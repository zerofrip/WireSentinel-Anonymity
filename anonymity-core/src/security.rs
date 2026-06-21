use serde::{Deserialize, Serialize};

use crate::types::AnonymityProvider;

/// Security constraints applied when launching anonymity providers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymitySecurityPolicy {
    /// Require provider binaries to exist before starting (no stub fallback).
    pub require_binaries: bool,
    /// Allowed provider kinds (empty = allow all).
    pub allowed_providers: Vec<AnonymityProvider>,
    /// Maximum subprocess restarts within a session.
    pub max_restarts: u32,
    /// Bind local SOCKS relay to loopback only.
    pub loopback_only: bool,
    /// Minimum privacy score required to keep a session active.
    pub min_privacy_score: f64,
}

impl Default for AnonymitySecurityPolicy {
    fn default() -> Self {
        Self {
            require_binaries: false,
            allowed_providers: vec![],
            max_restarts: 3,
            loopback_only: true,
            min_privacy_score: 0.0,
        }
    }
}

impl AnonymitySecurityPolicy {
    pub fn provider_allowed(&self, provider: &AnonymityProvider) -> bool {
        self.allowed_providers.is_empty()
            || self.allowed_providers.iter().any(|p| p == provider)
    }

    pub fn privacy_score_allowed(&self, score: f64) -> bool {
        score >= self.min_privacy_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_permits_all() {
        let policy = AnonymitySecurityPolicy::default();
        assert!(policy.provider_allowed(&AnonymityProvider::Nym));
        assert!(policy.provider_allowed(&AnonymityProvider::Katzenpost));
    }

    #[test]
    fn allowlist_restricts_providers() {
        let policy = AnonymitySecurityPolicy {
            allowed_providers: vec![AnonymityProvider::Nym],
            ..Default::default()
        };
        assert!(policy.provider_allowed(&AnonymityProvider::Nym));
        assert!(!policy.provider_allowed(&AnonymityProvider::Loopix));
    }
}
