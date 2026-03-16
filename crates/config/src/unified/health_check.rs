// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Canonical Health Check Configuration
//!
//! Unified health check configuration for general health monitoring across
//! observability, resilience, and monitoring domains.
//!
//! This consolidates health check configs that share the same core pattern:
//! interval-based checks with threshold-based state transitions.
//!
//! # Domain Separation
//!
//! This config is for **general health checks** (timeouts, thresholds, intervals).
//! For **HTTP-specific health checks** (endpoints, status codes, paths), see
//! domain-specific configs in ports, service composition, etc.
//!
//! # Evolutionary Design
//!
//! Optional fields allow different subsystems to adopt additional features:
//! - `auto_recovery`: Used by resilience subsystem
//! - `grace_period`: Used by monitoring subsystem
//!
//! Future evolution: All subsystems may adopt these features.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Canonical health check configuration
///
/// Used across observability, resilience, and monitoring subsystems for
/// general health status tracking with threshold-based state transitions.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use squirrel_mcp_config::unified::HealthCheckConfig;
///
/// let config = HealthCheckConfig {
///     enabled: true,
///     interval: Duration::from_secs(30),
///     timeout: Duration::from_secs(5),
///     failure_threshold: 3,
///     recovery_threshold: 2,
///     auto_recovery: Some(true),
///     grace_period: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Interval between health checks
    pub interval: Duration,

    /// Timeout for health check operations
    pub timeout: Duration,

    /// Number of consecutive failures before marking unhealthy
    ///
    /// Health state transitions from healthy → unhealthy after this many
    /// consecutive check failures.
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy
    ///
    /// Health state transitions from unhealthy → healthy after this many
    /// consecutive check successes.
    ///
    /// Also known as `success_threshold` in some subsystems - functionally equivalent.
    pub recovery_threshold: u32,

    /// Automatically trigger recovery actions on health failures
    ///
    /// Evolutionary feature: Used by resilience subsystem to enable automatic
    /// recovery attempts when health checks fail.
    ///
    /// Future: All subsystems may adopt automatic recovery capabilities.
    #[serde(default)]
    pub auto_recovery: Option<bool>,

    /// Grace period after system startup before health checks begin
    ///
    /// Evolutionary feature: Used by monitoring subsystem to allow systems
    /// time to initialize before health checks start failing.
    ///
    /// Future: All subsystems may benefit from startup grace periods.
    #[serde(default)]
    pub grace_period: Option<Duration>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            auto_recovery: None,
            grace_period: None,
        }
    }
}

impl HealthCheckConfig {
    /// Create a simple health check config (for observability)
    #[must_use]
    pub const fn simple(
        enabled: bool,
        interval: Duration,
        timeout: Duration,
        failure_threshold: u32,
        recovery_threshold: u32,
    ) -> Self {
        Self {
            enabled,
            interval,
            timeout,
            failure_threshold,
            recovery_threshold,
            auto_recovery: None,
            grace_period: None,
        }
    }

    /// Create config with auto-recovery enabled (for resilience)
    #[must_use]
    pub const fn with_auto_recovery(
        enabled: bool,
        interval: Duration,
        timeout: Duration,
        failure_threshold: u32,
        recovery_threshold: u32,
        auto_recovery: bool,
    ) -> Self {
        Self {
            enabled,
            interval,
            timeout,
            failure_threshold,
            recovery_threshold,
            auto_recovery: Some(auto_recovery),
            grace_period: None,
        }
    }

    /// Create config with grace period (for monitoring)
    #[must_use]
    pub const fn with_grace_period(
        enabled: bool,
        interval: Duration,
        timeout: Duration,
        failure_threshold: u32,
        recovery_threshold: u32,
        grace_period: Duration,
    ) -> Self {
        Self {
            enabled,
            interval,
            timeout,
            failure_threshold,
            recovery_threshold,
            auto_recovery: None,
            grace_period: Some(grace_period),
        }
    }

    /// Get success threshold (alias for `recovery_threshold`)
    ///
    /// Some subsystems use `success_threshold` terminology - this provides
    /// compatible access to the same value.
    #[must_use]
    pub const fn success_threshold(&self) -> u32 {
        self.recovery_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.recovery_threshold, 2);
        assert_eq!(config.auto_recovery, None);
        assert_eq!(config.grace_period, None);
    }

    #[test]
    fn test_success_threshold_alias() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.success_threshold(), config.recovery_threshold);
    }

    #[test]
    fn test_simple() {
        let config = HealthCheckConfig::simple(
            false,
            Duration::from_secs(60),
            Duration::from_secs(10),
            5,
            3,
        );
        assert!(!config.enabled);
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_threshold, 3);
        assert_eq!(config.auto_recovery, None);
        assert_eq!(config.grace_period, None);
    }

    #[test]
    fn test_with_auto_recovery() {
        let config = HealthCheckConfig::with_auto_recovery(
            true,
            Duration::from_secs(15),
            Duration::from_secs(3),
            2,
            1,
            true,
        );
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(15));
        assert_eq!(config.timeout, Duration::from_secs(3));
        assert_eq!(config.failure_threshold, 2);
        assert_eq!(config.recovery_threshold, 1);
        assert_eq!(config.auto_recovery, Some(true));
        assert_eq!(config.grace_period, None);
    }

    #[test]
    fn test_with_grace_period() {
        let grace = Duration::from_secs(120);
        let config = HealthCheckConfig::with_grace_period(
            true,
            Duration::from_secs(30),
            Duration::from_secs(5),
            3,
            2,
            grace,
        );
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.recovery_threshold, 2);
        assert_eq!(config.auto_recovery, None);
        assert_eq!(config.grace_period, Some(grace));
    }

    #[test]
    fn test_success_threshold_returns_recovery_threshold() {
        let config =
            HealthCheckConfig::simple(true, Duration::from_secs(30), Duration::from_secs(5), 3, 7);
        assert_eq!(config.success_threshold(), 7);
    }

    #[test]
    fn test_clone() {
        let config = HealthCheckConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.interval, cloned.interval);
        assert_eq!(config.failure_threshold, cloned.failure_threshold);
    }
}
