// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Memory tracking structure
struct MemoryTracker {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    peak_usage: AtomicUsize,
    current_usage: AtomicUsize,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            current_usage: AtomicUsize::new(0),
        }
    }

    fn allocate(&self, size: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let new_usage = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak usage if necessary
        let mut current_peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > current_peak {
            match self.peak_usage.compare_exchange_weak(
                current_peak,
                new_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    fn deallocate(&self, size: usize) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.allocations.load(Ordering::Relaxed),
            self.deallocations.load(Ordering::Relaxed),
            self.current_usage.load(Ordering::Relaxed),
            self.peak_usage.load(Ordering::Relaxed),
        )
    }
}

/// Mock data structure with complex memory patterns
struct ComplexDataStructure {
    id: String,
    data: HashMap<String, Vec<u8>>,
    history: VecDeque<String>,
    metadata: HashMap<String, String>,
    cache: Arc<HashMap<String, Vec<u8>>>,
}

impl ComplexDataStructure {
    fn new(id: String, initial_size: usize) -> Self {
        let mut data = HashMap::new();
        let mut metadata = HashMap::new();
        let mut cache_data = HashMap::new();
        
        // Pre-populate with varying sizes
        for i in 0..initial_size {
            let key = format!("key-{}", i);
            let value = vec![0u8; 100 + (i % 1000)]; // Varying sizes
            data.insert(key.clone(), value);
            
            // Add metadata
            metadata.insert(key.clone(), format!("metadata-{}", i));
            
            // Add to cache with larger data
            let cache_value = vec![0u8; 500 + (i % 2000)];
            cache_data.insert(key, cache_value);
        }
        
        Self {
            id,
            data,
            history: VecDeque::new(),
            metadata,
            cache: Arc::new(cache_data),
        }
    }

    fn add_data(&mut self, key: String, size: usize) {
        let value = vec![0u8; size];
        self.data.insert(key.clone(), value);
        self.history.push_back(key.clone());
        self.metadata.insert(key, "new_entry".to_string());
        
        // Limit history size to prevent unbounded growth
        if self.history.len() > 1000 {
            if let Some(old_key) = self.history.pop_front() {
                self.data.remove(&old_key);
                self.metadata.remove(&old_key);
            }
        }
    }

    fn get_memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        
        // Calculate data size
        for (key, value) in &self.data {
            size += key.len() + value.len() + std::mem::size_of::<String>() + std::mem::size_of::<Vec<u8>>();
        }
        
        // Calculate metadata size
        for (key, value) in &self.metadata {
            size += key.len() + value.len() + 2 * std::mem::size_of::<String>();
        }
        
        // Add history size
        for entry in &self.history {
            size += entry.len() + std::mem::size_of::<String>();
        }
        
        size
    }
}

