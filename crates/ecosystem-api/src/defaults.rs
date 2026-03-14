// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Default configuration values for ecosystem API
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Default ecosystem service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get service mesh endpoint from environment or default (capability-based)
    ///
    /// Multi-tier resolution:
    /// 1. `SERVICE_MESH_ENDPOINT` (full endpoint)
    /// 2. `SONGBIRD_ENDPOINT` (legacy env var)
    /// 3. `SERVICE_MESH_PORT` or `SONGBIRD_PORT` (port override)
    /// 4. Default: <http://localhost:8500>
    #[must_use]
    pub fn service_mesh_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| env::var("SONGBIRD_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SERVICE_MESH_PORT")
                    .or_else(|_| env::var("SONGBIRD_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8500);
                format!("http://localhost:{port}")
            })
    }

    /// Get compute service endpoint from environment or default (capability-based)
    ///
    /// Multi-tier resolution:
    /// 1. `COMPUTE_SERVICE_ENDPOINT` (full endpoint)
    /// 2. `TOADSTOOL_ENDPOINT` (legacy)
    /// 3. `COMPUTE_SERVICE_PORT` or `TOADSTOOL_PORT` (port override)
    /// 4. Default: <http://localhost:8081>
    #[must_use]
    pub fn compute_endpoint() -> String {
        env::var("COMPUTE_SERVICE_ENDPOINT")
            .or_else(|_| env::var("TOADSTOOL_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("COMPUTE_SERVICE_PORT")
                    .or_else(|_| env::var("TOADSTOOL_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8081);
                format!("http://localhost:{port}")
            })
    }

    /// Get storage service endpoint from environment or default (capability-based)
    ///
    /// Multi-tier resolution:
    /// 1. `STORAGE_SERVICE_ENDPOINT` (full endpoint)
    /// 2. `NESTGATE_ENDPOINT` (legacy)
    /// 3. `STORAGE_SERVICE_PORT` or `NESTGATE_PORT` (port override)
    /// 4. Default: <http://localhost:8082>
    #[must_use]
    pub fn storage_endpoint() -> String {
        env::var("STORAGE_SERVICE_ENDPOINT")
            .or_else(|_| env::var("NESTGATE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("STORAGE_SERVICE_PORT")
                    .or_else(|_| env::var("NESTGATE_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8082);
                format!("http://localhost:{port}")
            })
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    ///
    /// Multi-tier resolution:
    /// 1. `SECURITY_SERVICE_ENDPOINT` (full endpoint)
    /// 2. `SECURITY_AUTH_SERVICE_ENDPOINT` (alt full endpoint)
    /// 3. `SECURITY_AUTHENTICATION_PORT` (port override)
    /// 4. Default: <http://localhost:8443>
    #[must_use]
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8443); // Default security auth port
                format!("http://localhost:{port}")
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
            .unwrap_or_else(|_| format!("{}/api/v1/discovery", Self::service_mesh_endpoint()))
    }

    /// Get registration endpoint from environment or default
    #[must_use]
    pub fn registration_endpoint() -> String {
        env::var("REGISTRATION_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/api/v1/register", Self::service_mesh_endpoint()))
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

    /// Get service mesh path for a base URL
    #[must_use]
    pub fn service_mesh_path(base_url: &str) -> String {
        format!("{}/mesh", base_url.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== URL Building Tests ==========

    #[test]
    fn test_health_endpoint() {
        assert_eq!(
            DefaultEndpoints::health_endpoint("http://localhost:8080"),
            "http://localhost:8080/health"
        );
    }

    #[test]
    fn test_health_endpoint_trailing_slash() {
        assert_eq!(
            DefaultEndpoints::health_endpoint("http://localhost:8080/"),
            "http://localhost:8080/health"
        );
    }

    #[test]
    fn test_metrics_endpoint() {
        assert_eq!(
            DefaultEndpoints::metrics_endpoint("http://localhost:8080"),
            "http://localhost:8080/metrics"
        );
    }

    #[test]
    fn test_admin_endpoint() {
        assert_eq!(
            DefaultEndpoints::admin_endpoint("http://localhost:8080"),
            "http://localhost:8080/admin"
        );
    }

    #[test]
    fn test_websocket_endpoint_http() {
        assert_eq!(
            DefaultEndpoints::websocket_endpoint("http://localhost:8080"),
            "ws://localhost:8080/ws"
        );
    }

    #[test]
    fn test_websocket_endpoint_https() {
        assert_eq!(
            DefaultEndpoints::websocket_endpoint("https://example.com"),
            "wss://example.com/ws"
        );
    }

    #[test]
    fn test_websocket_endpoint_trailing_slash() {
        assert_eq!(
            DefaultEndpoints::websocket_endpoint("http://localhost:8080/"),
            "ws://localhost:8080/ws"
        );
    }

    #[test]
    fn test_mcp_endpoint() {
        assert_eq!(
            DefaultEndpoints::mcp_endpoint("http://localhost:8080"),
            "http://localhost:8080/mcp"
        );
    }

    #[test]
    fn test_ai_coordination_endpoint() {
        assert_eq!(
            DefaultEndpoints::ai_coordination_endpoint("http://localhost:8080"),
            "http://localhost:8080/ai"
        );
    }

    #[test]
    fn test_service_mesh_path() {
        assert_eq!(
            DefaultEndpoints::service_mesh_path("http://localhost:8080"),
            "http://localhost:8080/mesh"
        );
    }

    // ========== Environment Override Tests ==========
    // These tests modify environment variables and must run sequentially
    // to avoid race conditions. Combined into a single test function.

    fn clear_all_endpoint_env_vars() {
        for var in &[
            "DEV_BIND_ADDRESS",
            "SERVICE_MESH_ENDPOINT",
            "SERVICE_MESH_PORT",
            "SONGBIRD_ENDPOINT",
            "SONGBIRD_PORT",
            "COMPUTE_SERVICE_ENDPOINT",
            "COMPUTE_SERVICE_PORT",
            "TOADSTOOL_ENDPOINT",
            "TOADSTOOL_PORT",
            "STORAGE_SERVICE_ENDPOINT",
            "STORAGE_SERVICE_PORT",
            "NESTGATE_ENDPOINT",
            "NESTGATE_PORT",
            "SECURITY_SERVICE_ENDPOINT",
            "SECURITY_AUTH_SERVICE_ENDPOINT",
            "SECURITY_AUTHENTICATION_PORT",
            "DISCOVERY_ENDPOINT",
            "REGISTRATION_ENDPOINT",
        ] {
            env::remove_var(var);
        }
    }

    #[test]
    fn test_endpoint_env_overrides() {
        // Run all env-dependent tests sequentially in one test function
        // to prevent parallel env var races.
        clear_all_endpoint_env_vars();

        // --- dev_bind_address ---
        assert_eq!(DefaultEndpoints::dev_bind_address(), "127.0.0.1");

        env::set_var("DEV_BIND_ADDRESS", "0.0.0.0");
        assert_eq!(DefaultEndpoints::dev_bind_address(), "0.0.0.0");
        env::remove_var("DEV_BIND_ADDRESS");

        // --- service_mesh_endpoint ---
        assert_eq!(
            DefaultEndpoints::service_mesh_endpoint(),
            "http://localhost:8500"
        );

        env::set_var("SERVICE_MESH_ENDPOINT", "http://mesh:9000");
        assert_eq!(
            DefaultEndpoints::service_mesh_endpoint(),
            "http://mesh:9000"
        );
        env::remove_var("SERVICE_MESH_ENDPOINT");

        env::set_var("SERVICE_MESH_PORT", "9999");
        assert_eq!(
            DefaultEndpoints::service_mesh_endpoint(),
            "http://localhost:9999"
        );
        env::remove_var("SERVICE_MESH_PORT");

        // --- compute_endpoint ---
        assert_eq!(
            DefaultEndpoints::compute_endpoint(),
            "http://localhost:8081"
        );

        // --- storage_endpoint ---
        assert_eq!(
            DefaultEndpoints::storage_endpoint(),
            "http://localhost:8082"
        );

        // --- security_service_endpoint ---
        assert_eq!(
            DefaultEndpoints::security_service_endpoint(),
            "http://localhost:8443"
        );

        // --- discovery_endpoint ---
        assert_eq!(
            DefaultEndpoints::discovery_endpoint(),
            "http://localhost:8500/api/v1/discovery"
        );

        // --- registration_endpoint ---
        assert_eq!(
            DefaultEndpoints::registration_endpoint(),
            "http://localhost:8500/api/v1/register"
        );

        // Clean up
        clear_all_endpoint_env_vars();
    }
}
