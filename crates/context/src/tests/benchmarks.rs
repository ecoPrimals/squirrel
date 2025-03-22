//! Performance benchmarks for the context system
//!
//! This module provides benchmarks for measuring the performance of the
//! context system, particularly focused on concurrent access scenarios.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::Barrier;
use tokio::task;
use uuid::Uuid;

use crate::{
    ContextState, ContextManager, ContextManagerConfig, 
    ContextTracker, ContextTrackerFactory, create_manager
};

/// Result of a benchmark run
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Number of operations performed
    pub operations: usize,
    /// Number of concurrent tasks
    pub concurrency: usize,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(name: &str, duration: Duration, operations: usize, concurrency: usize) -> Self {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let ops_per_second = operations as f64 / duration.as_secs_f64();
        
        Self {
            name: name.to_string(),
            duration_ms,
            ops_per_second,
            operations,
            concurrency,
        }
    }
    
    /// Print the benchmark result
    pub fn print(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Duration: {:.2} ms", self.duration_ms);
        println!("  Operations: {}", self.operations);
        println!("  Concurrency: {}", self.concurrency);
        println!("  Throughput: {:.2} ops/s", self.ops_per_second);
        println!();
    }
}

/// Run all benchmarks
pub async fn run_all_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Context creation benchmarks
    for concurrent_tasks in [1, 4, 16, 32, 64] {
        let result = benchmark_context_creation(concurrent_tasks, 100).await;
        results.push(result);
    }
    
    // Context update benchmarks
    for concurrent_tasks in [1, 4, 16, 32, 64] {
        let result = benchmark_context_update(concurrent_tasks, 1000).await;
        results.push(result);
    }
    
    // Context read benchmarks
    for concurrent_tasks in [1, 4, 16, 32, 64] {
        let result = benchmark_context_read(concurrent_tasks, 1000).await;
        results.push(result);
    }
    
    // Mixed operation benchmarks
    for concurrent_tasks in [1, 4, 16, 32, 64] {
        let result = benchmark_mixed_operations(concurrent_tasks, 1000).await;
        results.push(result);
    }
    
    results
}

