//! Network Configuration Constants
//!
//! All network-related constants used throughout the Squirrel system,
//! consolidated from `crates/core/mcp/src/constants.rs`.
//!
//! # Categories
//!
//! - **Addresses**: Default bind addresses
//! - **Ports**: Default port numbers for various services
//! - **URL Templates**: Templates for constructing URLs

// ============================================================================
// Addresses
// ============================================================================

/// Default bind address for services
///
/// Services will bind to this address by default.
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// Localhost IPv4 address
pub const LOCALHOST_IPV4: &str = "127.0.0.1";

/// Default localhost hostname
pub const DEFAULT_LOCALHOST: &str = "localhost";

// ============================================================================
// Default Ports
// ============================================================================

/// Default WebSocket port (8080)
pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;

/// Default HTTP port (8081)
pub const DEFAULT_HTTP_PORT: u16 = 8081;

/// Default admin port (8082)
pub const DEFAULT_ADMIN_PORT: u16 = 8082;

/// Default metrics port (9090)
pub const DEFAULT_METRICS_PORT: u16 = 9090;

/// Default service discovery port (8500)
pub const DEFAULT_DISCOVERY_PORT: u16 = 8500;

// ============================================================================
// URL Templates
// ============================================================================

/// Default localhost HTTP URL template
///
/// Format: "http://localhost:{port}"
pub const LOCALHOST_HTTP_TEMPLATE: &str = "http://localhost:{}";

/// Default localhost WebSocket URL template
///
/// Format: "ws://localhost:{port}"
pub const LOCALHOST_WS_TEMPLATE: &str = "ws://localhost:{}";

// ============================================================================
// Endpoint Paths
// ============================================================================

/// Health check endpoint path
pub const HEALTH_ENDPOINT: &str = "/health";

/// Metrics endpoint path
pub const METRICS_ENDPOINT: &str = "/metrics";

/// Admin endpoint path
pub const ADMIN_ENDPOINT: &str = "/admin";

/// WebSocket endpoint path
pub const WS_ENDPOINT: &str = "/ws";

/// Service discovery endpoint path
pub const DISCOVERY_ENDPOINT: &str = "/discovery";

/// Registration endpoint path
pub const REGISTRATION_ENDPOINT: &str = "/register";

// ============================================================================
// Helper Functions
// ============================================================================

/// Get port from environment variable or use default
pub fn get_port_from_env(env_var: &str, default: u16) -> u16 {
    std::env::var(env_var)
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default)
}

/// Construct HTTP URL from components
pub fn http_url(host: &str, port: u16, path: &str) -> String {
    if path.is_empty() {
        format!("http://{}:{}", host, port)
    } else {
        format!("http://{}:{}{}", host, port, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addresses() {
        assert_eq!(DEFAULT_BIND_ADDRESS, "127.0.0.1");
        assert_eq!(DEFAULT_LOCALHOST, "localhost");
    }

    #[test]
    fn test_ports() {
        assert_eq!(DEFAULT_WEBSOCKET_PORT, 8080);
        assert_eq!(DEFAULT_HTTP_PORT, 8081);
        assert_eq!(DEFAULT_ADMIN_PORT, 8082);
        assert_eq!(DEFAULT_METRICS_PORT, 9090);
        assert_eq!(DEFAULT_DISCOVERY_PORT, 8500);
    }

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(HEALTH_ENDPOINT, "/health");
        assert_eq!(METRICS_ENDPOINT, "/metrics");
        assert_eq!(ADMIN_ENDPOINT, "/admin");
        assert_eq!(WS_ENDPOINT, "/ws");
    }

    #[test]
    fn test_url_templates() {
        assert_eq!(
            format!("{}", LOCALHOST_HTTP_TEMPLATE.replace("{}", "8080")),
            "http://localhost:8080"
        );
        assert_eq!(
            format!("{}", LOCALHOST_WS_TEMPLATE.replace("{}", "8080")),
            "ws://localhost:8080"
        );
    }
}
