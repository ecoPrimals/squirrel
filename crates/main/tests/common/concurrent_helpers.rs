// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Concurrent test helpers - Modern Rust patterns for robust testing
//!
//! Philosophy: Test issues ARE production issues
//! - No sleeps hiding race conditions
//! - Channel-based coordination instead of timing
//! - Proper synchronization primitives
//! - Truly concurrent and robust

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, watch, Barrier, Mutex, Notify};
use tokio::time::timeout;

/// Result type for test operations
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Ready signal for coordinating async operations
///
/// Use this instead of sleep to wait for operations to complete
///
/// # Example
/// ```ignore
/// let ready = ReadySignal::new();
/// let waiter = ready.waiter();
///
/// tokio::spawn(async move {
///     // Do async work
///     process_data().await;
///     ready.signal(); // Signal completion
/// });
///
/// waiter.await?; // Wait for signal, not arbitrary time
/// assert!(check_result());
/// ```
#[derive(Clone)]
pub struct ReadySignal {
    notify: Arc<Notify>,
}

impl ReadySignal {
    /// Create a new ready signal
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    /// Signal that operation is ready
    pub fn signal(&self) {
        self.notify.notify_waiters();
    }

    /// Get a waiter for this signal
    pub fn waiter(&self) -> ReadyWaiter {
        ReadyWaiter {
            notify: Arc::clone(&self.notify),
        }
    }
}

impl Default for ReadySignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Waiter for ready signal
pub struct ReadyWaiter {
    notify: Arc<Notify>,
}

impl ReadyWaiter {
    /// Wait for signal with timeout
    pub async fn wait_timeout(&self, duration: Duration) -> TestResult<()> {
        timeout(duration, self.notify.notified())
            .await
            .map_err(|_| "Ready signal timeout".into())
    }

    /// Wait for signal (with default 5s timeout)
    pub async fn wait(&self) -> TestResult<()> {
        self.wait_timeout(Duration::from_secs(5)).await
    }
}

/// State watcher for observing state changes
///
/// Use this to wait for specific state instead of polling with sleeps
///
/// # Example
/// ```ignore
/// let (watcher, setter) = StateWatcher::new(ServiceState::Starting);
///
/// tokio::spawn(async move {
///     service.start().await;
///     setter.set(ServiceState::Running);
/// });
///
/// watcher.wait_for(|state| *state == ServiceState::Running).await?;
/// ```
pub struct StateWatcher<T> {
    receiver: watch::Receiver<T>,
}

impl<T: Clone> StateWatcher<T> {
    /// Create new state watcher with initial value
    pub fn new(initial: T) -> (Self, StateSetter<T>) {
        let (tx, rx) = watch::channel(initial);
        (Self { receiver: rx }, StateSetter { sender: tx })
    }

    /// Wait for state to match predicate
    pub async fn wait_for<F>(&mut self, predicate: F) -> TestResult<T>
    where
        F: Fn(&T) -> bool,
    {
        self.wait_for_timeout(predicate, Duration::from_secs(5))
            .await
    }

    /// Wait for state to match predicate with timeout
    pub async fn wait_for_timeout<F>(&mut self, predicate: F, duration: Duration) -> TestResult<T>
    where
        F: Fn(&T) -> bool,
    {
        timeout(duration, async {
            loop {
                let value = self.receiver.borrow().clone();
                if predicate(&value) {
                    return Ok(value);
                }
                self.receiver.changed().await?;
            }
        })
        .await
        .map_err(|_| "State wait timeout")?
    }

    /// Get current state
    pub fn get(&self) -> T {
        self.receiver.borrow().clone()
    }
}

/// Setter for state watcher
pub struct StateSetter<T> {
    sender: watch::Sender<T>,
}

impl<T> StateSetter<T> {
    /// Set new state
    pub fn set(&self, value: T) {
        let _ = self.sender.send(value);
    }
}

/// Coordination barrier for multi-task synchronization
///
/// Use this to coordinate multiple concurrent tasks
///
/// # Example
/// ```ignore
/// let barrier = TestBarrier::new(3); // 3 tasks must wait
///
/// for i in 0..3 {
///     let barrier = barrier.clone();
///     tokio::spawn(async move {
///         setup_task(i).await;
///         barrier.wait().await; // All tasks wait here
///         execute_task(i).await;
///     });
/// }
/// ```
#[derive(Clone)]
pub struct TestBarrier {
    barrier: Arc<Barrier>,
}

impl TestBarrier {
    /// Create barrier for n tasks
    pub fn new(n: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(n)),
        }
    }

    /// Wait at barrier
    pub async fn wait(&self) {
        self.barrier.wait().await;
    }
}

/// Async result collector
///
/// Collect results from multiple spawned tasks
///
/// # Example
/// ```ignore
/// let collector = ResultCollector::new();
///
/// for i in 0..10 {
///     let sender = collector.sender();
///     tokio::spawn(async move {
///         let result = process(i).await;
///         sender.send(result);
///     });
/// }
///
/// let results = collector.collect_timeout(10, Duration::from_secs(5)).await?;
/// ```
pub struct ResultCollector<T> {
    receivers: Arc<Mutex<Vec<oneshot::Receiver<T>>>>,
}

