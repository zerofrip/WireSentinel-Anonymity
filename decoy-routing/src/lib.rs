//! Decoy routing framework — in-memory simulation only (Research/Simulation/Lab).

mod framework;
mod types;

pub use framework::DecoyRoutingFramework;
pub use types::{DecoyHop, DecoyMode, DecoyRoute, DecoySimulationResult};
