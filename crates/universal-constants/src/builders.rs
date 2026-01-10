//! URL and Configuration Builder Helpers
//!
//! Helper functions for building URLs, configurations, and parsing
//! environment variables. These were previously scattered across multiple
//! constant modules.
//!
//! # Design
//!
//! These functions help construct common patterns without duplicating logic.

use crate::{env_vars, limits, network, timeouts};
use std::env;
use std::time::Duration;

// ============================================================================
// URL Builders
// ============================================================================

/// Build HTTP URL from host and port
///
/// # Example
/// ```
/// use universal_constants::builders::build_http_url;
/// let url = build_http_url("localhost", 8080);
/// assert_eq!(url, "http://localhost:8080");
/// ```
#[must_use]
pub fn build_http_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}")
}

/// Build WebSocket URL from host and port
///
/// # Example
/// ```
/// use universal_constants::builders::build_ws_url;
/// let url = build_ws_url("localhost", 8080);
/// assert_eq!(url, "ws://localhost:8080");
/// ```
#[must_use]
pub fn build_ws_url(host: &str, port: u16) -> String {
    format!("ws://{host}:{port}")
}

/// Build localhost HTTP URL with default port
#[must_use]
pub fn localhost_http(port: u16) -> String {
    build_http_url(network::DEFAULT_LOCALHOST, port)
}

/// Build localhost WebSocket URL with default port
#[must_use]
pub fn localhost_ws(port: u16) -> String {
    build_ws_url(network::DEFAULT_LOCALHOST, port)
}

/// Build health check URL
#[must_use]
pub fn health_url(base_url: &str) -> String {
    format!("{}{}", base_url, network::HEALTH_ENDPOINT)
}

/// Build metrics URL
#[must_use]
pub fn metrics_url(base_url: &str) -> String {
    format!("{}{}", base_url, network::METRICS_ENDPOINT)
}

/// Build admin URL
#[must_use]
pub fn admin_url(base_url: &str) -> String {
    format!("{}{}", base_url, network::ADMIN_ENDPOINT)
}

/// Build WebSocket endpoint URL
#[must_use]
pub fn ws_url(base_url: &str) -> String {
    format!("{}{}", base_url, network::WS_ENDPOINT)
}

/// Build default localhost URLs (HTTP, health, metrics, admin, WS)
#[must_use]
pub fn default_localhost_urls() -> (String, String, String, String, String) {
    let http_url = localhost_http(network::DEFAULT_HTTP_PORT);
    (
        http_url.clone(),
        health_url(&http_url),
        metrics_url(&http_url),
        admin_url(&http_url),
        ws_url(&localhost_ws(network::DEFAULT_WEBSOCKET_PORT)),
    )
}

// ============================================================================
// Environment Variable Parsers
// ============================================================================

/// Parse timeout from environment variable with default fallback (returns Duration)
///
/// # Example
/// ```
/// use universal_constants::builders::parse_timeout_duration;
/// use universal_constants::timeouts;
/// use std::time::Duration;
///
/// std::env::set_var("TEST_TIMEOUT", "45");
/// let timeout = parse_timeout_duration("TEST_TIMEOUT", timeouts::DEFAULT_CONNECTION_TIMEOUT);
/// assert_eq!(timeout, Duration::from_secs(45));
/// ```
#[must_use]
pub fn parse_timeout_duration(env_var: &str, default: Duration) -> Duration {
    env::var(env_var)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(default, Duration::from_secs)
}

/// Parse limit from environment variable with default fallback
#[must_use]
pub fn parse_limit(env_var: &str, default: usize) -> usize {
    env::var(env_var)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default)
}

/// Parse u32 from environment variable with default fallback
#[must_use]
pub fn parse_u32(env_var: &str, default: u32) -> u32 {
    env::var(env_var)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(default)
}

/// Parse u16 (port) from environment variable with default fallback
#[must_use]
pub fn parse_port(env_var: &str, default: u16) -> u16 {
    env::var(env_var)
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(default)
}

