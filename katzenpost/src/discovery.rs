use serde::{Deserialize, Serialize};

use crate::types::KatzenpostGateway;

/// Discovers Katzenpost gateways from static JSON configuration.
#[derive(Debug, Clone, Default)]
pub struct KatzenpostGatewayDiscovery {
    gateways: Vec<KatzenpostGateway>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GatewayConfig {
    gateways: Vec<KatzenpostGateway>,
}

impl KatzenpostGatewayDiscovery {
    pub fn from_json(json: &str) -> shared_types::Result<Self> {
        let cfg: GatewayConfig = serde_json::from_str(json)
            .map_err(|e| shared_types::WireSentinelError::Other(e.to_string()))?;
        Ok(Self {
            gateways: cfg.gateways,
        })
    }

    pub fn gateways(&self) -> &[KatzenpostGateway] {
        &self.gateways
    }

    pub fn discover(&self) -> Vec<KatzenpostGateway> {
        self.gateways
            .iter()
            .filter(|g| g.healthy)
            .cloned()
            .collect()
    }

    pub fn select_best(&self) -> Option<KatzenpostGateway> {
        self.discover()
            .into_iter()
            .min_by_key(|g| g.latency_ms.unwrap_or(u64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_healthy_gateways() {
        let json = r#"{
            "gateways": [
                {"id":"gw1","address":"127.0.0.1:4444","identity_key":"k1","country":"DE","latency_ms":50,"healthy":true,"last_seen":null},
                {"id":"gw2","address":"127.0.0.1:4445","identity_key":"k2","country":"NL","latency_ms":30,"healthy":false,"last_seen":null}
            ]
        }"#;
        let discovery = KatzenpostGatewayDiscovery::from_json(json).expect("parse");
        assert_eq!(discovery.discover().len(), 1);
        assert_eq!(discovery.select_best().unwrap().id, "gw1");
    }
}
