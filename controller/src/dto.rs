use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Anonymity node reported by an endpoint agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymityNodeDto {
    pub id: Uuid,
    pub provider: String,
    pub gateway_id: String,
    pub country: Option<String>,
    pub latency_ms: Option<u64>,
    pub healthy: bool,
    pub privacy_score: f64,
}

/// Active anonymous route on an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymityRouteDto {
    pub route_id: Uuid,
    pub label: String,
    pub provider: String,
    pub hops: Vec<String>,
    pub socks_port: Option<u16>,
    pub cover_traffic_profile: Option<String>,
    pub active: bool,
}

/// Federated cross-mixnet route summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FederatedRouteDto {
    pub route_id: Uuid,
    pub providers: Vec<String>,
    pub hop_count: u32,
    pub privacy_score: f64,
}

/// Payload embedded in agent heartbeat metadata for anonymity status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymityHeartbeatPayload {
    pub agent_id: Uuid,
    pub reported_at: DateTime<Utc>,
    pub anonymity_connected: bool,
    pub stub_mode: bool,
    pub selected_node: Option<AnonymityNodeDto>,
    pub active_routes: Vec<AnonymityRouteDto>,
    pub federated_routes: Vec<FederatedRouteDto>,
    pub cover_traffic_profile: Option<String>,
    pub privacy_score: f64,
    pub path_diversity: f64,
}

impl AnonymityHeartbeatPayload {
    pub fn empty(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            reported_at: Utc::now(),
            anonymity_connected: false,
            stub_mode: false,
            selected_node: None,
            active_routes: Vec::new(),
            federated_routes: Vec::new(),
            cover_traffic_profile: None,
            privacy_score: 0.0,
            path_diversity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_payload_defaults() {
        let id = Uuid::new_v4();
        let payload = AnonymityHeartbeatPayload::empty(id);
        assert_eq!(payload.agent_id, id);
        assert!(!payload.anonymity_connected);
        assert!(payload.active_routes.is_empty());
    }
}
