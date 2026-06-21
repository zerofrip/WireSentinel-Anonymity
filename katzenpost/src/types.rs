use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Katzenpost gateway descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KatzenpostGateway {
    pub id: String,
    pub address: String,
    pub identity_key: String,
    pub country: Option<String>,
    pub latency_ms: Option<u64>,
    pub healthy: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Active Katzenpost route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KatzenpostRoute {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub entry_gateway: KatzenpostGateway,
    pub exit_gateway: KatzenpostGateway,
    pub hop_count: u32,
    pub hops: Vec<String>,
    pub socks_port: Option<u16>,
}