/// Parse boolean from environment variable with default fallback
#[must_use]
pub fn parse_bool(env_var: &str, default: bool) -> bool {
    env::var(env_var)
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

// ============================================================================
// Specific Environment Getters
// ============================================================================

/// Get database timeout from environment
#[must_use]
pub fn get_database_timeout() -> Duration {
    parse_timeout_duration(
        env_vars::DATABASE_TIMEOUT,
        timeouts::DEFAULT_DATABASE_TIMEOUT,
    )
}

/// Get heartbeat interval from environment
#[must_use]
pub fn get_heartbeat_interval() -> Duration {
    parse_timeout_duration(
        env_vars::HEARTBEAT_INTERVAL,
        timeouts::DEFAULT_HEARTBEAT_INTERVAL,
    )
}

/// Get initial delay from environment
#[must_use]
pub fn get_initial_delay() -> Duration {
    parse_timeout_duration(env_vars::INITIAL_DELAY, timeouts::DEFAULT_INITIAL_DELAY)
}

/// Get service mesh max services from environment
#[must_use]
pub fn get_service_mesh_max_services() -> usize {
    parse_limit(
        env_vars::SERVICE_MESH_MAX_SERVICES,
        limits::DEFAULT_MAX_SERVICES,
    )
}

/// Get max connections from environment
#[must_use]
pub fn get_max_connections() -> usize {
    parse_limit(env_vars::MAX_CONNECTIONS, limits::DEFAULT_MAX_CONNECTIONS)
}

/// Get buffer size from environment
#[must_use]
pub fn get_buffer_size() -> usize {
    parse_limit(env_vars::BUFFER_SIZE, limits::DEFAULT_BUFFER_SIZE)
}

/// Get connection timeout from environment
#[must_use]
pub fn get_connection_timeout() -> Duration {
    parse_timeout_duration(
        env_vars::CONNECTION_TIMEOUT,
        timeouts::DEFAULT_CONNECTION_TIMEOUT,
    )
}

/// Get request timeout from environment
#[must_use]
pub fn get_request_timeout() -> Duration {
    parse_timeout_duration(env_vars::REQUEST_TIMEOUT, timeouts::DEFAULT_REQUEST_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builders() {
        assert_eq!(build_http_url("localhost", 8080), "http://localhost:8080");
        assert_eq!(build_ws_url("localhost", 8080), "ws://localhost:8080");
        assert_eq!(localhost_http(8080), "http://localhost:8080");
        assert_eq!(localhost_ws(8080), "ws://localhost:8080");
    }

    #[test]
    fn test_endpoint_builders() {
        let base = "http://localhost:8080";
        assert_eq!(health_url(base), "http://localhost:8080/health");
        assert_eq!(metrics_url(base), "http://localhost:8080/metrics");
        assert_eq!(admin_url(base), "http://localhost:8080/admin");
    }

    #[test]
    fn test_parse_bool() {
        std::env::set_var("TEST_BOOL_TRUE", "true");
        std::env::set_var("TEST_BOOL_FALSE", "false");
        std::env::set_var("TEST_BOOL_1", "1");
        std::env::set_var("TEST_BOOL_0", "0");

        assert!(parse_bool("TEST_BOOL_TRUE", false));
        assert!(!parse_bool("TEST_BOOL_FALSE", true));
        assert!(parse_bool("TEST_BOOL_1", false));
        assert!(!parse_bool("TEST_BOOL_0", true));
        assert!(parse_bool("NONEXISTENT", true)); // Default
    }

    #[test]
    fn test_parse_port() {
        std::env::set_var("TEST_PORT", "9090");
        assert_eq!(parse_port("TEST_PORT", 8080), 9090);
        assert_eq!(parse_port("NONEXISTENT", 8080), 8080);
    }

    #[test]
    fn test_default_localhost_urls() {
        let (http, health, metrics, admin, ws) = default_localhost_urls();
        assert!(http.starts_with("http://"));
        assert!(health.contains("/health"));
        assert!(metrics.contains("/metrics"));
        assert!(admin.contains("/admin"));
        assert!(ws.starts_with("ws://"));
    }
}
