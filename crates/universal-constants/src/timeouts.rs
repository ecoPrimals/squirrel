// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
// Legacy Compatibility (milliseconds as u64)
// ============================================================================

/// Legacy: Connection timeout in milliseconds
#[deprecated(note = "Use DEFAULT_CONNECTION_TIMEOUT (Duration) instead")]
pub const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 30_000;

/// Legacy: Request timeout in milliseconds
#[deprecated(note = "Use DEFAULT_REQUEST_TIMEOUT (Duration) instead")]
pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 60_000;

/// Legacy: Operation timeout in milliseconds
#[deprecated(note = "Use DEFAULT_OPERATION_TIMEOUT (Duration) instead")]
pub const DEFAULT_OPERATION_TIMEOUT_MS: u64 = 10_000;

/// Legacy: Heartbeat interval in milliseconds
#[deprecated(note = "Use DEFAULT_HEARTBEAT_INTERVAL (Duration) instead")]
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 30_000;

/// Legacy: Initial delay in milliseconds
#[deprecated(note = "Use DEFAULT_INITIAL_DELAY (Duration) instead")]
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 1_000;

/// Legacy: Retry delay in milliseconds
#[deprecated(note = "Use DEFAULT_RETRY_DELAY (Duration) instead")]
pub const DEFAULT_RETRY_DELAY_MS: u64 = 5_000;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Duration to milliseconds as u64
///
/// Useful for APIs that require milliseconds.
#[must_use]
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
#[allow(deprecated)]
mod tests {
    use super::*;

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
    fn test_legacy_ms_values() {
        assert_eq!(DEFAULT_CONNECTION_TIMEOUT_MS, 30_000);
        assert_eq!(DEFAULT_REQUEST_TIMEOUT_MS, 60_000);
        assert_eq!(DEFAULT_OPERATION_TIMEOUT_MS, 10_000);
        assert_eq!(DEFAULT_HEARTBEAT_INTERVAL_MS, 30_000);
        assert_eq!(DEFAULT_INITIAL_DELAY_MS, 1_000);
        assert_eq!(DEFAULT_RETRY_DELAY_MS, 5_000);
    }

    #[test]
    fn test_duration_helpers() {
        assert_eq!(duration_to_millis(DEFAULT_CONNECTION_TIMEOUT), 30_000);
        assert_eq!(duration_to_secs(DEFAULT_CONNECTION_TIMEOUT), 30);
        assert_eq!(duration_to_millis(DEFAULT_OPERATION_TIMEOUT), 10_000);
        assert_eq!(duration_to_secs(DEFAULT_REQUEST_TIMEOUT), 60);
    }
}
