// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! Concurrent Stress Tests
//!
//! Proves robustness under extreme concurrent load without timing hacks.
//! Philosophy: "Test issues ARE production issues"
//!
//! These tests verify:
//! - Zero race conditions under high concurrency
//! - Zero deadlocks with complex locking
//! - Deterministic behavior without sleeps
//! - Production-ready concurrent patterns

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::{Barrier, Notify, RwLock, Semaphore};

/// Test: Massive concurrent atomic operations
///
/// Verifies: Zero lost updates under extreme load
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_atomic_operations() {
    const TASKS: usize = 1000;
    const OPS_PER_TASK: usize = 1000;

    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Barrier ensures true simultaneous start
    let barrier = Arc::new(Barrier::new(TASKS));

    for _ in 0..TASKS {
        let counter = Arc::clone(&counter);
        let barrier = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            // Perform atomic operations
            for _ in 0..OPS_PER_TASK {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.expect("should succeed");
    }

    // Verify: No lost updates
    let expected = (TASKS * OPS_PER_TASK) as u64;
    let actual = counter.load(Ordering::SeqCst);
    assert_eq!(
        actual, expected,
        "Lost updates detected: expected {expected}, got {actual}"
    );
}

/// Test: Concurrent read/write without deadlock
///
/// Verifies: `RwLock` patterns don't deadlock under load
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_rwlock_no_deadlock() {
    const READERS: usize = 100;
    const WRITERS: usize = 10;
    const OPS: usize = 100;

    let data = Arc::new(RwLock::new(0u64));
    let mut handles = vec![];

    // Readers
    for _ in 0..READERS {
        let data = Arc::clone(&data);
        handles.push(tokio::spawn(async move {
            for _ in 0..OPS {
                let _value = data.read().await;
                // Immediate release - no artificial delays
            }
        }));
    }

    // Writers
    for _ in 0..WRITERS {
        let data = Arc::clone(&data);
        handles.push(tokio::spawn(async move {
            for _ in 0..OPS {
                let mut value = data.write().await;
                *value += 1;
                // Immediate release
            }
        }));
    }

    // All tasks complete without deadlock
    for handle in handles {
        handle.await.expect("should succeed");
    }

    let final_value = *data.read().await;
    assert_eq!(final_value, (WRITERS * OPS) as u64);
}

/// Test: Semaphore-based rate limiting
///
/// Verifies: Proper concurrency control without races
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_semaphore_rate_limiting() {
    const MAX_CONCURRENT: usize = 10;
    const TOTAL_TASKS: usize = 100;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let active_count = Arc::new(AtomicU64::new(0));
    let max_observed = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    for _ in 0..TOTAL_TASKS {
        let sem = Arc::clone(&semaphore);
        let active = Arc::clone(&active_count);
        let max_obs = Arc::clone(&max_observed);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("should succeed");

            // Track concurrency
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            max_obs.fetch_max(current, Ordering::SeqCst);

            // Simulate work without sleep
            tokio::task::yield_now().await;

            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.await.expect("should succeed");
    }

    let max = max_observed.load(Ordering::SeqCst);
    assert!(
        max <= MAX_CONCURRENT as u64,
        "Semaphore violated: max {max} concurrent (limit {MAX_CONCURRENT})"
    );
}

/// Test: Notify-based wake-ups under load
///
/// Verifies: Event-driven coordination scales
///
/// DEEP DEBT SOLUTION: Uses barrier to ensure all waiters are registered before notify.
/// No timing hacks, deterministic behavior, production-ready pattern.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_notify_coordination() {
    const WAITERS: usize = 100;

    let notify = Arc::new(Notify::new());
    let ready_count = Arc::new(AtomicU64::new(0));
    let registration_barrier = Arc::new(Barrier::new(WAITERS + 1)); // +1 for coordinator
    let mut handles = vec![];

    // Spawn waiters with proper synchronization
    for _ in 0..WAITERS {
        let notify = Arc::clone(&notify);
        let count = Arc::clone(&ready_count);
        let barrier = Arc::clone(&registration_barrier);

        handles.push(tokio::spawn(async move {
            // Register for notification FIRST
            let notified = notify.notified();

            // Signal registration complete
            barrier.wait().await;

            // Now wait for notification
            notified.await;
            count.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Wait for ALL waiters to register (deterministic, no race)
    registration_barrier.wait().await;

    // Now ALL waiters are registered - wake them up
    notify.notify_waiters();

    // All should wake up
    for handle in handles {
        handle.await.expect("should succeed");
    }

    assert_eq!(ready_count.load(Ordering::SeqCst), WAITERS as u64);
}

/// Test: Barrier synchronization stress
///
/// Verifies: Barriers work correctly under high task count
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_barrier_synchronization() {
    const TASKS: usize = 100;

    let barrier = Arc::new(Barrier::new(TASKS));
    let before_barrier = Arc::new(AtomicU64::new(0));
    let after_barrier = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    for _ in 0..TASKS {
        let barrier = Arc::clone(&barrier);
        let before = Arc::clone(&before_barrier);
        let after = Arc::clone(&after_barrier);

        handles.push(tokio::spawn(async move {
            // Increment before barrier
            before.fetch_add(1, Ordering::SeqCst);

            // Wait for all tasks
            barrier.wait().await;

            // All tasks should see full count now
            let before_count = before.load(Ordering::SeqCst);
            assert_eq!(
                before_count, TASKS as u64,
                "Barrier didn't synchronize properly"
            );

            after.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.await.expect("should succeed");
    }

    assert_eq!(before_barrier.load(Ordering::SeqCst), TASKS as u64);
    assert_eq!(after_barrier.load(Ordering::SeqCst), TASKS as u64);
}

/// Test: Message passing under load
///
/// Verifies: Channels handle high throughput
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_channel_throughput() {
    const SENDERS: usize = 10;
    const MESSAGES_PER_SENDER: usize = 1000;
    const TOTAL_MESSAGES: usize = SENDERS * MESSAGES_PER_SENDER;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
    let mut handles = vec![];

    // Spawn senders
    for sender_id in 0..SENDERS {
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            for msg_id in 0..MESSAGES_PER_SENDER {
                tx.send((sender_id, msg_id)).await.expect("should succeed");
            }
        }));
    }

    // Drop original sender
    drop(tx);

    // Receiver counts messages
    let received = Arc::new(AtomicU64::new(0));
    let received_clone = Arc::clone(&received);

    let receiver_handle = tokio::spawn(async move {
        while let Some(_msg) = rx.recv().await {
            received_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Wait for senders
    for handle in handles {
        handle.await.expect("should succeed");
    }

    // Wait for receiver
    receiver_handle.await.expect("should succeed");

    assert_eq!(received.load(Ordering::SeqCst), TOTAL_MESSAGES as u64);
}

/// Test: Complex state machine under concurrency
///
/// Verifies: State transitions remain consistent
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_state_machine_consistency() {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum State {
        Init,
        Processing,
        Complete,
    }

    let state = Arc::new(RwLock::new(State::Init));
    let transition_count = Arc::new(AtomicU64::new(0));

    // State machine: Init -> Processing -> Complete
    let state1 = Arc::clone(&state);
    let count1 = Arc::clone(&transition_count);
    let handle1 = tokio::spawn(async move {
        *state1.write().await = State::Processing;
        count1.fetch_add(1, Ordering::SeqCst);
    });

    let state2 = Arc::clone(&state);
    let count2 = Arc::clone(&transition_count);
    let handle2 = tokio::spawn(async move {
        // Wait for Processing state
        loop {
            let current = *state2.read().await;
            if current == State::Processing {
                break;
            }
            tokio::task::yield_now().await;
        }

        *state2.write().await = State::Complete;
        count2.fetch_add(1, Ordering::SeqCst);
    });

    handle1.await.expect("should succeed");
    handle2.await.expect("should succeed");

    assert_eq!(*state.read().await, State::Complete);
    assert_eq!(transition_count.load(Ordering::SeqCst), 2);
}

/// Test: Performance baseline - concurrent throughput
///
/// Verifies: Concurrent atomic operations achieve expected throughput
///
/// DEEP DEBT SOLUTION: Bounded work (no sleep), measures actual concurrent throughput.
/// Deterministic, no timing hacks, verifies production-ready concurrency performance.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_performance_baseline() {
    const WORKERS: usize = 8;
    const OPS_PER_WORKER: u64 = 1_000_000; // 1M ops per worker
    const TOTAL_OPS: u64 = WORKERS as u64 * OPS_PER_WORKER;

    let counter = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(Barrier::new(WORKERS));
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn workers with synchronized start
    for _ in 0..WORKERS {
        let counter = Arc::clone(&counter);
        let barrier = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            // Synchronized start for fair measurement
            barrier.wait().await;

            // Each worker does exactly OPS_PER_WORKER operations
            for _ in 0..OPS_PER_WORKER {
                counter.fetch_add(1, Ordering::Relaxed);
                // Yield occasionally to allow other tasks to run
                if counter.load(Ordering::Relaxed).is_multiple_of(10000) {
                    tokio::task::yield_now().await;
                }
            }
        }));
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.await.expect("should succeed");
    }

    let elapsed = start.elapsed();
    let total_ops_actual = counter.load(Ordering::Relaxed);

    // Verify: No lost operations
    assert_eq!(
        total_ops_actual, TOTAL_OPS,
        "Lost operations: expected {TOTAL_OPS}, got {total_ops_actual}"
    );

    #[expect(
        clippy::cast_precision_loss,
        reason = "Test code: explicit unwrap/expect and local lint noise"
    )]
    let ops_per_sec = total_ops_actual as f64 / elapsed.as_secs_f64();
    println!(
        "Performance: {:.2} M ops/sec ({}M ops in {:?})",
        ops_per_sec / 1_000_000.0,
        total_ops_actual / 1_000_000,
        elapsed
    );

    // Sanity check: Should achieve millions of ops/sec
    // (lowered threshold for CI environments)
    assert!(
        ops_per_sec > 500_000.0,
        "Performance too low: {ops_per_sec:.0} ops/sec (expected >500K)"
    );
}

/// Test: Zero race conditions with complex data structure
///
/// Verifies: Concurrent modifications remain consistent
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_zero_race_conditions() {
    use std::collections::HashMap;

    const WRITERS: usize = 20;
    const ENTRIES_PER_WRITER: usize = 100;

    let map = Arc::new(RwLock::new(HashMap::<usize, usize>::new()));
    let mut handles = vec![];

    for writer_id in 0..WRITERS {
        let map = Arc::clone(&map);
        handles.push(tokio::spawn(async move {
            for i in 0..ENTRIES_PER_WRITER {
                let key = writer_id * ENTRIES_PER_WRITER + i;
                map.write().await.insert(key, writer_id);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("should succeed");
    }

    // Verify: All entries present and correct
    let map_guard = map.read().await;
    assert_eq!(map_guard.len(), WRITERS * ENTRIES_PER_WRITER);

    for writer_id in 0..WRITERS {
        for i in 0..ENTRIES_PER_WRITER {
            let key = writer_id * ENTRIES_PER_WRITER + i;
            assert_eq!(map_guard.get(&key), Some(&writer_id));
        }
    }
    drop(map_guard);
}
