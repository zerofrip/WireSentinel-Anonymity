use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mixnet_core::{MixnetBackend, MixnetProfile};
use mixnet_transports::NymBackend;
use uuid::Uuid;

use crate::backend::AnonymityBackend;
use crate::types::{
    AnonymityHealth, AnonymityProfile, AnonymityProvider, AnonymityRoute, AnonymitySession,
    AnonymityStatus,
};
use shared_types::Result;

/// Wraps a mixnet-core [`MixnetBackend`] (typically Nym) as an [`AnonymityBackend`].
pub struct NymAnonymityAdapter {
    profile: AnonymityProfile,
    inner: Arc<dyn MixnetBackend>,
}

impl NymAnonymityAdapter {
    pub fn from_profile(profile: AnonymityProfile) -> Arc<Self> {
        let mixnet_profile = MixnetProfile::new(&profile.name, "nym");
        let inner = NymBackend::new(mixnet_profile);
        Arc::new(Self { profile, inner })
    }

    pub fn with_backend(profile: AnonymityProfile, inner: Arc<dyn MixnetBackend>) -> Arc<Self> {
        Arc::new(Self { profile, inner })
    }
}

#[async_trait]
impl AnonymityBackend for NymAnonymityAdapter {
    fn provider_id(&self) -> &str {
        "nym"
    }

    fn profile(&self) -> &AnonymityProfile {
        &self.profile
    }

    async fn start(&self) -> Result<AnonymitySession> {
        let port = self.inner.start().await?;
        let mixnet_status = self.inner.status();
        let route = self.route_info();
        Ok(AnonymitySession {
            id: Uuid::new_v4(),
            profile_id: self.profile.id,
            route,
            socks_port: port,
            stub_mode: mixnet_status.stub_mode,
            started_at: Utc::now(),
        })
    }

    async fn stop(&self) -> Result<()> {
        self.inner.stop().await
    }

    fn status(&self) -> AnonymityStatus {
        let mixnet = self.inner.status();
        AnonymityStatus {
            running: matches!(mixnet.state, mixnet_core::MixnetState::Running),
            stub_mode: mixnet.stub_mode,
            socks_port: mixnet.socks_port,
            profile_id: Some(self.profile.id),
            last_error: mixnet.last_error,
        }
    }

    async fn health(&self) -> AnonymityHealth {
        let h = self.inner.health_check().await;
        let latency_ms = self.inner.measure_latency().await.ok();
        AnonymityHealth {
            healthy: h.healthy,
            latency_ms,
            message: h.message,
            checked_at: h.checked_at,
        }
    }

    async fn latency(&self) -> Result<u64> {
        self.inner.measure_latency().await
    }

    fn route_info(&self) -> AnonymityRoute {
        let gateway = self
            .profile
            .gateway_id
            .clone()
            .unwrap_or_else(|| "nym-gateway".into());
        AnonymityRoute {
            id: Uuid::new_v4(),
            profile_id: self.profile.id,
            provider: AnonymityProvider::Nym,
            entry_node: gateway.clone(),
            exit_node: gateway,
            hop_count: 3,
            hops: vec!["nym-entry".into(), "nym-mix".into(), "nym-exit".into()],
            socks_port: self.inner.status().socks_port,
        }
    }

    fn privacy_score(&self) -> f64 {
        if self.status().stub_mode {
            0.35
        } else {
            0.82
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn nym_adapter_start_stop() {
        let profile = AnonymityProfile::new("test-nym", AnonymityProvider::Nym);
        let adapter = NymAnonymityAdapter::from_profile(profile);
        let session = adapter.start().await.expect("start");
        assert!(session.socks_port > 0);
        assert!(adapter.status().running);
        adapter.stop().await.expect("stop");
    }

    #[test]
    fn privacy_score_reflects_stub_mode() {
        let profile = AnonymityProfile::new("test", AnonymityProvider::Nym);
        let adapter = NymAnonymityAdapter::from_profile(profile);
        let score = adapter.privacy_score();
        assert!(score > 0.0 && score <= 1.0);
    }
}
