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

use crate::discovery::LoopixProviderDiscovery;
use crate::types::{LoopixProvider, LoopixRoute};

/// Loopix backend with local SOCKS stub.
pub struct LoopixBackend {
    profile: AnonymityProfile,
    discovery: LoopixProviderDiscovery,
    selected_provider: RwLock<Option<LoopixProvider>>,
    listen_port: RwLock<Option<u16>>,
    running: RwLock<bool>,
    stub_mode: RwLock<bool>,
    shutdown: RwLock<Option<watch::Sender<bool>>>,
    relay_task: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl LoopixBackend {
    pub fn new(profile: AnonymityProfile, discovery: LoopixProviderDiscovery) -> Arc<Self> {
        Arc::new(Self {
            profile,
            discovery,
            selected_provider: RwLock::new(None),
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
            .map_err(|e| WireSentinelError::Other(format!("loopix stub bind: {e}")))?;
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
                        debug!(%peer, "loopix stub socks accepted");
                    }
                }
            }
        });
        *self.relay_task.write() = Some(task);
        Ok(port)
    }

    fn build_route(&self, port: Option<u16>) -> LoopixRoute {
        let provider = self
            .selected_provider
            .read()
            .clone()
            .unwrap_or_else(|| LoopixProvider {
                id: "loopix-stub".into(),
                address: "127.0.0.1:0".into(),
                public_key: "stub".into(),
                layer: 1,
                latency_ms: Some(38),
                healthy: true,
                last_seen: Some(Utc::now()),
            });
        LoopixRoute {
            id: Uuid::new_v4(),
            profile_id: self.profile.id,
            entry_provider: provider.clone(),
            exit_provider: provider,
            hop_count: 3,
            hops: vec![
                "loopix-entry".into(),
                "loopix-mix".into(),
                "loopix-exit".into(),
            ],
            socks_port: port,
        }
    }
}

#[async_trait]
impl AnonymityBackend for LoopixBackend {
    fn provider_id(&self) -> &str {
        "loopix"
    }

    fn profile(&self) -> &AnonymityProfile {
        &self.profile
    }

    async fn start(&self) -> Result<AnonymitySession> {
        if let Some(p) = self.discovery.select_best() {
            *self.selected_provider.write() = Some(p);
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
            AnonymityHealth::ok("loopix stub running")
        } else {
            AnonymityHealth::degraded("loopix stopped")
        }
    }

    async fn latency(&self) -> Result<u64> {
        Ok(self
            .selected_provider
            .read()
            .as_ref()
            .and_then(|p| p.latency_ms)
            .unwrap_or(38))
    }

    fn route_info(&self) -> AnonymityRoute {
        let lp_route = self.build_route(*self.listen_port.read());
        AnonymityRoute {
            id: lp_route.id,
            profile_id: lp_route.profile_id,
            provider: AnonymityProvider::Loopix,
            entry_node: lp_route.entry_provider.id.clone(),
            exit_node: lp_route.exit_provider.id.clone(),
            hop_count: lp_route.hop_count,
            hops: lp_route.hops,
            socks_port: lp_route.socks_port,
        }
    }

    fn privacy_score(&self) -> f64 {
        if *self.stub_mode.read() {
            0.42
        } else {
            0.8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_discovery() -> LoopixProviderDiscovery {
        let json = r#"{"providers":[{"id":"p1","address":"127.0.0.1:5555","public_key":"pk1","layer":1,"latency_ms":30,"healthy":true,"last_seen":null}]}"#;
        LoopixProviderDiscovery::from_json(json).expect("parse")
    }

    #[tokio::test]
    async fn loopix_start_stop() {
        let profile = AnonymityProfile::new("lp-test", AnonymityProvider::Loopix);
        let backend = LoopixBackend::new(profile, sample_discovery());
        let session = backend.start().await.expect("start");
        assert!(session.socks_port > 0);
        backend.stop().await.expect("stop");
    }
}
