use std::collections::HashMap;

use tracing::warn;
use uuid::Uuid;

use crate::types::{DecoyHop, DecoyMode, DecoyRoute, DecoySimulationResult};

/// In-memory decoy routing simulator. **Never performs real network operations.**
pub struct DecoyRoutingFramework {
    mode: DecoyMode,
    routes: HashMap<Uuid, DecoyRoute>,
}

impl DecoyRoutingFramework {
    pub fn new(mode: DecoyMode) -> Self {
        if !matches!(
            mode,
            DecoyMode::Research | DecoyMode::Simulation | DecoyMode::Lab
        ) {
            warn!("decoy routing restricted to research/simulation/lab modes");
        }
        Self {
            mode,
            routes: HashMap::new(),
        }
    }

    pub fn mode(&self) -> DecoyMode {
        self.mode
    }

    /// Build a simulated route with decoy hops injected in-memory.
    pub fn simulate_route(
        &mut self,
        real_hops: &[&str],
        decoy_count: u32,
    ) -> DecoySimulationResult {
        let mut hops: Vec<DecoyHop> = real_hops
            .iter()
            .map(|label| DecoyHop {
                id: Uuid::new_v4(),
                label: (*label).to_string(),
                is_decoy: false,
            })
            .collect();

        for i in 0..decoy_count {
            hops.push(DecoyHop {
                id: Uuid::new_v4(),
                label: format!("decoy-{i}"),
                is_decoy: true,
            });
        }

        let total = hops.len().max(1);
        let decoy_ratio = decoy_count as f64 / total as f64;
        let route = DecoyRoute {
            id: Uuid::new_v4(),
            hops: hops.clone(),
            decoy_ratio,
        };
        self.routes.insert(route.id, route.clone());

        DecoySimulationResult {
            effective_anonymity: (0.5 + decoy_ratio * 0.4).min(0.95),
            decoys_injected: decoy_count,
            mode: self.mode,
            route,
        }
    }

    pub fn get_route(&self, id: Uuid) -> Option<&DecoyRoute> {
        self.routes.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_injects_decoys_in_memory() {
        let mut fw = DecoyRoutingFramework::new(DecoyMode::Simulation);
        let result = fw.simulate_route(&["entry", "mix", "exit"], 2);
        assert_eq!(result.decoys_injected, 2);
        assert_eq!(result.route.hops.len(), 5);
        assert!(result.effective_anonymity > 0.5);
    }

    #[test]
    fn research_mode_never_leaves_framework() {
        let mut fw = DecoyRoutingFramework::new(DecoyMode::Research);
        let result = fw.simulate_route(&["a"], 1);
        assert_eq!(result.mode, DecoyMode::Research);
        assert!(fw.get_route(result.route.id).is_some());
    }
}
