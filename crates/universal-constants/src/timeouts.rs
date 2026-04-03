// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Timeout and Duration Constants
//!
//! All timeout values used throughout the Squirrel system, consolidated from:
//! - `crates/config/src/constants.rs`
//! - `crates/core/mcp/src/constants.rs`
//!
//! # Design
//!
//! All timeout values use `std::time::Duration` for type safety and clarity.
//! Avoid raw u64 millisecond values where possible.

use std::time::Duration;

// ============================================================================
// Connection & Network Timeouts
// ============================================================================

/// Default connection timeout (30 seconds)
///
/// Used for establishing connections to services, databases, and external APIs.
pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default request timeout (60 seconds)
///
/// Used for HTTP requests and RPC calls.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Default operation timeout (10 seconds)
///
/// Used for internal operations that should complete quickly.
pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);

// ============================================================================
// Heartbeat & Health Check Timeouts
// ============================================================================

/// Default heartbeat interval (30 seconds)
///
/// How often to send heartbeat/keepalive messages.
pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Default ping interval (30 seconds)
///
/// WebSocket ping interval.
pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

/// Default pong timeout (10 seconds)
///
/// How long to wait for pong response before considering connection dead.
pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(10);

/// Default health check interval (30 seconds)
///
/// How often to perform health checks on services.
pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Default monitoring interval (60 seconds)
///
/// How often to collect and report monitoring metrics.
pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60);

// ============================================================================
// Retry & Backoff Timeouts
// ============================================================================

/// Default initial delay (1 second)
///
/// Initial delay before first retry attempt.
pub const DEFAULT_INITIAL_DELAY: Duration = Duration::from_secs(1);

/// Default retry delay (5 seconds)
///
/// Delay between retry attempts.
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(5);

// ============================================================================
// Database Timeouts
// ============================================================================

/// Default database timeout (30 seconds)
///
/// Timeout for database operations.
pub const DEFAULT_DATABASE_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Context TTL
// ============================================================================

/// Default context TTL in seconds (1 hour)
///
/// Time-to-live for stored contexts before expiration.
pub const DEFAULT_CONTEXT_TTL_SECS: u64 = 3600;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Duration to milliseconds as u64
///
/// Useful for APIs that require milliseconds.
///
/// # Panics
///
/// Truncation occurs if duration exceeds `u64::MAX` milliseconds (~584 million years).
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "Duration.as_millis() fits u64 for realistic durations; panics doc covers overflow"
)]
pub const fn duration_to_millis(duration: Duration) -> u64 {
    duration.as_millis() as u64
}

/// Convert Duration to seconds as u64
///
/// Useful for APIs that require seconds.
#[must_use]
pub const fn duration_to_secs(duration: Duration) -> u64 {
    duration.as_secs()
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_CONNECTION_TIMEOUT, DEFAULT_CONTEXT_TTL_SECS, DEFAULT_DATABASE_TIMEOUT,
        DEFAULT_HEALTH_CHECK_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_INITIAL_DELAY,
        DEFAULT_MONITORING_INTERVAL, DEFAULT_OPERATION_TIMEOUT, DEFAULT_PING_INTERVAL,
        DEFAULT_PONG_TIMEOUT, DEFAULT_REQUEST_TIMEOUT, DEFAULT_RETRY_DELAY, duration_to_millis,
        duration_to_secs,
    };

    #[test]
    fn test_timeout_values() {
        assert_eq!(DEFAULT_CONNECTION_TIMEOUT.as_secs(), 30);
        assert_eq!(DEFAULT_REQUEST_TIMEOUT.as_secs(), 60);
        assert_eq!(DEFAULT_OPERATION_TIMEOUT.as_secs(), 10);
    }

    #[test]
    fn test_heartbeat_values() {
        assert_eq!(DEFAULT_HEARTBEAT_INTERVAL.as_secs(), 30);
        assert_eq!(DEFAULT_PING_INTERVAL.as_secs(), 30);
        assert_eq!(DEFAULT_PONG_TIMEOUT.as_secs(), 10);
    }

    #[test]
    fn test_health_check_and_monitoring() {
        assert_eq!(DEFAULT_HEALTH_CHECK_INTERVAL.as_secs(), 30);
        assert_eq!(DEFAULT_MONITORING_INTERVAL.as_secs(), 60);
    }

    #[test]
    fn test_retry_values() {
        assert_eq!(DEFAULT_INITIAL_DELAY.as_secs(), 1);
        assert_eq!(DEFAULT_RETRY_DELAY.as_secs(), 5);
    }

    #[test]
    fn test_database_timeout() {
        assert_eq!(DEFAULT_DATABASE_TIMEOUT.as_secs(), 30);
    }

    #[test]
    fn test_context_ttl() {
        assert_eq!(DEFAULT_CONTEXT_TTL_SECS, 3600);
    }

    #[test]
    fn test_duration_helpers() {
        assert_eq!(duration_to_millis(DEFAULT_CONNECTION_TIMEOUT), 30_000);
        assert_eq!(duration_to_secs(DEFAULT_CONNECTION_TIMEOUT), 30);
        assert_eq!(duration_to_millis(DEFAULT_OPERATION_TIMEOUT), 10_000);
        assert_eq!(duration_to_secs(DEFAULT_REQUEST_TIMEOUT), 60);
    }
}
