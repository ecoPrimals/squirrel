// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Type definitions for Universal Primal Ecosystem
//!
//! This module contains the core data structures for service discovery,
//! capability matching, and service health tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use crate::universal::PrimalContext;

/// Discovered service information (capability-based, not name-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service identifier (not primal name)
    pub service_id: String,
    /// Instance identifier
    pub instance_id: String,
    /// Service endpoint
    pub endpoint: String,
    /// Available capabilities
    pub capabilities: Vec<String>,
    /// Service health status
    pub health: ServiceHealth,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Last health check
    pub last_health_check: Option<DateTime<Utc>>,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    /// Service is healthy and operational.
    Healthy,
    /// Service is degraded but partially functional.
    Degraded,
    /// Service is unhealthy or failing.
    Unhealthy,
    /// Health status is unknown.
    Unknown,
}

/// Universal capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Optional capabilities
    pub optional_capabilities: Vec<String>,
    /// Context for capability matching
    pub context: PrimalContext,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Capability matching result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatch {
    /// Service that can fulfill the capability
    pub service: DiscoveredService,
    /// Matching score (0.0 to 1.0)
    pub score: f64,
    /// Matched capabilities
    pub matched_capabilities: Vec<String>,
    /// Missing capabilities
    pub missing_capabilities: Vec<String>,
}

/// Statistics for service connections
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of requests made to this endpoint
    pub request_count: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last successful request timestamp
    pub last_success: Option<Instant>,
    /// Number of failures
    pub failure_count: u64,
}

/// Connection pool health metrics for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionPoolHealthMetrics {
    /// Total number of active connections in the pool
    pub total_connections: usize,
    /// Number of healthy connections
    pub healthy_connections: usize,
    /// Number of unhealthy connections
    pub unhealthy_connections: usize,
    /// Total requests processed by the pool
    pub total_requests: u64,
    /// Total failures encountered by the pool
    pub total_failures: u64,
    /// Overall failure rate (0.0 to 1.0)
    pub overall_failure_rate: f64,
    /// Average response time across all connections
    pub avg_response_time_ms: f64,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Size of the capability discovery cache
    pub discovery_cache_size: usize,
    /// Size of the service capabilities cache
    pub capabilities_cache_size: usize,
    /// Total number of cache hits across all caches
    pub total_cache_hits: u64,
    /// Number of valid entries in the capability discovery cache
    pub valid_cache_entries: usize,
    /// Number of expired entries in the capability discovery cache
    pub expired_cache_entries: usize,
    /// Connection pool statistics
    pub connection_pool_stats: HashMap<String, ConnectionStats>,
}

/// Performance-optimized cache entry for capability discovery results
#[derive(Debug, Clone)]
pub struct CachedCapabilityMatch {
    /// The cached matches
    pub matches: Vec<CapabilityMatch>,
    /// When this cache entry was created
    pub cached_at: Instant,
    /// TTL for this cache entry (in seconds)
    pub ttl_seconds: u64,
    /// Number of times this cache entry has been accessed
    pub access_count: u64,
}

impl CachedCapabilityMatch {
    /// Check if this cache entry is still valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed().as_secs() < self.ttl_seconds
    }

    /// Update access statistics
    pub const fn accessed(&mut self) {
        self.access_count += 1;
    }
}

