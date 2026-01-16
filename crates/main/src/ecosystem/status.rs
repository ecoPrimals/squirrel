//! Ecosystem status and health monitoring types
//!
//! This module contains all status-related types for ecosystem monitoring,
//! health checks, and service mesh status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::registry::types::DiscoveredService;
use crate::universal::LoadBalancingStatus;

/// Overall ecosystem manager status (integration layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemIntegrationStatus {
    /// Overall status
    pub status: String,
    /// Status timestamp
    pub timestamp: DateTime<Utc>,
    /// Discovered services
    pub discovered_services: Vec<DiscoveredService>,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Service mesh status
    pub service_mesh_status: ServiceMeshStatus,
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
}

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Service mesh enabled
    pub enabled: bool,
    /// Registered with Songbird
    pub registered: bool,
    /// Load balancing status
    pub load_balancing: LoadBalancingStatus,
    /// Cross-primal communication status
    pub cross_primal_communication: CrossPrimalStatus,
}

/// Cross-primal communication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalStatus {
    /// Cross-primal communication enabled
    pub enabled: bool,
    /// Active connections
    pub active_connections: u32,
    /// Supported protocols
    pub supported_protocols: Vec<String>,
}

/// Ecosystem manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemManagerStatus {
    /// Current status
    pub status: String,
    /// Initialization timestamp
    pub initialized_at: Option<DateTime<Utc>>,
    /// Last successful registration
    pub last_registration: Option<DateTime<Utc>>,
    /// Active service registrations
    pub active_registrations: Vec<String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Error count
    pub error_count: u32,
    /// Last error message
    pub last_error: Option<String>,
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Component health statuses
    pub component_statuses: HashMap<String, ComponentHealth>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Health check errors
    pub health_errors: Vec<String>,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Health status
    pub status: String,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_integration_status_creation() {
        let status = EcosystemIntegrationStatus {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            discovered_services: Vec::new(),
            active_integrations: Vec::new(),
            service_mesh_status: ServiceMeshStatus {
                enabled: true,
                registered: true,
                load_balancing: LoadBalancingStatus {
                    enabled: true,
                    healthy: true,
                    active_connections: 0,
                    algorithm: "round-robin".to_string(),
                    health_score: 1.0,
                    last_check: chrono::Utc::now(),
                },
                cross_primal_communication: CrossPrimalStatus {
                    enabled: true,
                    active_connections: 2,
                    supported_protocols: vec!["http".to_string(), "grpc".to_string()],
                },
            },
            overall_health: 1.0,
        };

        assert_eq!(status.status, "healthy");
        assert!(status.service_mesh_status.enabled);
        assert_eq!(status.overall_health, 1.0);
    }

    #[test]
    fn test_health_status_with_errors() {
        let mut health = HealthStatus {
            health_score: 0.75,
            component_statuses: HashMap::new(),
            last_check: Utc::now(),
            health_errors: vec!["Database connection slow".to_string()],
        };

        health.component_statuses.insert(
            "database".to_string(),
            ComponentHealth {
                status: "degraded".to_string(),
                last_check: Utc::now(),
                error: Some("Connection timeout".to_string()),
                metadata: HashMap::new(),
            },
        );

        assert_eq!(health.health_score, 0.75);
        assert_eq!(health.health_errors.len(), 1);
        assert!(health.component_statuses.contains_key("database"));
    }
}
