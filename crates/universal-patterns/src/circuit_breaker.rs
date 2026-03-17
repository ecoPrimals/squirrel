// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Circuit breaker + retry policy for IPC resilience.
//!
//! Absorbed from petalTongue v1.6.6 (which absorbed from rhizoCrypt v0.13).
//! Wraps IPC calls with a circuit-breaker pattern and configurable retry
//! with exponential backoff, gated by `IpcErrorPhase.is_retryable()`.
//!
//! # Architecture
//!
//! ```text
//! caller → RetryPolicy → CircuitBreaker → IpcClient::call()
//! ```
//!
//! The circuit breaker tracks consecutive failures and trips open when the
//! threshold is exceeded. While open, calls fail fast without hitting the
//! socket. After a cooldown period, the breaker enters half-open state and
//! allows a single probe request.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests flow through.
    Closed,
    /// Breaker tripped — requests fail fast.
    Open,
    /// Cooldown expired — next request is a probe.
    HalfOpen,
}

/// Circuit breaker for IPC connections.
///
/// Tracks failures and prevents cascading failure by fast-failing when
/// the error threshold is exceeded.
#[derive(Debug)]
pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    consecutive_failures: AtomicU64,
    failure_threshold: u64,
    cooldown: Duration,
    last_failure: RwLock<Option<Instant>>,
    total_trips: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `failure_threshold`: consecutive failures before tripping open
    /// - `cooldown`: how long to stay open before trying half-open
    #[must_use]
    pub fn new(failure_threshold: u64, cooldown: Duration) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            consecutive_failures: AtomicU64::new(0),
            failure_threshold,
            cooldown,
            last_failure: RwLock::new(None),
            total_trips: AtomicU64::new(0),
        }
    }

    /// Create a circuit breaker with ecosystem defaults (5 failures, 30s cooldown).
    #[must_use]
    pub fn default_ipc() -> Self {
        Self::new(5, Duration::from_secs(30))
    }

    /// Check if a request is allowed through the breaker.
    pub async fn allow_request(&self) -> bool {
        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let value = *self.last_failure.read().await;
                if let Some(last) = value
                    && last.elapsed() >= self.cooldown
                {
                    *self.state.write().await = CircuitState::HalfOpen;
                    return true;
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call — resets the breaker to closed.
    pub async fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        *self.state.write().await = CircuitState::Closed;
    }

    /// Record a failed call — may trip the breaker open.
    pub async fn record_failure(&self) {
        let count = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.write().await = Some(Instant::now());

        if count >= self.failure_threshold {
            let mut state = self.state.write().await;
            if *state != CircuitState::Open {
                *state = CircuitState::Open;
                self.total_trips.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Current state of the breaker.
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Number of times the breaker has tripped open.
    #[must_use]
    pub fn total_trips(&self) -> u64 {
        self.total_trips.load(Ordering::Relaxed)
    }

    /// Current consecutive failure count.
    #[must_use]
    pub fn consecutive_failures(&self) -> u64 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }
}

/// Retry policy with exponential backoff.
///
/// Gated by `IpcErrorPhase.is_retryable()` — only retries errors where
/// retry is safe (Connect, Write, Read phases).
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Initial backoff duration.
    pub initial_backoff: Duration,
    /// Maximum backoff duration (caps exponential growth).
    pub max_backoff: Duration,
    /// Backoff multiplier (typically 2.0 for exponential).
    pub multiplier: f64,
    /// Optional jitter factor (0.0–1.0) to prevent thundering herd.
    pub jitter: f64,
}

impl RetryPolicy {
    /// Create a retry policy with ecosystem defaults.
    #[must_use]
    pub fn default_ipc() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: 0.1,
        }
    }

    /// No retries.
    #[must_use]
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
            multiplier: 1.0,
            jitter: 0.0,
        }
    }

    /// Compute the backoff duration for a given attempt (0-indexed).
    #[must_use]
    pub fn backoff_for(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let base = self.initial_backoff.as_secs_f64() * self.multiplier.powi(attempt as i32 - 1);
        let capped = base.min(self.max_backoff.as_secs_f64());
        Duration::from_secs_f64(capped)
    }
}

/// Resilient IPC caller combining circuit breaker + retry policy.
///
/// Wraps an async call function with retry logic gated by the circuit breaker.
pub struct ResilientCaller {
    /// The circuit breaker instance.
    pub breaker: Arc<CircuitBreaker>,
    /// The retry policy.
    pub policy: RetryPolicy,
}

