// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::future_not_send,
    missing_docs
)] // Test code: explicit unwrap/expect and local lint noise
//! Modern Concurrent Test Helpers
//!
//! This module provides utilities for writing truly concurrent tests without sleeps.
//!
//! ## Philosophy
//!
//! **Test issues = Production issues**
//!
//! Tests that rely on `sleep()` for synchronization are:
//! - Flaky (timing-dependent)
//! - Slow (artificial delays)
//! - Unrealistic (production doesn't sleep)
//! - Hiding race conditions
//!
//! ## Modern Patterns
//!
//! Instead of sleep-based timing, use:
//! 1. **Channels** for event notification
//! 2. **Barriers** for multi-task coordination
//! 3. **Atomics** for lock-free synchronization
//! 4. **Timeouts** for safety (not primary sync)
//!
//! ## Examples
//!
//! ### Event Notification
//! ```no_run
//! use concurrent_test_helpers::*;
//!
//! #[tokio::test]
//! async fn test_async_operation() {
//!     let (tx, rx) = event_channel();
//!    
//!     // Start background task
//!     tokio::spawn(async move {
//!         perform_operation().await;
//!         tx.send(()).expect("should succeed"); // Notify completion
//!     });
//!    
//!     // Wait for event (with timeout for safety)
//!     rx.await_with_timeout(Duration::from_secs(5)).await.expect("should succeed");
//!     assert!(operation_completed());
//! }
//! ```
//!
//! ### Concurrent Stress Test
//! ```no_run
//! use concurrent_test_helpers::*;
//!
//! #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
//! async fn test_concurrent_correctness() {
//!     let barrier = ConcurrentBarrier::new(100);
//!    
//!     let mut handles = vec![];
//!     for i in 0..100 {
//!         let barrier = barrier.clone();
//!         handles.push(tokio::spawn(async move {
//!             barrier.wait().await; // All start simultaneously
//!             perform_concurrent_operation(i).await
//!         }));
//!     }
//!    
//!     // All operations should succeed without race conditions
//!     for handle in handles {
//!         assert!(handle.await.expect("should succeed").is_ok());
//!     }
//! }
//! ```

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Barrier, Notify, Semaphore, oneshot};

// ============================================================================
// Event Channels
// ============================================================================

/// Event sender for test synchronization
pub struct EventSender<T = ()> {
    tx: oneshot::Sender<T>,
}

impl<T> EventSender<T> {
    /// Send event notification
    ///
    /// # Errors
    ///
    /// Returns `Err` with the value if the receiver was dropped before receiving.
    pub fn send(self, value: T) -> Result<(), T> {
        self.tx.send(value)
    }
}

/// Event receiver for test synchronization
pub struct EventReceiver<T = ()> {
    rx: oneshot::Receiver<T>,
}

impl<T> EventReceiver<T> {
    /// Wait for event with timeout
    ///
    /// # Errors
    ///
    /// Returns [`EventTimeoutError`] if the wait exceeds the timeout or the sender was dropped.
    pub async fn await_with_timeout(self, timeout: Duration) -> Result<T, EventTimeoutError> {
        tokio::time::timeout(timeout, self.rx)
            .await
            .map_err(|_| EventTimeoutError)?
            .map_err(|_| EventTimeoutError)
    }

    /// Wait for event (no timeout)
    ///
    /// # Errors
    ///
    /// Returns [`EventRecvError`] if the sender was dropped without sending a value.
    pub async fn await_event(self) -> Result<T, EventRecvError> {
        self.rx.await.map_err(|_| EventRecvError)
    }
}

/// Create an event channel for one-time notification
#[must_use]
pub fn event_channel<T>() -> (EventSender<T>, EventReceiver<T>) {
    let (tx, rx) = oneshot::channel();
    (EventSender { tx }, EventReceiver { rx })
}

#[derive(Debug)]
pub struct EventTimeoutError;

#[derive(Debug)]
pub struct EventRecvError;

// ============================================================================
// Concurrent Barriers
// ============================================================================

/// Barrier for coordinating concurrent tasks
#[derive(Clone)]
pub struct ConcurrentBarrier {
    barrier: Arc<Barrier>,
}

impl ConcurrentBarrier {
    /// Create a barrier for N concurrent tasks
    ///
    /// All tasks will block at `wait()` until N tasks have reached it,
    /// then all are released simultaneously.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(n)),
        }
    }

    /// Wait for all tasks to reach the barrier
    ///
    /// This ensures all tasks start their work simultaneously,
    /// maximizing the chance of catching race conditions.
    pub async fn wait(&self) {
        self.barrier.wait().await;
    }
}

