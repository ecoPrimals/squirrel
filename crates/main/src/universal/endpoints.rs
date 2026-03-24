// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]

//! Endpoint and port management for primals
//!
//! This module defines types for managing primal endpoints and dynamic
//! port allocation for network services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Primal endpoint configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// HTTP endpoint URL
    pub http: Option<String>,
    /// gRPC endpoint URL
    pub grpc: Option<String>,
    /// WebSocket endpoint URL
    pub websocket: Option<String>,
    /// Primary endpoint URL
    pub primary: Option<String>,
    /// Health check endpoint URL
    pub health: Option<String>,
    /// Metrics endpoint URL
    pub metrics: Option<String>,
    /// MCP protocol endpoint URL
    pub mcp: Option<String>,
    /// AI coordination endpoint URL
    pub ai_coordination: Option<String>,
    /// Admin endpoint URL
    pub admin: Option<String>,
    /// Service mesh endpoint URL
    pub service_mesh: Option<String>,
    /// Custom endpoint name-URL pairs
    pub custom: Vec<(String, String)>,
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        use universal_constants::network;

        // PRIMAL SELF-KNOWLEDGE PRINCIPLE:
        // Squirrel knows ONLY its own listening configuration from environment.
        // Discovery of OTHER primals happens at runtime via Songbird.

        let http_port = network::get_port_from_env("SQUIRREL_HTTP_PORT", 9010);
        let grpc_port = network::get_port_from_env("SQUIRREL_GRPC_PORT", 9011);
        let ws_port = network::get_port_from_env("SQUIRREL_WS_PORT", 9012);

        // Get bind address from environment (for service registration)
        // In production: 0.0.0.0, in development: localhost
        // NOTE: This is SQUIRREL'S listening address, not other primals' endpoints
        let bind_host = std::env::var("SQUIRREL_HOST").unwrap_or_else(|_| {
            if std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .eq_ignore_ascii_case("production")
            {
                network::DEFAULT_BIND_ADDRESS.to_string()
            } else {
                network::DEFAULT_LOCALHOST.to_string()
            }
        });

        let base_url = format!("http://{bind_host}:{http_port}");

        Self {
            http: Some(base_url.clone()),
            grpc: Some(format!("http://{bind_host}:{grpc_port}")),
            websocket: Some(format!("ws://{bind_host}:{ws_port}")),
            primary: Some(base_url.clone()),
            health: Some(format!("{base_url}/health")),
            metrics: Some(format!("{base_url}/metrics")),
            mcp: Some(format!("{base_url}/mcp")),
            ai_coordination: Some(format!("{base_url}/ai")),
            admin: Some(format!("{base_url}/admin")),
            service_mesh: Some(format!("{base_url}/mesh")),
            custom: vec![],
        }
    }
}

/// Dynamic port information for runtime port allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Base port number
    pub port: u16,
    /// Port that was assigned
    pub assigned_port: u16,
    /// Current port in use
    pub current_port: u16,
    /// Optional port range (min, max)
    pub port_range: Option<(u16, u16)>,
    /// Type of port
    pub port_type: PortType,
    /// Current port status
    pub status: PortStatus,
    /// When the port was allocated
    pub allocated_at: DateTime<Utc>,
    /// When the port was assigned
    pub assigned_at: DateTime<Utc>,
    /// Optional lease duration
    pub lease_duration: Option<chrono::Duration>,
    /// When the lease expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Type of network port
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// HTTP port (uppercase variant)
    HTTP,
    /// HTTP port
    Http,
    /// gRPC port (uppercase variant)
    GRPC,
    /// gRPC port
    Grpc,
    /// WebSocket port
    WebSocket,
    /// Custom port type
    Custom(String),
}

