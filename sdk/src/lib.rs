//! WireSentinel Anonymity SDK — public surface for anonymity provider plugins.

mod manifest;
mod plugin;

pub use anonymity_core::{
    AnonymityBackend, AnonymityHealth, AnonymityProfile, AnonymityProvider,
    AnonymityRoute, AnonymitySecurityPolicy, AnonymitySession, AnonymityStatus,
};
pub use manifest::AnonymityPluginManifest;
pub use plugin::AnonymityPlugin;
