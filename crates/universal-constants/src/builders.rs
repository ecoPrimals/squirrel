// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
    let endpoint = network::HEALTH_ENDPOINT;
    format!("{base_url}{endpoint}")
}

/// Build metrics URL
#[must_use]
pub fn metrics_url(base_url: &str) -> String {
    let endpoint = network::METRICS_ENDPOINT;
    format!("{base_url}{endpoint}")
}

/// Build admin URL
#[must_use]
pub fn admin_url(base_url: &str) -> String {
    let endpoint = network::ADMIN_ENDPOINT;
    format!("{base_url}{endpoint}")
}

/// Build WebSocket endpoint URL
#[must_use]
pub fn ws_url(base_url: &str) -> String {
    let endpoint = network::WS_ENDPOINT;
    format!("{base_url}{endpoint}")
}

/// Build default localhost URLs (HTTP, health, metrics, admin, WS)
#[must_use]
pub fn default_localhost_urls() -> (String, String, String, String, String) {
    let http_url = localhost_http(network::get_service_port("http"));
    (
        http_url.clone(),
        health_url(&http_url),
        metrics_url(&http_url),
        admin_url(&http_url),
        ws_url(&localhost_ws(network::get_service_port("websocket"))),
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
    use super::{
        admin_url, build_http_url, build_ws_url, default_localhost_urls, get_buffer_size,
        get_connection_timeout, get_database_timeout, get_heartbeat_interval, get_initial_delay,
        get_max_connections, get_request_timeout, get_service_mesh_max_services, health_url,
        localhost_http, localhost_ws, metrics_url, parse_bool, parse_limit, parse_port,
        parse_timeout_duration, parse_u32, ws_url,
    };
    use crate::{limits, timeouts};
    use std::time::Duration;

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
        assert_eq!(ws_url(base), "http://localhost:8080/ws");
    }

    #[test]
    fn test_parse_bool_all() {
        // Sequential to avoid env var conflicts
        std::env::set_var("UC_TEST_BOOL_T", "true");
        assert!(parse_bool("UC_TEST_BOOL_T", false));
        std::env::remove_var("UC_TEST_BOOL_T");

        std::env::set_var("UC_TEST_BOOL_F", "false");
        assert!(!parse_bool("UC_TEST_BOOL_F", true));
        std::env::remove_var("UC_TEST_BOOL_F");

        std::env::set_var("UC_TEST_BOOL_1", "1");
        assert!(parse_bool("UC_TEST_BOOL_1", false));
        std::env::remove_var("UC_TEST_BOOL_1");

        std::env::set_var("UC_TEST_BOOL_0", "0");
        assert!(!parse_bool("UC_TEST_BOOL_0", true));
        std::env::remove_var("UC_TEST_BOOL_0");

        std::env::set_var("UC_TEST_BOOL_YES", "yes");
        assert!(parse_bool("UC_TEST_BOOL_YES", false));
        std::env::remove_var("UC_TEST_BOOL_YES");

        std::env::set_var("UC_TEST_BOOL_NO", "no");
        assert!(!parse_bool("UC_TEST_BOOL_NO", true));
        std::env::remove_var("UC_TEST_BOOL_NO");

        std::env::set_var("UC_TEST_BOOL_ON", "on");
        assert!(parse_bool("UC_TEST_BOOL_ON", false));
        std::env::remove_var("UC_TEST_BOOL_ON");

        std::env::set_var("UC_TEST_BOOL_OFF", "off");
        assert!(!parse_bool("UC_TEST_BOOL_OFF", true));
        std::env::remove_var("UC_TEST_BOOL_OFF");

        // Invalid falls back to default
        std::env::set_var("UC_TEST_BOOL_INV", "maybe");
        assert!(parse_bool("UC_TEST_BOOL_INV", true));
        std::env::remove_var("UC_TEST_BOOL_INV");

        assert!(parse_bool("UC_NONEXISTENT_BOOL", true));
        assert!(!parse_bool("UC_NONEXISTENT_BOOL", false));
    }

    #[test]
    fn test_parse_port() {
        std::env::set_var("UC_TEST_PORT", "9090");
        assert_eq!(parse_port("UC_TEST_PORT", 8080), 9090);
        std::env::remove_var("UC_TEST_PORT");
        assert_eq!(parse_port("UC_NONEXISTENT_PORT", 8080), 8080);
    }

    #[test]
    fn test_parse_u32() {
        std::env::set_var("UC_TEST_U32", "42");
        assert_eq!(parse_u32("UC_TEST_U32", 10), 42);
        std::env::remove_var("UC_TEST_U32");
        assert_eq!(parse_u32("UC_NONEXISTENT_U32", 10), 10);
    }

    #[test]
    fn test_parse_limit() {
        std::env::set_var("UC_TEST_LIMIT", "500");
        assert_eq!(parse_limit("UC_TEST_LIMIT", 100), 500);
        std::env::remove_var("UC_TEST_LIMIT");
        assert_eq!(parse_limit("UC_NONEXISTENT_LIMIT", 100), 100);
    }

    #[test]
    fn test_parse_timeout_duration() {
        std::env::set_var("UC_TEST_TIMEOUT", "45");
        let timeout = parse_timeout_duration("UC_TEST_TIMEOUT", Duration::from_secs(30));
        assert_eq!(timeout, Duration::from_secs(45));
        std::env::remove_var("UC_TEST_TIMEOUT");

        let default = parse_timeout_duration("UC_NONEXISTENT_TIMEOUT", Duration::from_secs(30));
        assert_eq!(default, Duration::from_secs(30));
    }

    #[test]
    fn test_specific_env_getters_defaults() {
        // These should return defaults when env vars are not set
        let db_timeout = get_database_timeout();
        assert_eq!(db_timeout, timeouts::DEFAULT_DATABASE_TIMEOUT);

        let hb_interval = get_heartbeat_interval();
        assert_eq!(hb_interval, timeouts::DEFAULT_HEARTBEAT_INTERVAL);

        let initial = get_initial_delay();
        assert_eq!(initial, timeouts::DEFAULT_INITIAL_DELAY);

        let max_svcs = get_service_mesh_max_services();
        assert_eq!(max_svcs, limits::DEFAULT_MAX_SERVICES);

        let max_conn = get_max_connections();
        assert_eq!(max_conn, limits::DEFAULT_MAX_CONNECTIONS);

        let buf_size = get_buffer_size();
        assert_eq!(buf_size, limits::DEFAULT_BUFFER_SIZE);

        let conn_timeout = get_connection_timeout();
        assert_eq!(conn_timeout, timeouts::DEFAULT_CONNECTION_TIMEOUT);

        let req_timeout = get_request_timeout();
        assert_eq!(req_timeout, timeouts::DEFAULT_REQUEST_TIMEOUT);
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