// ============================================================================
// State Watchers
// ============================================================================

/// Watch for a state change with proper async polling
pub struct StateWatcher<F, T>
where
    F: Fn() -> Option<T>,
{
    check: F,
    notify: Arc<Notify>,
}

impl<F, T> StateWatcher<F, T>
where
    F: Fn() -> Option<T>,
{
    /// Create a state watcher
    pub fn new(check: F) -> Self {
        Self {
            check,
            notify: Arc::new(Notify::new()),
        }
    }

    /// Wait for condition to become true
    ///
    /// This is better than sleep-based polling because:
    /// - No wasted CPU cycles
    /// - No artificial delays
    /// - Responds immediately when condition is met
    ///
    /// # Errors
    ///
    /// Returns [`StateTimeoutError`] if the condition is not satisfied before the deadline.
    pub async fn wait_for(&self, timeout: Duration) -> Result<T, StateTimeoutError> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            // Check condition
            if let Some(value) = (self.check)() {
                return Ok(value);
            }

            // Calculate remaining time
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Err(StateTimeoutError);
            }

            // Wait for notification or timeout
            tokio::select! {
                () = self.notify.notified() => {
                    // Condition might have changed, loop to check
                }
                () = tokio::time::sleep(deadline - now) => {
                    // Timeout
                    return Err(StateTimeoutError);
                }
            }
        }
    }

    /// Notify waiters that state may have changed
    pub fn notify(&self) {
        self.notify.notify_waiters();
    }
}

#[derive(Debug)]
pub struct StateTimeoutError;

// ============================================================================
// Atomic Flag Helpers
// ============================================================================

/// Atomic flag for simple synchronization
#[derive(Default)]
pub struct AtomicFlag {
    flag: AtomicBool,
    notify: Notify,
}

impl AtomicFlag {
    /// Create a new flag (initially false)
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            flag: AtomicBool::new(false),
            notify: Notify::new(),
        })
    }

    /// Set the flag to true
    pub fn set(&self) {
        self.flag.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    /// Check if flag is set
    pub fn is_set(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// Wait for flag to be set (with timeout)
    ///
    /// # Errors
    ///
    /// Returns [`StateTimeoutError`] if the flag is not set before the timeout elapses.
    pub async fn wait_set(&self, timeout: Duration) -> Result<(), StateTimeoutError> {
        if self.is_set() {
            return Ok(());
        }

        tokio::time::timeout(timeout, async {
            while !self.is_set() {
                self.notify.notified().await;
            }
        })
        .await
        .map_err(|_| StateTimeoutError)
    }

    /// Reset the flag to false
    pub fn reset(&self) {
        self.flag.store(false, Ordering::Release);
    }
}

// ============================================================================
// Concurrent Counter
// ============================================================================

/// Thread-safe counter for tracking concurrent operations
pub struct ConcurrentCounter {
    count: AtomicUsize,
    notify: Notify,
}

impl ConcurrentCounter {
    /// Create a new counter
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            count: AtomicUsize::new(0),
            notify: Notify::new(),
        })
    }

    /// Increment counter
    pub fn increment(&self) -> usize {
        let prev = self.count.fetch_add(1, Ordering::AcqRel);
        self.notify.notify_waiters();
        prev + 1
    }

    /// Decrement counter
    pub fn decrement(&self) -> usize {
        let prev = self.count.fetch_sub(1, Ordering::AcqRel);
        self.notify.notify_waiters();
        prev.saturating_sub(1)
    }

    /// Get current count
    pub fn get(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Wait for count to reach target (with timeout)
    ///
    /// # Errors
    ///
    /// Returns [`StateTimeoutError`] if the count does not reach the target before the timeout elapses.
    pub async fn wait_for_count(
        &self,
        target: usize,
        timeout: Duration,
    ) -> Result<(), StateTimeoutError> {
        if self.get() == target {
            return Ok(());
        }

        tokio::time::timeout(timeout, async {
            while self.get() != target {
                self.notify.notified().await;
            }
        })
        .await
        .map_err(|_| StateTimeoutError)
    }
}

impl Default for ConcurrentCounter {
    fn default() -> Self {
        Self {
            count: AtomicUsize::new(0),
            notify: Notify::new(),
        }
    }
}

// ============================================================================
// Test Utilities
// ============================================================================

