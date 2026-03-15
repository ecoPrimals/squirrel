// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{Semaphore, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use uuid::Uuid;

// Mock components for stress testing
struct StressTestContext {
    active_connections: Arc<AtomicUsize>,
    total_requests: Arc<AtomicUsize>,
    failed_requests: Arc<AtomicUsize>,
    memory_usage: Arc<AtomicUsize>,
    is_running: Arc<AtomicBool>,
}

impl StressTestContext {
    fn new() -> Self {
        Self {
            active_connections: Arc::new(AtomicUsize::new(0)),
            total_requests: Arc::new(AtomicUsize::new(0)),
            failed_requests: Arc::new(AtomicUsize::new(0)),
            memory_usage: Arc::new(AtomicUsize::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    async fn simulate_request(&self, latency: Duration, should_fail: bool) -> Result<String, String> {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        // Simulate memory allocation
        let memory_increase = rand::random::<usize>() % 1024;
        self.memory_usage.fetch_add(memory_increase, Ordering::Relaxed);
        
        tokio::time::sleep(latency).await;
        
        let result = if should_fail {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
            Err("Simulated failure".to_string())
        } else {
            Ok(format!("Success-{}", Uuid::new_v4()))
        };
        
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.memory_usage.fetch_sub(memory_increase, Ordering::Relaxed);
        
        result
    }

    fn get_metrics(&self) -> (usize, usize, usize, usize) {
        (
            self.active_connections.load(Ordering::Relaxed),
            self.total_requests.load(Ordering::Relaxed),
            self.failed_requests.load(Ordering::Relaxed),
            self.memory_usage.load(Ordering::Relaxed),
        )
    }
}

/// Stress test with high concurrency
fn stress_test_high_concurrency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_high_concurrency");
    
    // Test different concurrency levels under stress
    for concurrency in [100, 500, 1000, 2000].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let context = Arc::new(StressTestContext::new());
                    let semaphore = Arc::new(Semaphore::new(concurrency / 2)); // Limit active requests
                    
                    let mut futures = FuturesUnordered::new();
                    
                    // Launch concurrent requests
                    for i in 0..concurrency {
                        let context_clone = context.clone();
                        let semaphore_clone = semaphore.clone();
                        
                        futures.push(tokio::spawn(async move {
                            let _permit = semaphore_clone.acquire().await.unwrap();
                            
                            // Vary request characteristics
                            let latency = Duration::from_millis(rand::random::<u64>() % 100);
                            let should_fail = rand::random::<f32>() < 0.05; // 5% failure rate
                            
                            context_clone.simulate_request(latency, should_fail).await
                        }));
                    }
                    
                    // Wait for all requests to complete
                    while let Some(result) = futures.next().await {
                        let _ = black_box(result.unwrap());
                    }
                    
                    let metrics = context.get_metrics();
                    black_box(metrics);
                });
            },
        );
    }
    
    group.finish();
}

/// Stress test memory allocation patterns
fn stress_test_memory_pressure(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_memory_pressure");
    
    // Test different memory pressure scenarios
    for memory_size in [1024, 10240, 102400, 1024000].iter() {
        group.throughput(Throughput::Bytes(*memory_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("memory_allocation", memory_size),
            memory_size,
            |b, &memory_size| {
                b.to_async(&rt).iter(|| async {
                    let mut allocations: Vec<Vec<u8>> = Vec::new();
                    
                    // Rapidly allocate and deallocate memory
                    for i in 0..100 {
                        // Allocate chunks of varying sizes
                        let chunk_size = (memory_size / 100) + (rand::random::<usize>() % 1024);
                        let mut chunk = vec![0u8; chunk_size];
                        
                        // Write to memory to ensure allocation
                        for j in 0..chunk.len() {
                            chunk[j] = (i + j) as u8;
                        }
                        
                        allocations.push(chunk);
                        
                        // Occasionally free some memory
                        if i % 10 == 0 && !allocations.is_empty() {
                            let remove_count = allocations.len() / 3;
                            for _ in 0..remove_count {
                                allocations.pop();
                            }
                        }
                    }
                    
                    // Verify memory was used
                    let total_allocated: usize = allocations.iter().map(|v| v.len()).sum();
                    black_box(total_allocated);
                });
            },
        );
    }
    
    group.finish();
}

/// Stress test resource contention
fn stress_test_resource_contention(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_resource_contention");
    
    // Test contention on shared resources
    for threads in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*threads as u64));
        
        group.bench_with_input(
            BenchmarkId::new("shared_resource_contention", threads),
            threads,
            |b, &threads| {
                b.to_async(&rt).iter(|| async {
                    let shared_state = Arc::new(RwLock::new(HashMap::<String, usize>::new()));
                    let mut handles = Vec::new();
                    
                    // Create contending tasks
                    for i in 0..threads {
                        let shared_state_clone = shared_state.clone();
                        
                        handles.push(tokio::spawn(async move {
                            let key = format!("resource-{}", i % 10); // Limited set of keys for contention
                            
                            // Mix of reads and writes
                            for _ in 0..50 {
                                if rand::random::<bool>() {
                                    // Read operation
                                    let guard = shared_state_clone.read().await;
                                    let _ = black_box(guard.get(&key));
                                } else {
                                    // Write operation
                                    let mut guard = shared_state_clone.write().await;
                                    let current = guard.get(&key).unwrap_or(&0) + 1;
                                    guard.insert(key.clone(), current);
                                }
                                
                                // Small delay to increase contention window
                                tokio::time::sleep(Duration::from_micros(10)).await;
                            }
                        }));
                    }
                    
                    // Wait for all tasks to complete
                    for handle in handles {
                        let _ = handle.await.unwrap();
                    }
                    
                    let final_state = shared_state.read().await;
                    black_box(final_state.len());
                });
            },
        );
    }
    
    group.finish();
}

