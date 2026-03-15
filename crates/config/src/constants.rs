// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration constants - re-exports from `universal-constants`
//!
//! This module re-exports constants from the `universal-constants` crate for
//! backward compatibility. New code should use `universal_constants` directly.

/// Re-export timeout constants from universal-constants
pub mod timeouts {
    pub use universal_constants::timeouts::{
        DEFAULT_CONNECTION_TIMEOUT, DEFAULT_DATABASE_TIMEOUT, DEFAULT_HEARTBEAT_INTERVAL,
        DEFAULT_INITIAL_DELAY, DEFAULT_OPERATION_TIMEOUT, DEFAULT_REQUEST_TIMEOUT,
        DEFAULT_RETRY_DELAY,
    };
}

/// Re-export limit constants from universal-constants
pub mod limits {
    pub use universal_constants::limits::{
        DEFAULT_BUFFER_SIZE, DEFAULT_MAX_CONNECTIONS, DEFAULT_MAX_SERVICES,
    };
}

/// Environment variable names (capability-based, re-exported from universal-constants)
pub mod env_vars {
    pub use universal_constants::env_vars::{
        BUFFER_SIZE, CONNECTION_TIMEOUT, DATABASE_TIMEOUT, HEARTBEAT_INTERVAL, INITIAL_DELAY,
        MAX_CONNECTIONS, OPERATION_TIMEOUT, REQUEST_TIMEOUT, SERVICE_MESH_MAX_SERVICES,
    };
}

/// Helper functions for parsing environment variables
pub mod env_helpers {
    use super::limits;
    use super::timeouts;
    use std::env;
    use std::time::Duration;
    use universal_constants::timeouts::duration_to_millis;