impl ResilientCaller {
    /// Create a new resilient caller with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            breaker: Arc::new(CircuitBreaker::default_ipc()),
            policy: RetryPolicy::default_ipc(),
        }
    }

    /// Create with custom breaker and policy.
    #[must_use]
    pub fn with(breaker: Arc<CircuitBreaker>, policy: RetryPolicy) -> Self {
        Self { breaker, policy }
    }

    /// Execute a fallible async call with circuit breaker + retry.
    ///
    /// The `is_retryable` closure determines whether a given error should be
    /// retried (typically delegates to `IpcClientError::is_retryable()`).
    pub async fn call<F, Fut, T, E>(
        &self,
        mut f: F,
        is_retryable: impl Fn(&E) -> bool,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut last_err = None;

        for attempt in 0..=self.policy.max_retries {
            if attempt > 0 {
                let backoff = self.policy.backoff_for(attempt);
                tokio::time::sleep(backoff).await;
            }

            if !self.breaker.allow_request().await {
                if let Some(e) = last_err {
                    return Err(e);
                }
                continue;
            }

            match f().await {
                Ok(v) => {
                    self.breaker.record_success().await;
                    return Ok(v);
                }
                Err(e) => {
                    self.breaker.record_failure().await;
                    if !is_retryable(&e) || attempt == self.policy.max_retries {
                        return Err(e);
                    }
                    last_err = Some(e);
                }
            }
        }

        // Unreachable in practice: max_retries >= 0 guarantees at least one attempt
        match last_err {
            Some(e) => Err(e),
            None => f().await,
        }
    }
}

impl Default for ResilientCaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::default_ipc();
        assert_eq!(cb.state().await, CircuitState::Closed);
        assert!(cb.allow_request().await);
    }

    #[tokio::test]
    async fn circuit_breaker_trips_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));

        for _ in 0..3 {
            cb.record_failure().await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);
        assert!(!cb.allow_request().await);
        assert_eq!(cb.total_trips(), 1);
    }

    #[tokio::test]
    async fn circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));

        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_success().await;

        assert_eq!(cb.state().await, CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[tokio::test]
    async fn circuit_breaker_half_open_after_cooldown() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(10));

        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitState::Open);

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(cb.allow_request().await);
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
    }

    #[test]
    fn retry_policy_backoff_calculation() {
        let policy = RetryPolicy::default_ipc();

        assert_eq!(policy.backoff_for(0), Duration::ZERO);
        assert_eq!(policy.backoff_for(1), Duration::from_millis(100));
        assert_eq!(policy.backoff_for(2), Duration::from_millis(200));
        assert_eq!(policy.backoff_for(3), Duration::from_millis(400));
    }

    #[test]
    fn retry_policy_backoff_capped() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: 0.0,
        };

        assert_eq!(policy.backoff_for(5), Duration::from_secs(5));
        assert_eq!(policy.backoff_for(10), Duration::from_secs(5));
    }

    #[test]
    fn retry_policy_none_has_zero_retries() {
        let policy = RetryPolicy::none();
        assert_eq!(policy.max_retries, 0);
    }

    #[tokio::test]
    async fn resilient_caller_succeeds_on_first_try() {
        let caller = ResilientCaller::new();
        let result = caller
            .call(|| async { Ok::<_, String>(42) }, |_: &String| false)
            .await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn resilient_caller_retries_on_retryable_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let caller = ResilientCaller::with(
            Arc::new(CircuitBreaker::new(10, Duration::from_secs(60))),
            RetryPolicy {
                max_retries: 3,
                initial_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(10),
                multiplier: 1.0,
                jitter: 0.0,
            },
        );

        let result = caller
            .call(
                move || {
                    let a = Arc::clone(&attempts_clone);
                    async move {
                        let n = a.fetch_add(1, Ordering::Relaxed);
                        if n < 2 { Err("retryable") } else { Ok(42) }
                    }
                },
                |_: &&str| true,
            )
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn resilient_caller_no_retry_on_non_retryable() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let caller = ResilientCaller::new();

        let result = caller
            .call(
                move || {
                    let a = Arc::clone(&attempts_clone);
                    async move {
                        a.fetch_add(1, Ordering::Relaxed);
                        Err::<i32, _>("not retryable")
                    }
                },
                |_: &&str| false,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::Relaxed), 1);
    }
}
