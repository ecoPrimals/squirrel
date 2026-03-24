// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience component builder
//!
//! This module provides a builder for composing resilience components.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::resilience::bulkhead::Bulkhead;
use crate::resilience::rate_limiter::RateLimiter;
use crate::resilience::circuit_breaker::StandardCircuitBreaker;
use super::retry_policy::RetryPolicy;

/// A builder for resilience components
pub struct ResilienceBuilder {
    bulkhead: Option<Arc<Bulkhead>>,
    rate_limiter: Option<Arc<RateLimiter>>,
    circuit_breaker: Option<StandardCircuitBreaker>,
    retry_policy: Option<Box<dyn RetryPolicy + Send + Sync>>,
    timeout: Option<Duration>,
}

impl fmt::Debug for ResilienceBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResilienceBuilder")
            .field("bulkhead", &format_args!("<bulkhead>"))
            .field("rate_limiter", &format_args!("<rate_limiter>"))
            .field("circuit_breaker", &format_args!("<circuit_breaker>"))
            .field("retry_policy", &format_args!("<retry_policy>"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Clone for ResilienceBuilder {
    fn clone(&self) -> Self {
        Self {
            bulkhead: self.bulkhead.clone(),
            rate_limiter: self.rate_limiter.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
            retry_policy: None,
            timeout: self.timeout,
        }
    }
}

impl ResilienceBuilder {
    pub fn new() -> Self {
        Self {
            bulkhead: None,
            rate_limiter: None,
            circuit_breaker: None,
            retry_policy: None,
            timeout: None,
        }
    }

    pub fn with_bulkhead(mut self, bulkhead: Arc<Bulkhead>) -> Self {
        self.bulkhead = Some(bulkhead);
        self
    }

    pub fn with_rate_limiter(mut self, rate_limiter: Arc<RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    pub fn with_circuit_breaker(mut self, circuit_breaker: StandardCircuitBreaker) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: Box<dyn RetryPolicy + Send + Sync>) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
