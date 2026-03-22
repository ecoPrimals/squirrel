// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Token bucket state and refill logic.

use std::time::{Duration, Instant};

/// Rate limit bucket for token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimitBucket {
    /// Current number of tokens
    pub(crate) tokens: f64,
    /// Maximum number of tokens
    pub(crate) capacity: f64,
    /// Token refill rate per second
    pub(crate) refill_rate: f64,
    /// Last refill timestamp
    pub(crate) last_refill: Instant,
    /// Request count in current window
    pub(crate) request_count: u32,
    /// Window start time
    pub(crate) window_start: Instant,
}

impl RateLimitBucket {
    pub(crate) fn new(capacity: u32, refill_rate: u32) -> Self {
        let now = Instant::now();
        Self {
            tokens: f64::from(capacity),
            capacity: f64::from(capacity),
            refill_rate: f64::from(refill_rate) / 60.0, // Convert per minute to per second
            last_refill: now,
            request_count: 0,
            window_start: now,
        }
    }

    /// Try to consume a token, returns true if allowed
    pub(crate) fn try_consume(&mut self, tokens_needed: f64) -> bool {
        self.refill_tokens();

        if self.tokens >= tokens_needed {
            self.tokens -= tokens_needed;
            self.request_count += 1;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Add tokens based on refill rate
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;

        // Reset request count if window expired
        if now.duration_since(self.window_start) >= Duration::from_secs(60) {
            self.request_count = 0;
            self.window_start = now;
        }
    }
}
