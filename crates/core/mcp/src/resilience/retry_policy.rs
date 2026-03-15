// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Retry policy trait and implementations
//!
//! This module provides the RetryPolicy trait and StandardRetryPolicy
//! for determining if and how operations should be retried.

use std::time::Duration;
use rand::Rng;

use crate::resilience::resilience_error::ResilienceError;
use crate::resilience::retry::BackoffStrategy;

/// Policy for determining if and how an operation should be retried
pub trait RetryPolicy: Send + Sync {
    /// Determines if a failed operation should be retried
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool;

    /// Determines the backoff duration for a retry attempt
    fn backoff_duration(&self, attempt: usize) -> Duration;
}

/// Standard implementation of the RetryPolicy trait
pub struct StandardRetryPolicy {
    /// Maximum number of retry attempts
    max_retries: usize,
    /// Base delay for retry attempts
    base_delay: Duration,
    /// Maximum delay for any retry attempt
    max_delay: Duration,
    /// Backoff strategy for calculating delays
    backoff_strategy: BackoffStrategy,
    /// Whether to use jitter to avoid retry storms
    use_jitter: bool,
}

impl StandardRetryPolicy {
    /// Creates a new retry policy with the specified parameters
    pub fn new(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
        backoff_strategy: BackoffStrategy,
        use_jitter: bool,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_strategy,
            use_jitter,
        }
    }

    /// Creates a new retry policy with default parameters
    pub fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_strategy: BackoffStrategy::Exponential,
            use_jitter: true,
        }
    }

    /// Creates a retry policy with the specified maximum retries
    pub fn with_max_retries(max_retries: usize) -> Self {
        Self {
            max_retries,
            ..Self::default()
        }
    }

    /// Creates a retry policy with exponential backoff
    pub fn with_exponential_backoff(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_strategy: BackoffStrategy::Exponential,
            use_jitter: true,
        }
    }

    /// Applies jitter to a delay value
    fn apply_jitter(&self, delay: Duration) -> Duration {
        if self.use_jitter {
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.0..1.0);
            let jitter_millis = (delay.as_millis() as f64 * jitter_factor) as u64;
            Duration::from_millis(jitter_millis)
        } else {
            delay
        }
    }
}

impl RetryPolicy for StandardRetryPolicy {
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        match error {
            ResilienceError::CircuitOpen(_) => false,

            ResilienceError::Timeout(_) => true,

            ResilienceError::RateLimit(_) => true,

            ResilienceError::OperationFailed(_) => true,

            ResilienceError::SyncFailed(_) => true,

            ResilienceError::RetryExceeded(_) => false,

            ResilienceError::RecoveryFailed(_) => false,

            ResilienceError::Bulkhead(_) => false,

            ResilienceError::HealthCheck(_) => false,

            ResilienceError::General(_) => true,
        }
    }

    fn backoff_duration(&self, attempt: usize) -> Duration {
        let base_delay = match self.backoff_strategy {
            BackoffStrategy::Constant => self.base_delay,

            BackoffStrategy::Linear => {
                self.base_delay.mul_f32(attempt as f32)
            },

            BackoffStrategy::Exponential => {
                let scale = 2u32.pow(attempt as u32) as f32;
                self.base_delay.mul_f32(scale)
            },

            BackoffStrategy::Fibonacci => {
                let mut a = 1;
                let mut b = 1;
                for _ in 0..attempt {
                    let temp = a;
                    a = b;
                    b = temp + b;
                }
                self.base_delay.mul_f32(a as f32)
            },
        };

        let delay_with_jitter = self.apply_jitter(base_delay);
        if delay_with_jitter > self.max_delay {
            self.max_delay
        } else {
            delay_with_jitter
        }
    }
}