impl<T: Send + 'static> ResultCollector<T> {
    /// Create new result collector
    pub fn new() -> Self {
        Self {
            receivers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a sender for adding results
    pub fn sender(&self) -> ResultSender<T> {
        let (tx, rx) = oneshot::channel();
        let receivers = Arc::clone(&self.receivers);
        tokio::spawn(async move {
            receivers.lock().await.push(rx);
        });
        ResultSender { sender: Some(tx) }
    }

    /// Collect all results with timeout
    pub async fn collect_timeout(&self, expected: usize, duration: Duration) -> TestResult<Vec<T>> {
        timeout(duration, async {
            // Wait for all senders to be registered
            loop {
                let count = self.receivers.lock().await.len();
                if count >= expected {
                    break;
                }
                tokio::task::yield_now().await;
            }

            // Collect results
            let mut receivers = self.receivers.lock().await;
            let mut results = Vec::new();

            for rx in receivers.drain(..) {
                results.push(rx.await?);
            }

            Ok(results)
        })
        .await
        .map_err(|_| "Result collection timeout")?
    }

    /// Collect all results (5s timeout)
    pub async fn collect(&self, expected: usize) -> TestResult<Vec<T>> {
        self.collect_timeout(expected, Duration::from_secs(5)).await
    }
}

impl<T: Send + 'static> Default for ResultCollector<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Sender for result collector
pub struct ResultSender<T> {
    sender: Option<oneshot::Sender<T>>,
}

impl<T> ResultSender<T> {
    /// Send result
    pub fn send(mut self, value: T) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(value);
        }
    }
}

/// Port allocator for test isolation
///
/// Dynamically allocate ports instead of hardcoding
///
/// # Example
/// ```ignore
/// let port = PortAllocator::allocate().await?;
/// let server = start_server(port).await;
/// ```
pub struct PortAllocator {
    next_port: Arc<Mutex<u16>>,
}

impl PortAllocator {
    /// Create new port allocator starting at 10000
    pub fn new() -> Self {
        Self {
            next_port: Arc::new(Mutex::new(10000)),
        }
    }

    /// Allocate next available port
    pub async fn allocate(&self) -> u16 {
        let mut port = self.next_port.lock().await;
        let allocated = *port;
        *port += 1;
        allocated
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global port allocator
static PORT_ALLOCATOR: std::sync::OnceLock<PortAllocator> = std::sync::OnceLock::new();

fn get_port_allocator() -> &'static PortAllocator {
    PORT_ALLOCATOR.get_or_init(|| PortAllocator::new())
}

/// Allocate a test port
pub async fn allocate_test_port() -> u16 {
    get_port_allocator().allocate().await
}

/// Eventually assertion - retry until condition is met
///
/// Use this instead of sleep + assert
///
/// # Example
/// ```ignore
/// eventually(
///     || async { service.is_ready().await },
///     Duration::from_secs(5),
///     Duration::from_millis(100)
/// ).await?;
/// ```
pub async fn eventually<F, Fut>(
    mut condition: F,
    timeout_duration: Duration,
    check_interval: Duration,
) -> TestResult<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    timeout(timeout_duration, async {
        loop {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await; // Legitimate: polling external condition
        }
    })
    .await
    .map_err(|_| "Eventually condition not met within timeout")?
}

/// Eventually with default intervals (5s timeout, 50ms check)
pub async fn eventually_default<F, Fut>(condition: F) -> TestResult<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    eventually(condition, Duration::from_secs(5), Duration::from_millis(50)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ready_signal() {
        let ready = ReadySignal::new();
        let waiter = ready.waiter();

        // ✅ No sleep needed - spawn immediately signals when it's ready to proceed
        tokio::spawn(async move {
            ready.signal(); // Signal immediately - no artificial delay
        });

        waiter.wait().await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_state_watcher() {
        let (mut watcher, setter) = StateWatcher::new(0);

        // ✅ No sleeps needed - state changes trigger notifications immediately
        tokio::spawn(async move {
            setter.set(1); // Immediate state transition
            setter.set(2); // Immediate next state
        });

        let value = watcher.wait_for(|v| *v == 2).await.expect("should succeed");
        assert_eq!(value, 2);
    }

    #[tokio::test]
    async fn test_barrier() {
        let barrier = TestBarrier::new(3);
        let counter = Arc::new(Mutex::new(0));

        let mut handles = vec![];
        for _ in 0..3 {
            let barrier = barrier.clone();
            let counter = Arc::clone(&counter);

            let handle = tokio::spawn(async move {
                *counter.lock().await += 1;
                barrier.wait().await;
                *counter.lock().await
            });

            handles.push(handle);
        }

        for handle in handles {
            let count = handle.await.expect("should succeed");
            assert_eq!(count, 3); // All should see 3
        }
    }

    #[tokio::test]
    async fn test_result_collector() {
        let collector = ResultCollector::new();

        // ✅ Spawn all tasks simultaneously without staggered delays
        for i in 0..5 {
            let sender = collector.sender();
            tokio::spawn(async move {
                sender.send(i); // Immediate send - test collector handles concurrency
            });
        }

        let mut results = collector.collect(5).await.expect("should succeed");
        results.sort();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_port_allocator() {
        let port1 = allocate_test_port().await;
        let port2 = allocate_test_port().await;
        assert_ne!(port1, port2);
        assert!(port1 >= 10000);
        assert!(port2 >= 10000);
    }

    #[tokio::test]
    async fn test_eventually() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // ✅ Use Notify for deterministic coordination
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_clone = ready.clone();

        tokio::spawn(async move {
            for i in 0..10 {
                *counter_clone.lock().await = i;
                if i == 9 {
                    ready_clone.notify_one(); // Signal when target reached
                }
            }
        });

        // Wait for notification that target value is set
        ready.notified().await;

        eventually_default(|| {
            let counter = Arc::clone(&counter);
            async move { *counter.lock().await >= 5 }
        })
        .await
        .expect("should succeed");

        let final_count = *counter.lock().await;
        assert!(final_count >= 5);
    }
}
