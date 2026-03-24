// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Public and internal types for rate limiting.

use serde::Serialize;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Token bucket state for a client's request counting.
#[derive(Debug)]
pub struct ClientRequestCounter {
    /// Current number of tokens
    pub tokens: f64,
    /// Maximum number of tokens
    pub capacity: f64,
    /// Token refill rate per second
    pub refill_rate: f64,
    /// Last refill timestamp
    pub last_refill: Instant,
    /// Request count in current window
    pub request_count: u32,
    /// Window start time
    pub window_start: Instant,
}

impl Default for ClientRequestCounter {
    fn default() -> Self {
        Self {
            tokens: 0.0,
            capacity: 0.0,
            refill_rate: 0.0,
            last_refill: Instant::now(),
            request_count: 0,
            window_start: Instant::now(),
        }
    }
}

/// Rate limiting result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request was allowed
    pub allowed: bool,
    /// Human-readable reason if blocked
    pub reason: Option<String>,
    /// Suggested retry delay if rate limited
    pub retry_after: Option<Duration>,
    /// Remaining tokens in the bucket (if applicable)
    pub remaining_tokens: Option<u32>,
    /// Whether the client is temporarily banned
    pub client_banned: bool,
}

/// Endpoint classification for different rate limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointType {
    /// General API endpoints
    Api,
    /// Authentication endpoints (stricter limits)
    Authentication,
    /// Compute-intensive operations
    Compute,
    /// Health check endpoints (more lenient)
    HealthCheck,
    /// Administrative endpoints (most restrictive)
    Admin,
}

/// Rate limiting statistics
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitStatistics {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests blocked due to rate limiting
    pub blocked_requests: u64,
    /// Number of clients currently banned
    pub banned_clients: u64,
    /// Count of suspicious activities detected
    pub suspicious_activities: u64,
    /// Number of active (tracked) clients
    pub active_clients: usize,
    /// Average requests per second
    pub requests_per_second: f64,
    /// Fraction of requests blocked (0.0 to 1.0)
    pub block_rate: f64,
    /// Time since rate limiter started
    pub uptime: Duration,
    /// Current adaptive rate multiplier
    pub adaptive_rate_multiplier: f64,
    /// System CPU usage (0.0 to 1.0)
    pub system_cpu_usage: f64,
    /// System memory usage (0.0 to 1.0)
    pub system_memory_usage: f64,
}

/// Security violation tracking
#[derive(Debug, Clone)]
pub(crate) struct SecurityViolation {
    pub(crate) timestamp: Instant,
    pub(crate) violation_type: ViolationType,
    pub(crate) severity: ViolationSeverity,
    pub(crate) details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ViolationType {
    RateLimitExceeded,
    SuspiciousActivity,
    RepeatedViolations,
    MaliciousRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Client tracking information
#[derive(Debug, Clone)]
pub(crate) struct ClientInfo {
    pub(crate) ip_address: IpAddr,
    pub(crate) user_agent: Option<String>,
    pub(crate) first_seen: Instant,
    pub(crate) last_activity: Instant,
    pub(crate) total_requests: u64,
    pub(crate) violations: Vec<SecurityViolation>,
    pub(crate) is_banned: bool,
    pub(crate) ban_expires_at: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct GlobalRateLimitMetrics {
    pub(crate) total_requests: u64,
    pub(crate) blocked_requests: u64,
    pub(crate) banned_clients: u64,
    pub(crate) suspicious_activities: u64,
    pub(crate) last_reset: Instant,
}

impl Default for GlobalRateLimitMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            blocked_requests: 0,
            banned_clients: 0,
            suspicious_activities: 0,
            last_reset: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AdaptiveRateLimitState {
    pub(crate) system_load: f64,
    pub(crate) active_connections: u32,
    pub(crate) memory_usage: f64,
    pub(crate) cpu_usage: f64,
    pub(crate) rate_multiplier: f64, // 1.0 = normal, < 1.0 = stricter, > 1.0 = more lenient
    pub(crate) last_update: Instant,
}

impl Default for AdaptiveRateLimitState {
    fn default() -> Self {
        Self {
            system_load: 0.0,
            active_connections: 0,
            memory_usage: 0.0,
            cpu_usage: 0.0,
            rate_multiplier: 1.0,
            last_update: Instant::now(),
        }
    }
}
