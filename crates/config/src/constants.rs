//! Configuration constants for the Squirrel system
//!
//! **DEPRECATED**: This module is being phased out in favor of `universal-constants` crate.
//! Please migrate to `universal-constants` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use squirrel_mcp_config::constants::timeouts;
//! // New:
//! use universal_constants::timeouts;
//! ```
//!
//! This module contains all hardcoded configuration values used throughout the system,
//! centralized for easy maintenance and configuration.

#![deprecated(since = "0.2.0", note = "Use `universal-constants` crate instead")]

use std::time::Duration;

/// Default timeout values using Duration for type safety
pub mod timeouts {
    use super::Duration;

    /// Default database timeout (30 seconds)
    pub const DEFAULT_DATABASE_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default heartbeat interval (30 seconds)
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

    /// Default initial delay (1000 milliseconds)
    pub const DEFAULT_INITIAL_DELAY: Duration = Duration::from_millis(1000);

    /// Default retry delay (5000 milliseconds)
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(5000);

    /// Default operation timeout (10000 milliseconds)
    pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_millis(10000);

    /// Default connection timeout (30 seconds)
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default request timeout (60 seconds)
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
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
    use super::Duration;
    use std::env;

    /// Parse timeout from environment variable with default fallback (returns Duration)
    pub fn parse_timeout_duration(env_var: &str, default: Duration) -> Duration {
        env::var(env_var)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(default)
    }

    /// Parse timeout from environment variable with default fallback (legacy u64 version)
    #[deprecated(note = "Use parse_timeout_duration instead for type-safe Duration")]
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

    /// Get database timeout from environment (returns Duration)
    pub fn get_database_timeout() -> Duration {
        parse_timeout_duration(
            super::env_vars::DATABASE_TIMEOUT,
            timeouts::DEFAULT_DATABASE_TIMEOUT,
        )
    }

    /// Get database timeout as milliseconds (legacy compatibility)
    pub fn get_database_timeout_ms() -> u64 {
        get_database_timeout().as_millis() as u64
    }

    /// Get heartbeat interval from environment (returns Duration)
    pub fn get_heartbeat_interval() -> Duration {
        parse_timeout_duration(
            super::env_vars::HEARTBEAT_INTERVAL,
            timeouts::DEFAULT_HEARTBEAT_INTERVAL,
        )
    }

    /// Get heartbeat interval as milliseconds (legacy compatibility)
    pub fn get_heartbeat_interval_ms() -> u64 {
        get_heartbeat_interval().as_millis() as u64
    }

    /// Get initial delay from environment (returns Duration)
    pub fn get_initial_delay() -> Duration {
        parse_timeout_duration(
            super::env_vars::INITIAL_DELAY,
            timeouts::DEFAULT_INITIAL_DELAY,
        )
    }

    /// Get initial delay as milliseconds (legacy compatibility)
    pub fn get_initial_delay_ms() -> u64 {
        get_initial_delay().as_millis() as u64
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

#[cfg(test)]
#[allow(deprecated)] // Allow deprecated items in tests - testing legacy constants being migrated
mod tests {
    // NOTE: Tests for deprecated constants removed - functionality migrated to universal-constants crate
    // See: crates/universal-constants for current implementation and tests
    // These deprecated items will be removed in a future version once all references are migrated

    // NOTE: All deprecated helper function tests removed
    // These functions are deprecated in favor of universal-constants crate
    // For new code, use: use universal_constants::*;
}