/// Stress test error cascade scenarios
fn stress_test_error_cascades(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_error_cascades");
    
    // Simulate error cascades and recovery
    group.bench_function("error_cascade_recovery", |b| {
        b.to_async(&rt).iter(|| async {
            let context = Arc::new(StressTestContext::new());
            let error_rate = Arc::new(AtomicUsize::new(5)); // Start with 5% error rate
            
            // Simulate system under increasing error pressure
            let mut phase_handles = Vec::new();
            
            // Phase 1: Normal operation
            for i in 0..100 {
                let context_clone = context.clone();
                let error_rate_clone = error_rate.clone();
                
                phase_handles.push(tokio::spawn(async move {
                    let should_fail = (rand::random::<usize>() % 100) < error_rate_clone.load(Ordering::Relaxed);
                    context_clone.simulate_request(Duration::from_millis(10), should_fail).await
                }));
            }
            
            // Phase 2: Increase error rate (cascade begins)
            error_rate.store(25, Ordering::Relaxed);
            
            for i in 0..100 {
                let context_clone = context.clone();
                let error_rate_clone = error_rate.clone();
                
                phase_handles.push(tokio::spawn(async move {
                    let should_fail = (rand::random::<usize>() % 100) < error_rate_clone.load(Ordering::Relaxed);
                    context_clone.simulate_request(Duration::from_millis(20), should_fail).await
                }));
            }
            
            // Phase 3: Recovery (reduce error rate)
            error_rate.store(10, Ordering::Relaxed);
            
            for i in 0..100 {
                let context_clone = context.clone();
                let error_rate_clone = error_rate.clone();
                
                phase_handles.push(tokio::spawn(async move {
                    let should_fail = (rand::random::<usize>() % 100) < error_rate_clone.load(Ordering::Relaxed);
                    context_clone.simulate_request(Duration::from_millis(15), should_fail).await
                }));
            }
            
            // Wait for all phases to complete
            for handle in phase_handles {
                let _ = black_box(handle.await.unwrap());
            }
            
            let (active, total, failed, memory) = context.get_metrics();
            black_box((active, total, failed, memory));
        });
    });
    
    group.finish();
}

/// Stress test timeout handling under load
fn stress_test_timeout_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_timeout_handling");
    
    // Test timeout handling under different load levels
    for load_level in [50, 100, 200, 500].iter() {
        group.throughput(Throughput::Elements(*load_level as u64));
        
        group.bench_with_input(
            BenchmarkId::new("timeout_under_load", load_level),
            load_level,
            |b, &load_level| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    let timeout_count = Arc::new(AtomicUsize::new(0));
                    let success_count = Arc::new(AtomicUsize::new(0));
                    
                    for i in 0..load_level {
                        let timeout_count_clone = timeout_count.clone();
                        let success_count_clone = success_count.clone();
                        
                        handles.push(tokio::spawn(async move {
                            // Vary operation duration to trigger timeouts
                            let operation_duration = Duration::from_millis(
                                50 + (rand::random::<u64>() % 150)
                            );
                            let timeout_duration = Duration::from_millis(100);
                            
                            match tokio::time::timeout(
                                timeout_duration,
                                tokio::time::sleep(operation_duration)
                            ).await {
                                Ok(_) => {
                                    success_count_clone.fetch_add(1, Ordering::Relaxed);
                                    Ok(())
                                },
                                Err(_) => {
                                    timeout_count_clone.fetch_add(1, Ordering::Relaxed);
                                    Err("Timeout")
                                }
                            }
                        }));
                    }
                    
                    // Wait for all operations
                    for handle in handles {
                        let _ = black_box(handle.await.unwrap());
                    }
                    
                    let timeouts = timeout_count.load(Ordering::Relaxed);
                    let successes = success_count.load(Ordering::Relaxed);
                    black_box((timeouts, successes));
                });
            },
        );
    }
    
    group.finish();
}