    /// Parse timeout from environment variable with default fallback (returns Duration)
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
            .unwrap_or_else(|_| default.to_string())
            .parse::<usize>()
            .unwrap_or(default)
    }

    /// Parse u32 from environment variable with default fallback
    #[must_use]
    pub fn parse_u32(env_var: &str, default: u32) -> u32 {
        env::var(env_var)
            .unwrap_or_else(|_| default.to_string())
            .parse::<u32>()
            .unwrap_or(default)
    }

    /// Get database timeout from environment (returns Duration)
    #[must_use]
    pub fn get_database_timeout() -> Duration {
        parse_timeout_duration(
            super::env_vars::DATABASE_TIMEOUT,
            timeouts::DEFAULT_DATABASE_TIMEOUT,
        )
    }

    /// Get database timeout as milliseconds (legacy compatibility)
    #[must_use]
    pub fn get_database_timeout_ms() -> u64 {
        duration_to_millis(get_database_timeout())
    }

    /// Get heartbeat interval from environment (returns Duration)
    #[must_use]
    pub fn get_heartbeat_interval() -> Duration {
        parse_timeout_duration(
            super::env_vars::HEARTBEAT_INTERVAL,
            timeouts::DEFAULT_HEARTBEAT_INTERVAL,
        )
    }

    /// Get heartbeat interval as milliseconds (legacy compatibility)
    #[must_use]
    pub fn get_heartbeat_interval_ms() -> u64 {
        duration_to_millis(get_heartbeat_interval())
    }

    /// Get initial delay from environment (returns Duration)
    #[must_use]
    pub fn get_initial_delay() -> Duration {
        parse_timeout_duration(
            super::env_vars::INITIAL_DELAY,
            timeouts::DEFAULT_INITIAL_DELAY,
        )
    }

    /// Get initial delay as milliseconds (legacy compatibility)
    #[must_use]
    pub fn get_initial_delay_ms() -> u64 {
        duration_to_millis(get_initial_delay())
    }

    /// Get service mesh max services from environment
    #[must_use]
    pub fn get_service_mesh_max_services() -> usize {
        parse_limit(
            super::env_vars::SERVICE_MESH_MAX_SERVICES,
            limits::DEFAULT_MAX_SERVICES,
        )
    }

    /// Get max connections from environment
    #[must_use]
    pub fn get_max_connections() -> u32 {
        parse_u32(
            super::env_vars::MAX_CONNECTIONS,
            u32::try_from(limits::DEFAULT_MAX_CONNECTIONS).unwrap_or(u32::MAX),
        )
    }

    /// Get buffer size from environment
    #[must_use]
    pub fn get_buffer_size() -> usize {
        parse_limit(super::env_vars::BUFFER_SIZE, limits::DEFAULT_BUFFER_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // TIMEOUT CONSTANTS
    // ====================================================================

    #[test]
    fn test_default_database_timeout() {
        assert_eq!(
            timeouts::DEFAULT_DATABASE_TIMEOUT,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_default_heartbeat_interval() {
        assert_eq!(
            timeouts::DEFAULT_HEARTBEAT_INTERVAL,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_default_initial_delay() {
        assert_eq!(
            timeouts::DEFAULT_INITIAL_DELAY,
            std::time::Duration::from_millis(1000)
        );
    }

    #[test]
    fn test_default_retry_delay() {
        assert_eq!(
            timeouts::DEFAULT_RETRY_DELAY,
            std::time::Duration::from_millis(5000)
        );
    }

    #[test]
    fn test_default_operation_timeout() {
        assert_eq!(
            timeouts::DEFAULT_OPERATION_TIMEOUT,
            std::time::Duration::from_millis(10000)
        );
    }

    #[test]
    fn test_default_connection_timeout() {
        assert_eq!(
            timeouts::DEFAULT_CONNECTION_TIMEOUT,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_default_request_timeout() {
        assert_eq!(
            timeouts::DEFAULT_REQUEST_TIMEOUT,
            std::time::Duration::from_secs(60)
        );
    }

    // ====================================================================
    // LIMIT CONSTANTS
    // ====================================================================

    #[test]
    fn test_default_max_services() {
        assert_eq!(limits::DEFAULT_MAX_SERVICES, 1000);
    }

    #[test]
    fn test_default_buffer_size() {
        assert_eq!(limits::DEFAULT_BUFFER_SIZE, 8192);
    }

    #[test]
    fn test_default_max_connections() {
        assert_eq!(limits::DEFAULT_MAX_CONNECTIONS, 100);
    }

    // ====================================================================
    // ENV VAR NAME CONSTANTS
    // ====================================================================

    #[test]
    fn test_env_var_names() {
        assert_eq!(env_vars::DATABASE_TIMEOUT, "DATABASE_TIMEOUT");
        assert_eq!(env_vars::HEARTBEAT_INTERVAL, "SONGBIRD_HEARTBEAT_INTERVAL");
        assert_eq!(env_vars::INITIAL_DELAY, "SONGBIRD_INITIAL_DELAY_MS");
        assert_eq!(
            env_vars::SERVICE_MESH_MAX_SERVICES,
            "SERVICE_MESH_MAX_SERVICES"
        );
        assert_eq!(env_vars::MAX_CONNECTIONS, "MAX_CONNECTIONS");
        assert_eq!(env_vars::BUFFER_SIZE, "BUFFER_SIZE");
        assert_eq!(env_vars::CONNECTION_TIMEOUT, "MCP_CONNECTION_TIMEOUT");
        assert_eq!(env_vars::REQUEST_TIMEOUT, "REQUEST_TIMEOUT");
        assert_eq!(env_vars::OPERATION_TIMEOUT, "OPERATION_TIMEOUT");
    }

    // ====================================================================
    // ENV HELPER FUNCTIONS
    // ====================================================================

    #[test]
    fn test_parse_timeout_duration_default() {
        let result = env_helpers::parse_timeout_duration(
            "SQUIRREL_TEST_NONEXISTENT_TIMEOUT_VAR",
            std::time::Duration::from_secs(42),
        );
        assert_eq!(result, std::time::Duration::from_secs(42));
    }

    #[test]
    fn test_parse_timeout_duration_from_env() {
        std::env::set_var("SQUIRREL_TEST_PARSE_TIMEOUT_DUR", "99");
        let result = env_helpers::parse_timeout_duration(
            "SQUIRREL_TEST_PARSE_TIMEOUT_DUR",
            std::time::Duration::from_secs(1),
        );
        assert_eq!(result, std::time::Duration::from_secs(99));
        std::env::remove_var("SQUIRREL_TEST_PARSE_TIMEOUT_DUR");
    }

    #[test]
    fn test_parse_timeout_duration_invalid_env() {
        std::env::set_var("SQUIRREL_TEST_PARSE_TIMEOUT_INVALID", "not_a_number");
        let result = env_helpers::parse_timeout_duration(
            "SQUIRREL_TEST_PARSE_TIMEOUT_INVALID",
            std::time::Duration::from_secs(5),
        );
        assert_eq!(result, std::time::Duration::from_secs(5));
        std::env::remove_var("SQUIRREL_TEST_PARSE_TIMEOUT_INVALID");
    }

    #[test]
    fn test_parse_limit_default() {
        let result = env_helpers::parse_limit("SQUIRREL_TEST_NONEXISTENT_LIMIT_VAR", 500);
        assert_eq!(result, 500);
    }

    #[test]
    fn test_parse_limit_from_env() {
        std::env::set_var("SQUIRREL_TEST_PARSE_LIMIT", "2048");
        let result = env_helpers::parse_limit("SQUIRREL_TEST_PARSE_LIMIT", 100);
        assert_eq!(result, 2048);
        std::env::remove_var("SQUIRREL_TEST_PARSE_LIMIT");
    }

    #[test]
    fn test_parse_limit_invalid_env() {
        std::env::set_var("SQUIRREL_TEST_PARSE_LIMIT_BAD", "abc");
        let result = env_helpers::parse_limit("SQUIRREL_TEST_PARSE_LIMIT_BAD", 64);
        assert_eq!(result, 64);
        std::env::remove_var("SQUIRREL_TEST_PARSE_LIMIT_BAD");
    }

    #[test]
    fn test_parse_u32_default() {
        let result = env_helpers::parse_u32("SQUIRREL_TEST_NONEXISTENT_U32_VAR", 77);
        assert_eq!(result, 77);
    }

    #[test]
    fn test_parse_u32_from_env() {
        std::env::set_var("SQUIRREL_TEST_PARSE_U32", "256");
        let result = env_helpers::parse_u32("SQUIRREL_TEST_PARSE_U32", 10);
        assert_eq!(result, 256);
        std::env::remove_var("SQUIRREL_TEST_PARSE_U32");
    }

    #[test]
    fn test_get_database_timeout_default() {
        std::env::remove_var(env_vars::DATABASE_TIMEOUT);
        let result = env_helpers::get_database_timeout();
        assert_eq!(result, timeouts::DEFAULT_DATABASE_TIMEOUT);
    }

    #[test]
    fn test_get_database_timeout_ms_default() {
        std::env::remove_var(env_vars::DATABASE_TIMEOUT);
        let result = env_helpers::get_database_timeout_ms();
        assert_eq!(result, 30_000);
    }

    #[test]
    fn test_get_heartbeat_interval_default() {
        std::env::remove_var(env_vars::HEARTBEAT_INTERVAL);
        let result = env_helpers::get_heartbeat_interval();
        assert_eq!(result, timeouts::DEFAULT_HEARTBEAT_INTERVAL);
    }

    #[test]
    fn test_get_heartbeat_interval_ms_default() {
        std::env::remove_var(env_vars::HEARTBEAT_INTERVAL);
        let result = env_helpers::get_heartbeat_interval_ms();
        assert_eq!(result, 30_000);
    }

    #[test]
    fn test_get_initial_delay_default() {
        std::env::remove_var(env_vars::INITIAL_DELAY);
        let result = env_helpers::get_initial_delay();
        assert_eq!(result, timeouts::DEFAULT_INITIAL_DELAY);
    }

    #[test]
    fn test_get_initial_delay_ms_default() {
        std::env::remove_var(env_vars::INITIAL_DELAY);
        let result = env_helpers::get_initial_delay_ms();
        assert_eq!(result, 1_000);
    }

    #[test]
    fn test_get_service_mesh_max_services_default() {
        std::env::remove_var(env_vars::SERVICE_MESH_MAX_SERVICES);
        let result = env_helpers::get_service_mesh_max_services();
        assert_eq!(result, limits::DEFAULT_MAX_SERVICES);
    }

    #[test]
    fn test_get_max_connections_default() {
        std::env::remove_var(env_vars::MAX_CONNECTIONS);
        let result = env_helpers::get_max_connections();
        assert_eq!(
            result,
            u32::try_from(limits::DEFAULT_MAX_CONNECTIONS).unwrap_or(u32::MAX)
        );
    }

    #[test]
    fn test_get_buffer_size_default() {
        std::env::remove_var(env_vars::BUFFER_SIZE);
        let result = env_helpers::get_buffer_size();
        assert_eq!(result, limits::DEFAULT_BUFFER_SIZE);
    }
}
