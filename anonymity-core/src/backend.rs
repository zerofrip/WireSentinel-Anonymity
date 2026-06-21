use async_trait::async_trait;

use crate::types::{
    AnonymityHealth, AnonymityProfile, AnonymityRoute, AnonymitySession, AnonymityStatus,
};
use shared_types::Result;

/// Pluggable anonymity transport backend (Nym, Katzenpost, Loopix, federated).
#[async_trait]
pub trait AnonymityBackend: Send + Sync {
    fn provider_id(&self) -> &str;
    fn profile(&self) -> &AnonymityProfile;

    async fn start(&self) -> Result<AnonymitySession>;
    async fn stop(&self) -> Result<()>;
    fn status(&self) -> AnonymityStatus;
    async fn health(&self) -> AnonymityHealth;
    async fn latency(&self) -> Result<u64>;
    fn route_info(&self) -> AnonymityRoute;
    fn privacy_score(&self) -> f64;
}