/// Stress test graceful degradation
fn stress_test_graceful_degradation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_graceful_degradation");
    
    group.bench_function("degradation_under_pressure", |b| {
        b.to_async(&rt).iter(|| async {
            let max_capacity = Arc::new(Semaphore::new(100));
            let degradation_mode = Arc::new(AtomicBool::new(false));
            let processed_requests = Arc::new(AtomicUsize::new(0));
            let rejected_requests = Arc::new(AtomicUsize::new(0));
            
            let mut request_handles = Vec::new();
            
            // Generate high load (more than capacity)
            for i in 0..500 {
                let max_capacity_clone = max_capacity.clone();
                let degradation_mode_clone = degradation_mode.clone();
                let processed_clone = processed_requests.clone();
                let rejected_clone = rejected_requests.clone();
                
                request_handles.push(tokio::spawn(async move {
                    // Try to acquire capacity
                    if let Ok(permit) = max_capacity_clone.try_acquire() {
                        // Normal processing
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        processed_clone.fetch_add(1, Ordering::Relaxed);
                        drop(permit);
                        Ok("Processed")
                    } else {
                        // Capacity exceeded - check degradation mode
                        if degradation_mode_clone.load(Ordering::Relaxed) {
                            // Degraded processing (faster, lower quality)
                            tokio::time::sleep(Duration::from_millis(5)).await;
                            processed_clone.fetch_add(1, Ordering::Relaxed);
                            Ok("Degraded")
                        } else {
                            // Reject request
                            rejected_clone.fetch_add(1, Ordering::Relaxed);
                            Err("Rejected")
                        }
                    }
                }));
                
                // Enable degradation mode after capacity is exceeded
                if i == 150 {
                    degradation_mode.store(true, Ordering::Relaxed);
                }
            }
            
            // Wait for all requests
            for handle in request_handles {
                let _ = black_box(handle.await.unwrap());
            }
            
            let processed = processed_requests.load(Ordering::Relaxed);
            let rejected = rejected_requests.load(Ordering::Relaxed);
            black_box((processed, rejected));
        });
    });
    
    group.finish();
}

/// Stress test system recovery after failure
fn stress_test_failure_recovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("stress_test_failure_recovery");
    
    group.bench_function("recovery_after_total_failure", |b| {
        b.to_async(&rt).iter(|| async {
            let system_healthy = Arc::new(AtomicBool::new(true));
            let recovery_attempts = Arc::new(AtomicUsize::new(0));
            let successful_operations = Arc::new(AtomicUsize::new(0));
            
            let mut operation_handles = Vec::new();
            
            // Phase 1: Normal operation
            for i in 0..50 {
                let system_healthy_clone = system_healthy.clone();
                let successful_clone = successful_operations.clone();
                
                operation_handles.push(tokio::spawn(async move {
                    if system_healthy_clone.load(Ordering::Relaxed) {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        successful_clone.fetch_add(1, Ordering::Relaxed);
                        Ok("Success")
                    } else {
                        Err("System unhealthy")
                    }
                }));
            }
            
            // Trigger system failure
            tokio::time::sleep(Duration::from_millis(100)).await;
            system_healthy.store(false, Ordering::Relaxed);
            
            // Phase 2: Failure period with recovery attempts
            for i in 0..100 {
                let system_healthy_clone = system_healthy.clone();
                let recovery_attempts_clone = recovery_attempts.clone();
                let successful_clone = successful_operations.clone();
                
                operation_handles.push(tokio::spawn(async move {
                    if !system_healthy_clone.load(Ordering::Relaxed) {
                        // Attempt recovery
                        recovery_attempts_clone.fetch_add(1, Ordering::Relaxed);
                        
                        // Simulate recovery logic
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        
                        // Recovery success after several attempts
                        if recovery_attempts_clone.load(Ordering::Relaxed) > 20 {
                            system_healthy_clone.store(true, Ordering::Relaxed);
                        }
                        
                        Err("Recovery in progress")
                    } else {
                        successful_clone.fetch_add(1, Ordering::Relaxed);
                        Ok("Recovered")
                    }
                }));
            }
            
            // Wait for all operations
            for handle in operation_handles {
                let _ = black_box(handle.await.unwrap());
            }
            
            let attempts = recovery_attempts.load(Ordering::Relaxed);
            let successes = successful_operations.load(Ordering::Relaxed);
            let recovered = system_healthy.load(Ordering::Relaxed);
            
            black_box((attempts, successes, recovered));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    stress_test_high_concurrency,
    stress_test_memory_pressure,
    stress_test_resource_contention,
    stress_test_error_cascades,
    stress_test_timeout_handling,
    stress_test_graceful_degradation,
    stress_test_failure_recovery
);
criterion_main!(benches); 