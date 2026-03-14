// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{Semaphore, Notify, broadcast, mpsc};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use futures::future::join_all;

/// Shared state for concurrent operations testing
#[derive(Debug)]
struct SharedState {
    counter: AtomicUsize,
    data: RwLock<HashMap<String, String>>,
    operations_completed: AtomicUsize,
    errors_encountered: AtomicUsize,
}

impl SharedState {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            data: RwLock::new(HashMap::new()),
            operations_completed: AtomicUsize::new(0),
            errors_encountered: AtomicUsize::new(0),
        }
    }

    fn increment_counter(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    fn read_data(&self, key: &str) -> Option<String> {
        let guard = self.data.read().unwrap();
        guard.get(key).cloned()
    }

    fn write_data(&self, key: String, value: String) -> bool {
        match self.data.write() {
            Ok(mut guard) => {
                guard.insert(key, value);
                self.operations_completed.fetch_add(1, Ordering::Relaxed);
                true
            },
            Err(_) => {
                self.errors_encountered.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    fn get_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.counter.load(Ordering::Relaxed),
            self.data.read().unwrap().len(),
            self.operations_completed.load(Ordering::Relaxed),
            self.errors_encountered.load(Ordering::Relaxed),
        )
    }
}

/// Benchmark basic concurrent operations
fn benchmark_basic_concurrency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("basic_concurrency");
    
    // Test different thread counts
    for thread_count in [1, 4, 8, 16, 32].iter() {
        group.throughput(Throughput::Elements(*thread_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("atomic_operations", thread_count),
            thread_count,
            |b, &thread_count| {
                b.to_async(&rt).iter(|| async {
                    let state = Arc::new(SharedState::new());
                    let mut handles = Vec::new();
                    
                    // Create concurrent tasks performing atomic operations
                    for i in 0..thread_count {
                        let state_clone = state.clone();
                        
                        handles.push(tokio::spawn(async move {
                            // Each task performs multiple operations
                            for j in 0..1000 {
                                // Atomic counter increment
                                let current = state_clone.increment_counter();
                                
                                // Write operation with contention
                                let key = format!("key-{}-{}", i, j % 100);
                                let value = format!("value-{}-{}", current, j);
                                state_clone.write_data(key.clone(), value);
                                
                                // Read operation
                                if j % 10 == 0 {
                                    let _ = state_clone.read_data(&key);
                                }
                            }
                        }));
                    }
                    
                    // Wait for all tasks to complete
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    let stats = state.get_stats();
                    black_box(stats);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark channel-based communication
fn benchmark_channel_communication(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("channel_communication");
    
    // Test different message volumes
    for message_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*message_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("mpsc_channel", message_count),
            message_count,
            |b, &message_count| {
                b.to_async(&rt).iter(|| async {
                    let (tx, mut rx) = mpsc::channel(100);
                    
                    // Sender task
                    let sender_handle = tokio::spawn(async move {
                        for i in 0..message_count {
                            let message = format!("message-{}", i);
                            if let Err(_) = tx.send(message).await {
                                break;
                            }
                        }
                    });
                    
                    // Receiver task
                    let receiver_handle = tokio::spawn(async move {
                        let mut received_count = 0;
                        while let Some(message) = rx.recv().await {
                            black_box(message);
                            received_count += 1;
                            if received_count >= message_count {
                                break;
                            }
                        }
                        received_count
                    });
                    
                    // Wait for both tasks
                    let (_, received_count) = tokio::join!(sender_handle, receiver_handle);
                    let received = received_count.unwrap();
                    
                    black_box(received);
                });
            },
        );
    }
    
    // Test broadcast channels
    for subscriber_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("broadcast_channel", subscriber_count),
            subscriber_count,
            |b, &subscriber_count| {
                b.to_async(&rt).iter(|| async {
                    let (tx, _) = broadcast::channel(100);
                    
                    // Create subscribers
                    let mut subscriber_handles = Vec::new();
                    for i in 0..subscriber_count {
                        let mut rx = tx.subscribe();
                        subscriber_handles.push(tokio::spawn(async move {
                            let mut received = 0;
                            while let Ok(message) = rx.recv().await {
                                black_box(message);
                                received += 1;
                                if received >= 100 {
                                    break;
                                }
                            }
                            received
                        }));
                    }
                    
                    // Broadcaster
                    let broadcaster_handle = tokio::spawn(async move {
                        for i in 0..100 {
                            let message = format!("broadcast-{}", i);
                            if let Err(_) = tx.send(message) {
                                break;
                            }
                        }
                    });
                    
                    // Wait for all tasks
                    let _ = broadcaster_handle.await;
                    let results = join_all(subscriber_handles).await;
                    
                    let total_received: usize = results.into_iter().map(|r| r.unwrap()).sum();
                    black_box(total_received);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark lock contention scenarios
fn benchmark_lock_contention(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("lock_contention");
    
    // Test different contention levels
    for contention_level in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("mutex_contention", contention_level),
            contention_level,
            |b, &contention_level| {
                b.to_async(&rt).iter(|| async {
                    let shared_data = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
                    let mut handles = Vec::new();
                    
                    for i in 0..contention_level {
                        let data_clone = shared_data.clone();
                        
                        handles.push(tokio::spawn(async move {
                            for j in 0..500 {
                                // High contention: all threads access same keys
                                let key = format!("shared-key-{}", j % 10);
                                
                                // Lock for write
                                {
                                    let mut guard = data_clone.lock().unwrap();
                                    let current = guard.get(&key).unwrap_or(&0) + 1;
                                    guard.insert(key.clone(), current);
                                }
                                
                                // Small delay to increase contention window
                                tokio::time::sleep(Duration::from_micros(1)).await;
                                
                                // Lock for read
                                {
                                    let guard = data_clone.lock().unwrap();
                                    let _ = guard.get(&key);
                                }
                            }
                        }));
                    }
                    
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    let final_size = shared_data.lock().unwrap().len();
                    black_box(final_size);
                });
            },
        );
    }
    
    // Test RwLock performance with read-heavy workload
    for reader_count in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("rwlock_read_heavy", reader_count),
            reader_count,
            |b, &reader_count| {
                b.to_async(&rt).iter(|| async {
                    let shared_data = Arc::new(RwLock::new(HashMap::<String, String>::new()));
                    
                    // Pre-populate data
                    {
                        let mut guard = shared_data.write().unwrap();
                        for i in 0..100 {
                            guard.insert(format!("key-{}", i), format!("value-{}", i));
                        }
                    }
                    
                    let mut handles = Vec::new();
                    
                    // Many readers
                    for i in 0..reader_count {
                        let data_clone = shared_data.clone();
                        handles.push(tokio::spawn(async move {
                            let mut read_count = 0;
                            for j in 0..1000 {
                                let key = format!("key-{}", j % 100);
                                let guard = data_clone.read().unwrap();
                                if let Some(value) = guard.get(&key) {
                                    black_box(value);
                                    read_count += 1;
                                }
                            }
                            read_count
                        }));
                    }
                    
                    // One occasional writer
                    let data_clone = shared_data.clone();
                    handles.push(tokio::spawn(async move {
                        for i in 0..10 {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            let key = format!("new-key-{}", i);
                            let value = format!("new-value-{}", i);
                            let mut guard = data_clone.write().unwrap();
                            guard.insert(key, value);
                        }
                    }));
                    
                    let results = join_all(handles).await;
                    let total_reads: usize = results.into_iter()
                        .filter_map(|r| r.unwrap().try_into().ok())
                        .sum();
                    
                    black_box(total_reads);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark async task spawning and coordination
fn benchmark_async_coordination(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("async_coordination");
    
    // Test different task counts
    for task_count in [10, 50, 200, 1000].iter() {
        group.throughput(Throughput::Elements(*task_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("task_spawning", task_count),
            task_count,
            |b, &task_count| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    // Spawn many tasks
                    for i in 0..task_count {
                        handles.push(tokio::spawn(async move {
                            // Simulate some async work
                            tokio::time::sleep(Duration::from_micros(100)).await;
                            
                            // Some computation
                            let mut result = 0;
                            for j in 0..100 {
                                result += (i + j) % 1000;
                            }
                            
                            result
                        }));
                    }
                    
                    // Wait for all tasks
                    let results = join_all(handles).await;
                    let total: usize = results.into_iter()
                        .filter_map(|r| r.ok())
                        .sum();
                    
                    black_box(total);
                });
            },
        );
    }
    
    // Test coordination with Notify
    group.bench_function("notify_coordination", |b| {
        b.to_async(&rt).iter(|| async {
            let notify = Arc::new(Notify::new());
            let completion_count = Arc::new(AtomicUsize::new(0));
            
            let mut handles = Vec::new();
            
            // Worker tasks waiting for notification
            for i in 0..20 {
                let notify_clone = notify.clone();
                let count_clone = completion_count.clone();
                
                handles.push(tokio::spawn(async move {
                    // Wait for notification
                    notify_clone.notified().await;
                    
                    // Do work after notification
                    tokio::time::sleep(Duration::from_micros(50)).await;
                    let mut result = 0;
                    for j in 0..50 {
                        result += (i * j) % 100;
                    }
                    
                    count_clone.fetch_add(1, Ordering::Relaxed);
                    result
                }));
            }
            
            // Coordinator task
            let coordinator_handle = tokio::spawn(async move {
                // Wait a bit, then notify all
                tokio::time::sleep(Duration::from_millis(10)).await;
                notify.notify_waiters();
            });
            
            // Wait for coordination and all workers
            coordinator_handle.await.unwrap();
            let results = join_all(handles).await;
            
            let completed = completion_count.load(Ordering::Relaxed);
            let total: usize = results.into_iter().filter_map(|r| r.ok()).sum();
            
            black_box((completed, total));
        });
    });
    
    group.finish();
}

/// Benchmark semaphore-based rate limiting
fn benchmark_semaphore_rate_limiting(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("semaphore_rate_limiting");
    
    // Test different permit counts and request loads
    for (permits, requests) in [(5, 100), (10, 500), (20, 1000), (50, 2000)].iter() {
        group.bench_with_input(
            BenchmarkId::new("rate_limited_requests", format!("{}permits_{}requests", permits, requests)),
            &(*permits, *requests),
            |b, &(permits, requests)| {
                b.to_async(&rt).iter(|| async {
                    let semaphore = Arc::new(Semaphore::new(permits));
                    let completed_count = Arc::new(AtomicUsize::new(0));
                    let mut handles = Vec::new();
                    
                    // Create more requests than permits (rate limiting)
                    for i in 0..requests {
                        let semaphore_clone = semaphore.clone();
                        let completed_clone = completed_count.clone();
                        
                        handles.push(tokio::spawn(async move {
                            // Acquire permit (may block if none available)
                            let _permit = semaphore_clone.acquire().await.unwrap();
                            
                            // Simulate work while holding permit
                            tokio::time::sleep(Duration::from_millis(5)).await;
                            
                            // Do some computation
                            let mut result = 0;
                            for j in 0..10 {
                                result += (i + j) % 100;
                            }
                            
                            completed_clone.fetch_add(1, Ordering::Relaxed);
                            result
                        }));
                    }
                    
                    // Wait for all requests to complete
                    let results = join_all(handles).await;
                    let completed = completed_count.load(Ordering::Relaxed);
                    let total: usize = results.into_iter().filter_map(|r| r.ok()).sum();
                    
                    black_box((completed, total));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark futures coordination patterns
fn benchmark_futures_coordination(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("futures_coordination");
    
    group.bench_function("unordered_futures", |b| {
        b.to_async(&rt).iter(|| async {
            let mut futures = FuturesUnordered::new();
            
            // Create futures with varying completion times
            for i in 0..100 {
                let delay = Duration::from_millis(i % 50);
                futures.push(tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    i * i
                }));
            }
            
            // Collect results as they complete
            let mut results = Vec::new();
            while let Some(result) = futures.next().await {
                results.push(result.unwrap());
            }
            
            black_box(results);
        });
    });
    
    group.bench_function("select_coordination", |b| {
        b.to_async(&rt).iter(|| async {
            let mut completed_operations = Vec::new();
            
            for i in 0..50 {
                // Race between fast and slow operations
                let fast_op = tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    format!("fast-{}", i)
                });
                
                let slow_op = tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    format!("slow-{}", i)
                });
                
                // Select the first to complete
                let result = tokio::select! {
                    fast_result = fast_op => fast_result.unwrap(),
                    slow_result = slow_op => slow_result.unwrap(),
                };
                
                completed_operations.push(result);
            }
            
            black_box(completed_operations);
        });
    });
    
    group.finish();
}

/// Benchmark error propagation in concurrent scenarios
fn benchmark_concurrent_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_error_handling");
    
    group.bench_function("error_propagation", |b| {
        b.to_async(&rt).iter(|| async {
            let success_count = Arc::new(AtomicUsize::new(0));
            let error_count = Arc::new(AtomicUsize::new(0));
            let mut handles = Vec::new();
            
            // Create tasks with varying failure rates
            for i in 0..200 {
                let success_clone = success_count.clone();
                let error_clone = error_count.clone();
                
                handles.push(tokio::spawn(async move {
                    // 20% failure rate
                    if i % 5 == 0 {
                        error_clone.fetch_add(1, Ordering::Relaxed);
                        Err(format!("Simulated error {}", i))
                    } else {
                        success_clone.fetch_add(1, Ordering::Relaxed);
                        Ok(i * 2)
                    }
                }));
            }
            
            // Collect all results, handling errors
            let mut successful_results = Vec::new();
            let mut error_messages = Vec::new();
            
            for handle in handles {
                match handle.await.unwrap() {
                    Ok(value) => successful_results.push(value),
                    Err(error) => error_messages.push(error),
                }
            }
            
            let success_final = success_count.load(Ordering::Relaxed);
            let error_final = error_count.load(Ordering::Relaxed);
            
            black_box((success_final, error_final, successful_results.len(), error_messages.len()));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_basic_concurrency,
    benchmark_channel_communication,
    benchmark_lock_contention,
    benchmark_async_coordination,
    benchmark_semaphore_rate_limiting,
    benchmark_futures_coordination,
    benchmark_concurrent_error_handling
);
criterion_main!(benches); 