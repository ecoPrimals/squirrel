// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Constants for MCP configuration
//!
//! **DEPRECATED**: This module is being phased out in favor of `universal-constants` crate.
//! Please migrate to `universal-constants` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use squirrel_mcp::constants::network;
//! // New:
//! use universal_constants::network;
//! ```
//!
//! This module contains all hardcoded values used throughout the MCP system,
//! centralized for easy maintenance and configuration.

#![deprecated(since = "0.2.0", note = "Use `universal-constants` crate instead")]

use std::time::Duration;

/// Network Configuration Constants
pub mod network {
    /// Default bind address for services
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

    /// Default WebSocket port
    pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;

    /// Default HTTP port
    pub const DEFAULT_HTTP_PORT: u16 = 8081;

    /// Default admin port
    pub const DEFAULT_ADMIN_PORT: u16 = 8082;

    /// Default metrics port
    pub const DEFAULT_METRICS_PORT: u16 = 9090;

    /// Maximum number of connections
    pub const DEFAULT_MAX_CONNECTIONS: usize = 100;

    /// Default service discovery port
    pub const DEFAULT_DISCOVERY_PORT: u16 = 8500;
}

/// Timeout Configuration Constants
pub mod timeouts {
    use super::Duration;

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default ping interval
    pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

    /// Default pong timeout
    pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(10);

    /// Default heartbeat interval
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

    /// Default initial delay
    pub const DEFAULT_INITIAL_DELAY: Duration = Duration::from_millis(1000);

    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(5000);

    /// Default operation timeout
    pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_millis(10000);
}

/// Message Size Configuration Constants
pub mod message_sizes {
    /// Default maximum message size (16MB)
    pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

    /// Default buffer size (8KB)
    pub const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;

    /// Default chunk size (4KB)
    pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024;

    /// Default maximum context length
    pub const DEFAULT_MAX_CONTEXT_LENGTH: usize = 128_000;

    /// Default channel buffer size
    pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 1000;
}

/// Protocol Configuration Constants
pub mod protocol {
    /// Default MCP subprotocol
    pub const DEFAULT_MCP_SUBPROTOCOL: &str = "mcp";

    /// Default protocol version
    pub const DEFAULT_PROTOCOL_VERSION: &str = "1.0";

    /// Default user agent
    pub const DEFAULT_USER_AGENT: &str = "squirrel-mcp/1.0";

    /// Default content type
    pub const DEFAULT_CONTENT_TYPE: &str = "application/json";
}

/// Service Configuration Constants
pub mod services {
    use std::time::Duration;

    /// Default service mesh maximum services
    pub const DEFAULT_MAX_SERVICES: usize = 1000;

    /// Default maximum retries
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Default health check interval
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60);
}

/// URL Templates
pub mod url_templates {
    /// Default localhost HTTP URL template
    pub const LOCALHOST_HTTP_TEMPLATE: &str = "http://localhost:{}";

    /// Default localhost WebSocket URL template
    pub const LOCALHOST_WS_TEMPLATE: &str = "ws://localhost:{}";

    /// Default health check endpoint
    pub const HEALTH_ENDPOINT: &str = "/health";

    /// Default metrics endpoint
    pub const METRICS_ENDPOINT: &str = "/metrics";

    /// Default admin endpoint
    pub const ADMIN_ENDPOINT: &str = "/admin";

    /// Default WebSocket endpoint
    pub const WS_ENDPOINT: &str = "/ws";

    /// Default discovery endpoint
    pub const DISCOVERY_ENDPOINT: &str = "/discovery";

    /// Default registration endpoint
    pub const REGISTRATION_ENDPOINT: &str = "/register";
}

/// Environment Variable Names
pub mod env_vars {
    /// Bind address environment variable
    pub const BIND_ADDRESS: &str = "MCP_BIND_ADDRESS";

    /// WebSocket port environment variable
    pub const WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";

    /// HTTP port environment variable
    pub const HTTP_PORT: &str = "MCP_HTTP_PORT";

