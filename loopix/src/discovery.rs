use serde::{Deserialize, Serialize};

use crate::types::LoopixProvider;

/// Discovers Loopix providers from static JSON configuration.
#[derive(Debug, Clone, Default)]
pub struct LoopixProviderDiscovery {
    providers: Vec<LoopixProvider>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProviderConfig {
    providers: Vec<LoopixProvider>,
}

impl LoopixProviderDiscovery {
    pub fn from_json(json: &str) -> shared_types::Result<Self> {
        let cfg: ProviderConfig = serde_json::from_str(json)
            .map_err(|e| shared_types::WireSentinelError::Other(e.to_string()))?;
        Ok(Self {
            providers: cfg.providers,
        })
    }

    pub fn providers(&self) -> &[LoopixProvider] {
        &self.providers
    }

    pub fn discover(&self) -> Vec<LoopixProvider> {
        self.providers
            .iter()
            .filter(|p| p.healthy)
            .cloned()
            .collect()
    }

    pub fn select_best(&self) -> Option<LoopixProvider> {
        self.discover()
            .into_iter()
            .min_by_key(|p| p.latency_ms.unwrap_or(u64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_healthy_providers() {
        let json = r#"{
            "providers": [
                {"id":"p1","address":"127.0.0.1:5555","public_key":"pk1","layer":1,"latency_ms":40,"healthy":true,"last_seen":null},
                {"id":"p2","address":"127.0.0.1:5556","public_key":"pk2","layer":2,"latency_ms":20,"healthy":false,"last_seen":null}
            ]
        }"#;
        let discovery = LoopixProviderDiscovery::from_json(json).expect("parse");
        assert_eq!(discovery.discover().len(), 1);
        assert_eq!(discovery.select_best().unwrap().id, "p1");
    }
}
