use std::sync::Arc;

use async_trait::async_trait;

use anonymity_core::{AnonymityBackend, AnonymityProfile};
use shared_types::Result;

use crate::manifest::AnonymityPluginManifest;

/// Stable hook for Wasm/native anonymity provider loaders.
#[async_trait]
pub trait AnonymityPlugin: Send + Sync {
    fn manifest(&self) -> &AnonymityPluginManifest;

    /// Create a backend instance for the given profile.
    fn create_backend(&self, profile: AnonymityProfile) -> Result<Arc<dyn AnonymityBackend>>;
}
