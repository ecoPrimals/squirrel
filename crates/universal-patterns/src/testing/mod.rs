// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Modern concurrent testing utilities
//!
//! This module provides utilities for writing robust, concurrent tests
//! without relying on sleeps or arbitrary delays.

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Notify, RwLock};
use tokio::time::timeout;

/// A coordination primitive for tests that need to wait for conditions
/// without using sleep.
#[derive(Clone)]
pub struct TestBarrier {
    ready: Arc<Notify>,
    count: Arc<AtomicU32>,
    target: u32,
}

impl TestBarrier {
    /// Create a new barrier that waits for `n` parties
    pub fn new(n: u32) -> Self {
        Self {
            ready: Arc::new(Notify::new()),
            count: Arc::new(AtomicU32::new(0)),
            target: n,
        }
    }

    /// Wait for all parties to arrive at the barrier
    pub async fn wait(&self) -> Result<(), TestError> {
        let current = self.count.fetch_add(1, Ordering::SeqCst) + 1;

        if current >= self.target {
            // Last one in, notify everyone
            self.ready.notify_waiters();
            Ok(())
        } else {
            // Wait for notification with timeout
            timeout(Duration::from_secs(10), self.ready.notified())
                .await
                .map_err(|_| TestError::Timeout("Barrier wait timed out".into()))?;
            Ok(())
        }
    }
}

/// A signal for tests to coordinate state transitions
#[derive(Clone)]
pub struct TestSignal {
    notifier: Arc<Notify>,
    fired: Arc<AtomicBool>,
}

impl Default for TestSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl TestSignal {
    /// Create a new test signal
    pub fn new() -> Self {
        Self {
            notifier: Arc::new(Notify::new()),
            fired: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Fire the signal, waking all waiters
    pub fn fire(&self) {
        self.fired.store(true, Ordering::SeqCst);
        self.notifier.notify_waiters();
    }

    /// Wait for the signal with a timeout
    pub async fn wait(&self, timeout_duration: Duration) -> Result<(), TestError> {
        if self.fired.load(Ordering::SeqCst) {
            return Ok(());
        }

        timeout(timeout_duration, self.notifier.notified())
            .await
            .map_err(|_| TestError::Timeout("Signal wait timed out".into()))?;

        Ok(())
    }

    /// Check if the signal has been fired without waiting
    pub fn is_fired(&self) -> bool {
        self.fired.load(Ordering::SeqCst)
    }
}

/// A condition variable for tests
pub struct TestCondition<T> {
    state: Arc<RwLock<Option<T>>>,
    notifier: Arc<Notify>,
}

impl<T: Clone> Default for TestCondition<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> TestCondition<T> {
    /// Create a new test condition
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
            notifier: Arc::new(Notify::new()),
        }
    }

    /// Set the condition value and notify waiters
    pub async fn set(&self, value: T) {
        let mut state = self.state.write().await;
        *state = Some(value);
        drop(state);
        self.notifier.notify_waiters();
    }

    /// Wait for the condition to be set
    pub async fn wait(&self, timeout_duration: Duration) -> Result<T, TestError> {
        // Check if already set
        {
            let state = self.state.read().await;
            if let Some(ref value) = *state {
                return Ok(value.clone());
            }
        }

        // Wait for notification
        timeout(timeout_duration, async {
            loop {
                self.notifier.notified().await;
                let state = self.state.read().await;
                if let Some(ref value) = *state {
                    return value.clone();
                }
            }
        })
        .await
        .map_err(|_| TestError::Timeout("Condition wait timed out".into()))
    }

    /// Get the current value if set
    pub async fn get(&self) -> Option<T> {
        self.state.read().await.clone()
    }
}

impl<T: Clone> Clone for TestCondition<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            notifier: Arc::clone(&self.notifier),
        }
    }
}

/// Broadcast channel wrapper for test coordination
pub struct TestBroadcast<T: Clone> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone> TestBroadcast<T> {
    /// Create a new broadcast channel with given capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Send a message to all subscribers
    pub fn send(&self, msg: T) -> Result<(), TestError> {
        self.sender
            .send(msg)
            .map(|_| ())
            .map_err(|_| TestError::ChannelClosed)
    }

    /// Subscribe to receive messages
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

/// Wait for a condition with timeout
pub async fn wait_for<F, Fut>(
    condition: F,
    timeout_duration: Duration,
    check_interval: Duration,
) -> Result<(), TestError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    timeout(timeout_duration, async {
        let mut interval = tokio::time::interval(check_interval);
        loop {
            interval.tick().await;
            if condition().await {
                return;
            }
        }
    })
    .await
    .map_err(|_| TestError::Timeout("Condition check timed out".into()))
}

/// Wait for a future with exponential backoff retries
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut backoff = initial_backoff;
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_retries => return Err(e),
            Err(_) => {
                attempts += 1;
                // NOTE: This sleep is intentional for exponential backoff delay
                // It's not for synchronization - it's rate limiting between retries
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
        }
    }
}

/// Errors that can occur during test coordination
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    /// Operation timed out waiting for a condition
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Communication channel was closed unexpectedly
    #[error("Channel closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_barrier_coordination() {
        let barrier = TestBarrier::new(3);
        let b1 = barrier.clone();
        let b2 = barrier.clone();
        let b3 = barrier.clone();

        let h1 = tokio::spawn(async move { b1.wait().await });
        let h2 = tokio::spawn(async move { b2.wait().await });
        let h3 = tokio::spawn(async move { b3.wait().await });

        // All should complete without timeout
        assert!(h1.await.unwrap().is_ok());
        assert!(h2.await.unwrap().is_ok());
        assert!(h3.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn test_signal_coordination() {
        let signal = TestSignal::new();
        let s1 = signal.clone();
        let s2 = signal.clone();

        // Spawn waiter task
        let h1 = tokio::spawn(async move { s1.wait(Duration::from_secs(1)).await });

        // Fire signal immediately (no sleep needed - proper async coordination)
        s2.fire();

        // Should complete successfully
        assert!(h1.await.unwrap().is_ok());
        assert!(signal.is_fired());
    }

    #[tokio::test]
    async fn test_condition_coordination() {
        let condition = TestCondition::new();
        let c1 = condition.clone();
        let c2 = condition.clone();

        // Spawn waiter task
        let h1 = tokio::spawn(async move { c1.wait(Duration::from_secs(1)).await });

        // Set condition value immediately (no sleep - proper async coordination)
        c2.set(42).await;

        // Should receive the value
        let result = h1.await.unwrap();
        assert_eq!(result.unwrap(), 42);
    }
}
