// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Scoring and health calculation logic
//!
//! Provides algorithms for calculating system health, performance,
//! and reliability scores from collected metrics.

use super::types::{CacheMetrics, DiscoveryMetrics, ErrorMetrics, RoutingMetrics};

/// Calculate overall health score (0.0 to 1.0)
///
/// Health score is a composite of discovery success, routing success,
/// and error rates. A score of 1.0 indicates perfect health.
///
/// # Formula
/// ```text
/// health_score = (discovery_health + routing_health + error_health) / 3
/// ```
///
/// Where:
/// - `discovery_health` = discovery_success_rate / 100
/// - `routing_health` = routing_success_rate / 100
/// - `error_health` = (100 - error_rate) / 100
pub fn calculate_health_score(
    discovery: &DiscoveryMetrics,
    routing: &RoutingMetrics,
    errors: &ErrorMetrics,
) -> f64 {
    let discovery_health = discovery.discovery_success_rate / 100.0;
    let routing_health = routing.routing_success_rate / 100.0;
    let error_health = if errors.total_errors == 0 {
        1.0
    } else {
        (100.0 - errors.error_rate) / 100.0
    };

    (discovery_health + routing_health + error_health) / 3.0
}

/// Calculate performance score (0.0 to 1.0)
///
/// Performance score is based on operation speed and cache effectiveness.
/// A score of 1.0 indicates optimal performance.
///
/// # Formula
/// ```text
/// performance_score = (speed_score + cache_score) / 2
/// ```
///
/// Where:
/// - `speed_score` = 1.0 if avg < 10ms, scales down to 0.5 at 100ms
/// - `cache_score` = cache_hit_rate / 100
pub fn calculate_performance_score(
    discovery: &DiscoveryMetrics,
    cache: &CacheMetrics,
    routing: &RoutingMetrics,
) -> f64 {
    // Speed score based on discovery and routing times
    let avg_time = (discovery.avg_discovery_time_ms + routing.avg_routing_time_ms) / 2.0;
    let speed_score = if avg_time < 10.0 {
        1.0
    } else if avg_time < 100.0 {
        1.0 - ((avg_time - 10.0) / 180.0) // Scale from 1.0 at 10ms to 0.5 at 100ms
    } else {
        0.5 - ((avg_time - 100.0) / 1000.0).min(0.5) // Continue scaling down
    };

    // Cache effectiveness score
    let cache_score = cache.hit_rate / 100.0;

    (speed_score + cache_score) / 2.0
}

/// Calculate reliability score (0.0 to 1.0)
///
/// Reliability score combines routing success and error recovery rates.
/// A score of 1.0 indicates maximum reliability.
///
/// # Formula
/// ```text
/// reliability_score = (routing_success + recovery_success) / 2
/// ```
///
/// Where:
/// - `routing_success` = routing_success_rate / 100
/// - `recovery_success` = recovery_success_rate / 100
pub fn calculate_reliability_score(routing: &RoutingMetrics, errors: &ErrorMetrics) -> f64 {
    let routing_reliability = routing.routing_success_rate / 100.0;
    let recovery_reliability = errors.recovery_success_rate / 100.0;

    (routing_reliability + recovery_reliability) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_perfect() {
        let discovery = DiscoveryMetrics {
            discovery_success_rate: 100.0,
            ..Default::default()
        };
        let routing = RoutingMetrics {
            routing_success_rate: 100.0,
            ..Default::default()
        };
        let errors = ErrorMetrics {
            total_errors: 0,
            error_rate: 0.0,
            ..Default::default()
        };

        let score = calculate_health_score(&discovery, &routing, &errors);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_health_score_degraded() {
        let discovery = DiscoveryMetrics {
            discovery_success_rate: 80.0,
            ..Default::default()
        };
        let routing = RoutingMetrics {
            routing_success_rate: 90.0,
            ..Default::default()
        };
        let errors = ErrorMetrics {
            total_errors: 10,
            error_rate: 5.0,
            ..Default::default()
        };

        let score = calculate_health_score(&discovery, &routing, &errors);
        assert!(score > 0.8 && score < 0.9);
    }

    #[test]
    fn test_performance_score_excellent() {
        let discovery = DiscoveryMetrics {
            avg_discovery_time_ms: 5.0,
            ..Default::default()
        };
        let cache = CacheMetrics {
            hit_rate: 95.0,
            ..Default::default()
        };
        let routing = RoutingMetrics {
            avg_routing_time_ms: 3.0,
            ..Default::default()
        };

        let score = calculate_performance_score(&discovery, &cache, &routing);
        assert!(score > 0.9);
    }

    #[test]
    fn test_performance_score_moderate() {
        let discovery = DiscoveryMetrics {
            avg_discovery_time_ms: 50.0,
            ..Default::default()
        };
        let cache = CacheMetrics {
            hit_rate: 60.0,
            ..Default::default()
        };
        let routing = RoutingMetrics {
            avg_routing_time_ms: 40.0,
            ..Default::default()
        };

        let score = calculate_performance_score(&discovery, &cache, &routing);
        assert!(score > 0.5 && score < 0.8);
    }

    #[test]
    fn test_reliability_score_high() {
        let routing = RoutingMetrics {
            routing_success_rate: 99.0,
            ..Default::default()
        };
        let errors = ErrorMetrics {
            recovery_success_rate: 95.0,
            ..Default::default()
        };

        let score = calculate_reliability_score(&routing, &errors);
        assert!((score - 0.97).abs() < 0.01);
    }

    #[test]
    fn test_reliability_score_perfect() {
        let routing = RoutingMetrics {
            routing_success_rate: 100.0,
            ..Default::default()
        };
        let errors = ErrorMetrics {
            recovery_success_rate: 100.0,
            ..Default::default()
        };

        let score = calculate_reliability_score(&routing, &errors);
        assert!((score - 1.0).abs() < 0.01);
    }
}
