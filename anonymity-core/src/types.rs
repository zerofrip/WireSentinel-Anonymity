use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_types::Result;
use uuid::Uuid;

/// Supported anonymity transport providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum AnonymityProvider {
    Nym,
    Katzenpost,
    Loopix,
    Plugin(Uuid),
    Federated,
}

/// Configuration for an anonymity backend profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymityProfile {
    pub id: Uuid,
    pub name: String,
    pub provider: AnonymityProvider,
    pub gateway_id: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AnonymityProfile {
    pub fn new(name: impl Into<String>, provider: AnonymityProvider) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            provider,
            gateway_id: None,
            config_json: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn provider_id(&self) -> String {
        match &self.provider {
            AnonymityProvider::Nym => "nym".into(),
            AnonymityProvider::Katzenpost => "katzenpost".into(),
            AnonymityProvider::Loopix => "loopix".into(),
            AnonymityProvider::Plugin(id) => format!("plugin:{id}"),
            AnonymityProvider::Federated => "federated".into(),
        }
    }
}

/// Active anonymous route metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymityRoute {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub provider: AnonymityProvider,
    pub entry_node: String,
    pub exit_node: String,
    pub hop_count: u32,
    pub hops: Vec<String>,
    pub socks_port: Option<u16>,
}

/// Runtime session for an anonymity backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymitySession {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub route: AnonymityRoute,
    pub socks_port: u16,
    pub stub_mode: bool,
    pub started_at: DateTime<Utc>,
}

/// Live status reported by an anonymity backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymityStatus {
    pub running: bool,
    pub stub_mode: bool,
    pub socks_port: Option<u16>,
    pub profile_id: Option<Uuid>,
    pub last_error: Option<String>,
}

/// Health snapshot for an anonymity backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymityHealth {
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

impl AnonymityHealth {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            healthy: true,
            latency_ms: None,
            message: Some(message.into()),
            checked_at: Utc::now(),
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            latency_ms: None,
            message: Some(message.into()),
            checked_at: Utc::now(),
        }
    }
}

pub type AnonymityResult<T> = Result<T>;
