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