/// Cache configuration for performance optimization
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for capability discovery cache (seconds)
    pub capability_discovery_ttl: u64,
    /// Default TTL for service capabilities cache (seconds)
    pub service_capabilities_ttl: u64,
    /// Maximum cache entries to maintain
    pub max_cache_entries: usize,
    /// Enable/disable caching
    pub enable_caching: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capability_discovery_ttl: 300, // 5 minutes
            service_capabilities_ttl: 600, // 10 minutes
            max_cache_entries: 1000,
            enable_caching: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_health_variants() {
        let variants = vec![
            ServiceHealth::Healthy,
            ServiceHealth::Degraded,
            ServiceHealth::Unhealthy,
            ServiceHealth::Unknown,
        ];

        assert_eq!(variants.len(), 4);

        // Test serialization
        for variant in variants {
            let serialized = serde_json::to_string(&variant).unwrap();
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_discovered_service_creation() {
        use chrono::Utc;

        let service = DiscoveredService {
            service_id: "svc-123".to_string(),
            instance_id: "inst-456".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["compute".to_string(), "storage".to_string()],
            health: ServiceHealth::Healthy,
            discovered_at: Utc::now(),
            last_health_check: None,
        };

        assert_eq!(service.service_id, "svc-123");
        assert_eq!(service.capabilities.len(), 2);
        assert!(matches!(service.health, ServiceHealth::Healthy));
    }

    #[test]
    fn test_capability_request_creation() {
        let request = CapabilityRequest {
            required_capabilities: vec!["auth".to_string()],
            optional_capabilities: vec!["logging".to_string()],
            context: crate::universal::PrimalContext {
                biome_id: Some("biome123".to_string()),
                user_id: "user123".to_string(),
                device_id: "device456".to_string(),
                session_id: Some("session789".to_string()),
                network_location: crate::universal::NetworkLocation {
                    region: "local".to_string(),
                    data_center: None,
                    availability_zone: None,
                    ip_address: Some("127.0.0.1".to_string()),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: crate::universal::SecurityLevel::Standard,
                metadata: std::collections::HashMap::new(),
            },
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(request.required_capabilities.len(), 1);
        assert_eq!(request.optional_capabilities.len(), 1);
    }

    #[test]
    fn test_capability_match_scoring() {
        use chrono::Utc;

        let service = DiscoveredService {
            service_id: "svc-1".to_string(),
            instance_id: "inst-1".to_string(),
            endpoint: "http://test:8080".to_string(),
            capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            health: ServiceHealth::Healthy,
            discovered_at: Utc::now(),
            last_health_check: Some(Utc::now()),
        };

        let match_result = CapabilityMatch {
            service,
            score: 0.95,
            matched_capabilities: vec!["cap1".to_string()],
            missing_capabilities: vec!["cap3".to_string()],
        };

        assert_eq!(match_result.score, 0.95);
        assert_eq!(match_result.matched_capabilities.len(), 1);
        assert_eq!(match_result.missing_capabilities.len(), 1);
    }

    #[test]
    fn test_connection_stats() {
        use std::time::Instant;

        let stats = ConnectionStats {
            request_count: 100,
            avg_response_time_ms: 25.5,
            last_success: Some(Instant::now()),
            failure_count: 5,
        };

        assert_eq!(stats.request_count, 100);
        assert_eq!(stats.failure_count, 5);
        assert!(stats.last_success.is_some());
    }

    #[test]
    fn test_connection_pool_health_metrics() {
        let metrics = ConnectionPoolHealthMetrics {
            total_connections: 10,
            healthy_connections: 8,
            unhealthy_connections: 2,
            total_requests: 1000,
            total_failures: 50,
            overall_failure_rate: 0.05,
            avg_response_time_ms: 30.0,
        };

        assert_eq!(metrics.total_connections, 10);
        assert_eq!(metrics.healthy_connections, 8);
        assert_eq!(metrics.overall_failure_rate, 0.05);
    }

    #[test]
    fn test_cache_statistics() {
        let stats = CacheStatistics {
            discovery_cache_size: 100,
            capabilities_cache_size: 50,
            total_cache_hits: 500,
            valid_cache_entries: 90,
            expired_cache_entries: 10,
            connection_pool_stats: std::collections::HashMap::new(),
        };

        assert_eq!(stats.discovery_cache_size, 100);
        assert_eq!(stats.total_cache_hits, 500);
        assert!(stats.connection_pool_stats.is_empty());
    }

    #[test]
    fn test_cached_capability_match_validity() {
        use std::time::Instant;

        let mut cached = CachedCapabilityMatch {
            matches: Vec::new(),
            cached_at: Instant::now(),
            ttl_seconds: 300,
            access_count: 0,
        };

        // Should be valid immediately
        assert!(cached.is_valid());

        // Test access counter
        cached.accessed();
        assert_eq!(cached.access_count, 1);

        cached.accessed();
        assert_eq!(cached.access_count, 2);
    }

    #[test]
    fn test_cached_capability_match_expiry() {
        use std::time::{Duration, Instant};

        let cached = CachedCapabilityMatch {
            matches: Vec::new(),
            cached_at: Instant::now() - Duration::from_secs(400),
            ttl_seconds: 300,
            access_count: 0,
        };

        // Should be expired
        assert!(!cached.is_valid());
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert_eq!(config.capability_discovery_ttl, 300);
        assert_eq!(config.service_capabilities_ttl, 600);
        assert_eq!(config.max_cache_entries, 1000);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig {
            capability_discovery_ttl: 60,
            service_capabilities_ttl: 120,
            max_cache_entries: 500,
            enable_caching: false,
        };

        assert_eq!(config.capability_discovery_ttl, 60);
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_service_health_serialization() {
        let health = ServiceHealth::Degraded;
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ServiceHealth = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, ServiceHealth::Degraded));
    }

    #[test]
    fn test_discovered_service_serialization() {
        use chrono::Utc;

        let service = DiscoveredService {
            service_id: "test".to_string(),
            instance_id: "inst".to_string(),
            endpoint: "http://test".to_string(),
            capabilities: vec!["cap1".to_string()],
            health: ServiceHealth::Healthy,
            discovered_at: Utc::now(),
            last_health_check: None,
        };

        let json = serde_json::to_string(&service).unwrap();
        let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

        assert_eq!(service.service_id, deserialized.service_id);
    }
}
