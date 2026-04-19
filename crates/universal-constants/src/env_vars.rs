// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Environment Variable Names
//!
//! All environment variable names used throughout the Squirrel system,
//! consolidated from:
//! - `crates/config/src/constants.rs`
//! - `crates/core/mcp/src/constants.rs`
//!
//! # Organization
//!
//! Variables are organized by domain for clarity.

// ============================================================================
// Network & Connection
// ============================================================================

/// Bind address environment variable
pub const BIND_ADDRESS: &str = "MCP_BIND_ADDRESS";

/// WebSocket port environment variable
pub const WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";

/// HTTP port environment variable
pub const HTTP_PORT: &str = "MCP_HTTP_PORT";

/// Admin port environment variable
pub const ADMIN_PORT: &str = "MCP_ADMIN_PORT";

/// Metrics port environment variable
pub const METRICS_PORT: &str = "MCP_METRICS_PORT";

/// Maximum connections environment variable
pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";

// ============================================================================
// Timeouts
// ============================================================================

/// Connection timeout environment variable
pub const CONNECTION_TIMEOUT: &str = "MCP_CONNECTION_TIMEOUT";

/// Request timeout environment variable
pub const REQUEST_TIMEOUT: &str = "REQUEST_TIMEOUT";

/// Operation timeout environment variable
pub const OPERATION_TIMEOUT: &str = "OPERATION_TIMEOUT";

/// Database timeout environment variable
pub const DATABASE_TIMEOUT: &str = "DATABASE_TIMEOUT";

/// Heartbeat interval environment variable (capability-first; legacy `SONGBIRD_HEARTBEAT_INTERVAL` still read as fallback in config loader)
pub const HEARTBEAT_INTERVAL: &str = "SERVICE_MESH_HEARTBEAT_INTERVAL";

/// Initial delay environment variable (capability-first; legacy `SONGBIRD_INITIAL_DELAY_MS` still read as fallback in config loader)
pub const INITIAL_DELAY: &str = "SERVICE_MESH_INITIAL_DELAY_MS";

// ============================================================================
// Limits & Sizes
// ============================================================================

/// Maximum message size environment variable
pub const MAX_MESSAGE_SIZE: &str = "MCP_MAX_MESSAGE_SIZE";

/// Buffer size environment variable
pub const BUFFER_SIZE: &str = "BUFFER_SIZE";

/// Service mesh max services environment variable
pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";

// ============================================================================
// Ecosystem Orchestration (legacy name: biomeOS)
// ============================================================================

/// Ecosystem registration URL (capability-first; legacy `BIOMEOS_REGISTRATION_URL` read as fallback)
pub const ECOSYSTEM_REGISTRATION_URL: &str = "ECOSYSTEM_REGISTRATION_URL";

/// Ecosystem health URL (capability-first; legacy `BIOMEOS_HEALTH_URL` read as fallback)
pub const ECOSYSTEM_HEALTH_URL: &str = "ECOSYSTEM_HEALTH_URL";

/// Ecosystem metrics URL (capability-first; legacy `BIOMEOS_METRICS_URL` read as fallback)
pub const ECOSYSTEM_METRICS_URL: &str = "ECOSYSTEM_METRICS_URL";

// ============================================================================
// Feature Flags
// ============================================================================

/// Enable debug mode environment variable
pub const DEBUG_MODE: &str = "SQUIRREL_DEBUG";

/// Enable verbose logging environment variable
pub const VERBOSE_LOGGING: &str = "SQUIRREL_VERBOSE";

/// Log level environment variable
pub const LOG_LEVEL: &str = "RUST_LOG";

#[cfg(test)]
mod tests {
    use super::{
        ADMIN_PORT, BIND_ADDRESS, BUFFER_SIZE, CONNECTION_TIMEOUT, DATABASE_TIMEOUT, DEBUG_MODE,
        ECOSYSTEM_HEALTH_URL, ECOSYSTEM_METRICS_URL, ECOSYSTEM_REGISTRATION_URL,
        HEARTBEAT_INTERVAL, HTTP_PORT, INITIAL_DELAY, LOG_LEVEL, MAX_CONNECTIONS, MAX_MESSAGE_SIZE,
        METRICS_PORT, OPERATION_TIMEOUT, REQUEST_TIMEOUT, SERVICE_MESH_MAX_SERVICES,
        VERBOSE_LOGGING, WEBSOCKET_PORT,
    };

    #[test]
    fn test_network_vars() {
        assert_eq!(BIND_ADDRESS, "MCP_BIND_ADDRESS");
        assert_eq!(WEBSOCKET_PORT, "MCP_WEBSOCKET_PORT");
        assert_eq!(HTTP_PORT, "MCP_HTTP_PORT");
        assert_eq!(ADMIN_PORT, "MCP_ADMIN_PORT");
        assert_eq!(METRICS_PORT, "MCP_METRICS_PORT");
        assert_eq!(MAX_CONNECTIONS, "MAX_CONNECTIONS");
    }

    #[test]
    fn test_timeout_vars() {
        assert_eq!(CONNECTION_TIMEOUT, "MCP_CONNECTION_TIMEOUT");
        assert_eq!(REQUEST_TIMEOUT, "REQUEST_TIMEOUT");
        assert_eq!(OPERATION_TIMEOUT, "OPERATION_TIMEOUT");
        assert_eq!(DATABASE_TIMEOUT, "DATABASE_TIMEOUT");
        assert_eq!(HEARTBEAT_INTERVAL, "SERVICE_MESH_HEARTBEAT_INTERVAL");
        assert_eq!(INITIAL_DELAY, "SERVICE_MESH_INITIAL_DELAY_MS");
    }

    #[test]
    fn test_limit_vars() {
        assert_eq!(MAX_MESSAGE_SIZE, "MCP_MAX_MESSAGE_SIZE");
        assert_eq!(BUFFER_SIZE, "BUFFER_SIZE");
        assert_eq!(SERVICE_MESH_MAX_SERVICES, "SERVICE_MESH_MAX_SERVICES");
    }

    #[test]
    fn test_ecosystem_vars() {
        assert_eq!(ECOSYSTEM_REGISTRATION_URL, "ECOSYSTEM_REGISTRATION_URL");
        assert_eq!(ECOSYSTEM_HEALTH_URL, "ECOSYSTEM_HEALTH_URL");
        assert_eq!(ECOSYSTEM_METRICS_URL, "ECOSYSTEM_METRICS_URL");
    }

    #[test]
    fn test_feature_flag_vars() {
        assert_eq!(DEBUG_MODE, "SQUIRREL_DEBUG");
        assert_eq!(VERBOSE_LOGGING, "SQUIRREL_VERBOSE");
        assert_eq!(LOG_LEVEL, "RUST_LOG");
    }
}
