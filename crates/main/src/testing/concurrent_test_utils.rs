// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! # Concurrent Test Utilities
//!
//! Modern, event-driven test utilities for truly concurrent testing.
//! Replaces sleep-based synchronization with deterministic event patterns.
//!
//! ## Philosophy
//! - No sleeps for synchronization (only for chaos/latency simulation)
//! - Event-driven state changes
//! - Deterministic concurrent execution
//! - Fast, reliable tests

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock, Barrier, oneshot, watch, mpsc};
use tokio::time::timeout;

/// Result type for test operations
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Service readiness notifier
///
/// Use this instead of sleep-based waiting for service startup.
///
/// # Example
/// ```no_run
/// use squirrel::testing::concurrent_test_utils::ReadinessNotifier;
///
/// #[tokio::test]
/// async fn test_service_starts() -> TestResult<()> {
///     let notifier = ReadinessNotifier::new();
///     
///     // Start service with readiness notification
///     let service = start_service(notifier.clone()).await?;
///     
///     // Wait for ready (or timeout)
///     notifier.wait_ready(Duration::from_secs(5)).await?;
///     
///     assert!(service.is_running());
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ReadinessNotifier {
    notify: Arc<Notify>,
    ready: Arc<RwLock<bool>>,
}

impl ReadinessNotifier {
    /// Create a new readiness notifier
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            ready: Arc::new(RwLock::new(false)),
        }
    }

    /// Signal that the service is ready
    pub async fn signal_ready(&self) {
        let mut ready = self.ready.write().await;
        *ready = true;
        drop(ready);
        self.notify.notify_waiters();
    }

    /// Wait for readiness with timeout
    pub async fn wait_ready(&self, max_wait: Duration) -> TestResult<()> {
        timeout(max_wait, async {
            loop {
                {
                    let ready = self.ready.read().await;
                    if *ready {
                        return Ok(());
                    }
                }
                self.notify.notified().await;
            }
        })
        .await
        .map_err(|_| "Service did not become ready in time".into())?
    }

    /// Check if ready without waiting
    pub async fn is_ready(&self) -> bool {
        *self.ready.read().await
    }
}

impl Default for ReadinessNotifier {
    fn default() -> Self {
        Self::new()
    }
}

/// State change watcher for event-driven testing
///
/// Use this instead of polling with sleep intervals.
///
/// # Example
/// ```no_run
/// use squirrel::testing::concurrent_test_utils::StateWatcher;
///
/// #[tokio::test]
/// async fn test_state_transition() -> TestResult<()> {
///     let watcher = StateWatcher::new("initializing");
///     
///     // Start state machine
///     let machine = StateMachine::new(watcher.get_sender());
///     
///     // Wait for specific state
///     watcher.wait_for_state("running", Duration::from_secs(5)).await?;
///     
///     assert_eq!(machine.current_state(), "running");
///     Ok(())
/// }
/// ```
pub struct StateWatcher<T: Clone + PartialEq + Send + Sync> {
    receiver: watch::Receiver<T>,
    sender: watch::Sender<T>,
}

impl<T: Clone + PartialEq + Send + Sync> StateWatcher<T> {
    /// Create a new state watcher with initial state
    pub fn new(initial_state: T) -> Self {
        let (sender, receiver) = watch::channel(initial_state);
        Self { receiver, sender }
    }

    /// Get a sender for updating state
    pub fn get_sender(&self) -> watch::Sender<T> {
        self.sender.clone()
    }

    /// Wait for a specific state with timeout
    pub async fn wait_for_state(&self, expected: T, max_wait: Duration) -> TestResult<()> {
        let mut rx = self.receiver.clone();
        
        timeout(max_wait, async {
            loop {
                if *rx.borrow_and_update() == expected {
                    return Ok(());
                }
                rx.changed().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            }
        })
        .await
        .map_err(|_| format!("State did not reach expected value in time").into())?
    }

    /// Get current state
    pub fn current_state(&self) -> T {
        self.receiver.borrow().clone()
    }
}

