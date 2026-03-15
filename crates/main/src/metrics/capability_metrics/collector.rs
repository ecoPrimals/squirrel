// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics collection implementation
//!
//! Provides the main `CapabilityMetrics` collector with thread-safe
//! recording methods for all metric types.

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::helpers::{get_score_bucket, get_time_bucket};
use super::scoring::{
    calculate_health_score, calculate_performance_score, calculate_reliability_score,
};
use super::types::{
    CacheMetrics, DiscoveryMetrics, ErrorEvent, ErrorMetrics, MetricsSummary, RoutingMetrics,
    SelectionMetrics,
};

/// Comprehensive metrics collector for capability-based operations
///
/// Thread-safe metrics collection using `Arc<RwLock>` for concurrent access.
/// All recording methods are `async` and internally manage locking.
#[derive(Debug)]
pub struct CapabilityMetrics {
    /// Discovery performance metrics
    discovery_metrics: Arc<RwLock<DiscoveryMetrics>>,
    /// Service selection metrics
    selection_metrics: Arc<RwLock<SelectionMetrics>>,
    /// Cache performance metrics
    cache_metrics: Arc<RwLock<CacheMetrics>>,
    /// Request routing metrics
    routing_metrics: Arc<RwLock<RoutingMetrics>>,
    /// Error tracking metrics
    error_metrics: Arc<RwLock<ErrorMetrics>>,
}

