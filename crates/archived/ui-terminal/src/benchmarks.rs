//! Benchmarks for UI terminal components

use std::collections::HashMap;
use std::time::{Instant, Duration};
use chrono::{DateTime, Utc};

use crate::util::{CompressedTimeSeries, CachedMetrics, CachedWidget, CachedMap};

/// Run a timed test function multiple times and return statistics
fn bench_function<F>(name: &str, iterations: usize, f: F) -> (Duration, Duration, Duration) 
where
    F: Fn() -> ()
{
    println!("Benchmarking: {}", name);
    
    let mut durations = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..5 {
        f();
    }
    
    // Actual benchmark
    for i in 0..iterations {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        durations.push(elapsed);
        
        if i % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!(" done!");
    
    // Calculate statistics
    durations.sort();
    
    let min = durations.first().unwrap_or(&Duration::from_secs(0)).clone();
    let max = durations.last().unwrap_or(&Duration::from_secs(0)).clone();
    
    let total: Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;
    
    println!("  Min: {:?}", min);
    println!("  Avg: {:?}", avg);
    println!("  Max: {:?}", max);
    
    (min, avg, max)
}

/// Benchmark the CompressedTimeSeries
pub fn bench_compressed_time_series() {
    println!("\n=== Compressed Time Series Benchmark ===");
    
    // Create test data - 10,000 points
    let now = Utc::now();
    let mut points = Vec::with_capacity(10_000);
    
    for i in 0..10_000 {
        let timestamp = now + chrono::Duration::milliseconds(i * 100);
        let value = (i as f64 / 100.0).sin() + 1.0; // Value between 0 and 2
        points.push((timestamp, value));
    }
    
    // Benchmark regular Vec storage
    bench_function("Vec storage - insert 10k points", 10, || {
        let mut regular_vec = Vec::with_capacity(10_000);
        for point in &points {
            regular_vec.push(*point);
        }
    });
    
    // Benchmark CompressedTimeSeries
    bench_function("CompressedTimeSeries - insert 10k points", 10, || {
        let mut ts = CompressedTimeSeries::<f64>::new(10_000);
        for (timestamp, value) in &points {
            ts.add(*timestamp, *value);
        }
    });
    
    // Create filled time series for read tests
    let mut ts = CompressedTimeSeries::<f64>::new(10_000);
    for (timestamp, value) in &points {
        ts.add(*timestamp, *value);
    }
    
    // Benchmark read operations
    bench_function("CompressedTimeSeries - read all points", 100, || {
        let _all_points = ts.points();
    });
    
    bench_function("CompressedTimeSeries - downsample to 100 points", 100, || {
        let _downsampled = ts.downsample(100);
    });
    
    // Compare memory usage
    let regular_size = std::mem::size_of::<(DateTime<Utc>, f64)>() * 10_000;
    let compressed_size = std::mem::size_of_val(&ts);
    
    println!("Memory usage:");
    println!("  Vec<(DateTime<Utc>, f64)>: {} bytes", regular_size);
    println!("  CompressedTimeSeries: {} bytes", compressed_size);
    println!("  Compression ratio: {:.2}x", regular_size as f64 / compressed_size as f64);
}

/// Benchmark cached metrics
pub fn bench_cached_metrics() {
    println!("\n=== Cached Metrics Benchmark ===");
    
    let heavy_computation = || {
        // Simulate an expensive computation
        let mut result = 0;
        for i in 0..1_000_000 {
            result += i;
        }
        result
    };
    
    // Benchmark without caching
    bench_function("Without caching", 10, || {
        for _ in 0..100 {
            heavy_computation();
        }
    });
    
    // Benchmark with caching
    bench_function("With CachedMetrics", 10, || {
        let mut cache = CachedMetrics::<i32>::new(1000);
        
        for _ in 0..100 {
            if let Some(value) = cache.get() {
                // Use cached value
                let _ = value;
            } else {
                // Compute and cache
                let result = heavy_computation();
                cache.update(result);
            }
        }
    });
}

/// Run all benchmarks
pub fn run_all_benchmarks() {
    bench_compressed_time_series();
    bench_cached_metrics();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmarks() {
        // This is a placeholder test to make sure the benchmarks compile
        // For actual benchmark execution, use the command-line interface
    }
} 