/// Benchmark context creation
async fn benchmark_context_creation(concurrent_tasks: usize, contexts_per_task: usize) -> BenchmarkResult {
    let manager = create_manager();
    let barrier = Arc::new(Barrier::new(concurrent_tasks));
    
    // Create tasks
    let mut handles = Vec::with_capacity(concurrent_tasks);
    for task_id in 0..concurrent_tasks {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            
            // Create contexts
            for i in 0..contexts_per_task {
                let id = format!("bench-{}-{}", task_id, i);
                let state = create_test_state();
                manager_clone.create_context(&id, state).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Start timing
    let start = Instant::now();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // End timing
    let duration = start.elapsed();
    let operations = concurrent_tasks * contexts_per_task;
    
    // Cleanup
    // (In a real benchmark, we'd clean up the created contexts)
    
    BenchmarkResult::new(
        &format!("Context Creation ({}x concurrent)", concurrent_tasks),
        duration,
        operations,
        concurrent_tasks,
    )
}

/// Benchmark context update
async fn benchmark_context_update(concurrent_tasks: usize, updates_per_task: usize) -> BenchmarkResult {
    let manager = create_manager();
    
    // Create a single context for all tasks to update
    let context_id = "bench-update";
    let initial_state = create_test_state();
    manager.create_context(context_id, initial_state).await.unwrap();
    
    let barrier = Arc::new(Barrier::new(concurrent_tasks));
    
    // Create tasks
    let mut handles = Vec::with_capacity(concurrent_tasks);
    for task_id in 0..concurrent_tasks {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            
            // Update context
            for i in 0..updates_per_task {
                let mut state = manager_clone.get_context_state(context_id).await.unwrap();
                state.version += 1;
                state.data.insert(format!("key-{}-{}", task_id, i), format!("value-{}", i));
                manager_clone.update_context_state(context_id, state).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Start timing
    let start = Instant::now();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // End timing
    let duration = start.elapsed();
    let operations = concurrent_tasks * updates_per_task;
    
    BenchmarkResult::new(
        &format!("Context Update ({}x concurrent)", concurrent_tasks),
        duration,
        operations,
        concurrent_tasks,
    )
}

/// Benchmark context read
async fn benchmark_context_read(concurrent_tasks: usize, reads_per_task: usize) -> BenchmarkResult {
    let manager = create_manager();
    
    // Create contexts for reading
    let num_contexts = 10;
    for i in 0..num_contexts {
        let context_id = format!("bench-read-{}", i);
        let state = create_test_state();
        manager.create_context(&context_id, state).await.unwrap();
    }
    
    let barrier = Arc::new(Barrier::new(concurrent_tasks));
    
    // Create tasks
    let mut handles = Vec::with_capacity(concurrent_tasks);
    for _ in 0..concurrent_tasks {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            
            // Read contexts
            for i in 0..reads_per_task {
                let context_id = format!("bench-read-{}", i % num_contexts);
                let _state = manager_clone.get_context_state(&context_id).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Start timing
    let start = Instant::now();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // End timing
    let duration = start.elapsed();
    let operations = concurrent_tasks * reads_per_task;
    
    BenchmarkResult::new(
        &format!("Context Read ({}x concurrent)", concurrent_tasks),
        duration,
        operations,
        concurrent_tasks,
    )
}

/// Benchmark mixed operations (read, update, create)
async fn benchmark_mixed_operations(concurrent_tasks: usize, operations_per_task: usize) -> BenchmarkResult {
    let manager = create_manager();
    
    // Create initial contexts
    let num_contexts = 10;
    for i in 0..num_contexts {
        let context_id = format!("bench-mixed-{}", i);
        let state = create_test_state();
        manager.create_context(&context_id, state).await.unwrap();
    }
    
    let barrier = Arc::new(Barrier::new(concurrent_tasks));
    
    // Create tasks
    let mut handles = Vec::with_capacity(concurrent_tasks);
    for task_id in 0..concurrent_tasks {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            
            // Perform mixed operations
            for i in 0..operations_per_task {
                // Determine operation based on i
                match i % 3 {
                    0 => {
                        // Read operation
                        let context_id = format!("bench-mixed-{}", i % num_contexts);
                        let _state = manager_clone.get_context_state(&context_id).await.unwrap();
                    },
                    1 => {
                        // Update operation
                        let context_id = format!("bench-mixed-{}", i % num_contexts);
                        let mut state = manager_clone.get_context_state(&context_id).await.unwrap();
                        state.version += 1;
                        state.data.insert(format!("key-{}-{}", task_id, i), format!("value-{}", i));
                        manager_clone.update_context_state(&context_id, state).await.unwrap();
                    },
                    2 => {
                        // Create operation
                        let context_id = format!("bench-mixed-{}-{}", task_id, i);
                        let state = create_test_state();
                        let _ = manager_clone.create_context(&context_id, state).await;
                    },
                    _ => unreachable!(),
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Start timing
    let start = Instant::now();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // End timing
    let duration = start.elapsed();
    let operations = concurrent_tasks * operations_per_task;
    
    BenchmarkResult::new(
        &format!("Mixed Operations ({}x concurrent)", concurrent_tasks),
        duration,
        operations,
        concurrent_tasks,
    )
}

/// Create a test state for benchmarking
fn create_test_state() -> ContextState {
    ContextState {
        id: Uuid::new_v4().to_string(),
        version: 1,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        data: HashMap::new(),
        metadata: HashMap::new(),
        synchronized: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_benchmarks() {
        // This test is marked as ignore by default because it's a performance benchmark
        // and not a functional test. Run it explicitly with `cargo test -- --ignored`.
        
        println!("Running context system benchmarks...");
        let results = run_all_benchmarks().await;
        
        for result in results {
            result.print();
        }
    }
} 