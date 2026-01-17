//! API server types and response structures
//!
//! Common types used across API endpoints, following ecosystem standards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health check response following ecosystem standards
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Service mesh status
    pub service_mesh: ServiceMeshHealthStatus,
    /// Ecosystem integration status
    pub ecosystem: EcosystemHealthStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service mesh health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshHealthStatus {
    /// Registered with Songbird
    pub registered: bool,
    /// Last heartbeat
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// Connection status
    pub connection_status: String,
    /// Load balancing active
    pub load_balancing_active: bool,
}

/// Ecosystem integration health status
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemHealthStatus {
    /// Discovered primals
    pub discovered_primals: u32,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Cross-primal communication status
    pub cross_primal_status: String,
    /// Ecosystem health score
    pub ecosystem_health_score: f64,
}

/// Ecosystem status response
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemStatusResponse {
    /// Registered primals
    pub registered_primals: Vec<String>,
    /// Primal capabilities  
    pub capabilities: HashMap<String, Vec<String>>,
}

/// Service mesh status response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshStatusResponse {
    /// Load balancing status
    pub load_balancing: LoadBalancingResponse,
    /// Cross-primal communication
    pub cross_primal_communication: CrossPrimalCommunicationResponse,
}

/// Load balancing response
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadBalancingResponse {
    /// Active status
    pub active: bool,
    /// Algorithm used
    pub algorithm: String,
}

/// Cross-primal communication response
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossPrimalCommunicationResponse {
    /// Enabled status
    pub enabled: bool,
    /// Protocol used
    pub protocol: String,
}

/// Primal status response
/// Uses String for API boundaries (serde compatibility)
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalStatusResponse {
    /// Primal name
    pub name: String,
    /// Health status
    pub health: String,
    /// Capabilities
    pub capabilities: Vec<String>,
}

/// Metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Request count
    pub request_count: u64,
    /// Active connections
    pub active_connections: u32,
    /// Uptime seconds
    pub uptime_seconds: u64,
}

/// Services response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResponse {
    /// List of services
    pub services: Vec<ServiceInfo>,
    /// Service mesh integration
    pub service_mesh_integration: ServiceMeshIntegrationStatus,
}

/// Service mesh integration status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshIntegrationStatus {
    /// Registered with Songbird
    pub songbird_registered: bool,
    /// Health reporting active
    pub health_reporting_active: bool,
}

/// Service information
/// Service information
/// Uses String for API boundaries (serde compatibility)
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service endpoint
    pub endpoint: String,
    /// Service health
    pub health: String,
}

/// Service mesh registration response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshRegistrationResponse {
    /// Registration status
    pub status: String,
    /// Registration message
    pub message: String,
}

/// Service mesh heartbeat response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshHeartbeatResponse {
    /// Heartbeat status
    pub status: String,
}

/// Deprecated: Use ServiceMeshRegistrationResponse instead
#[deprecated(note = "Use ServiceMeshRegistrationResponse for capability-based discovery")]
pub type SongbirdRegistrationResponse = ServiceMeshRegistrationResponse;

/// Deprecated: Use ServiceMeshHeartbeatResponse instead
#[deprecated(note = "Use ServiceMeshHeartbeatResponse for capability-based discovery")]
pub type SongbirdHeartbeatResponse = ServiceMeshHeartbeatResponse;

