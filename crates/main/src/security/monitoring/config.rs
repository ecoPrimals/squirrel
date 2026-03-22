// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security monitoring configuration
//!
//! Configuration types for the security monitoring system, including
//! thresholds, retention policies, and analysis parameters.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Security monitoring configuration
///
/// Comprehensive configuration for the security monitoring system,
/// controlling monitoring behavior, analysis, and alert thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    /// Enable real-time monitoring
    pub enable_real_time_monitoring: bool,

    /// Event buffer size before flushing
    pub event_buffer_size: usize,

    /// Event retention period
    pub event_retention_period: Duration,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,

    /// Enable behavioral analysis
    pub enable_behavioral_analysis: bool,

    /// Behavioral analysis window
    pub behavioral_window: Duration,

    /// Enable automated response
    pub enable_automated_response: bool,

    /// Maximum events to analyze per batch
    pub max_events_per_batch: usize,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_real_time_monitoring: true,
            event_buffer_size: 1000,
            event_retention_period: Duration::from_secs(24 * 3600), // 24 hours
            alert_thresholds: AlertThresholds::default(),
            enable_behavioral_analysis: true,
            behavioral_window: Duration::from_secs(3600), // 1 hour
            enable_automated_response: true,
            max_events_per_batch: 100,
        }
    }
}

/// Alert threshold configuration
///
/// Defines thresholds that trigger security alerts when exceeded.
/// All thresholds are evaluated per time window (typically hourly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Failed authentication attempts per IP per hour
    pub failed_auth_per_hour: u32,

    /// Rate limit violations per IP per hour
    pub rate_limit_violations_per_hour: u32,

    /// Input validation violations per IP per hour
    pub input_violations_per_hour: u32,

    /// Suspicious activities per IP per hour
    pub suspicious_activities_per_hour: u32,

    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions_per_user: u32,

    /// Maximum failed requests ratio (0.0 to 1.0)
    pub max_failed_requests_ratio: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            failed_auth_per_hour: 10,
            rate_limit_violations_per_hour: 50,
            input_violations_per_hour: 20,
            suspicious_activities_per_hour: 5,
            max_concurrent_sessions_per_user: 5,
            max_failed_requests_ratio: 0.3,
        }
    }
}

impl SecurityMonitoringConfig {
    /// Create a new configuration with custom thresholds
    #[must_use]
    pub const fn with_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.alert_thresholds = thresholds;
        self
    }

    /// Set event buffer size
    #[must_use]
    pub const fn with_buffer_size(mut self, size: usize) -> Self {
        self.event_buffer_size = size;
        self
    }

    /// Set event retention period
    #[must_use]
    pub const fn with_retention_period(mut self, period: Duration) -> Self {
        self.event_retention_period = period;
        self
    }

    /// Set behavioral analysis window
    #[must_use]
    pub const fn with_behavioral_window(mut self, window: Duration) -> Self {
        self.behavioral_window = window;
        self
    }

    /// Enable/disable automated response
    #[must_use]
    pub const fn with_automated_response(mut self, enabled: bool) -> Self {
        self.enable_automated_response = enabled;
        self
    }
}

impl AlertThresholds {
    /// Create aggressive thresholds for high-security environments
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            failed_auth_per_hour: 5,
            rate_limit_violations_per_hour: 20,
            input_violations_per_hour: 10,
            suspicious_activities_per_hour: 2,
            max_concurrent_sessions_per_user: 3,
            max_failed_requests_ratio: 0.15,
        }
    }

    /// Create relaxed thresholds for development environments
    #[must_use]
    pub const fn relaxed() -> Self {
        Self {
            failed_auth_per_hour: 50,
            rate_limit_violations_per_hour: 200,
            input_violations_per_hour: 100,
            suspicious_activities_per_hour: 20,
            max_concurrent_sessions_per_user: 10,
            max_failed_requests_ratio: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityMonitoringConfig::default();
        assert!(config.enable_real_time_monitoring);
        assert_eq!(config.event_buffer_size, 1000);
        assert!(config.enable_behavioral_analysis);
    }

    #[test]
    fn test_config_builder() {
        let config = SecurityMonitoringConfig::default()
            .with_buffer_size(2000)
            .with_automated_response(false)
            .with_behavioral_window(Duration::from_secs(1800));

        assert_eq!(config.event_buffer_size, 2000);
        assert!(!config.enable_automated_response);
        assert_eq!(config.behavioral_window, Duration::from_secs(1800));
    }

    #[test]
    fn test_strict_thresholds() {
        let thresholds = AlertThresholds::strict();
        assert_eq!(thresholds.failed_auth_per_hour, 5);
        assert_eq!(thresholds.max_concurrent_sessions_per_user, 3);
        assert!((thresholds.max_failed_requests_ratio - 0.15).abs() < 0.01);
    }

    #[test]
    fn test_relaxed_thresholds() {
        let thresholds = AlertThresholds::relaxed();
        assert_eq!(thresholds.failed_auth_per_hour, 50);
        assert!(thresholds.max_failed_requests_ratio > 0.4);
    }
}
