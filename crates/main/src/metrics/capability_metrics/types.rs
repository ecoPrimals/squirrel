// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core metric types for capability-based operations
//!
//! This module defines the fundamental metric structures used to track
//! discovery, selection, caching, routing, and error behavior.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Discovery performance metrics
///
/// Tracks the performance and behavior of capability discovery operations,
/// including timing, success rates, and usage patterns.
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

    /// Discovery success rate (percentage)
    pub discovery_success_rate: f64,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
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

/// Service selection metrics
///
/// Tracks service selection operations, including timing, patterns,
/// and scoring distributions.
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

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
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

/// Cache performance metrics
///
/// Tracks cache hit rates, lookup times, evictions, and overall
/// cache effectiveness for capability lookups.
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

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
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

/// Request routing metrics
///
/// Tracks request routing operations, including success rates, latency,
/// and fallback usage patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Total requests routed
    pub total_routed_requests: u64,

    /// Routing success rate (percentage)
    pub routing_success_rate: f64,

    /// Average routing time in milliseconds
    pub avg_routing_time_ms: f64,

    /// Requests by operation type
    pub requests_by_operation: HashMap<String, u64>,

    /// Fallback usage statistics
    pub fallback_usage: HashMap<String, u64>, // e.g., "local_security", "local_storage"

    /// Network latency distribution
    pub latency_distribution: HashMap<String, u64>,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
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

/// Error tracking metrics
///
/// Tracks errors encountered during capability operations, including
/// categorization, recovery attempts, and recent error patterns.
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

    /// Recent error patterns (last 100)
    pub recent_errors: Vec<ErrorEvent>,

    /// Error recovery success rate (percentage)
    pub recovery_success_rate: f64,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
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

/// Individual error event for tracking
///
/// Represents a single error occurrence with context about the error,
/// service, capability, and recovery attempt.
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

impl ErrorEvent {
    /// Create a new error event
    pub fn new(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            category: category.into(),
            message: message.into(),
            service_id: None,
            capability: None,
            recovery_attempted: false,
            recovery_successful: false,
        }
    }

    /// Set the service ID
    pub fn with_service(mut self, service_id: impl Into<String>) -> Self {
        self.service_id = Some(service_id.into());
        self
    }

    /// Set the capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }

    /// Mark recovery as attempted
    pub fn with_recovery(mut self, successful: bool) -> Self {
        self.recovery_attempted = true;
        self.recovery_successful = successful;
        self
    }
}

/// Comprehensive metrics summary for monitoring dashboards
///
/// Aggregates all metric types with calculated health and performance scores.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_metrics_default() {
        let metrics = DiscoveryMetrics::default();
        assert_eq!(metrics.total_discovery_requests, 0);
        assert_eq!(metrics.avg_discovery_time_ms, 0.0);
        assert_eq!(metrics.discovery_success_rate, 100.0);
    }

    #[test]
    fn test_selection_metrics_default() {
        let metrics = SelectionMetrics::default();
        assert_eq!(metrics.total_selections, 0);
        assert_eq!(metrics.avg_selection_time_ms, 0.0);
    }

    #[test]
    fn test_cache_metrics_default() {
        let metrics = CacheMetrics::default();
        assert_eq!(metrics.total_hits, 0);
        assert_eq!(metrics.total_misses, 0);
        assert_eq!(metrics.hit_rate, 0.0);
    }

    #[test]
    fn test_routing_metrics_default() {
        let metrics = RoutingMetrics::default();
        assert_eq!(metrics.total_routed_requests, 0);
        assert_eq!(metrics.routing_success_rate, 100.0);
    }

    #[test]
    fn test_error_metrics_default() {
        let metrics = ErrorMetrics::default();
        assert_eq!(metrics.total_errors, 0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.recovery_success_rate, 100.0);
    }

    #[test]
    fn test_error_event_builder() {
        let event = ErrorEvent::new("timeout", "Request timed out")
            .with_service("auth-service")
            .with_capability("authentication")
            .with_recovery(true);

        assert_eq!(event.category, "timeout");
        assert_eq!(event.message, "Request timed out");
        assert_eq!(event.service_id, Some("auth-service".to_string()));
        assert_eq!(event.capability, Some("authentication".to_string()));
        assert!(event.recovery_attempted);
        assert!(event.recovery_successful);
    }
}
