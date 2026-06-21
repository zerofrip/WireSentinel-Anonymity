use serde::{Deserialize, Serialize};

/// Entropy analysis report for candidate anonymity paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntropyReport {
    pub path_count: usize,
    pub mean_entropy: f64,
    pub best_path_index: usize,
    pub best_score: f64,
    pub path_scores: Vec<f64>,
}
