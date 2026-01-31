//! Default configuration values for ecosystem API
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Default ecosystem service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get Songbird endpoint from environment or default (now using `service_mesh_endpoint`)
    ///
    /// Multi-tier resolution:
    /// 1. SERVICE_MESH_ENDPOINT (full endpoint)
    /// 2. SONGBIRD_ENDPOINT (songbird-specific)
    /// 3. SONGBIRD_PORT (port override)
    /// 4. Default: http://localhost:8500
    #[must_use]
    pub fn songbird_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| env::var("SONGBIRD_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SONGBIRD_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8500); // Default Songbird service mesh port
                format!("http://localhost:{}", port)
            })
    }

    /// Get `ToadStool` endpoint from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. TOADSTOOL_ENDPOINT (full endpoint)
    /// 2. TOADSTOOL_PORT (port override)
    /// 3. Default: http://localhost:8081
    #[must_use]
    pub fn toadstool_endpoint() -> String {
        env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            let port = env::var("TOADSTOOL_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8081); // Default ToadStool compute port (alt)
            format!("http://localhost:{}", port)
        })
    }

    /// Get `NestGate` endpoint from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. NESTGATE_ENDPOINT (full endpoint)
    /// 2. NESTGATE_PORT (port override)
    /// 3. Default: http://localhost:8082
    #[must_use]
    pub fn nestgate_endpoint() -> String {
        env::var("NESTGATE_ENDPOINT").unwrap_or_else(|_| {
            let port = env::var("NESTGATE_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8082); // Default NestGate UniBin port
            format!("http://localhost:{}", port)
        })
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    ///
    /// Multi-tier resolution:
    /// 1. SECURITY_SERVICE_ENDPOINT (full endpoint)
    /// 2. SECURITY_AUTH_SERVICE_ENDPOINT (alt full endpoint)
    /// 3. SECURITY_AUTHENTICATION_PORT (port override)
    /// 4. Default: http://localhost:8443
    #[must_use]
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8443); // Default security auth port
                format!("http://localhost:{}", port)
            })
    }

    /// Get development bind address from environment or default
    #[must_use]
    pub fn dev_bind_address() -> String {
        env::var("DEV_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Get discovery endpoint from environment or default
    #[must_use]
    pub fn discovery_endpoint() -> String {
        env::var("DISCOVERY_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/api/v1/discovery", Self::songbird_endpoint()))
    }

    /// Get registration endpoint from environment or default
    #[must_use]
    pub fn registration_endpoint() -> String {
        env::var("REGISTRATION_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/api/v1/register", Self::songbird_endpoint()))
    }

    /// Get health endpoint from environment or default
    #[must_use]
    pub fn health_endpoint(base_url: &str) -> String {
        format!("{}/health", base_url.trim_end_matches('/'))
    }

    /// Get metrics endpoint from environment or default
    #[must_use]
    pub fn metrics_endpoint(base_url: &str) -> String {
        format!("{}/metrics", base_url.trim_end_matches('/'))
    }

    /// Get admin endpoint from environment or default
    #[must_use]
    pub fn admin_endpoint(base_url: &str) -> String {
        format!("{}/admin", base_url.trim_end_matches('/'))
    }

    /// Get WebSocket URL
    #[must_use]
    pub fn websocket_endpoint(base_url: &str) -> String {
        let ws_base = base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        format!("{}/ws", ws_base.trim_end_matches('/'))
    }

    /// Get MCP endpoint
    #[must_use]
    pub fn mcp_endpoint(base_url: &str) -> String {
        format!("{}/mcp", base_url.trim_end_matches('/'))
    }

    /// Get AI coordination endpoint
    #[must_use]
    pub fn ai_coordination_endpoint(base_url: &str) -> String {
        format!("{}/ai", base_url.trim_end_matches('/'))
    }

    /// Get service mesh endpoint
    #[must_use]
    pub fn service_mesh_endpoint(base_url: &str) -> String {
        format!("{}/mesh", base_url.trim_end_matches('/'))
    }
}
