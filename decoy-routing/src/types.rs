use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Operating mode for decoy routing experiments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecoyMode {
    /// Research-only analysis, no network I/O.
    Research,
    /// In-memory simulation of decoy paths.
    Simulation,
    /// Lab harness for controlled experiments.
    Lab,
}

/// Simulated decoy hop (never touches a real network).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyHop {
    pub id: Uuid,
    pub label: String,
    pub is_decoy: bool,
}

/// Simulated decoy route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoyRoute {
    pub id: Uuid,
    pub hops: Vec<DecoyHop>,
    pub decoy_ratio: f64,
}

/// Result of an in-memory decoy routing simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoySimulationResult {
    pub route: DecoyRoute,
    pub effective_anonymity: f64,
    pub decoys_injected: u32,
    pub mode: DecoyMode,
}
