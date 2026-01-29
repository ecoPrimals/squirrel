//! Comprehensive metrics for capability-based service discovery and selection
//!
//! This module provides detailed metrics and observability for the performance
//! and behavior of the capability-based ecosystem integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Comprehensive metrics collector for capability-based operations
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

/// Discovery performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetrics {
    /// Total number of capability discovery requests
    pub total_discovery_requests: u64,
    /// Average discovery time in milliseconds
    pub avg_discovery_time_ms: f64,
    /// Distribution of discovery times
    pub discovery_time_histogram: HashMap<String, u64>, // e.g., "0-10ms", "10-50ms", etc.
    /// Number of services found per request (average)
    pub avg_services_found: f64,
    /// Most requested capabilities
    pub top_capabilities: HashMap<String, u64>,
    /// Discovery success rate
    pub discovery_success_rate: f64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Service selection metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionMetrics {
    /// Total number of service selection operations
    pub total_selections: u64,
    /// Average selection time in milliseconds
    pub avg_selection_time_ms: f64,
    /// Service selection distribution by capability
    pub selections_by_capability: HashMap<String, u64>,
    /// Service selection distribution by service ID
    pub selections_by_service: HashMap<String, u64>,
    /// Score distribution for selected services
    pub score_distribution: HashMap<String, u64>, // e.g., "0.9-1.0", "0.8-0.9", etc.
    /// Context-based selection patterns
    pub selections_by_context: HashMap<String, u64>, // security level, user type, etc.
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Cache hit rate (percentage)
    pub hit_rate: f64,
    /// Total cache hits
    pub total_hits: u64,
    /// Total cache misses
    pub total_misses: u64,
    /// Average cache lookup time in microseconds
    pub avg_lookup_time_us: f64,
    /// Cache eviction count
    pub eviction_count: u64,
    /// Cache size utilization (percentage)
    pub utilization_percentage: f64,
    /// Time saved by caching (milliseconds)
    pub time_saved_ms: f64,
    /// Cache entries by TTL bucket
    pub entries_by_ttl: HashMap<String, u64>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Request routing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Total requests routed
    pub total_routed_requests: u64,
    /// Routing success rate
    pub routing_success_rate: f64,
    /// Average routing time in milliseconds
    pub avg_routing_time_ms: f64,
    /// Requests by operation type
    pub requests_by_operation: HashMap<String, u64>,
    /// Fallback usage statistics
    pub fallback_usage: HashMap<String, u64>, // e.g., "local_security", "local_storage"
    /// Network latency distribution
    pub latency_distribution: HashMap<String, u64>,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Error tracking metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors encountered
    pub total_errors: u64,
    /// Error rate (percentage)
    pub error_rate: f64,
    /// Errors by category
    pub errors_by_category: HashMap<String, u64>,
    /// Errors by service
    pub errors_by_service: HashMap<String, u64>,
    /// Recent error patterns
    pub recent_errors: Vec<ErrorEvent>,
    /// Error recovery success rate
    pub recovery_success_rate: f64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Individual error event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Error category
    pub category: String,
    /// Error message
    pub message: String,
    /// Service involved (if applicable)
    pub service_id: Option<String>,
    /// Capability being accessed
    pub capability: Option<String>,
    /// Whether recovery was attempted
    pub recovery_attempted: bool,
    /// Whether recovery was successful
    pub recovery_successful: bool,
}

/// Comprehensive metrics summary for monitoring dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Overall system health score (0.0 to 1.0)
    pub health_score: f64,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Discovery metrics summary
    pub discovery: DiscoveryMetrics,
    /// Selection metrics summary
    pub selection: SelectionMetrics,
    /// Cache metrics summary
    pub cache: CacheMetrics,
    /// Routing metrics summary
    pub routing: RoutingMetrics,
    /// Error metrics summary
    pub errors: ErrorMetrics,
    /// Timestamp of this summary
    pub generated_at: DateTime<Utc>,
}