impl CapabilityMetrics {
    /// Create new metrics collector
    ///
    /// Initializes all metric types with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery_metrics: Arc::new(RwLock::new(DiscoveryMetrics::default())),
            selection_metrics: Arc::new(RwLock::new(SelectionMetrics::default())),
            cache_metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            routing_metrics: Arc::new(RwLock::new(RoutingMetrics::default())),
            error_metrics: Arc::new(RwLock::new(ErrorMetrics::default())),
        }
    }

    /// Record a capability discovery operation
    ///
    /// # Parameters
    /// - `capabilities`: List of capabilities being discovered
    /// - `duration`: Time taken for discovery
    /// - `services_found`: Number of services that matched
    /// - `success`: Whether the discovery was successful
    pub async fn record_discovery(
        &self,
        capabilities: &[String],
        duration: Duration,
        services_found: usize,
        success: bool,
    ) {
        let mut metrics = self.discovery_metrics.write().await;

        metrics.total_discovery_requests += 1;

        // Update average discovery time
        let duration_ms = duration.as_millis() as f64;
        metrics.avg_discovery_time_ms = (metrics.avg_discovery_time_ms
            * (metrics.total_discovery_requests - 1) as f64
            + duration_ms)
            / metrics.total_discovery_requests as f64;

        // Update histogram
        let time_bucket = get_time_bucket(duration_ms);
        *metrics
            .discovery_time_histogram
            .entry(time_bucket)
            .or_insert(0) += 1;

        // Update average services found
        metrics.avg_services_found = (metrics.avg_services_found
            * (metrics.total_discovery_requests - 1) as f64
            + services_found as f64)
            / metrics.total_discovery_requests as f64;

        // Update top capabilities
        for capability in capabilities {
            *metrics
                .top_capabilities
                .entry(capability.clone())
                .or_insert(0) += 1;
        }

        // Update success rate
        let successful_requests = if success {
            (metrics.discovery_success_rate / 100.0 * (metrics.total_discovery_requests - 1) as f64)
                + 1.0
        } else {
            metrics.discovery_success_rate / 100.0 * (metrics.total_discovery_requests - 1) as f64
        };
        metrics.discovery_success_rate =
            (successful_requests / metrics.total_discovery_requests as f64) * 100.0;

        metrics.last_updated = Utc::now();
    }

    /// Record a service selection operation
    ///
    /// # Parameters
    /// - `capability`: Capability being selected for
    /// - `service_id`: ID of the selected service
    /// - `score`: Selection score (0.0 to 1.0)
    /// - `duration`: Time taken for selection
    /// - `context`: Selection context (e.g., "high_security", "user_type:admin")
    pub async fn record_selection(
        &self,
        capability: &str,
        service_id: &str,
        score: f64,
        duration: Duration,
        context: &str,
    ) {
        let mut metrics = self.selection_metrics.write().await;

        metrics.total_selections += 1;

        // Update average selection time
        let duration_ms = duration.as_millis() as f64;
        metrics.avg_selection_time_ms =
            (metrics.avg_selection_time_ms * (metrics.total_selections - 1) as f64 + duration_ms)
                / metrics.total_selections as f64;

        // Update selections by capability
        *metrics
            .selections_by_capability
            .entry(capability.to_string())
            .or_insert(0) += 1;

        // Update selections by service
        *metrics
            .selections_by_service
            .entry(service_id.to_string())
            .or_insert(0) += 1;

        // Update score distribution
        let score_bucket = get_score_bucket(score);
        *metrics.score_distribution.entry(score_bucket).or_insert(0) += 1;

        // Update context-based selections
        *metrics
            .selections_by_context
            .entry(context.to_string())
            .or_insert(0) += 1;

        metrics.last_updated = Utc::now();
    }

    /// Record cache operation
    ///
    /// # Parameters
    /// - `hit`: Whether this was a cache hit (true) or miss (false)
    /// - `lookup_time`: Time taken for cache lookup
    /// - `cache_size`: Current number of entries in cache
    /// - `max_size`: Maximum cache capacity
    pub async fn record_cache_operation(
        &self,
        hit: bool,
        lookup_time: Duration,
        cache_size: usize,
        max_size: usize,
    ) {
        let mut metrics = self.cache_metrics.write().await;

        if hit {
            metrics.total_hits += 1;
            // Estimate time saved (assuming cache lookup is ~10x faster than discovery)
            metrics.time_saved_ms += 50.0; // Estimated average savings
        } else {
            metrics.total_misses += 1;
        }

        // Update hit rate
        let total_operations = metrics.total_hits + metrics.total_misses;
        if total_operations > 0 {
            metrics.hit_rate = (metrics.total_hits as f64 / total_operations as f64) * 100.0;
        }

        // Update average lookup time
        let lookup_time_us = lookup_time.as_micros() as f64;
        metrics.avg_lookup_time_us = (metrics.avg_lookup_time_us * (total_operations - 1) as f64
            + lookup_time_us)
            / total_operations as f64;

        // Update utilization
        if max_size > 0 {
            metrics.utilization_percentage = (cache_size as f64 / max_size as f64) * 100.0;
        }

        metrics.last_updated = Utc::now();
    }

    /// Record a request routing operation
    ///
    /// # Parameters
    /// - `operation`: Type of operation being routed (e.g., "authentication", "storage")
    /// - `duration`: Time taken for routing
    /// - `success`: Whether routing was successful
    /// - `used_fallback`: Whether a fallback service was used
    /// - `fallback_type`: Type of fallback (if used)
    pub async fn record_routing(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
        used_fallback: bool,
        fallback_type: Option<&str>,
    ) {
        let mut metrics = self.routing_metrics.write().await;

        metrics.total_routed_requests += 1;

        // Update average routing time
        let duration_ms = duration.as_millis() as f64;
        metrics.avg_routing_time_ms = (metrics.avg_routing_time_ms
            * (metrics.total_routed_requests - 1) as f64
            + duration_ms)
            / metrics.total_routed_requests as f64;

        // Update operation distribution
        *metrics
            .requests_by_operation
            .entry(operation.to_string())
            .or_insert(0) += 1;

        // Update fallback usage
        if used_fallback {
            if let Some(fallback) = fallback_type {
                *metrics
                    .fallback_usage
                    .entry(fallback.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Update success rate
        let successful_requests = if success {
            (metrics.routing_success_rate / 100.0 * (metrics.total_routed_requests - 1) as f64)
                + 1.0
        } else {
            metrics.routing_success_rate / 100.0 * (metrics.total_routed_requests - 1) as f64
        };
        metrics.routing_success_rate =
            (successful_requests / metrics.total_routed_requests as f64) * 100.0;

        // Update latency distribution
        let latency_bucket = get_time_bucket(duration_ms);
        *metrics
            .latency_distribution
            .entry(latency_bucket)
            .or_insert(0) += 1;

        metrics.last_updated = Utc::now();
    }

    /// Record an error event
    ///
    /// # Parameters
    /// - `category`: Error category (e.g., "timeout", "connection_failed")
    /// - `message`: Error message
    /// - `service_id`: Service that caused the error (if applicable)
    /// - `capability`: Capability being accessed when error occurred
    /// - `recovery_attempted`: Whether error recovery was attempted
    /// - `recovery_successful`: Whether recovery was successful
    pub async fn record_error(
        &self,
        category: &str,
        message: &str,
        service_id: Option<&str>,
        capability: Option<&str>,
        recovery_attempted: bool,
        recovery_successful: bool,
    ) {
        let mut metrics = self.error_metrics.write().await;

        metrics.total_errors += 1;

        // Update errors by category
        *metrics
            .errors_by_category
            .entry(category.to_string())
            .or_insert(0) += 1;

        // Update errors by service
        if let Some(service) = service_id {
            *metrics
                .errors_by_service
                .entry(service.to_string())
                .or_insert(0) += 1;
        }

        // Add to recent errors (keep last 100)
        let error_event = ErrorEvent {
            timestamp: Utc::now(),
            category: category.to_string(),
            message: message.to_string(),
            service_id: service_id.map(|s| s.to_string()),
            capability: capability.map(|c| c.to_string()),
            recovery_attempted,
            recovery_successful,
        };

        metrics.recent_errors.push(error_event);
        if metrics.recent_errors.len() > 100 {
            metrics.recent_errors.remove(0);
        }

        // Update recovery success rate
        if recovery_attempted {
            let recovery_attempts = metrics
                .recent_errors
                .iter()
                .filter(|e| e.recovery_attempted)
                .count() as f64;
            let successful_recoveries = metrics
                .recent_errors
                .iter()
                .filter(|e| e.recovery_attempted && e.recovery_successful)
                .count() as f64;

            metrics.recovery_success_rate = if recovery_attempts > 0.0 {
                (successful_recoveries / recovery_attempts) * 100.0
            } else {
                100.0
            };
        }

        // Update error rate (errors per total operations)
        // This is a simplified calculation - you may want to customize based on your needs
        let total_operations = self.discovery_metrics.read().await.total_discovery_requests
            + self.routing_metrics.read().await.total_routed_requests;

        if total_operations > 0 {
            metrics.error_rate = (metrics.total_errors as f64 / total_operations as f64) * 100.0;
        }

        metrics.last_updated = Utc::now();
    }

    /// Get comprehensive metrics summary
    ///
    /// Returns a snapshot of all metrics with calculated health scores.
    pub async fn get_summary(&self) -> MetricsSummary {
        let discovery = self.discovery_metrics.read().await.clone();
        let selection = self.selection_metrics.read().await.clone();
        let cache = self.cache_metrics.read().await.clone();
        let routing = self.routing_metrics.read().await.clone();
        let errors = self.error_metrics.read().await.clone();

        // Calculate health scores
        let health_score = calculate_health_score(&discovery, &routing, &errors);
        let performance_score = calculate_performance_score(&discovery, &cache, &routing);
        let reliability_score = calculate_reliability_score(&routing, &errors);

        MetricsSummary {
            health_score,
            performance_score,
            reliability_score,
            discovery,
            selection,
            cache,
            routing,
            errors,
            generated_at: Utc::now(),
        }
    }

    /// Reset all metrics to initial state
    ///
    /// Useful for testing or periodic metric rotation.
    pub async fn reset(&self) {
        *self.discovery_metrics.write().await = DiscoveryMetrics::default();
        *self.selection_metrics.write().await = SelectionMetrics::default();
        *self.cache_metrics.write().await = CacheMetrics::default();
        *self.routing_metrics.write().await = RoutingMetrics::default();
        *self.error_metrics.write().await = ErrorMetrics::default();
    }
}

impl Default for CapabilityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_metrics_new() {
        let metrics = CapabilityMetrics::new();
        let summary = metrics.get_summary().await;

        assert_eq!(summary.discovery.total_discovery_requests, 0);
        assert_eq!(summary.selection.total_selections, 0);
        assert_eq!(summary.cache.total_hits, 0);
        assert_eq!(summary.routing.total_routed_requests, 0);
        assert_eq!(summary.errors.total_errors, 0);
    }

    #[tokio::test]
    async fn test_record_discovery_success() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_discovery(
                &["authentication".to_string()],
                Duration::from_millis(15),
                3,
                true,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 1);
        assert_eq!(summary.discovery.avg_discovery_time_ms, 15.0);
        assert_eq!(summary.discovery.avg_services_found, 3.0);
        assert_eq!(summary.discovery.discovery_success_rate, 100.0);
    }

    #[tokio::test]
    async fn test_record_selection() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_selection(
                "authentication",
                "beardog-auth",
                0.95,
                Duration::from_millis(5),
                "high_security",
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.selection.total_selections, 1);
        assert_eq!(summary.selection.avg_selection_time_ms, 5.0);
        assert_eq!(
            summary
                .selection
                .selections_by_capability
                .get("authentication"),
            Some(&1)
        );
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let metrics = CapabilityMetrics::new();

        // Record 7 hits and 3 misses
        for _ in 0..7 {
            metrics
                .record_cache_operation(true, Duration::from_micros(50), 100, 1000)
                .await;
        }
        for _ in 0..3 {
            metrics
                .record_cache_operation(false, Duration::from_micros(100), 100, 1000)
                .await;
        }

        let summary = metrics.get_summary().await;
        assert_eq!(summary.cache.total_hits, 7);
        assert_eq!(summary.cache.total_misses, 3);
        assert!((summary.cache.hit_rate - 70.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_routing_success_rate() {
        let metrics = CapabilityMetrics::new();

        // 9 successful, 1 failed
        for _ in 0..9 {
            metrics
                .record_routing(
                    "authentication",
                    Duration::from_millis(10),
                    true,
                    false,
                    None,
                )
                .await;
        }
        metrics
            .record_routing(
                "authentication",
                Duration::from_millis(10),
                false,
                false,
                None,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.routing.total_routed_requests, 10);
        assert!((summary.routing.routing_success_rate - 90.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_error(
                "timeout",
                "Connection timed out",
                Some("auth-service"),
                Some("authentication"),
                true,
                true,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.errors.total_errors, 1);
        assert_eq!(summary.errors.recent_errors.len(), 1);
        assert_eq!(summary.errors.recovery_success_rate, 100.0);
    }

    #[tokio::test]
    async fn test_reset() {
        let metrics = CapabilityMetrics::new();

        // Record some metrics
        metrics
            .record_discovery(&["auth".to_string()], Duration::from_millis(10), 2, true)
            .await;
        metrics
            .record_selection("auth", "service1", 0.9, Duration::from_millis(5), "test")
            .await;

        // Reset
        metrics.reset().await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 0);
        assert_eq!(summary.selection.total_selections, 0);
    }
}
