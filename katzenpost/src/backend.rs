use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use parking_lot::RwLock;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing::debug;
use uuid::Uuid;

use anonymity_core::{
    AnonymityBackend, AnonymityHealth, AnonymityProfile, AnonymityProvider, AnonymityRoute,
    AnonymitySession, AnonymityStatus,
};
use shared_types::{Result, WireSentinelError};

use crate::discovery::KatzenpostGatewayDiscovery;
use crate::types::{KatzenpostGateway, KatzenpostRoute};

/// Katzenpost backend with local SOCKS stub (same pattern as Nym).
pub struct KatzenpostBackend {
    profile: AnonymityProfile,
    discovery: KatzenpostGatewayDiscovery,
    selected_gateway: RwLock<Option<KatzenpostGateway>>,
    listen_port: RwLock<Option<u16>>,
    running: RwLock<bool>,
    stub_mode: RwLock<bool>,
    shutdown: RwLock<Option<watch::Sender<bool>>>,
    relay_task: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl KatzenpostBackend {
    pub fn new(profile: AnonymityProfile, discovery: KatzenpostGatewayDiscovery) -> Arc<Self> {
        Arc::new(Self {
            profile,
            discovery,
            selected_gateway: RwLock::new(None),
            listen_port: RwLock::new(None),
            running: RwLock::new(false),
            stub_mode: RwLock::new(true),
            shutdown: RwLock::new(None),
            relay_task: RwLock::new(None),
        })
    }

    async fn start_stub_socks(&self) -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| WireSentinelError::Other(format!("katzenpost stub bind: {e}")))?;
        let port = listener
            .local_addr()
            .map_err(|e| WireSentinelError::Other(e.to_string()))?
            .port();

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        *self.shutdown.write() = Some(shutdown_tx);

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() { break; }
                    }
                    accept = listener.accept() => {
                        let Ok((_stream, peer)) = accept else { continue };
                        debug!(%peer, "katzenpost stub socks accepted");
                    }
                }
            }
        });
        *self.relay_task.write() = Some(task);
        Ok(port)
    }

    fn build_route(&self, port: Option<u16>) -> KatzenpostRoute {
        let gateway = self
            .selected_gateway
            .read()
            .clone()
            .unwrap_or_else(|| KatzenpostGateway {
                id: "katzenpost-stub".into(),
                address: "127.0.0.1:0".into(),
                identity_key: "stub".into(),
                country: None,
                latency_ms: Some(42),
                healthy: true,
                last_seen: Some(Utc::now()),
            });
        KatzenpostRoute {
            id: Uuid::new_v4(),
            profile_id: self.profile.id,
            entry_gateway: gateway.clone(),
            exit_gateway: gateway,
            hop_count: 2,
            hops: vec!["katzenpost-entry".into(), "katzenpost-exit".into()],
            socks_port: port,
        }
    }
}

#[async_trait]
impl AnonymityBackend for KatzenpostBackend {
    fn provider_id(&self) -> &str {
        "katzenpost"
    }

    fn profile(&self) -> &AnonymityProfile {
        &self.profile
    }

    async fn start(&self) -> Result<AnonymitySession> {
        if let Some(gw) = self.discovery.select_best() {
            *self.selected_gateway.write() = Some(gw);
        }
        let port = self.start_stub_socks().await?;
        *self.listen_port.write() = Some(port);
        *self.running.write() = true;
        *self.stub_mode.write() = true;

        let route = self.route_info();
        Ok(AnonymitySession {
            id: Uuid::new_v4(),
            profile_id: self.profile.id,
            route,
            socks_port: port,
            stub_mode: true,
            started_at: Utc::now(),
        })
    }

    async fn stop(&self) -> Result<()> {
        if let Some(tx) = self.shutdown.write().take() {
            let _ = tx.send(true);
        }
        if let Some(task) = self.relay_task.write().take() {
            task.abort();
        }
        *self.running.write() = false;
        *self.listen_port.write() = None;
        Ok(())
    }

    fn status(&self) -> AnonymityStatus {
        AnonymityStatus {
            running: *self.running.read(),
            stub_mode: *self.stub_mode.read(),
            socks_port: *self.listen_port.read(),
            profile_id: Some(self.profile.id),
            last_error: None,
        }
    }

    async fn health(&self) -> AnonymityHealth {
        if *self.running.read() {
            AnonymityHealth::ok("katzenpost stub running")
        } else {
            AnonymityHealth::degraded("katzenpost stopped")
        }
    }

    async fn latency(&self) -> Result<u64> {
        Ok(self
            .selected_gateway
            .read()
            .as_ref()
            .and_then(|g| g.latency_ms)
            .unwrap_or(42))
    }

    fn route_info(&self) -> AnonymityRoute {
        let kp_route = self.build_route(*self.listen_port.read());
        AnonymityRoute {
            id: kp_route.id,
            profile_id: kp_route.profile_id,
            provider: AnonymityProvider::Katzenpost,
            entry_node: kp_route.entry_gateway.id.clone(),
            exit_node: kp_route.exit_gateway.id.clone(),
            hop_count: kp_route.hop_count,
            hops: kp_route.hops,
            socks_port: kp_route.socks_port,
        }
    }

    fn privacy_score(&self) -> f64 {
        if *self.stub_mode.read() {
            0.4
        } else {
            0.78
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_discovery() -> KatzenpostGatewayDiscovery {
        let json = r#"{"gateways":[{"id":"gw1","address":"127.0.0.1:4444","identity_key":"k1","country":"DE","latency_ms":25,"healthy":true,"last_seen":null}]}"#;
        KatzenpostGatewayDiscovery::from_json(json).expect("parse")
    }

    #[tokio::test]
    async fn katzenpost_start_stop() {
        let profile = AnonymityProfile::new("kp-test", AnonymityProvider::Katzenpost);
        let backend = KatzenpostBackend::new(profile, sample_discovery());
        let session = backend.start().await.expect("start");
        assert!(session.socks_port > 0);
        backend.stop().await.expect("stop");
    }
}