/// Benchmark memory allocation patterns
fn benchmark_allocation_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("allocation_patterns");
    
    // Test different allocation sizes
    for size in [1024, 10240, 102400, 1024000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("rapid_allocation_deallocation", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let tracker = MemoryTracker::new();
                    let mut allocations: Vec<Vec<u8>> = Vec::new();
                    
                    // Phase 1: Rapid allocation
                    for i in 0..100 {
                        let chunk_size = size / 100 + (i % 1000);
                        let chunk = vec![0u8; chunk_size];
                        tracker.allocate(chunk_size);
                        allocations.push(chunk);
                    }
                    
                    // Phase 2: Partial deallocation
                    for _ in 0..50 {
                        if let Some(chunk) = allocations.pop() {
                            tracker.deallocate(chunk.len());
                        }
                    }
                    
                    // Phase 3: More allocation
                    for i in 0..75 {
                        let chunk_size = size / 75 + (i % 500);
                        let chunk = vec![1u8; chunk_size]; // Different pattern
                        tracker.allocate(chunk_size);
                        allocations.push(chunk);
                    }
                    
                    let stats = tracker.get_stats();
                    black_box(stats);
                    black_box(allocations);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory fragmentation patterns
fn benchmark_fragmentation_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("fragmentation_patterns");
    
    group.bench_function("fragmentation_simulation", |b| {
        b.to_async(&rt).iter(|| async {
            let tracker = MemoryTracker::new();
            let mut small_allocations: Vec<Vec<u8>> = Vec::new();
            let mut large_allocations: Vec<Vec<u8>> = Vec::new();
            
            // Create fragmentation by alternating small and large allocations
            for i in 0..200 {
                if i % 3 == 0 {
                    // Large allocation
                    let size = 50000 + (i % 50000);
                    let chunk = vec![0u8; size];
                    tracker.allocate(size);
                    large_allocations.push(chunk);
                } else {
                    // Small allocation
                    let size = 100 + (i % 1000);
                    let chunk = vec![0u8; size];
                    tracker.allocate(size);
                    small_allocations.push(chunk);
                }
            }
            
            // Free every other small allocation (creates holes)
            for i in (0..small_allocations.len()).step_by(2) {
                if i < small_allocations.len() {
                    let chunk = &small_allocations[i];
                    tracker.deallocate(chunk.len());
                }
            }
            small_allocations = small_allocations.into_iter().enumerate()
                .filter_map(|(idx, chunk)| if idx % 2 == 0 { None } else { Some(chunk) })
                .collect();
            
            // Try to allocate medium-sized chunks (may not fit in holes)
            let mut medium_allocations: Vec<Vec<u8>> = Vec::new();
            for i in 0..50 {
                let size = 5000 + (i % 10000);
                let chunk = vec![0u8; size];
                tracker.allocate(size);
                medium_allocations.push(chunk);
            }
            
            let stats = tracker.get_stats();
            black_box(stats);
            black_box((small_allocations, large_allocations, medium_allocations));
        });
    });
    
    group.finish();
}

/// Benchmark complex data structure memory usage
fn benchmark_complex_structures(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("complex_structures");
    
    // Test different structure sizes
    for initial_size in [100, 500, 1000, 2000].iter() {
        group.throughput(Throughput::Elements(*initial_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("complex_structure_operations", initial_size),
            initial_size,
            |b, &initial_size| {
                b.to_async(&rt).iter(|| async {
                    let mut structures: Vec<ComplexDataStructure> = Vec::new();
                    
                    // Create multiple complex structures
                    for i in 0..10 {
                        let id = format!("structure-{}", i);
                        let mut structure = ComplexDataStructure::new(id, initial_size);
                        
                        // Perform operations that change memory patterns
                        for j in 0..100 {
                            let key = format!("dynamic-{}-{}", i, j);
                            let size = 200 + (j % 1000);
                            structure.add_data(key, size);
                        }
                        
                        structures.push(structure);
                    }
                    
                    // Calculate total memory usage
                    let total_memory: usize = structures.iter().map(|s| s.get_memory_usage()).sum();
                    
                    black_box(total_memory);
                    black_box(structures);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage under concurrent access
fn benchmark_concurrent_memory_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_memory_access");
    
    // Test different concurrency levels
    for concurrency in [10, 25, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        
        group.bench_with_input(
            BenchmarkId::new("concurrent_allocations", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let tracker = Arc::new(MemoryTracker::new());
                    let mut handles = Vec::new();
                    
                    // Create concurrent tasks that allocate and deallocate memory
                    for i in 0..concurrency {
                        let tracker_clone = tracker.clone();
                        
                        handles.push(tokio::spawn(async move {
                            let mut local_allocations: Vec<Vec<u8>> = Vec::new();
                            
                            // Each task performs mixed allocation/deallocation patterns
                            for j in 0..100 {
                                let operation = (i + j) % 4;
                                
                                match operation {
                                    0 => {
                                        // Allocate small chunk
                                        let size = 100 + (j % 500);
                                        let chunk = vec![0u8; size];
                                        tracker_clone.allocate(size);
                                        local_allocations.push(chunk);
                                    },
                                    1 => {
                                        // Allocate large chunk
                                        let size = 5000 + (j % 10000);
                                        let chunk = vec![0u8; size];
                                        tracker_clone.allocate(size);
                                        local_allocations.push(chunk);
                                    },
                                    2 => {
                                        // Deallocate if possible
                                        if let Some(chunk) = local_allocations.pop() {
                                            tracker_clone.deallocate(chunk.len());
                                        }
                                    },
                                    _ => {
                                        // Create temporary allocation (immediate deallocation)
                                        let size = 1000 + (j % 2000);
                                        let chunk = vec![0u8; size];
                                        tracker_clone.allocate(size);
                                        tracker_clone.deallocate(size);
                                        // chunk goes out of scope immediately
                                    }
                                }
                                
                                // Small delay to increase contention
                                if j % 10 == 0 {
                                    tokio::time::sleep(Duration::from_micros(1)).await;
                                }
                            }
                            
                            // Clean up remaining allocations
                            for chunk in local_allocations {
                                tracker_clone.deallocate(chunk.len());
                            }
                        }));
                    }
                    
                    // Wait for all tasks to complete
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    let stats = tracker.get_stats();
                    black_box(stats);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory leak detection patterns
fn benchmark_leak_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("leak_detection");
    
    group.bench_function("potential_leak_patterns", |b| {
        b.to_async(&rt).iter(|| async {
            let tracker = MemoryTracker::new();
            
            // Pattern 1: Growing structures without cleanup
            let mut growing_map: HashMap<String, Vec<u8>> = HashMap::new();
            for i in 0..1000 {
                let key = format!("growing-{}", i);
                let size = 1000 + (i % 5000);
                let value = vec![0u8; size];
                tracker.allocate(size);
                growing_map.insert(key, value);
            }
            
            // Pattern 2: Circular references (simulated)
            let mut circular_refs: Vec<Arc<HashMap<String, Vec<u8>>>> = Vec::new();
            for i in 0..100 {
                let mut inner_map = HashMap::new();
                for j in 0..50 {
                    let key = format!("circular-{}-{}", i, j);
                    let size = 500 + (j % 1000);
                    let value = vec![0u8; size];
                    tracker.allocate(size);
                    inner_map.insert(key, value);
                }
                circular_refs.push(Arc::new(inner_map));
            }
            
            // Pattern 3: Cached data that grows indefinitely
            let mut cache: HashMap<String, Vec<Vec<u8>>> = HashMap::new();
            for i in 0..500 {
                let key = format!("cache-group-{}", i % 10); // Limited keys, growing values
                let entry = cache.entry(key).or_insert_with(Vec::new);
                
                let size = 200 + (i % 1000);
                let value = vec![0u8; size];
                tracker.allocate(size);
                entry.push(value);
            }
            
            // Partial cleanup (some leaks remain)
            let mut keys_to_remove: Vec<String> = growing_map.keys()
                .enumerate()
                .filter_map(|(idx, key)| if idx % 3 == 0 { Some(key.clone()) } else { None })
                .collect();
            
            for key in keys_to_remove {
                if let Some(value) = growing_map.remove(&key) {
                    tracker.deallocate(value.len());
                }
            }
            
            let stats = tracker.get_stats();
            black_box(stats);
            black_box((growing_map, circular_refs, cache));
        });
    });
    
    group.finish();
}

/// Benchmark memory-intensive operations
fn benchmark_memory_intensive_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_intensive_operations");
    
    // Test operations with different memory footprints
    for memory_multiplier in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("intensive_operations", memory_multiplier),
            memory_multiplier,
            |b, &memory_multiplier| {
                b.to_async(&rt).iter(|| async {
                    let base_size = 1000 * memory_multiplier;
                    let tracker = MemoryTracker::new();
                    
                    // Operation 1: Large vector operations
                    let mut large_vector = Vec::with_capacity(base_size);
                    for i in 0..base_size {
                        let data = format!("item-{}-{}", i, "x".repeat(100));
                        tracker.allocate(data.len());
                        large_vector.push(data);
                    }
                    
                    // Operation 2: Matrix-like operations
                    let mut matrix: Vec<Vec<f64>> = Vec::new();
                    let matrix_size = (base_size as f64).sqrt() as usize;
                    for i in 0..matrix_size {
                        let mut row = Vec::new();
                        for j in 0..matrix_size {
                            row.push((i * j) as f64);
                        }
                        tracker.allocate(row.len() * std::mem::size_of::<f64>());
                        matrix.push(row);
                    }
                    
                    // Operation 3: Deep nested structures
                    let mut nested: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for i in 0..100 {
                        let mut map = HashMap::new();
                        for j in 0..50 {
                            let key = format!("nested-key-{}-{}", i, j);
                            let mut values = Vec::new();
                            for k in 0..memory_multiplier {
                                let value = format!("nested-value-{}-{}-{}", i, j, k);
                                tracker.allocate(value.len());
                                values.push(value);
                            }
                            tracker.allocate(key.len());
                            map.insert(key, values);
                        }
                        nested.push(map);
                    }
                    
                    let stats = tracker.get_stats();
                    black_box(stats);
                    black_box((large_vector, matrix, nested));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_allocation_patterns,
    benchmark_fragmentation_patterns,
    benchmark_complex_structures,
    benchmark_concurrent_memory_access,
    benchmark_leak_detection,
    benchmark_memory_intensive_operations
);
criterion_main!(benches); 