/// Aggregated Phase 13 anonymity analytics metrics.
#[derive(Debug, Clone, Copy)]
pub struct AnonymityAnalyticsSnapshot {
    pub cover_traffic_efficiency: f64,
    pub mixnet_diversity: f64,
    pub federation_diversity: f64,
}

/// Computes advanced anonymity metrics from runtime counters.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnonymityAnalytics;

impl AnonymityAnalytics {
    pub fn new() -> Self {
        Self
    }

    pub fn compute(
        &self,
        active_providers: u32,
        federated_providers: u32,
        adaptive_cover: bool,
        distinct_route_types: u32,
    ) -> AnonymityAnalyticsSnapshot {
        let provider_factor = (active_providers as f64 / 4.0).min(1.0);
        let federated_factor = (federated_providers as f64 / 3.0).min(1.0);
        let route_factor = (distinct_route_types as f64 / 8.0).min(1.0);
        let adaptive_bonus = if adaptive_cover { 0.15 } else { 0.0 };

        AnonymityAnalyticsSnapshot {
            cover_traffic_efficiency: (provider_factor * 0.5 + adaptive_bonus).min(1.0),
            mixnet_diversity: route_factor,
            federation_diversity: federated_factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_snapshot_scales_with_providers() {
        let analytics = AnonymityAnalytics::new();
        let low = analytics.compute(1, 0, false, 1);
        let high = analytics.compute(4, 3, true, 8);
        assert!(high.cover_traffic_efficiency > low.cover_traffic_efficiency);
        assert!(high.federation_diversity > low.federation_diversity);
    }
}
