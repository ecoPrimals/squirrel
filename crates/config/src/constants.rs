//! Configuration constants for the Squirrel system
//!
//! This module contains all hardcoded configuration values used throughout the system,
//! centralized for easy maintenance and configuration.

/// Default timeout values in seconds
pub mod timeouts {
    /// Default database timeout
    pub const DEFAULT_DATABASE_TIMEOUT: u64 = 30;

    /// Default heartbeat interval
    pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30;

    /// Default initial delay
    pub const DEFAULT_INITIAL_DELAY: u64 = 1000;

    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: u64 = 5000;

    /// Default operation timeout
    pub const DEFAULT_OPERATION_TIMEOUT: u64 = 10000;

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;

    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: u64 = 60;
}

/// Default size and limit values
pub mod limits {
    /// Default maximum services
    pub const DEFAULT_MAX_SERVICES: usize = 1000;

    /// Default buffer size
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;

    /// Default maximum connections
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 100;
}

/// Environment variable names
pub mod env_vars {
    /// Database timeout environment variable
    pub const DATABASE_TIMEOUT: &str = "DATABASE_TIMEOUT";

    /// Heartbeat interval environment variable  
    pub const HEARTBEAT_INTERVAL: &str = "SONGBIRD_HEARTBEAT_INTERVAL";

    /// Initial delay environment variable
    pub const INITIAL_DELAY: &str = "SONGBIRD_INITIAL_DELAY_MS";

    /// Service mesh max services environment variable
    pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";

    /// Max connections environment variable
    pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";

    /// Buffer size environment variable
    pub const BUFFER_SIZE: &str = "BUFFER_SIZE";

    /// Connection timeout environment variable
    pub const CONNECTION_TIMEOUT: &str = "CONNECTION_TIMEOUT";

    /// Request timeout environment variable
    pub const REQUEST_TIMEOUT: &str = "REQUEST_TIMEOUT";

    /// Operation timeout environment variable
    pub const OPERATION_TIMEOUT: &str = "OPERATION_TIMEOUT";
}

/// Helper functions for parsing environment variables
pub mod env_helpers {
    use super::limits;
    use super::timeouts;
    use std::env;

    /// Parse timeout from environment variable with default fallback
    pub fn parse_timeout(env_var: &str, default: u64) -> u64 {
        env::var(env_var)
            .unwrap_or_else(|_| default.to_string())
            .parse::<u64>()
            .unwrap_or(default)
    }

    /// Parse limit from environment variable with default fallback
    pub fn parse_limit(env_var: &str, default: usize) -> usize {
        env::var(env_var)
            .unwrap_or_else(|_| default.to_string())
            .parse::<usize>()
            .unwrap_or(default)
    }

    /// Parse u32 from environment variable with default fallback
    pub fn parse_u32(env_var: &str, default: u32) -> u32 {
        env::var(env_var)
            .unwrap_or_else(|_| default.to_string())
            .parse::<u32>()
            .unwrap_or(default)
    }

    /// Get database timeout from environment
    pub fn get_database_timeout() -> u64 {
        parse_timeout(
            super::env_vars::DATABASE_TIMEOUT,
            timeouts::DEFAULT_DATABASE_TIMEOUT,
        )
    }

    /// Get heartbeat interval from environment
    pub fn get_heartbeat_interval() -> u64 {
        parse_timeout(
            super::env_vars::HEARTBEAT_INTERVAL,
            timeouts::DEFAULT_HEARTBEAT_INTERVAL,
        )
    }

    /// Get initial delay from environment
    pub fn get_initial_delay() -> u64 {
        parse_timeout(
            super::env_vars::INITIAL_DELAY,
            timeouts::DEFAULT_INITIAL_DELAY,
        )
    }

    /// Get service mesh max services from environment
    pub fn get_service_mesh_max_services() -> usize {
        parse_limit(
            super::env_vars::SERVICE_MESH_MAX_SERVICES,
            limits::DEFAULT_MAX_SERVICES,
        )
    }

    /// Get max connections from environment
    pub fn get_max_connections() -> u32 {
        parse_u32(
            super::env_vars::MAX_CONNECTIONS,
            limits::DEFAULT_MAX_CONNECTIONS,
        )
    }

    /// Get buffer size from environment
    pub fn get_buffer_size() -> usize {
        parse_limit(super::env_vars::BUFFER_SIZE, limits::DEFAULT_BUFFER_SIZE)
    }
}
