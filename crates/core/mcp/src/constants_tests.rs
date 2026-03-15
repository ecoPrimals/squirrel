// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for MCP constants module
//!
//! Testing deprecated constants to ensure they work during migration period.

use super::*;

#[cfg(test)]
// Tests deprecated path for backward compatibility
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_network_constants() {
        assert_eq!(network::DEFAULT_BIND_ADDRESS, "127.0.0.1");
        assert_eq!(network::DEFAULT_WEBSOCKET_PORT, 8080);
        assert_eq!(network::DEFAULT_HTTP_PORT, 8081);
        assert_eq!(network::DEFAULT_ADMIN_PORT, 8082);
        assert_eq!(network::DEFAULT_METRICS_PORT, 9090);
        assert_eq!(network::DEFAULT_MAX_CONNECTIONS, 100);
        assert_eq!(network::DEFAULT_DISCOVERY_PORT, 8500);
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(timeouts::DEFAULT_CONNECTION_TIMEOUT.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_REQUEST_TIMEOUT.as_secs(), 60);
        assert_eq!(timeouts::DEFAULT_PING_INTERVAL.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_PONG_TIMEOUT.as_secs(), 10);
        assert_eq!(timeouts::DEFAULT_HEARTBEAT_INTERVAL.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_INITIAL_DELAY.as_millis(), 1000);
        assert_eq!(timeouts::DEFAULT_RETRY_DELAY.as_millis(), 5000);
        assert_eq!(timeouts::DEFAULT_OPERATION_TIMEOUT.as_millis(), 10000);
    }

    #[test]
    fn test_message_size_constants() {
        assert_eq!(message_sizes::DEFAULT_MAX_MESSAGE_SIZE, 16 * 1024 * 1024);
        assert_eq!(message_sizes::DEFAULT_BUFFER_SIZE, 8 * 1024);
        assert_eq!(message_sizes::DEFAULT_CHUNK_SIZE, 4 * 1024);
        assert_eq!(message_sizes::DEFAULT_MAX_CONTEXT_LENGTH, 128_000);
        assert_eq!(message_sizes::DEFAULT_CHANNEL_BUFFER_SIZE, 1000);
    }

    #[test]
    fn test_protocol_constants() {
        assert_eq!(protocol::DEFAULT_MCP_SUBPROTOCOL, "mcp");
        assert_eq!(protocol::DEFAULT_PROTOCOL_VERSION, "1.0");
        assert_eq!(protocol::DEFAULT_USER_AGENT, "squirrel-mcp/1.0");
        assert_eq!(protocol::DEFAULT_CONTENT_TYPE, "application/json");
    }

    #[test]
    fn test_service_constants() {
        assert_eq!(services::DEFAULT_MAX_SERVICES, 1000);
        assert_eq!(services::DEFAULT_MAX_RETRIES, 3);
        assert_eq!(services::DEFAULT_HEALTH_CHECK_INTERVAL.as_secs(), 30);
        assert_eq!(services::DEFAULT_MONITORING_INTERVAL.as_secs(), 60);
    }

    #[test]
    fn test_url_templates() {
        assert_eq!(
            url_templates::LOCALHOST_HTTP_TEMPLATE,
            "http://localhost:{}"
        );
        assert_eq!(url_templates::LOCALHOST_WS_TEMPLATE, "ws://localhost:{}");
        assert_eq!(url_templates::HEALTH_ENDPOINT, "/health");
        assert_eq!(url_templates::METRICS_ENDPOINT, "/metrics");
        assert_eq!(url_templates::ADMIN_ENDPOINT, "/admin");
        assert_eq!(url_templates::WS_ENDPOINT, "/ws");
        assert_eq!(url_templates::DISCOVERY_ENDPOINT, "/discovery");
        assert_eq!(url_templates::REGISTRATION_ENDPOINT, "/register");
    }

    #[test]
    fn test_env_var_names() {
        assert_eq!(env_vars::BIND_ADDRESS, "MCP_BIND_ADDRESS");
        assert_eq!(env_vars::WEBSOCKET_PORT, "MCP_WEBSOCKET_PORT");
        assert_eq!(env_vars::HTTP_PORT, "MCP_HTTP_PORT");
        assert_eq!(env_vars::CONNECTION_TIMEOUT, "MCP_CONNECTION_TIMEOUT");
        assert_eq!(env_vars::MAX_CONNECTIONS, "MCP_MAX_CONNECTIONS");
        assert_eq!(env_vars::MAX_MESSAGE_SIZE, "MCP_MAX_MESSAGE_SIZE");
        assert_eq!(env_vars::HEARTBEAT_INTERVAL, "MCP_HEARTBEAT_INTERVAL");
        assert_eq!(
            env_vars::SERVICE_MESH_MAX_SERVICES,
            "SERVICE_MESH_MAX_SERVICES"
        );
    }

    #[test]
    fn test_url_builders() {
        // Test with default localhost
        let http_url = url_builders::localhost_http(8080);
        assert!(http_url.starts_with("http://"));
        assert!(http_url.contains(":8080"));

        let ws_url = url_builders::localhost_ws(9000);
        assert!(ws_url.starts_with("ws://"));
        assert!(ws_url.contains(":9000"));

        // Test URL building helpers
        let base = "http://example.com";
        assert_eq!(url_builders::health_url(base), "http://example.com/health");
        assert_eq!(
            url_builders::metrics_url(base),
            "http://example.com/metrics"
        );
        assert_eq!(url_builders::admin_url(base), "http://example.com/admin");
        assert_eq!(url_builders::ws_url(base), "http://example.com/ws");
    }

    #[test]
    fn test_default_localhost_urls() {
        let (http, health, metrics, admin, ws) = url_builders::default_localhost_urls();

        assert!(http.starts_with("http://"));
        assert!(http.contains(":8081"));

        assert!(health.contains("/health"));
        assert!(metrics.contains("/metrics"));
        assert!(admin.contains("/admin"));

        assert!(ws.starts_with("ws://"));
        assert!(ws.contains(":8080"));
    }
}