    /// Connection timeout environment variable
    pub const CONNECTION_TIMEOUT: &str = "MCP_CONNECTION_TIMEOUT";

    /// Maximum connections environment variable
    pub const MAX_CONNECTIONS: &str = "MCP_MAX_CONNECTIONS";

    /// Maximum message size environment variable
    pub const MAX_MESSAGE_SIZE: &str = "MCP_MAX_MESSAGE_SIZE";

    /// Heartbeat interval environment variable
    pub const HEARTBEAT_INTERVAL: &str = "MCP_HEARTBEAT_INTERVAL";

    /// Service mesh max services environment variable
    pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";
}

/// Helper functions for building common URLs
pub mod url_builders {
    use super::url_templates;
    use universal_constants::network;

    /// Build HTTP URL using centralized host configuration
    #[must_use]
    pub fn localhost_http(port: u16) -> String {
        let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "localhost".to_string());
        format!("http://{host}:{port}")
    }

    /// Build WebSocket URL using centralized host configuration
    #[must_use]
    pub fn localhost_ws(port: u16) -> String {
        let host = std::env::var("MCP_HOST").unwrap_or_else(|_| "localhost".to_string());
        format!("ws://{host}:{port}")
    }

    /// Build health check URL
    #[must_use]
    pub fn health_url(base_url: &str) -> String {
        format!("{}{}", base_url, url_templates::HEALTH_ENDPOINT)
    }

    /// Build metrics URL
    #[must_use]
    pub fn metrics_url(base_url: &str) -> String {
        format!("{}{}", base_url, url_templates::METRICS_ENDPOINT)
    }

    /// Build admin URL
    #[must_use]
    pub fn admin_url(base_url: &str) -> String {
        format!("{}{}", base_url, url_templates::ADMIN_ENDPOINT)
    }

    /// Build WebSocket URL
    #[must_use]
    pub fn ws_url(base_url: &str) -> String {
        format!("{}{}", base_url, url_templates::WS_ENDPOINT)
    }

    /// Build default localhost URLs
    #[must_use]
    pub fn default_localhost_urls() -> (String, String, String, String, String) {
        let http_port = network::get_service_port("http");
        let ws_port = network::get_service_port("websocket");
        let http_url = localhost_http(http_port);
        (
            http_url.clone(),
            health_url(&http_url),
            metrics_url(&http_url),
            admin_url(&http_url),
            ws_url(&localhost_ws(ws_port)),
        )
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::message_sizes;
    use super::network;
    use super::protocol;
    use super::timeouts;
    use super::url_builders;
    use super::url_templates;

    #[test]
    fn network_ports_and_sizes_are_sane() {
        assert_eq!(network::DEFAULT_WEBSOCKET_PORT, 8080);
        assert!(network::DEFAULT_MAX_CONNECTIONS > 0);
        assert_eq!(message_sizes::DEFAULT_CHUNK_SIZE, 4 * 1024);
        assert_eq!(protocol::DEFAULT_MCP_SUBPROTOCOL, "mcp");
    }

    #[test]
    fn timeouts_are_nonzero() {
        assert!(timeouts::DEFAULT_CONNECTION_TIMEOUT.as_secs() > 0);
        assert!(timeouts::DEFAULT_REQUEST_TIMEOUT >= timeouts::DEFAULT_CONNECTION_TIMEOUT);
    }

    #[test]
    fn url_helpers_build_expected_paths() {
        assert!(url_builders::health_url("http://h:1").ends_with(url_templates::HEALTH_ENDPOINT));
        assert!(url_builders::metrics_url("http://h:1").contains("/metrics"));
        let (http, health, metrics, admin, ws) = url_builders::default_localhost_urls();
        assert!(health.contains("/health"));
        assert!(metrics.contains("/metrics"));
        assert!(admin.contains("/admin"));
        assert!(ws.starts_with("ws://"));
        assert!(http.starts_with("http://"));
    }
}
