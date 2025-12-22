//! Service mesh integration types
//!
//! This module defines types for service mesh integration including
//! load balancing, circuit breaking, and mesh status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Load balancing status and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatus {
    pub enabled: bool,
    pub healthy: bool,
    pub active_connections: u32,
    pub algorithm: String,
    pub health_score: f64,
    pub last_check: DateTime<Utc>,
}

/// Circuit breaker status for fault tolerance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CircuitBreakerStatus {
    pub open: bool,
    pub failures: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub next_retry: Option<DateTime<Utc>>,
}

/// Service mesh integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    pub registered: bool,
    pub connected: bool,
    pub songbird_endpoint: Option<String>,
    pub registration_time: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub mesh_version: String,
    pub instance_id: String,
    pub load_balancing_enabled: bool,
    pub circuit_breaker_status: CircuitBreakerStatus,
    pub last_registration: Option<DateTime<Utc>>,
    pub mesh_health: String,
    pub active_connections: u32,
    pub load_balancing: LoadBalancingStatus,
}

impl Default for ServiceMeshStatus {
    fn default() -> Self {
        Self {
            registered: false,
            connected: false,
            songbird_endpoint: None,
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