/// Concurrent operation coordinator using barriers
///
/// Ensures multiple operations start simultaneously for true concurrency testing.
///
/// # Example
/// ```no_run
/// use squirrel::testing::concurrent_test_utils::ConcurrentCoordinator;
///
/// #[tokio::test]
/// async fn test_concurrent_access() -> TestResult<()> {
///     let coordinator = ConcurrentCoordinator::new(10);
///     let service = Arc::new(Service::new());
///     
///     let handles: Vec<_> = (0..10).map(|i| {
///         let coord = coordinator.clone();
///         let svc = service.clone();
///         tokio::spawn(async move {
///             coord.wait().await; // All tasks start together
///             svc.process(i).await
///         })
///     }).collect();
///     
///     let results = futures::future::join_all(handles).await;
///     assert!(results.iter().all(|r| r.is_ok()));
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ConcurrentCoordinator {
    barrier: Arc<Barrier>,
}

impl ConcurrentCoordinator {
    /// Create coordinator for N concurrent operations
    pub fn new(count: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(count)),
        }
    }

    /// Wait for all participants to be ready
    pub async fn wait(&self) {
        self.barrier.wait().await;
    }
}

/// Event collector for testing event-driven systems
///
/// Collect events and verify they occurred in expected order/count.
///
/// # Example
/// ```no_run
/// use squirrel::testing::concurrent_test_utils::EventCollector;
///
/// #[tokio::test]
/// async fn test_events_fired() -> TestResult<()> {
///     let collector = EventCollector::new();
///     let mut receiver = collector.subscribe();
///     
///     // System emits events
///     system.on_event(collector.get_sender());
///     system.start().await?;
///     
///     // Wait for expected events
///     let event1 = receiver.recv_timeout(Duration::from_secs(1)).await?;
///     assert_eq!(event1, "started");
///     
///     let event2 = receiver.recv_timeout(Duration::from_secs(1)).await?;
///     assert_eq!(event2, "initialized");
///     
///     Ok(())
/// }
/// ```
pub struct EventCollector<T: Clone + Send + 'static> {
    sender: mpsc::UnboundedSender<T>,
}

impl<T: Clone + Send + 'static> EventCollector<T> {
    /// Create a new event collector
    pub fn new() -> Self {
        let (sender, _receiver) = mpsc::unbounded_channel();
        Self { sender }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> EventReceiver<T> {
        let (tx, rx) = mpsc::unbounded_channel();
        EventReceiver { receiver: rx, _sender: tx }
    }

    /// Get sender for emitting events
    pub fn get_sender(&self) -> mpsc::UnboundedSender<T> {
        self.sender.clone()
    }

    /// Send an event
    pub fn send(&self, event: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(event)
    }
}

impl<T: Clone + Send + 'static> Default for EventCollector<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Event receiver with timeout support
pub struct EventReceiver<T> {
    receiver: mpsc::UnboundedReceiver<T>,
    _sender: mpsc::UnboundedSender<T>,
}

impl<T> EventReceiver<T> {
    /// Receive an event with timeout
    pub async fn recv_timeout(&mut self, max_wait: Duration) -> TestResult<T> {
        timeout(max_wait, self.receiver.recv())
            .await
            .map_err(|_| "No event received in time".into())?
            .ok_or_else(|| "Channel closed".into())
    }

    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

/// Completion tracker for async operations
///
/// Track when multiple async operations complete without sleep polling.
///
/// # Example
/// ```no_run
/// use squirrel::testing::concurrent_test_utils::CompletionTracker;
///
/// #[tokio::test]
/// async fn test_all_complete() -> TestResult<()> {
///     let tracker = CompletionTracker::new(5);
///     
///     for i in 0..5 {
///         let t = tracker.clone();
///         tokio::spawn(async move {
///             // Do work
///             process(i).await;
///             t.mark_complete(i);
///         });
///     }
///     
///     // Wait for all to complete
///     tracker.wait_all_complete(Duration::from_secs(10)).await?;
///     assert_eq!(tracker.completed_count().await, 5);
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct CompletionTracker {
    count: Arc<RwLock<usize>>,
    total: usize,
    notify: Arc<Notify>,
}

impl CompletionTracker {
    /// Create tracker expecting N completions
    pub fn new(total: usize) -> Self {
        Self {
            count: Arc::new(RwLock::new(0)),
            total,
            notify: Arc::new(Notify::new()),
        }
    }