/// Status of a port (consolidated from universal-patterns)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    /// Port is available for allocation
    Available,
    /// Port has been allocated
    Allocated,
    /// Port is active and in use
    Active,
    /// Port is currently in use
    InUse,
    /// Port is reserved but not yet active
    Reserved,
    /// Port is being released
    Releasing,
    /// Port has been released
    Released,
    /// Port lease has expired and should be cleaned up
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_endpoints_default() {
        temp_env::with_vars_unset(
            [
                "SQUIRREL_HTTP_PORT",
                "SQUIRREL_GRPC_PORT",
                "SQUIRREL_WS_PORT",
                "SQUIRREL_HOST",
                "ENVIRONMENT",
            ],
            || {
                let endpoints = PrimalEndpoints::default();
                assert!(endpoints.http.is_some());
                assert!(endpoints.grpc.is_some());
                assert!(endpoints.websocket.is_some());
                assert!(endpoints.primary.is_some());
                assert!(endpoints.health.is_some());
                assert!(endpoints.metrics.is_some());
                assert!(endpoints.mcp.is_some());
                assert!(endpoints.ai_coordination.is_some());
                assert!(endpoints.admin.is_some());
                assert!(endpoints.service_mesh.is_some());
                assert!(endpoints.custom.is_empty());

                let http = endpoints.http.expect("should succeed");
                assert!(http.contains("localhost:9010"), "got: {http}");
            },
        );
    }

    #[test]
    fn test_primal_endpoints_env_override() {
        temp_env::with_vars(
            [
                ("SQUIRREL_HTTP_PORT", Some("8080")),
                ("SQUIRREL_HOST", Some("0.0.0.0")),
                ("ENVIRONMENT", None::<&str>),
            ],
            || {
                let endpoints = PrimalEndpoints::default();
                let http = endpoints.http.expect("should succeed");
                assert!(http.contains("0.0.0.0:8080"), "got: {http}");
            },
        );
    }

    #[test]
    fn test_primal_endpoints_serde() {
        let endpoints = PrimalEndpoints {
            http: Some("http://localhost:9010".to_string()),
            grpc: Some("http://localhost:9011".to_string()),
            websocket: Some("ws://localhost:9012".to_string()),
            primary: Some("http://localhost:9010".to_string()),
            health: Some("http://localhost:9010/health".to_string()),
            metrics: Some("http://localhost:9010/metrics".to_string()),
            mcp: Some("http://localhost:9010/mcp".to_string()),
            ai_coordination: Some("http://localhost:9010/ai".to_string()),
            admin: Some("http://localhost:9010/admin".to_string()),
            service_mesh: Some("http://localhost:9010/mesh".to_string()),
            custom: vec![("test".to_string(), "http://test".to_string())],
        };
        let json = serde_json::to_string(&endpoints).expect("serialize");
        let deser: PrimalEndpoints = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, endpoints);
    }

    #[test]
    fn test_port_type_serde() {
        for pt in [
            PortType::HTTP,
            PortType::Http,
            PortType::GRPC,
            PortType::Grpc,
            PortType::WebSocket,
            PortType::Custom("test".to_string()),
        ] {
            let json = serde_json::to_string(&pt).expect("serialize");
            let deser: PortType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, pt);
        }
    }

    #[test]
    fn test_port_status_serde() {
        for ps in [
            PortStatus::Available,
            PortStatus::Allocated,
            PortStatus::Active,
            PortStatus::InUse,
            PortStatus::Reserved,
            PortStatus::Releasing,
            PortStatus::Released,
            PortStatus::Expired,
        ] {
            let json = serde_json::to_string(&ps).expect("serialize");
            let deser: PortStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, ps);
        }
    }

    #[test]
    fn test_dynamic_port_info_serde() {
        let info = DynamicPortInfo {
            port: 9010,
            assigned_port: 9010,
            current_port: 9010,
            port_range: Some((9000, 9100)),
            port_type: PortType::HTTP,
            status: PortStatus::Active,
            allocated_at: Utc::now(),
            assigned_at: Utc::now(),
            lease_duration: None,
            expires_at: None,
            metadata: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let deser: DynamicPortInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.port, 9010);
        assert_eq!(deser.port_type, PortType::HTTP);
        assert_eq!(deser.status, PortStatus::Active);
    }
}