/// Shutdown response
#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownResponse {
    /// Shutdown status
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_health_response_creation() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            uptime_seconds: 3600,
            service_mesh: ServiceMeshHealthStatus {
                registered: true,
                last_heartbeat: Some(Utc::now()),
                connection_status: "connected".to_string(),
                load_balancing_active: true,
            },
            ecosystem: EcosystemHealthStatus {
                discovered_primals: 5,
                active_integrations: vec!["songbird".to_string(), "beardog".to_string()],
                cross_primal_status: "healthy".to_string(),
                ecosystem_health_score: 0.95,
            },
            metadata: HashMap::new(),
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.uptime_seconds, 3600);
        assert_eq!(response.ecosystem.discovered_primals, 5);
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            uptime_seconds: 100,
            service_mesh: ServiceMeshHealthStatus {
                registered: false,
                last_heartbeat: None,
                connection_status: "disconnected".to_string(),
                load_balancing_active: false,
            },
            ecosystem: EcosystemHealthStatus {
                discovered_primals: 0,
                active_integrations: vec![],
                cross_primal_status: "initializing".to_string(),
                ecosystem_health_score: 0.5,
            },
            metadata: HashMap::new(),
        };

        let json =
            serde_json::to_string(&response).expect("HealthResponse should serialize to JSON");
        assert!(json.contains("healthy"));

        let deserialized: HealthResponse =
            serde_json::from_str(&json).expect("JSON should deserialize to HealthResponse");
        assert_eq!(deserialized.status, "healthy");
        assert!(!deserialized.service_mesh.registered);
    }

    #[test]
    fn test_service_mesh_health_status() {
        let status = ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: Some(Utc::now()),
            connection_status: "active".to_string(),
            load_balancing_active: true,
        };

        let json = serde_json::to_string(&status)
            .expect("ServiceMeshHealthStatus should serialize to JSON");
        let deserialized: ServiceMeshHealthStatus = serde_json::from_str(&json)
            .expect("JSON should deserialize to ServiceMeshHealthStatus");
        assert!(deserialized.registered);
        assert!(deserialized.load_balancing_active);
        assert_eq!(deserialized.connection_status, "active");
    }

    #[test]
    fn test_ecosystem_health_status() {
        let status = EcosystemHealthStatus {
            discovered_primals: 10,
            active_integrations: vec!["primal1".to_string(), "primal2".to_string()],
            cross_primal_status: "optimal".to_string(),
            ecosystem_health_score: 0.99,
        };

        assert_eq!(status.discovered_primals, 10);
        assert_eq!(status.active_integrations.len(), 2);
        assert!(status.ecosystem_health_score > 0.9);
    }

    #[test]
    fn test_ecosystem_status_response() {
        let mut capabilities = HashMap::new();
        capabilities.insert("squirrel".to_string(), vec!["ai".to_string()]);

        let response = EcosystemStatusResponse {
            registered_primals: vec!["squirrel".to_string(), "songbird".to_string()],
            capabilities,
        };

        assert_eq!(response.registered_primals.len(), 2);
        assert_eq!(response.capabilities.len(), 1);
    }

    #[test]
    fn test_service_mesh_status_response() {
        let response = ServiceMeshStatusResponse {
            load_balancing: LoadBalancingResponse {
                active: true,
                algorithm: "round-robin".to_string(),
            },
            cross_primal_communication: CrossPrimalCommunicationResponse {
                enabled: true,
                protocol: "http2".to_string(),
            },
        };

        assert!(response.load_balancing.active);
        assert!(response.cross_primal_communication.enabled);
        assert_eq!(response.load_balancing.algorithm, "round-robin");
    }

    #[test]
    fn test_load_balancing_response() {
        let lb = LoadBalancingResponse {
            active: false,
            algorithm: "least-connections".to_string(),
        };

        let json =
            serde_json::to_string(&lb).expect("LoadBalancingResponse should serialize to JSON");
        let deserialized: LoadBalancingResponse =
            serde_json::from_str(&json).expect("JSON should deserialize to LoadBalancingResponse");
        assert!(!deserialized.active);
        assert_eq!(deserialized.algorithm, "least-connections");
    }

    #[test]
    fn test_cross_primal_communication_response() {
        let comm = CrossPrimalCommunicationResponse {
            enabled: true,
            protocol: "grpc".to_string(),
        };

        assert!(comm.enabled);
        assert_eq!(comm.protocol, "grpc");
    }

    #[test]
    fn test_primal_status_response() {
        let status = PrimalStatusResponse {
            name: "squirrel".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["inference".to_string(), "coordination".to_string()],
        };

        assert_eq!(status.name, "squirrel");
        assert_eq!(status.capabilities.len(), 2);
    }

    #[test]
    fn test_metrics_response() {
        let response = MetricsResponse {
            request_count: 1000,
            active_connections: 25,
            uptime_seconds: 3600,
        };

        assert_eq!(response.request_count, 1000);
        assert_eq!(response.active_connections, 25);
        assert_eq!(response.uptime_seconds, 3600);
    }

    #[test]
    fn test_metrics_response_serialization() {
        let response = MetricsResponse {
            request_count: 500,
            active_connections: 10,
            uptime_seconds: 7200,
        };

        let json =
            serde_json::to_string(&response).expect("MetricsResponse should serialize to JSON");
        let deserialized: MetricsResponse =
            serde_json::from_str(&json).expect("JSON should deserialize to MetricsResponse");
        assert_eq!(deserialized.request_count, 500);
        assert_eq!(deserialized.active_connections, 10);
    }

    #[test]
    fn test_services_response() {
        let services = vec![
            ServiceInfo {
                name: "service1".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                health: "healthy".to_string(),
            },
            ServiceInfo {
                name: "service2".to_string(),
                endpoint: "http://localhost:9000".to_string(),
                health: "degraded".to_string(),
            },
        ];

        let response = ServicesResponse {
            services,
            service_mesh_integration: ServiceMeshIntegrationStatus {
                songbird_registered: true,
                health_reporting_active: true,
            },
        };

        assert_eq!(response.services.len(), 2);
        assert!(response.service_mesh_integration.songbird_registered);
    }

    #[test]
    fn test_shutdown_response() {
        let response = ShutdownResponse {
            status: "shutting_down".to_string(),
        };

        assert_eq!(response.status, "shutting_down");

        let json =
            serde_json::to_string(&response).expect("ShutdownResponse should serialize to JSON");
        let deserialized: ShutdownResponse =
            serde_json::from_str(&json).expect("JSON should deserialize to ShutdownResponse");
        assert_eq!(deserialized.status, "shutting_down");
    }

    #[test]
    fn test_songbird_registration_response() {
        let response = SongbirdRegistrationResponse {
            status: "registered".to_string(),
            message: "Successfully registered with Songbird".to_string(),
        };

        assert_eq!(response.status, "registered");
        assert!(response.message.contains("Successfully"));
    }

    #[test]
    fn test_songbird_heartbeat_response() {
        let response = SongbirdHeartbeatResponse {
            status: "active".to_string(),
        };

        assert_eq!(response.status, "active");

        let json =
            serde_json::to_string(&response).expect("StatusResponse should serialize to JSON");
        assert!(json.contains("active"));
    }
}