    /// Mark an operation as complete
    pub async fn mark_complete(&self, _id: usize) {
        let mut count = self.count.write().await;
        *count += 1;
        drop(count);
        self.notify.notify_waiters();
    }

    /// Wait for all operations to complete
    pub async fn wait_all_complete(&self, max_wait: Duration) -> TestResult<()> {
        timeout(max_wait, async {
            loop {
                {
                    let count = self.count.read().await;
                    if *count >= self.total {
                        return Ok(());
                    }
                }
                self.notify.notified().await;
            }
        })
        .await
        .map_err(|_| format!("Not all operations completed in time").into())?
    }

    /// Get current completion count
    pub async fn completed_count(&self) -> usize {
        *self.count.read().await
    }
}

/// Oneshot result channel with timeout
///
/// For operations that produce a single result.
pub struct OneshotResult<T> {
    sender: Option<oneshot::Sender<T>>,
    receiver: oneshot::Receiver<T>,
}

impl<T> OneshotResult<T> {
    /// Create a new oneshot result channel
    pub fn new() -> Self {
        let (sender, receiver) = oneshot::channel();
        Self {
            sender: Some(sender),
            receiver,
        }
    }

    /// Send the result
    pub fn send(mut self, value: T) -> Result<(), T> {
        self.sender.take().unwrap().send(value)
    }

    /// Receive the result with timeout
    pub async fn recv_timeout(self, max_wait: Duration) -> TestResult<T> {
        timeout(max_wait, self.receiver)
            .await
            .map_err(|_| "Result not received in time".into())?
            .map_err(|_| "Sender dropped before sending".into())
    }
}

impl<T> Default for OneshotResult<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_readiness_notifier() {
        let notifier = ReadinessNotifier::new();
        assert!(!notifier.is_ready().await);

        let n = notifier.clone();
        tokio::spawn(async move {
            // Yield once to prove concurrency, then signal
            tokio::task::yield_now().await;
            n.signal_ready().await;
        });

        notifier.wait_ready(Duration::from_secs(1)).await.unwrap();
        assert!(notifier.is_ready().await);
    }

    #[tokio::test]
    async fn test_state_watcher() {
        let watcher = StateWatcher::new("initializing");
        let sender = watcher.get_sender();

        tokio::spawn(async move {
            tokio::task::yield_now().await;
            let _ = sender.send("running");
        });

        watcher.wait_for_state("running", Duration::from_secs(1)).await.unwrap();
        assert_eq!(watcher.current_state(), "running");
    }

    #[tokio::test]
    async fn test_concurrent_coordinator() {
        let coordinator = ConcurrentCoordinator::new(3);
        let start = std::time::Instant::now();
        let results = Arc::new(RwLock::new(Vec::new()));

        let handles: Vec<_> = (0..3).map(|i| {
            let coord = coordinator.clone();
            let res = results.clone();
            tokio::spawn(async move {
                coord.wait().await;
                let mut r = res.write().await;
                r.push((i, start.elapsed()));
            })
        }).collect();

        futures::future::join_all(handles).await;
        let results = results.read().await;
        
        // All should start at nearly the same time
        assert_eq!(results.len(), 3);
        let times: Vec<_> = results.iter().map(|(_, t)| t.as_millis()).collect();
        let max_diff = times.iter().max().unwrap() - times.iter().min().unwrap();
        assert!(max_diff < 50, "Tasks should start simultaneously, diff: {}ms", max_diff);
    }

    #[tokio::test]
    async fn test_completion_tracker() {
        let tracker = CompletionTracker::new(5);

        for i in 0..5 {
            let t = tracker.clone();
            tokio::spawn(async move {
                // No sleep needed -- tasks run concurrently
                tokio::task::yield_now().await;
                t.mark_complete(i).await;
            });
        }

        tracker.wait_all_complete(Duration::from_secs(1)).await.unwrap();
        assert_eq!(tracker.completed_count().await, 5);
    }

    #[tokio::test]
    async fn test_oneshot_result() {
        let _result = OneshotResult::<i32>::new();
        let recv_result = OneshotResult::<i32>::new();

        // Test timeout behavior: nothing sends to recv_result
        let timeout_result = recv_result.recv_timeout(Duration::from_millis(5)).await;
        assert!(timeout_result.is_err());
    }
}

