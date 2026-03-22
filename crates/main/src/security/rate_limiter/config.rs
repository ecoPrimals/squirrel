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

impl Default for RateLimitConfig {
    fn default() -> Self {
        // SAFETY: These IP addresses are valid and hardcoded.
        // If parsing fails (which is impossible for these constants),
        // we'll use a safe default empty whitelist.
        let whitelist = vec!["127.0.0.1".parse().ok(), "::1".parse().ok()]
            .into_iter()
            .flatten()
            .collect();

        Self {
            api_requests_per_minute: 100,
            auth_requests_per_minute: 10,
            compute_requests_per_minute: 20,
            burst_capacity: 150,
            ban_duration: Duration::from_secs(300), // 5 minutes
            ban_threshold: 5,
            violation_window: Duration::from_secs(60),
            adaptive_limiting: true,
            whitelist,
        }
    }
}
