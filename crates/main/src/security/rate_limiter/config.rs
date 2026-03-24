// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Rate limiting configuration.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Rate limiting configuration for different endpoint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute for general API endpoints
    pub api_requests_per_minute: u32,

    /// Requests per minute for authentication endpoints
    pub auth_requests_per_minute: u32,

    /// Requests per minute for compute-intensive operations
    pub compute_requests_per_minute: u32,

    /// Maximum burst capacity
    pub burst_capacity: u32,

    /// Ban duration for repeat offenders
    pub ban_duration: Duration,

    /// Threshold for temporary ban (violations in time window)
    pub ban_threshold: u32,

    /// Time window for counting violations
    pub violation_window: Duration,

    /// Enable adaptive rate limiting based on system load
    pub adaptive_limiting: bool,

    /// Whitelist of IPs that bypass rate limiting
    pub whitelist: Vec<IpAddr>,
}

impl RateLimitConfig {
    /// Parse whitelist from `SQUIRREL_RATE_LIMIT_WHITELIST` (comma-separated IPs),
    /// falling back to loopback addresses when the variable is absent.
    fn default_whitelist() -> Vec<IpAddr> {
        if let Ok(env_val) = std::env::var("SQUIRREL_RATE_LIMIT_WHITELIST") {
            return env_val
                .split(',')
                .filter_map(|s| s.trim().parse::<IpAddr>().ok())
                .collect();
        }
        [
            "127.0.0.1".parse::<IpAddr>().ok(),
            "::1".parse::<IpAddr>().ok(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            api_requests_per_minute: 100,
            auth_requests_per_minute: 10,
            compute_requests_per_minute: 20,
            burst_capacity: 150,
            ban_duration: Duration::from_secs(300),
            ban_threshold: 5,
            violation_window: Duration::from_secs(60),
            adaptive_limiting: true,
            whitelist: Self::default_whitelist(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_config_default_values() {
        let c = RateLimitConfig::default();
        assert_eq!(c.api_requests_per_minute, 100);
        assert_eq!(c.auth_requests_per_minute, 10);
        assert_eq!(c.compute_requests_per_minute, 20);
        assert_eq!(c.burst_capacity, 150);
        assert_eq!(c.ban_duration, Duration::from_secs(300));
        assert_eq!(c.ban_threshold, 5);
        assert_eq!(c.violation_window, Duration::from_secs(60));
        assert!(c.adaptive_limiting);
        assert!(
            c.whitelist.iter().any(std::net::IpAddr::is_loopback),
            "expected loopback in whitelist"
        );
    }

    #[test]
    fn rate_limit_config_serde_roundtrip() {
        let c = RateLimitConfig::default();
        let json = serde_json::to_string(&c).expect("should succeed");
        let back: RateLimitConfig = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.api_requests_per_minute, c.api_requests_per_minute);
        assert_eq!(back.whitelist.len(), c.whitelist.len());
    }
}
