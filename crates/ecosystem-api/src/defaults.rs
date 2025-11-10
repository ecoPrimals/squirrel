//! Default configuration values for ecosystem API
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Default ecosystem service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get Songbird endpoint from environment or default (now using service_mesh_endpoint)
    pub fn songbird_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8500".to_string())
    }

    /// Get ToadStool endpoint from environment or default
    pub fn toadstool_endpoint() -> String {
        env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| "http://localhost:8081".to_string())
    }

    /// Get NestGate endpoint from environment or default
    pub fn nestgate_endpoint() -> String {
        env::var("NESTGATE_ENDPOINT").unwrap_or_else(|_| "http://localhost:8082".to_string())
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| "http://localhost:8443".to_string())
    }

    /// Get development bind address from environment or default
    pub fn dev_bind_address() -> String {
        env::var("DEV_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Get discovery endpoint from environment or default
    pub fn discovery_endpoint() -> String {
        env::var("DISCOVERY_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/api/v1/discovery", Self::songbird_endpoint()))
    }

    /// Get registration endpoint from environment or default
    pub fn registration_endpoint() -> String {
        env::var("REGISTRATION_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/api/v1/register", Self::songbird_endpoint()))
    }

    /// Get health endpoint from environment or default
    pub fn health_endpoint(base_url: &str) -> String {
        format!("{}/health", base_url.trim_end_matches('/'))
    }

    /// Get metrics endpoint from environment or default
    pub fn metrics_endpoint(base_url: &str) -> String {
        format!("{}/metrics", base_url.trim_end_matches('/'))
    }

    /// Get admin endpoint from environment or default
    pub fn admin_endpoint(base_url: &str) -> String {
        format!("{}/admin", base_url.trim_end_matches('/'))
    }

    /// Get WebSocket URL
    pub fn websocket_endpoint(base_url: &str) -> String {
        let ws_base = base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        format!("{}/ws", ws_base.trim_end_matches('/'))
    }

    /// Get MCP endpoint
    pub fn mcp_endpoint(base_url: &str) -> String {
        format!("{}/mcp", base_url.trim_end_matches('/'))
    }

    /// Get AI coordination endpoint
    pub fn ai_coordination_endpoint(base_url: &str) -> String {
        format!("{}/ai", base_url.trim_end_matches('/'))
    }

    /// Get service mesh endpoint
    pub fn service_mesh_endpoint(base_url: &str) -> String {
        format!("{}/mesh", base_url.trim_end_matches('/'))
    }
}
