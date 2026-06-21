use std::collections::HashMap;

use anonymity_core::{AnonymityHealth, AnonymityProfile};
use chrono::Utc;
use parking_lot::RwLock;
use uuid::Uuid;

/// Registered anonymous service endpoint.
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub id: Uuid,
    pub profile: AnonymityProfile,
    pub endpoint: String,
    pub registered_at: chrono::DateTime<Utc>,
}

/// In-memory discovery registry for anonymity providers.
pub struct AnonymousDiscoveryEngine {
    services: RwLock<HashMap<Uuid, DiscoveredService>>,
}

impl AnonymousDiscoveryEngine {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, profile: AnonymityProfile, endpoint: impl Into<String>) -> Uuid {
        let id = Uuid::new_v4();
        self.services.write().insert(
            id,
            DiscoveredService {
                id,
                profile,
                endpoint: endpoint.into(),
                registered_at: Utc::now(),
            },
        );
        id
    }

    pub fn discover(&self) -> Vec<DiscoveredService> {
        self.services.read().values().cloned().collect()
    }

    pub fn discover_by_provider(&self, provider_id: &str) -> Vec<DiscoveredService> {
        self.discover()
            .into_iter()
            .filter(|s| s.profile.provider_id() == provider_id)
            .collect()
    }

    pub fn health_check(&self, id: Uuid) -> AnonymityHealth {
        if self.services.read().contains_key(&id) {
            AnonymityHealth::ok("service registered")
        } else {
            AnonymityHealth::degraded("service not found")
        }
    }
}

impl Default for AnonymousDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anonymity_core::AnonymityProvider;

    #[test]
    fn register_discover_health() {
        let engine = AnonymousDiscoveryEngine::new();
        let profile = AnonymityProfile::new("svc", AnonymityProvider::Nym);
        let id = engine.register(profile, "127.0.0.1:1080");
        assert_eq!(engine.discover().len(), 1);
        assert!(engine.health_check(id).healthy);
        assert!(!engine.health_check(Uuid::new_v4()).healthy);
    }
}
