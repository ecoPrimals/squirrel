//! Endpoint and port management for primals
//!
//! This module defines types for managing primal endpoints and dynamic
//! port allocation for network services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Primal endpoint configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    pub http: Option<String>,
    pub grpc: Option<String>,
    pub websocket: Option<String>,
    pub primary: Option<String>,
    pub health: Option<String>,
    pub metrics: Option<String>,
    pub mcp: Option<String>,
    pub ai_coordination: Option<String>,
    pub admin: Option<String>,
    pub service_mesh: Option<String>,
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

        let base_url = format!("http://{}:{}", bind_host, http_port);

        Self {
            http: Some(base_url.clone()),
            grpc: Some(format!("http://{}:{}", bind_host, grpc_port)),
            websocket: Some(format!("ws://{}:{}", bind_host, ws_port)),
            primary: Some(base_url.clone()),
            health: Some(format!("{}/health", base_url)),
            metrics: Some(format!("{}/metrics", base_url)),
            mcp: Some(format!("{}/mcp", base_url)),
            ai_coordination: Some(format!("{}/ai", base_url)),
            admin: Some(format!("{}/admin", base_url)),
            service_mesh: Some(format!("{}/mesh", base_url)),
            custom: vec![],
        }
    }
}

/// Dynamic port information for runtime port allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    pub port: u16,
    pub assigned_port: u16,
    pub current_port: u16,
    pub port_range: Option<(u16, u16)>,
    pub port_type: PortType,
    pub status: PortStatus,
    pub allocated_at: DateTime<Utc>,
    pub assigned_at: DateTime<Utc>,
    pub lease_duration: Option<chrono::Duration>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Type of network port
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    HTTP,
    Http,
    GRPC,
    Grpc,
    WebSocket,
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
