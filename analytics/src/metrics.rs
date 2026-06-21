use anonymity_core::{AnonymityProfile, AnonymityRoute};

/// Measure unique-hop diversity across paths (0.0–1.0).
pub fn path_diversity(paths: &[AnonymityRoute]) -> f64 {
    if paths.is_empty() {
        return 0.0;
    }
    let mut all_hops = std::collections::HashSet::new();
    let mut total = 0usize;
    for path in paths {
        total += path.hops.len();
        for hop in &path.hops {
            all_hops.insert(hop.as_str());
        }
    }
    if total == 0 {
        return 0.0;
    }
    all_hops.len() as f64 / total as f64
}

/// Measure provider spread in a federated deployment (0.0–1.0).
pub fn federation_diversity(profiles: &[AnonymityProfile]) -> f64 {
    if profiles.is_empty() {
        return 0.0;
    }
    let unique: std::collections::HashSet<_> = profiles
        .iter()
        .map(|p| format!("{:?}", p.provider))
        .collect();
    unique.len() as f64 / profiles.len() as f64
}

/// Estimate anonymity set size from route and provider diversity.
pub fn anonymity_set_estimate(paths: &[AnonymityRoute], profiles: &[AnonymityProfile]) -> f64 {
    let path_div = path_diversity(paths);
    let fed_div = federation_diversity(profiles);
    let hop_bonus = paths
        .iter()
        .map(|p| p.hop_count as f64)
        .sum::<f64>()
        / paths.len().max(1) as f64;
    let base = (path_div * 0.4 + fed_div * 0.4).min(1.0);
    (base * hop_bonus.ln_1p()).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anonymity_core::AnonymityProvider;
    use uuid::Uuid;

    fn route(hops: Vec<&str>) -> AnonymityRoute {
        AnonymityRoute {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            provider: AnonymityProvider::Nym,
            entry_node: "a".into(),
            exit_node: "z".into(),
            hop_count: hops.len() as u32,
            hops: hops.into_iter().map(String::from).collect(),
            socks_port: None,
        }
    }

    #[test]
    fn path_diversity_increases_with_unique_hops() {
        let diverse = vec![route(vec!["a", "b", "c"]), route(vec!["d", "e"])];
        let flat = vec![route(vec!["a", "a"]), route(vec!["a", "a"])];
        assert!(path_diversity(&diverse) > path_diversity(&flat));
    }

    #[test]
    fn federation_diversity_counts_providers() {
        let profiles = vec![
            AnonymityProfile::new("n", AnonymityProvider::Nym),
            AnonymityProfile::new("k", AnonymityProvider::Katzenpost),
        ];
        assert_eq!(federation_diversity(&profiles), 1.0);
    }

    #[test]
    fn anonymity_set_estimate_positive() {
        let paths = vec![route(vec!["a", "b", "c"])];
        let profiles = vec![AnonymityProfile::new("n", AnonymityProvider::Nym)];
        assert!(anonymity_set_estimate(&paths, &profiles) >= 1.0);
    }
}