impl Default for DiscoveryMetrics {
    fn default() -> Self {
        Self {
            total_discovery_requests: 0,
            avg_discovery_time_ms: 0.0,
            discovery_time_histogram: HashMap::new(),
            avg_services_found: 0.0,
            top_capabilities: HashMap::new(),
            discovery_success_rate: 100.0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for SelectionMetrics {
    fn default() -> Self {
        Self {
            total_selections: 0,
            avg_selection_time_ms: 0.0,
            selections_by_capability: HashMap::new(),
            selections_by_service: HashMap::new(),
            score_distribution: HashMap::new(),
            selections_by_context: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            hit_rate: 0.0,
            total_hits: 0,
            total_misses: 0,
            avg_lookup_time_us: 0.0,
            eviction_count: 0,
            utilization_percentage: 0.0,
            time_saved_ms: 0.0,
            entries_by_ttl: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for RoutingMetrics {
    fn default() -> Self {
        Self {
            total_routed_requests: 0,
            routing_success_rate: 100.0,
            avg_routing_time_ms: 0.0,
            requests_by_operation: HashMap::new(),
            fallback_usage: HashMap::new(),
            latency_distribution: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            error_rate: 0.0,
            errors_by_category: HashMap::new(),
            errors_by_service: HashMap::new(),
            recent_errors: Vec::new(),
            recovery_success_rate: 100.0,
            last_updated: Utc::now(),
        }
    }
}

impl CapabilityMetrics {
    /// Create new metrics collector
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
        let time_bucket = Self::get_time_bucket(duration_ms);
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
        let score_bucket = Self::get_score_bucket(score);
        *metrics.score_distribution.entry(score_bucket).or_insert(0) += 1;

        // Update context-based selections
        *metrics
            .selections_by_context
            .entry(context.to_string())
            .or_insert(0) += 1;

        metrics.last_updated = Utc::now();
    }

    /// Record cache operation
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
            // Estimate time saved (assuming cache lookup is 10x faster than discovery)
            metrics.time_saved_ms += 50.0; // Estimated savings
        } else {
            metrics.total_misses += 1;
        }

        // Update hit rate
        let total_operations = metrics.total_hits + metrics.total_misses;
        metrics.hit_rate = (metrics.total_hits as f64 / total_operations as f64) * 100.0;

        // Update average lookup time
        let lookup_time_us = lookup_time.as_micros() as f64;
        metrics.avg_lookup_time_us = (metrics.avg_lookup_time_us * (total_operations - 1) as f64
            + lookup_time_us)
            / total_operations as f64;

        // Update utilization
        metrics.utilization_percentage = (cache_size as f64 / max_size as f64) * 100.0;

        metrics.last_updated = Utc::now();
    }

    /// Record a request routing operation
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
        let latency_bucket = Self::get_time_bucket(duration_ms);
        *metrics
            .latency_distribution
            .entry(latency_bucket)
            .or_insert(0) += 1;

        metrics.last_updated = Utc::now();
    }

    /// Record an error event
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

        metrics.last_updated = Utc::now();
    }

    /// Get comprehensive metrics summary
    pub async fn get_summary(&self) -> MetricsSummary {
        let discovery = self.discovery_metrics.read().await.clone();
        let selection = self.selection_metrics.read().await.clone();
        let cache = self.cache_metrics.read().await.clone();
        let routing = self.routing_metrics.read().await.clone();
        let errors = self.error_metrics.read().await.clone();

        // Calculate health scores
        let health_score = Self::calculate_health_score(&discovery, &routing, &errors);
        let performance_score = Self::calculate_performance_score(&discovery, &cache, &routing);
        let reliability_score = Self::calculate_reliability_score(&routing, &errors);

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

    /// Calculate overall health score (0.0 to 1.0)
    fn calculate_health_score(
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
    fn calculate_performance_score(
        discovery: &DiscoveryMetrics,
        cache: &CacheMetrics,
        routing: &RoutingMetrics,
    ) -> f64 {
        // Good performance: discovery < 100ms, cache hit rate > 80%, routing < 50ms
        let discovery_score = if discovery.avg_discovery_time_ms < 100.0 {
            1.0
        } else {
            100.0 / discovery.avg_discovery_time_ms
        };
        let cache_score = cache.hit_rate / 100.0;
        let routing_score = if routing.avg_routing_time_ms < 50.0 {
            1.0
        } else {
            50.0 / routing.avg_routing_time_ms
        };

        (discovery_score + cache_score + routing_score) / 3.0
    }

    /// Calculate reliability score (0.0 to 1.0)
    fn calculate_reliability_score(routing: &RoutingMetrics, errors: &ErrorMetrics) -> f64 {
        let routing_reliability = routing.routing_success_rate / 100.0;
        let error_reliability = if errors.total_errors == 0 {
            1.0
        } else {
            errors.recovery_success_rate / 100.0
        };

        (routing_reliability + error_reliability) / 2.0
    }

    /// Get time bucket for histogram
    fn get_time_bucket(time_ms: f64) -> String {
        match time_ms {
            t if t < 10.0 => "0-10ms".to_string(),
            t if t < 50.0 => "10-50ms".to_string(),
            t if t < 100.0 => "50-100ms".to_string(),
            t if t < 500.0 => "100-500ms".to_string(),
            t if t < 1000.0 => "500ms-1s".to_string(),
            _ => "1s+".to_string(),
        }
    }

    /// Get score bucket for distribution
    fn get_score_bucket(score: f64) -> String {
        match score {
            s if s >= 0.9 => "0.9-1.0".to_string(),
            s if s >= 0.8 => "0.8-0.9".to_string(),
            s if s >= 0.7 => "0.7-0.8".to_string(),
            s if s >= 0.6 => "0.6-0.7".to_string(),
            s if s >= 0.5 => "0.5-0.6".to_string(),
            _ => "0.0-0.5".to_string(),
        }
    }

    /// Reset all metrics (useful for testing)
    pub async fn reset(&self) {
        *self.discovery_metrics.write().await = DiscoveryMetrics::default();
        *self.selection_metrics.write().await = SelectionMetrics::default();
        *self.cache_metrics.write().await = CacheMetrics::default();
        *self.routing_metrics.write().await = RoutingMetrics::default();
        *self.error_metrics.write().await = ErrorMetrics::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
        let capabilities = vec!["ai.complete".to_string(), "ai.chat".to_string()];
        let duration = Duration::from_millis(50);

        metrics
            .record_discovery(&capabilities, duration, 3, true)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 1);
        assert_eq!(summary.discovery.avg_discovery_time_ms, 50.0);
        assert_eq!(summary.discovery.avg_services_found, 3.0);
        assert_eq!(summary.discovery.discovery_success_rate, 100.0);
        assert_eq!(
            *summary
                .discovery
                .top_capabilities
                .get("ai.complete")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary.discovery.top_capabilities.get("ai.chat").unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_record_discovery_multiple() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_discovery(
                &vec!["ai.complete".to_string()],
                Duration::from_millis(100),
                2,
                true,
            )
            .await;
        metrics
            .record_discovery(
                &vec!["ai.complete".to_string()],
                Duration::from_millis(200),
                4,
                true,
            )
            .await;
        metrics
            .record_discovery(
                &vec!["ai.chat".to_string()],
                Duration::from_millis(150),
                3,
                false,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 3);
        assert_eq!(summary.discovery.avg_discovery_time_ms, 150.0); // (100 + 200 + 150) / 3
        assert_eq!(summary.discovery.avg_services_found, 3.0); // (2 + 4 + 3) / 3
        assert!((summary.discovery.discovery_success_rate - 66.67).abs() < 0.1); // 2/3 success
        assert_eq!(
            *summary
                .discovery
                .top_capabilities
                .get("ai.complete")
                .unwrap(),
            2
        );
        assert_eq!(
            *summary.discovery.top_capabilities.get("ai.chat").unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_discovery_time_histogram() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_discovery(&vec!["test".to_string()], Duration::from_millis(5), 1, true)
            .await;
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(75),
                1,
                true,
            )
            .await;
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(150),
                1,
                true,
            )
            .await;
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(600),
                1,
                true,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(
            *summary
                .discovery
                .discovery_time_histogram
                .get("0-10ms")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary
                .discovery
                .discovery_time_histogram
                .get("50-100ms")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary
                .discovery
                .discovery_time_histogram
                .get("100-500ms")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary
                .discovery
                .discovery_time_histogram
                .get("500ms-1s")
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_record_selection() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_selection(
                "ai.complete",
                "provider-1",
                0.95,
                Duration::from_millis(10),
                "production",
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.selection.total_selections, 1);
        assert_eq!(summary.selection.avg_selection_time_ms, 10.0);
        assert_eq!(
            *summary
                .selection
                .selections_by_capability
                .get("ai.complete")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary
                .selection
                .selections_by_service
                .get("provider-1")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary.selection.score_distribution.get("0.9-1.0").unwrap(),
            1
        );
        assert_eq!(
            *summary
                .selection
                .selections_by_context
                .get("production")
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_record_selection_multiple() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_selection(
                "ai.complete",
                "provider-1",
                0.95,
                Duration::from_millis(10),
                "production",
            )
            .await;
        metrics
            .record_selection(
                "ai.complete",
                "provider-2",
                0.85,
                Duration::from_millis(15),
                "staging",
            )
            .await;
        metrics
            .record_selection(
                "http.request",
                "provider-1",
                0.75,
                Duration::from_millis(20),
                "production",
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.selection.total_selections, 3);
        assert_eq!(summary.selection.avg_selection_time_ms, 15.0); // (10 + 15 + 20) / 3
        assert_eq!(
            *summary
                .selection
                .selections_by_capability
                .get("ai.complete")
                .unwrap(),
            2
        );
        assert_eq!(
            *summary
                .selection
                .selections_by_capability
                .get("http.request")
                .unwrap(),
            1
        );
        assert_eq!(
            *summary
                .selection
                .selections_by_service
                .get("provider-1")
                .unwrap(),
            2
        );
        assert_eq!(
            *summary
                .selection
                .selections_by_service
                .get("provider-2")
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_score_bucket_distribution() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_selection("test", "p1", 0.95, Duration::from_millis(10), "ctx")
            .await;
        metrics
            .record_selection("test", "p2", 0.85, Duration::from_millis(10), "ctx")
            .await;
        metrics
            .record_selection("test", "p3", 0.75, Duration::from_millis(10), "ctx")
            .await;
        metrics
            .record_selection("test", "p4", 0.55, Duration::from_millis(10), "ctx")
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(
            *summary.selection.score_distribution.get("0.9-1.0").unwrap(),
            1
        );
        assert_eq!(
            *summary.selection.score_distribution.get("0.8-0.9").unwrap(),
            1
        );
        assert_eq!(
            *summary.selection.score_distribution.get("0.7-0.8").unwrap(),
            1
        );
        assert_eq!(
            *summary.selection.score_distribution.get("0.5-0.6").unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_record_cache_hit() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_cache_operation(true, Duration::from_micros(100), 50, 100)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.cache.total_hits, 1);
        assert_eq!(summary.cache.total_misses, 0);
        assert_eq!(summary.cache.hit_rate, 100.0);
        assert_eq!(summary.cache.avg_lookup_time_us, 100.0);
        assert_eq!(summary.cache.utilization_percentage, 50.0);
        assert_eq!(summary.cache.time_saved_ms, 50.0);
    }

    #[tokio::test]
    async fn test_record_cache_miss() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_cache_operation(false, Duration::from_micros(200), 50, 100)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.cache.total_hits, 0);
        assert_eq!(summary.cache.total_misses, 1);
        assert_eq!(summary.cache.hit_rate, 0.0);
        assert_eq!(summary.cache.avg_lookup_time_us, 200.0);
        assert_eq!(summary.cache.time_saved_ms, 0.0);
    }

    #[tokio::test]
    async fn test_cache_hit_rate_calculation() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_cache_operation(true, Duration::from_micros(100), 50, 100)
            .await;
        metrics
            .record_cache_operation(true, Duration::from_micros(100), 50, 100)
            .await;
        metrics
            .record_cache_operation(true, Duration::from_micros(100), 50, 100)
            .await;
        metrics
            .record_cache_operation(false, Duration::from_micros(100), 50, 100)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.cache.total_hits, 3);
        assert_eq!(summary.cache.total_misses, 1);
        assert_eq!(summary.cache.hit_rate, 75.0); // 3/4
        assert_eq!(summary.cache.time_saved_ms, 150.0); // 3 hits * 50ms
    }

    #[tokio::test]
    async fn test_record_routing_success() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_routing("query", Duration::from_millis(25), true, false, None)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.routing.total_routed_requests, 1);
        assert_eq!(summary.routing.avg_routing_time_ms, 25.0);
        assert_eq!(summary.routing.routing_success_rate, 100.0);
        assert_eq!(
            *summary.routing.requests_by_operation.get("query").unwrap(),
            1
        );
        assert!(summary.routing.fallback_usage.is_empty());
    }

    #[tokio::test]
    async fn test_record_routing_with_fallback() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_routing(
                "query",
                Duration::from_millis(25),
                true,
                true,
                Some("local_security"),
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.routing.total_routed_requests, 1);
        assert_eq!(
            *summary
                .routing
                .fallback_usage
                .get("local_security")
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_routing_success_rate() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_routing("query", Duration::from_millis(25), true, false, None)
            .await;
        metrics
            .record_routing("query", Duration::from_millis(30), true, false, None)
            .await;
        metrics
            .record_routing("query", Duration::from_millis(35), false, false, None)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.routing.total_routed_requests, 3);
        assert!((summary.routing.routing_success_rate - 66.67).abs() < 0.1); // 2/3 success
        assert_eq!(summary.routing.avg_routing_time_ms, 30.0); // (25 + 30 + 35) / 3
    }

    #[tokio::test]
    async fn test_record_error() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_error(
                "network",
                "Connection timeout",
                Some("service-1"),
                Some("ai.complete"),
                true,
                false,
            )
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.errors.total_errors, 1);
        assert_eq!(
            *summary.errors.errors_by_category.get("network").unwrap(),
            1
        );
        assert_eq!(
            *summary.errors.errors_by_service.get("service-1").unwrap(),
            1
        );
        assert_eq!(summary.errors.recent_errors.len(), 1);

        let error = &summary.errors.recent_errors[0];
        assert_eq!(error.category, "network");
        assert_eq!(error.message, "Connection timeout");
        assert_eq!(error.service_id.as_ref().unwrap(), "service-1");
        assert_eq!(error.capability.as_ref().unwrap(), "ai.complete");
        assert!(error.recovery_attempted);
        assert!(!error.recovery_successful);
    }

    #[tokio::test]
    async fn test_error_recent_errors_limit() {
        let metrics = CapabilityMetrics::new();

        // Add 105 errors
        for i in 0..105 {
            metrics
                .record_error("test", &format!("Error {}", i), None, None, false, false)
                .await;
        }

        let summary = metrics.get_summary().await;
        assert_eq!(summary.errors.total_errors, 105);
        assert_eq!(summary.errors.recent_errors.len(), 100); // Capped at 100
    }

    #[tokio::test]
    async fn test_error_recovery_success_rate() {
        let metrics = CapabilityMetrics::new();

        metrics
            .record_error("test", "Error 1", None, None, true, true)
            .await;
        metrics
            .record_error("test", "Error 2", None, None, true, false)
            .await;
        metrics
            .record_error("test", "Error 3", None, None, true, true)
            .await;
        metrics
            .record_error("test", "Error 4", None, None, true, true)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.errors.total_errors, 4);
        assert_eq!(summary.errors.recovery_success_rate, 75.0); // 3/4 successful recoveries
    }

    #[tokio::test]
    async fn test_health_score_calculation() {
        let metrics = CapabilityMetrics::new();

        // Perfect scenario
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(50),
                1,
                true,
            )
            .await;
        metrics
            .record_routing("test", Duration::from_millis(25), true, false, None)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.health_score, 1.0); // Perfect health
    }

    #[tokio::test]
    async fn test_performance_score_calculation() {
        let metrics = CapabilityMetrics::new();

        // Good performance scenario
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(50),
                1,
                true,
            )
            .await;
        metrics
            .record_cache_operation(true, Duration::from_micros(100), 80, 100)
            .await;
        metrics
            .record_routing("test", Duration::from_millis(25), true, false, None)
            .await;

        let summary = metrics.get_summary().await;
        assert!(summary.performance_score > 0.8); // Good performance
    }

    #[tokio::test]
    async fn test_reliability_score_calculation() {
        let metrics = CapabilityMetrics::new();

        // Good reliability scenario
        metrics
            .record_routing("test", Duration::from_millis(25), true, false, None)
            .await;
        metrics
            .record_routing("test", Duration::from_millis(30), true, false, None)
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.reliability_score, 1.0); // Perfect reliability
    }

    #[tokio::test]
    async fn test_get_time_bucket() {
        assert_eq!(CapabilityMetrics::get_time_bucket(5.0), "0-10ms");
        assert_eq!(CapabilityMetrics::get_time_bucket(25.0), "10-50ms");
        assert_eq!(CapabilityMetrics::get_time_bucket(75.0), "50-100ms");
        assert_eq!(CapabilityMetrics::get_time_bucket(200.0), "100-500ms");
        assert_eq!(CapabilityMetrics::get_time_bucket(750.0), "500ms-1s");
        assert_eq!(CapabilityMetrics::get_time_bucket(1500.0), "1s+");
    }

    #[tokio::test]
    async fn test_get_score_bucket() {
        assert_eq!(CapabilityMetrics::get_score_bucket(0.95), "0.9-1.0");
        assert_eq!(CapabilityMetrics::get_score_bucket(0.85), "0.8-0.9");
        assert_eq!(CapabilityMetrics::get_score_bucket(0.75), "0.7-0.8");
        assert_eq!(CapabilityMetrics::get_score_bucket(0.65), "0.6-0.7");
        assert_eq!(CapabilityMetrics::get_score_bucket(0.55), "0.5-0.6");
        assert_eq!(CapabilityMetrics::get_score_bucket(0.25), "0.0-0.5");
    }

    #[tokio::test]
    async fn test_reset() {
        let metrics = CapabilityMetrics::new();

        // Add some metrics
        metrics
            .record_discovery(
                &vec!["test".to_string()],
                Duration::from_millis(50),
                1,
                true,
            )
            .await;
        metrics
            .record_selection("test", "service-1", 0.95, Duration::from_millis(10), "ctx")
            .await;
        metrics
            .record_cache_operation(true, Duration::from_micros(100), 50, 100)
            .await;
        metrics
            .record_routing("test", Duration::from_millis(25), true, false, None)
            .await;
        metrics
            .record_error("test", "Error", None, None, false, false)
            .await;

        // Reset
        metrics.reset().await;

        // Verify all metrics are reset
        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 0);
        assert_eq!(summary.selection.total_selections, 0);
        assert_eq!(summary.cache.total_hits, 0);
        assert_eq!(summary.routing.total_routed_requests, 0);
        assert_eq!(summary.errors.total_errors, 0);
    }

    #[tokio::test]
    async fn test_default_metrics() {
        let discovery = DiscoveryMetrics::default();
        assert_eq!(discovery.total_discovery_requests, 0);
        assert_eq!(discovery.discovery_success_rate, 100.0);

        let selection = SelectionMetrics::default();
        assert_eq!(selection.total_selections, 0);

        let cache = CacheMetrics::default();
        assert_eq!(cache.hit_rate, 0.0);

        let routing = RoutingMetrics::default();
        assert_eq!(routing.routing_success_rate, 100.0);

        let errors = ErrorMetrics::default();
        assert_eq!(errors.error_rate, 0.0);
        assert_eq!(errors.recovery_success_rate, 100.0);
    }

    #[tokio::test]
    async fn test_concurrent_metric_updates() {
        let metrics = Arc::new(CapabilityMetrics::new());
        let mut handles = vec![];

        // Spawn 10 concurrent tasks
        for i in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = tokio::spawn(async move {
                metrics_clone
                    .record_discovery(
                        &vec![format!("test-{}", i)],
                        Duration::from_millis(50),
                        1,
                        true,
                    )
                    .await;
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let summary = metrics.get_summary().await;
        assert_eq!(summary.discovery.total_discovery_requests, 10);
    }
}
