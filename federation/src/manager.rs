use std::collections::HashMap;
use std::sync::Arc;

use anonymity_core::{
    AnonymityBackend, AnonymityHealth, AnonymityProfile, AnonymityProvider, AnonymityRoute,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registered provider in a federated mixnet deployment.
#[derive(Clone)]
pub struct FederatedProvider {
    pub id: Uuid,
    pub profile: AnonymityProfile,
    pub backend: Arc<dyn AnonymityBackend>,
}

/// Result of cross-mixnet route validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FederationValidation {
    pub valid: bool,
    pub providers: Vec<String>,
    pub hop_count: u32,
    pub message: Option<String>,
}

/// Manages multiple anonymity backends across mixnet providers.
pub struct MixnetFederationManager {
    providers: RwLock<HashMap<Uuid, FederatedProvider>>,
}

impl MixnetFederationManager {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, backend: Arc<dyn AnonymityBackend>) -> Uuid {
        let id = Uuid::new_v4();
        let profile = backend.profile().clone();
        self.providers.write().insert(
            id,
            FederatedProvider {
                id,
                profile,
                backend,
            },
        );
        id
    }

    pub fn discover(&self) -> Vec<AnonymityProfile> {
        self.providers
            .read()
            .values()
            .map(|p| p.profile.clone())
            .collect()
    }

    pub async fn health_poll(&self) -> HashMap<Uuid, AnonymityHealth> {
        let providers: Vec<_> = self.providers.read().values().cloned().collect();
        let mut out = HashMap::new();
        for p in providers {
            let health = p.backend.health().await;
            out.insert(p.id, health);
        }
        out
    }

    pub fn optimize_route(&self) -> Option<AnonymityRoute> {
        let providers: Vec<_> = self.providers.read().values().cloned().collect();
        if providers.is_empty() {
            return None;
        }

        let best = providers
            .iter()
            .max_by(|a, b| {
                a.backend
                    .privacy_score()
                    .partial_cmp(&b.backend.privacy_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;

        let mut route = best.backend.route_info();
        if providers.len() > 1 {
            route.provider = AnonymityProvider::Federated;
            route.hop_count = providers.len() as u32;
            route.hops = providers
                .iter()
                .map(|p| p.backend.provider_id().to_string())
                .collect();
        }
        Some(route)
    }

    pub fn validate_cross_mixnet(&self, route: &AnonymityRoute) -> FederationValidation {
        let provider_ids: Vec<String> = self
            .providers
            .read()
            .values()
            .map(|p| p.backend.provider_id().to_string())
            .collect();

        let unique_hops: std::collections::HashSet<_> = route.hops.iter().cloned().collect();
        let valid = route.hop_count >= 2
            && !route.hops.is_empty()
            && (route.provider == AnonymityProvider::Federated || unique_hops.len() == route.hops.len());

        FederationValidation {
            valid,
            providers: provider_ids,
            hop_count: route.hop_count,
            message: if valid {
                Some("cross-mixnet route validated".into())
            } else {
                Some("invalid federated route".into())
            },
        }
    }
}

impl Default for MixnetFederationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anonymity_core::AnonymityProvider;
    use katzenpost::{KatzenpostBackend, KatzenpostGatewayDiscovery};
    use loopix::{LoopixBackend, LoopixProviderDiscovery};

    fn katzenpost_backend() -> Arc<dyn AnonymityBackend> {
        let json = r#"{"gateways":[{"id":"gw1","address":"127.0.0.1:4444","identity_key":"k1","country":"DE","latency_ms":25,"healthy":true,"last_seen":null}]}"#;
        let profile = AnonymityProfile::new("kp", AnonymityProvider::Katzenpost);
        KatzenpostBackend::new(
            profile,
            KatzenpostGatewayDiscovery::from_json(json).expect("parse"),
        )
    }

    fn loopix_backend() -> Arc<dyn AnonymityBackend> {
        let json = r#"{"providers":[{"id":"p1","address":"127.0.0.1:5555","public_key":"pk1","layer":1,"latency_ms":30,"healthy":true,"last_seen":null}]}"#;
        let profile = AnonymityProfile::new("lp", AnonymityProvider::Loopix);
        LoopixBackend::new(
            profile,
            LoopixProviderDiscovery::from_json(json).expect("parse"),
        )
    }

    #[test]
    fn register_discover_optimize() {
        let mgr = MixnetFederationManager::new();
        mgr.register(katzenpost_backend());
        mgr.register(loopix_backend());
        assert_eq!(mgr.discover().len(), 2);
        let route = mgr.optimize_route().expect("route");
        assert_eq!(route.provider, AnonymityProvider::Federated);
    }

    #[tokio::test]
    async fn health_poll_returns_entries() {
        let mgr = MixnetFederationManager::new();
        let id = mgr.register(katzenpost_backend());
        let health = mgr.health_poll().await;
        assert!(health.contains_key(&id));
    }
}
