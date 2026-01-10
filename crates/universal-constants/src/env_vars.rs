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

/// Heartbeat interval environment variable
pub const HEARTBEAT_INTERVAL: &str = "SONGBIRD_HEARTBEAT_INTERVAL";

/// Initial delay environment variable
pub const INITIAL_DELAY: &str = "SONGBIRD_INITIAL_DELAY_MS";

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
// BiomeOS Integration
// ============================================================================

/// `BiomeOS` registration URL environment variable
pub const BIOMEOS_REGISTRATION_URL: &str = "BIOMEOS_REGISTRATION_URL";

/// `BiomeOS` health URL environment variable
pub const BIOMEOS_HEALTH_URL: &str = "BIOMEOS_HEALTH_URL";

/// `BiomeOS` metrics URL environment variable
pub const BIOMEOS_METRICS_URL: &str = "BIOMEOS_METRICS_URL";

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
    use super::*;

    #[test]
    fn test_network_vars() {
        assert_eq!(BIND_ADDRESS, "MCP_BIND_ADDRESS");
        assert_eq!(WEBSOCKET_PORT, "MCP_WEBSOCKET_PORT");
        assert_eq!(HTTP_PORT, "MCP_HTTP_PORT");
    }

    #[test]
    fn test_timeout_vars() {
        assert_eq!(CONNECTION_TIMEOUT, "MCP_CONNECTION_TIMEOUT");
        assert_eq!(REQUEST_TIMEOUT, "REQUEST_TIMEOUT");
        assert_eq!(DATABASE_TIMEOUT, "DATABASE_TIMEOUT");
    }

    #[test]
    fn test_limit_vars() {
        assert_eq!(MAX_MESSAGE_SIZE, "MCP_MAX_MESSAGE_SIZE");
        assert_eq!(BUFFER_SIZE, "BUFFER_SIZE");
        assert_eq!(SERVICE_MESH_MAX_SERVICES, "SERVICE_MESH_MAX_SERVICES");
    }

    #[test]
    fn test_biomeos_vars() {
        assert_eq!(BIOMEOS_REGISTRATION_URL, "BIOMEOS_REGISTRATION_URL");
        assert_eq!(BIOMEOS_HEALTH_URL, "BIOMEOS_HEALTH_URL");
        assert_eq!(BIOMEOS_METRICS_URL, "BIOMEOS_METRICS_URL");
    }
}
