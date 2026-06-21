use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Manifest describing an anonymity provider plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymityPluginManifest {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub description: Option<String>,
    pub min_privacy_score: f64,
}

impl AnonymityPluginManifest {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            provider: provider.into(),
            description: None,
            min_privacy_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_fields_roundtrip() {
        let manifest = AnonymityPluginManifest::new("test-plugin", "0.1.0", "custom");
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.provider, "custom");
    }
}
