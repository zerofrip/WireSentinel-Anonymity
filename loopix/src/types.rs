use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Loopix mix provider descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopixProvider {
    pub id: String,
    pub address: String,
    pub public_key: String,
    pub layer: u8,
    pub latency_ms: Option<u64>,
    pub healthy: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Active Loopix route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopixRoute {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub entry_provider: LoopixProvider,
    pub exit_provider: LoopixProvider,
    pub hop_count: u32,
    pub hops: Vec<String>,
    pub socks_port: Option<u16>,
}
