// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service mesh integration types
//!
//! This module defines types for service mesh integration including
//! load balancing, circuit breaking, and mesh status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Load balancing status and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatus {
    /// Whether load balancing is enabled
    pub enabled: bool,
    /// Whether the load balancer is healthy
    pub healthy: bool,
    /// Number of active connections
    pub active_connections: u32,
    /// Load balancing algorithm (e.g., "round_robin", "least_connections")
    pub algorithm: String,
    /// Health score from 0.0 to 1.0
    pub health_score: f64,
    /// Timestamp of last health check
    pub last_check: DateTime<Utc>,
}

/// Circuit breaker status for fault tolerance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CircuitBreakerStatus {
    /// Whether the circuit is open (rejecting requests)
    pub open: bool,
    /// Number of consecutive failures
    pub failures: u32,
    /// Timestamp of last failure
    pub last_failure: Option<DateTime<Utc>>,
    /// When the next retry is scheduled
    pub next_retry: Option<DateTime<Utc>>,
}

/// Service mesh integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Whether the service is registered with the mesh
    pub registered: bool,
    /// Whether the service is connected to the mesh
    pub connected: bool,
    /// Service mesh endpoint URL if configured
    pub service_mesh_endpoint: Option<String>,
    /// When the service was registered
    pub registration_time: Option<DateTime<Utc>>,
    /// Timestamp of last heartbeat sent
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Version of the mesh protocol
    pub mesh_version: String,
    /// Unique instance identifier
    pub instance_id: String,
    /// Whether load balancing is enabled
    pub load_balancing_enabled: bool,
    /// Current circuit breaker state
    pub circuit_breaker_status: CircuitBreakerStatus,
    /// Timestamp of last registration attempt
    pub last_registration: Option<DateTime<Utc>>,
    /// Overall mesh health status
    pub mesh_health: String,
    /// Number of active connections
    pub active_connections: u32,
    /// Load balancing status and configuration
    pub load_balancing: LoadBalancingStatus,
}

impl Default for ServiceMeshStatus {
    fn default() -> Self {
        Self {
            registered: false,
            connected: false,
            service_mesh_endpoint: None,
            registration_time: None,
            last_heartbeat: None,
            mesh_version: "1.0.0".to_string(),
            instance_id: "unknown".to_string(),
            load_balancing_enabled: false,
            circuit_breaker_status: CircuitBreakerStatus::default(),
            last_registration: None,
            mesh_health: "unknown".to_string(),
            active_connections: 0,
            load_balancing: LoadBalancingStatus {
                enabled: false,
                healthy: false,
                active_connections: 0,
                algorithm: "round_robin".to_string(),
                health_score: 0.0,
                last_check: Utc::now(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_status_default() {
        let status = CircuitBreakerStatus::default();
        assert!(!status.open);
        assert_eq!(status.failures, 0);
        assert!(status.last_failure.is_none());
        assert!(status.next_retry.is_none());
    }

    #[test]
    fn test_circuit_breaker_status_serde() {
        let status = CircuitBreakerStatus {
            open: true,
            failures: 5,
            last_failure: Some(Utc::now()),
            next_retry: Some(Utc::now()),
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let deser: CircuitBreakerStatus = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.open);
        assert_eq!(deser.failures, 5);
    }

    #[test]
    fn test_service_mesh_status_default() {
        let status = ServiceMeshStatus::default();
        assert!(!status.registered);
        assert!(!status.connected);
        assert!(status.service_mesh_endpoint.is_none());
        assert_eq!(status.mesh_version, "1.0.0");
        assert_eq!(status.instance_id, "unknown");
        assert!(!status.load_balancing_enabled);
        assert!(!status.circuit_breaker_status.open);
        assert_eq!(status.mesh_health, "unknown");
        assert_eq!(status.active_connections, 0);
        assert!(!status.load_balancing.enabled);
        assert_eq!(status.load_balancing.algorithm, "round_robin");
    }

    #[test]
    fn test_service_mesh_status_serde() {
        let status = ServiceMeshStatus::default();
        let json = serde_json::to_string(&status).expect("serialize");
        let deser: ServiceMeshStatus = serde_json::from_str(&json).expect("deserialize");
        assert!(!deser.registered);
        assert_eq!(deser.mesh_version, "1.0.0");
    }

    #[test]
    fn test_load_balancing_status_serde() {
        let status = LoadBalancingStatus {
            enabled: true,
            healthy: true,
            active_connections: 42,
            algorithm: "least_connections".to_string(),
            health_score: 0.95,
            last_check: Utc::now(),
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let deser: LoadBalancingStatus = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.enabled);
        assert!(deser.healthy);
        assert_eq!(deser.active_connections, 42);
        assert_eq!(deser.algorithm, "least_connections");
    }
}
