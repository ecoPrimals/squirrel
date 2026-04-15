// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Retry policy trait and implementations
//!
//! This module provides the RetryPolicy trait and StandardRetryPolicy
//! for determining if and how operations should be retried.

use rand::Rng;
use std::time::Duration;

use crate::resilience::resilience_error::ResilienceError;
use crate::resilience::retry::BackoffStrategy;

/// Retry limits parsed from `SQUIRREL_RETRY_*` / `IPC_RETRY_*` (primal-first, then ecosystem).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryEnvParams {
    /// Maximum retries (same field as [`StandardRetryPolicy`] internal limit).
    pub max_retries: usize,
    /// Base delay between retries.
    pub base_delay: Duration,
    /// Upper bound on per-attempt delay (after backoff).
    pub max_delay: Duration,
}

/// Read retry timing from environment variables with primal → IPC → default precedence.
#[must_use]
pub fn retry_env_params() -> RetryEnvParams {
    let max_retries = std::env::var("SQUIRREL_RETRY_MAX_ATTEMPTS")
        .or_else(|_| std::env::var("IPC_RETRY_MAX_ATTEMPTS"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);

    let base_delay_ms: u64 = std::env::var("SQUIRREL_RETRY_BASE_DELAY_MS")
        .or_else(|_| std::env::var("IPC_RETRY_BASE_DELAY_MS"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let max_delay_ms: u64 = std::env::var("SQUIRREL_RETRY_MAX_DELAY_MS")
        .or_else(|_| std::env::var("IPC_RETRY_MAX_DELAY_MS"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    RetryEnvParams {
        max_retries,
        base_delay: Duration::from_millis(base_delay_ms),
        max_delay: Duration::from_millis(max_delay_ms),
    }
}

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
    #[must_use]
    pub const fn new(
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

    /// Creates a retry policy with the specified maximum retries
    #[must_use]
    pub fn with_max_retries(max_retries: usize) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Creates a retry policy with exponential backoff
    #[must_use]
    pub const fn with_exponential_backoff(
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

    /// Create a retry policy from environment variables, with sensible defaults.
    ///
    /// Environment variables:
    /// - `SQUIRREL_RETRY_MAX_ATTEMPTS` — max retry attempts (default: 3)
    /// - `SQUIRREL_RETRY_BASE_DELAY_MS` — base delay in milliseconds (default: 100)
    /// - `SQUIRREL_RETRY_MAX_DELAY_MS` — max delay in milliseconds (default: 5000)
    /// - `IPC_RETRY_MAX_ATTEMPTS` — ecosystem-wide fallback (default: 3)
    /// - `IPC_RETRY_BASE_DELAY_MS` — ecosystem-wide fallback (default: 100)
    /// - `IPC_RETRY_MAX_DELAY_MS` — ecosystem-wide fallback (default: 5000)
    #[must_use]
    pub fn from_env() -> Self {
        let p = retry_env_params();
        Self::with_exponential_backoff(p.max_retries, p.base_delay, p.max_delay)
    }

    /// Applies jitter to a delay value
    fn apply_jitter(&self, delay: Duration) -> Duration {
        if self.use_jitter {
            let mut rng = rand::rng();
            let millis = delay.as_millis();
            let numer = rng.random_range(0u128..100u128);
            let jitter_scaled = millis.saturating_mul(numer).saturating_div(100);
            let jitter_millis =
                u64::try_from(jitter_scaled.min(u128::from(u64::MAX))).unwrap_or(u64::MAX);
            Duration::from_millis(jitter_millis)
        } else {
            delay
        }
    }
}

impl Default for StandardRetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_strategy: BackoffStrategy::Exponential,
            use_jitter: true,
        }
    }
}

impl RetryPolicy for StandardRetryPolicy {
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        match error {
            ResilienceError::CircuitOpen(_)
            | ResilienceError::RetryExceeded(_)
            | ResilienceError::RecoveryFailed(_)
            | ResilienceError::Bulkhead(_)
            | ResilienceError::HealthCheck(_) => false,

            ResilienceError::Timeout(_)
            | ResilienceError::RateLimit(_)
            | ResilienceError::OperationFailed(_)
            | ResilienceError::SyncFailed(_)
            | ResilienceError::General(_) => true,
        }
    }

    fn backoff_duration(&self, attempt: usize) -> Duration {
        let base_delay = match self.backoff_strategy {
            BackoffStrategy::Constant => self.base_delay,

            BackoffStrategy::Linear => self.base_delay.mul_f32(attempt as f32),

            BackoffStrategy::Exponential => {
                let scale = 2u32.pow(attempt as u32) as f32;
                self.base_delay.mul_f32(scale)
            }

            BackoffStrategy::Fibonacci => {
                let mut a = 1;
                let mut b = 1;
                for _ in 0..attempt {
                    let temp = a;
                    a = b;
                    b += temp;
                }
                self.base_delay.mul_f32(a as f32)
            }
        };

        let delay_with_jitter = self.apply_jitter(base_delay);
        if delay_with_jitter > self.max_delay {
            self.max_delay
        } else {
            delay_with_jitter
        }
    }
}

#[cfg(test)]
impl StandardRetryPolicy {
    pub(crate) fn test_max_retries(&self) -> usize {
        self.max_retries
    }

    pub(crate) fn test_base_delay(&self) -> Duration {
        self.base_delay
    }

    pub(crate) fn test_max_delay(&self) -> Duration {
        self.max_delay
    }
}

#[cfg(test)]
mod from_env_tests {
    use super::{RetryEnvParams, StandardRetryPolicy, retry_env_params};
    use std::time::Duration;

    const RETRY_ENV_VARS: [&str; 6] = [
        "SQUIRREL_RETRY_MAX_ATTEMPTS",
        "SQUIRREL_RETRY_BASE_DELAY_MS",
        "SQUIRREL_RETRY_MAX_DELAY_MS",
        "IPC_RETRY_MAX_ATTEMPTS",
        "IPC_RETRY_BASE_DELAY_MS",
        "IPC_RETRY_MAX_DELAY_MS",
    ];

    #[test]
    fn from_env_uses_defaults_when_no_env_vars_set() {
        temp_env::with_vars_unset(RETRY_ENV_VARS, || {
            let p = retry_env_params();
            assert_eq!(
                p,
                RetryEnvParams {
                    max_retries: 3,
                    base_delay: Duration::from_millis(100),
                    max_delay: Duration::from_millis(5000),
                }
            );
            let policy = StandardRetryPolicy::from_env();
            assert_eq!(policy.test_max_retries(), 3);
            assert_eq!(policy.test_base_delay(), Duration::from_millis(100));
            assert_eq!(policy.test_max_delay(), Duration::from_millis(5000));
        });
    }

    #[test]
    fn primal_specific_env_takes_precedence_over_ipc() {
        temp_env::with_vars(
            [
                ("SQUIRREL_RETRY_MAX_ATTEMPTS", Some("7")),
                ("IPC_RETRY_MAX_ATTEMPTS", Some("2")),
                ("SQUIRREL_RETRY_BASE_DELAY_MS", Some("50")),
                ("IPC_RETRY_BASE_DELAY_MS", Some("200")),
                ("SQUIRREL_RETRY_MAX_DELAY_MS", Some("9000")),
                ("IPC_RETRY_MAX_DELAY_MS", Some("1000")),
            ],
            || {
                let p = retry_env_params();
                assert_eq!(p.max_retries, 7);
                assert_eq!(p.base_delay, Duration::from_millis(50));
                assert_eq!(p.max_delay, Duration::from_millis(9000));
            },
        );
    }

    #[test]
    fn ipc_env_used_when_squirrel_unset() {
        temp_env::with_vars(
            [
                ("SQUIRREL_RETRY_MAX_ATTEMPTS", None::<&str>),
                ("SQUIRREL_RETRY_BASE_DELAY_MS", None::<&str>),
                ("SQUIRREL_RETRY_MAX_DELAY_MS", None::<&str>),
                ("IPC_RETRY_MAX_ATTEMPTS", Some("5")),
                ("IPC_RETRY_BASE_DELAY_MS", Some("250")),
                ("IPC_RETRY_MAX_DELAY_MS", Some("8000")),
            ],
            || {
                let p = retry_env_params();
                assert_eq!(p.max_retries, 5);
                assert_eq!(p.base_delay, Duration::from_millis(250));
                assert_eq!(p.max_delay, Duration::from_millis(8000));
            },
        );
    }

    #[test]
    fn invalid_env_values_fall_back_to_defaults() {
        temp_env::with_vars(
            [
                ("SQUIRREL_RETRY_MAX_ATTEMPTS", Some("not-a-number")),
                ("SQUIRREL_RETRY_BASE_DELAY_MS", Some("")),
                ("SQUIRREL_RETRY_MAX_DELAY_MS", Some("xyz")),
            ],
            || {
                let p = retry_env_params();
                assert_eq!(p.max_retries, 3);
                assert_eq!(p.base_delay, Duration::from_millis(100));
                assert_eq!(p.max_delay, Duration::from_millis(5000));
            },
        );
    }

    #[test]
    fn partial_ipc_fallback_per_field() {
        temp_env::with_vars(
            [
                ("SQUIRREL_RETRY_MAX_ATTEMPTS", Some("4")),
                ("SQUIRREL_RETRY_BASE_DELAY_MS", None::<&str>),
                ("SQUIRREL_RETRY_MAX_DELAY_MS", None::<&str>),
                ("IPC_RETRY_BASE_DELAY_MS", Some("333")),
                ("IPC_RETRY_MAX_DELAY_MS", Some("4444")),
            ],
            || {
                let p = retry_env_params();
                assert_eq!(p.max_retries, 4);
                assert_eq!(p.base_delay, Duration::from_millis(333));
                assert_eq!(p.max_delay, Duration::from_millis(4444));
            },
        );
    }
}
