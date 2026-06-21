use anonymity_core::AnonymityRoute;

use crate::report::EntropyReport;

/// Scores and optimizes anonymity path entropy.
#[derive(Debug, Clone, Default)]
pub struct RouteEntropyEngine;

impl RouteEntropyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Score each path by hop diversity and provider spread.
    pub fn score_paths(&self, paths: &[AnonymityRoute]) -> EntropyReport {
        let path_scores: Vec<f64> = paths.iter().map(|p| self.score_single(p)).collect();
        let mean_entropy = if path_scores.is_empty() {
            0.0
        } else {
            path_scores.iter().sum::<f64>() / path_scores.len() as f64
        };
        let (best_path_index, best_score) = path_scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, s)| (i, *s))
            .unwrap_or((0, 0.0));

        EntropyReport {
            path_count: paths.len(),
            mean_entropy,
            best_path_index,
            best_score,
            path_scores,
        }
    }

    pub fn optimize(&self, paths: &[AnonymityRoute]) -> Option<AnonymityRoute> {
        let report = self.score_paths(paths);
        paths.get(report.best_path_index).cloned()
    }

    fn score_single(&self, route: &AnonymityRoute) -> f64 {
        if route.hops.is_empty() {
            return 0.0;
        }
        let unique_hops = route
            .hops
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        let hop_factor = unique_hops as f64 / route.hops.len() as f64;
        let depth_factor = (route.hop_count as f64).ln_1p() / 3.0;
        (hop_factor * 0.6 + depth_factor.min(1.0) * 0.4).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anonymity_core::{AnonymityProvider};
    use uuid::Uuid;

    fn sample_route(hops: Vec<&str>) -> AnonymityRoute {
        AnonymityRoute {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            provider: AnonymityProvider::Nym,
            entry_node: hops.first().unwrap_or(&"a").to_string(),
            exit_node: hops.last().unwrap_or(&"z").to_string(),
            hop_count: hops.len() as u32,
            hops: hops.into_iter().map(String::from).collect(),
            socks_port: None,
        }
    }

    #[test]
    fn score_paths_picks_diverse_route() {
        let engine = RouteEntropyEngine::new();
        let paths = vec![
            sample_route(vec!["a", "a", "a"]),
            sample_route(vec!["a", "b", "c", "d"]),
        ];
        let report = engine.score_paths(&paths);
        assert_eq!(report.best_path_index, 1);
        assert!(report.best_score > report.path_scores[0]);
    }

    #[test]
    fn optimize_returns_best_path() {
        let engine = RouteEntropyEngine::new();
        let paths = vec![
            sample_route(vec!["x"]),
            sample_route(vec!["x", "y", "z"]),
        ];
        let best = engine.optimize(&paths).expect("best");
        assert_eq!(best.hop_count, 3);
    }
}