/// Execute a function with timeout
///
/// This is useful for ensuring tests don't hang, but the timeout
/// should be generous (5-10 seconds). It's a safety net, not a
/// synchronization mechanism.
///
/// # Errors
///
/// Returns [`TestTimeoutError`] if the future does not complete before the timeout elapses.
pub async fn with_timeout<F, T>(timeout: Duration, f: F) -> Result<T, TestTimeoutError>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(timeout, f)
        .await
        .map_err(|_| TestTimeoutError)
}

#[derive(Debug)]
pub struct TestTimeoutError;

/// Run N concurrent tasks and collect results
///
/// All tasks start simultaneously (via barrier) to maximize
/// concurrency and catch race conditions.
///
/// # Panics
///
/// Panics if any spawned task panics.
pub async fn run_concurrent<F, Fut, T>(n: usize, f: F) -> Vec<T>
where
    F: Fn(usize) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = T> + Send,
    T: Send + 'static,
{
    let barrier = ConcurrentBarrier::new(n);
    let mut handles = Vec::with_capacity(n);

    for i in 0..n {
        let barrier = barrier.clone();
        let f = f.clone();

        handles.push(tokio::spawn(async move {
            barrier.wait().await; // Synchronize start
            f(i).await
        }));
    }

    let mut results = Vec::with_capacity(n);
    for handle in handles {
        results.push(handle.await.expect("Task panicked"));
    }

    results
}

/// Stress test helper - run operation many times concurrently
///
/// This is useful for catching race conditions and verifying
/// thread safety.
///
/// # Panics
///
/// Panics if the semaphore is closed (should not happen under normal usage).
///
/// # Errors
///
/// Returns [`StressTestError`] if a spawned task panics or its future returns `Err`.
pub async fn stress_test<F, Fut>(
    iterations: usize,
    concurrency: usize,
    f: F,
) -> Result<(), StressTestError>
where
    F: Fn(usize) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("should succeed");
        let f = f.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit; // Hold for duration
            f(i).await
        }));
    }

    // Collect results
    for (i, handle) in handles.into_iter().enumerate() {
        handle
            .await
            .map_err(|e| StressTestError::TaskPanicked(i, e))?
            .map_err(|e| StressTestError::TaskFailed(i, e))?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum StressTestError {
    TaskPanicked(usize, tokio::task::JoinError),
    TaskFailed(usize, Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for StressTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskPanicked(i, e) => {
                write!(f, "Task {i} panicked: {e}")
            }
            Self::TaskFailed(i, e) => {
                write!(f, "Task {i} failed: {e}")
            }
        }
    }
}

impl std::error::Error for StressTestError {}

// ============================================================================
// Tests for Test Helpers (Meta!)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_channel() {
        let (tx, rx) = event_channel();

        tokio::spawn(async move {
            tokio::task::yield_now().await;
            tx.send(42).expect("should succeed");
        });

        let value = rx
            .await_with_timeout(Duration::from_secs(1))
            .await
            .expect("should succeed");
        assert_eq!(value, 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_barrier() {
        let barrier = ConcurrentBarrier::new(10);
        let counter = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];
        for _ in 0..10 {
            let barrier = barrier.clone();
            let counter = counter.clone();

            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                counter.fetch_add(1, Ordering::SeqCst);
            }));
        }

        for handle in handles {
            handle.await.expect("should succeed");
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_atomic_flag() {
        let flag = AtomicFlag::new();
        assert!(!flag.is_set());

        flag.set();
        assert!(flag.is_set());

        flag.reset();
        assert!(!flag.is_set());
    }

    #[tokio::test]
    async fn test_atomic_flag_wait() {
        let flag = AtomicFlag::new();
        let flag_clone = flag.clone();

        // ✅ Event-driven: Use Notify to signal when ready
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_clone = ready.clone();

        tokio::spawn(async move {
            ready_clone.notified().await; // Wait for signal to proceed
            flag_clone.set();
        });

        // Signal the spawned task immediately
        ready.notify_one();

        flag.wait_set(Duration::from_secs(1))
            .await
            .expect("should succeed");
        assert!(flag.is_set());
    }

    #[tokio::test]
    async fn test_concurrent_counter() {
        let counter = ConcurrentCounter::new();

        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);

        counter.decrement();
        assert_eq!(counter.get(), 1);
    }

    #[tokio::test]
    async fn test_run_concurrent() {
        let results = run_concurrent(10, |i| async move { i * 2 }).await;

        assert_eq!(results.len(), 10);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(*result, i * 2);
        }
    }

    #[tokio::test]
    async fn test_stress_test_success() {
        let counter = Arc::new(AtomicUsize::new(0));

        let result = stress_test(100, 10, {
            let counter = counter.clone();
            move |_| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
