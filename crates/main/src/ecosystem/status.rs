// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem status and health monitoring types
//!
//! This module contains all status-related types for ecosystem monitoring,
//! health checks, and service mesh status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::registry::types::DiscoveredService;

fn serialize_arc_str_vec<S>(vec: &[Arc<str>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let strings: Vec<&str> = vec.iter().map(std::convert::AsRef::as_ref).collect();
    strings.serialize(serializer)
}

fn deserialize_arc_str_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    Ok(strings.into_iter().map(Arc::from).collect())
}

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
    /// Active service registrations (`Arc<str>` for O(1) clone)
    #[serde(
        serialize_with = "serialize_arc_str_vec",
        deserialize_with = "deserialize_arc_str_vec"
    )]
    pub active_registrations: Vec<Arc<str>>,
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
                    supported_protocols: vec!["http".to_string(), "tarpc".to_string()],
                },
            },
            overall_health: 1.0,
        };

        assert_eq!(status.status, "healthy");
        assert!(status.service_mesh_status.enabled);
        assert!((status.overall_health - 1.0).abs() < f64::EPSILON);
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

        assert!((health.health_score - 0.75).abs() < f64::EPSILON);
        assert_eq!(health.health_errors.len(), 1);
        assert!(health.component_statuses.contains_key("database"));
    }

    #[test]
    fn test_ecosystem_integration_status_serde() {
        let status = EcosystemIntegrationStatus {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            discovered_services: Vec::new(),
            active_integrations: vec!["ai".to_string(), "storage".to_string()],
            service_mesh_status: ServiceMeshStatus {
                enabled: true,
                registered: false,
                load_balancing: LoadBalancingStatus {
                    enabled: false,
                    healthy: false,
                    active_connections: 0,
                    algorithm: "random".to_string(),
                    health_score: 0.5,
                    last_check: Utc::now(),
                },
                cross_primal_communication: CrossPrimalStatus {
                    enabled: false,
                    active_connections: 0,
                    supported_protocols: vec![],
                },
            },
            overall_health: 0.85,
        };

        let json = serde_json::to_string(&status).expect("should succeed");
        let deserialized: EcosystemIntegrationStatus =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.status, "healthy");
        assert_eq!(deserialized.active_integrations.len(), 2);
        assert!((deserialized.overall_health - 0.85).abs() < f64::EPSILON);
        assert!(deserialized.service_mesh_status.enabled);
        assert!(!deserialized.service_mesh_status.registered);
    }

    #[test]
    fn test_service_mesh_status_serde() {
        let status = ServiceMeshStatus {
            enabled: true,
            registered: true,
            load_balancing: LoadBalancingStatus {
                enabled: true,
                healthy: true,
                active_connections: 5,
                algorithm: "round-robin".to_string(),
                health_score: 0.95,
                last_check: Utc::now(),
            },
            cross_primal_communication: CrossPrimalStatus {
                enabled: true,
                active_connections: 3,
                supported_protocols: vec!["json-rpc".to_string(), "tarpc".to_string()],
            },
        };

        let json = serde_json::to_string(&status).expect("should succeed");
        let deserialized: ServiceMeshStatus = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.enabled);
        assert!(deserialized.registered);
        assert_eq!(deserialized.load_balancing.active_connections, 5);
        assert_eq!(
            deserialized.cross_primal_communication.active_connections,
            3
        );
    }

    #[test]
    fn test_cross_primal_status_serde() {
        let status = CrossPrimalStatus {
            enabled: true,
            active_connections: 10,
            supported_protocols: vec!["http".to_string(), "ws".to_string()],
        };

        let json = serde_json::to_string(&status).expect("should succeed");
        let deserialized: CrossPrimalStatus = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.enabled);
        assert_eq!(deserialized.active_connections, 10);
        assert_eq!(deserialized.supported_protocols.len(), 2);
    }

    #[test]
    fn test_ecosystem_manager_status_creation() {
        let status = EcosystemManagerStatus {
            status: "running".to_string(),
            initialized_at: Some(Utc::now()),
            last_registration: None,
            active_registrations: vec![Arc::from("service-1")],
            health_status: HealthStatus {
                health_score: 1.0,
                component_statuses: HashMap::new(),
                last_check: Utc::now(),
                health_errors: vec![],
            },
            error_count: 0,
            last_error: None,
        };

        assert_eq!(status.status, "running");
        assert!(status.initialized_at.is_some());
        assert!(status.last_registration.is_none());
        assert_eq!(status.active_registrations.len(), 1);
        assert_eq!(status.error_count, 0);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn test_ecosystem_manager_status_serde() {
        let status = EcosystemManagerStatus {
            status: "error".to_string(),
            initialized_at: None,
            last_registration: None,
            active_registrations: vec![],
            health_status: HealthStatus {
                health_score: 0.0,
                component_statuses: HashMap::new(),
                last_check: Utc::now(),
                health_errors: vec!["Critical failure".to_string()],
            },
            error_count: 5,
            last_error: Some("Connection refused".to_string()),
        };

        let json = serde_json::to_string(&status).expect("should succeed");
        let deserialized: EcosystemManagerStatus =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.status, "error");
        assert_eq!(deserialized.error_count, 5);
        assert_eq!(
            deserialized.last_error.as_deref(),
            Some("Connection refused")
        );
    }

    #[test]
    fn test_component_health_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());

        let health = ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            error: None,
            metadata,
        };

        assert_eq!(health.status, "healthy");
        assert!(health.error.is_none());
        assert_eq!(
            health.metadata.get("version").expect("should succeed"),
            "1.0"
        );
    }

    #[test]
    fn test_component_health_serde() {
        let health = ComponentHealth {
            status: "unhealthy".to_string(),
            last_check: Utc::now(),
            error: Some("Out of memory".to_string()),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&health).expect("should succeed");
        let deserialized: ComponentHealth = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.status, "unhealthy");
        assert_eq!(deserialized.error.as_deref(), Some("Out of memory"));
    }

    #[test]
    fn test_health_status_serde() {
        let health = HealthStatus {
            health_score: 0.92,
            component_statuses: HashMap::new(),
            last_check: Utc::now(),
            health_errors: vec![],
        };

        let json = serde_json::to_string(&health).expect("should succeed");
        let deserialized: HealthStatus = serde_json::from_str(&json).expect("should succeed");
        assert!((deserialized.health_score - 0.92).abs() < f64::EPSILON);
        assert!(deserialized.health_errors.is_empty());
    }
}
