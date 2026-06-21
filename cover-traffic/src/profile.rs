use serde::{Deserialize, Serialize};

/// Adaptive cover traffic intensity preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdaptiveCoverProfile {
    Conservative,
    Balanced,
    Aggressive,
    Maximum,
}

impl AdaptiveCoverProfile {
    pub fn packets_per_second(self) -> u32 {
        match self {
            Self::Conservative => 2,
            Self::Balanced => 8,
            Self::Aggressive => 20,
            Self::Maximum => 50,
        }
    }

    pub fn bandwidth_bps(self) -> u64 {
        match self {
            Self::Conservative => 4_096,
            Self::Balanced => 16_384,
            Self::Aggressive => 65_536,
            Self::Maximum => 262_144,
        }
    }
}

impl Default for AdaptiveCoverProfile {
    fn default() -> Self {
        Self::Balanced
    }
}
