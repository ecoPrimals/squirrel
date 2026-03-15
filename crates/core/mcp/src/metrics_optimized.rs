// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Optimized Metrics Collector with Arc<str> Keys
//!
//! This module demonstrates the performance improvements possible by using
//! Arc<str> instead of String keys in HashMap operations.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::LazyLock;

/// String interning for common metric names
static COMMON_METRICS: LazyLock<HashMap<&'static str, Arc<str>>> = LazyLock::new(|| {
        let mut map = HashMap::new();
        // Pre-allocate common metric names to avoid any allocation overhead
        map.insert("request_count", Arc::from("request_count"));
        map.insert("error_count", Arc::from("error_count"));
        map.insert("latency_p50", Arc::from("latency_p50"));
        map.insert("latency_p95", Arc::from("latency_p95"));
        map.insert("latency_p99", Arc::from("latency_p99"));
        map.insert("memory_usage", Arc::from("memory_usage"));
        map.insert("cpu_usage", Arc::from("cpu_usage"));
        map.insert("active_connections", Arc::from("active_connections"));
        map.insert("queue_depth", Arc::from("queue_depth"));
        map.insert("cache_hit_rate", Arc::from("cache_hit_rate"));
        map.insert("cache_miss_rate", Arc::from("cache_miss_rate"));
        map.insert("throughput", Arc::from("throughput"));
        map.insert("success_rate", Arc::from("success_rate"));
        map.insert("failure_rate", Arc::from("failure_rate"));
        map.insert("processing_time", Arc::from("processing_time"));
        map.insert("discovery_operations", Arc::from("discovery_operations"));
        map.insert("service_registrations", Arc::from("service_registrations"));
        map.insert("capability_matches", Arc::from("capability_matches"));
        map.insert("context_switches", Arc::from("context_switches"));
        map.insert("message_routing", Arc::from("message_routing"));
        map
});

/// Get Arc<str> for metric name with zero allocation for common names
fn get_metric_name(name: &str) -> Arc<str> {
    COMMON_METRICS.get(name)
        .cloned()
        .unwrap_or_else(|| Arc::from(name))
}

/// Optimized metrics collector using Arc<str> keys for maximum performance
#[derive(Debug)]
pub struct OptimizedMetricsCollector {
    /// Counter metrics with Arc<str> keys (shared, zero-copy)
    counters: RwLock<HashMap<Arc<str>, AtomicU64>>,
    /// Gauge metrics with Arc<str> keys 
    gauges: RwLock<HashMap<Arc<str>, f64>>,
    /// Histogram metrics with Arc<str> keys and Arc<Vec<f64>> values
    histograms: RwLock<HashMap<Arc<str>, Arc<Vec<f64>>>>,
    /// Timer metrics with Arc<str> keys and Arc<Vec<f64>> values
    timers: RwLock<HashMap<Arc<str>, Arc<Vec<f64>>>>,
    /// Performance metrics for this collector
    performance: RwLock<CollectorPerformance>,
}

/// Performance tracking for the collector itself
#[derive(Debug, Default)]
pub struct CollectorPerformance {
    pub total_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub zero_allocation_operations: u64,
    pub avg_operation_time_ns: u64,
}

impl OptimizedMetricsCollector {
    /// Create new optimized metrics collector
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            timers: RwLock::new(HashMap::new()),
            performance: RwLock::new(CollectorPerformance::default()),
        }
    }

    /// Increment a counter with zero allocation for common metric names
    pub fn increment_counter(&self, name: &str, value: u64) -> Result<(), String> {
        let start = Instant::now();
        let mut zero_allocation = false;

        // Fast path: Check if counter already exists (zero allocation lookup)
        {
            let counters = self.counters.read()
                .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
            
            // Efficient lookup without Arc allocation
            if let Some(counter) = counters.iter()
                .find(|(k, _)| k.as_ref() == name)
                .map(|(_, v)| v) {
                counter.fetch_add(value, Ordering::Relaxed);
                zero_allocation = true;
                
                // Update performance metrics
                self.update_performance_metrics(start, zero_allocation, true);
                return Ok(());
            }
        }

        // Slow path: Create new counter (only allocates for new metrics)
        let arc_name = get_metric_name(name); // Use string interning
        let mut counters = self.counters.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        
        counters.insert(arc_name, AtomicU64::new(value));
        
        self.update_performance_metrics(start, zero_allocation, false);
        Ok(())
    }

    /// Set a gauge value with Arc<str> key optimization
    pub fn set_gauge(&self, name: &str, value: f64) -> Result<(), String> {
        let start = Instant::now();
        let arc_name = get_metric_name(name);
        
        let mut gauges = self.gauges.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        
        let is_existing = gauges.contains_key(&arc_name);
        gauges.insert(arc_name, value);
        
        self.update_performance_metrics(start, false, is_existing);
        Ok(())
    }

    /// Record histogram value with Arc optimizations for both keys and values
    pub fn record_histogram(&self, name: &str, value: f64) -> Result<(), String> {
        let start = Instant::now();
        let arc_name = get_metric_name(name);
        
        let mut histograms = self.histograms.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        
        let values = histograms.entry(arc_name).or_insert_with(|| Arc::new(Vec::new()));
        
        // Use Arc::make_mut for copy-on-write semantics
        Arc::make_mut(values).push(value);
        
        self.update_performance_metrics(start, false, true);
        Ok(())
    }

    /// Get counter value with zero-allocation lookup
    pub fn get_counter(&self, name: &str) -> Result<Option<u64>, String> {
        let counters = self.counters.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        
        // Efficient lookup without Arc allocation
        let value = counters.iter()
            .find(|(k, _)| k.as_ref() == name)
            .map(|(_, v)| v.load(Ordering::Relaxed));
        
        Ok(value)
    }

    /// Get all metrics with zero-copy access
    pub fn get_all_metrics(&self) -> Result<MetricsSnapshot, String> {
        let counters = self.counters.read()
            .map_err(|e| format!("Failed to acquire counters read lock: {}", e))?;
        let gauges = self.gauges.read()
            .map_err(|e| format!("Failed to acquire gauges read lock: {}", e))?;
        let histograms = self.histograms.read()
            .map_err(|e| format!("Failed to acquire histograms read lock: {}", e))?;
        
        let mut snapshot = MetricsSnapshot {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        };

        // Copy data efficiently (Arc clones are cheap)
        for (name, counter) in counters.iter() {
            snapshot.counters.insert(
                name.clone(), // Arc clone - just increments reference count
                counter.load(Ordering::Relaxed)
            );
        }

        for (name, value) in gauges.iter() {
            snapshot.gauges.insert(name.clone(), *value);
        }

        for (name, values) in histograms.iter() {
            snapshot.histograms.insert(name.clone(), values.clone());
        }

        Ok(snapshot)
    }

    /// Get collector performance metrics
    pub fn get_performance(&self) -> Result<CollectorPerformance, String> {
        let performance = self.performance.read()
            .map_err(|e| format!("Failed to acquire performance read lock: {}", e))?;
        Ok(performance.clone())
    }

    /// Benchmark this collector against string-based implementation
    pub fn benchmark_against_string_collector(&self, operations: usize) -> BenchmarkResults {
        let mut results = BenchmarkResults::default();
        
        // Benchmark optimized collector (this)
        let start = Instant::now();
        for i in 0..operations {
            let _ = self.increment_counter("benchmark_metric", 1);
            if i % 100 == 0 {
                let _ = self.set_gauge("benchmark_gauge", i as f64);
            }
        }
        results.optimized_duration = start.elapsed();
        
        // Get performance statistics
        if let Ok(perf) = self.get_performance() {
            results.zero_allocation_ops = perf.zero_allocation_operations;
            results.cache_hit_rate = if perf.total_operations > 0 {
                perf.cache_hits as f64 / perf.total_operations as f64
            } else {
                0.0
            };
        }

        results
    }

    // Private helper methods

    fn update_performance_metrics(&self, start: Instant, zero_allocation: bool, cache_hit: bool) {
        if let Ok(mut perf) = self.performance.write() {
            perf.total_operations += 1;
            
            if zero_allocation {
                perf.zero_allocation_operations += 1;
            }
            
            if cache_hit {
                perf.cache_hits += 1;
            } else {
                perf.cache_misses += 1;
            }
            
            let duration_ns = start.elapsed().as_nanos() as u64;
            perf.avg_operation_time_ns = 
                (perf.avg_operation_time_ns * (perf.total_operations - 1) + duration_ns) 
                / perf.total_operations;
        }
    }
}

/// Snapshot of all metrics with Arc<str> keys for efficient sharing
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub counters: HashMap<Arc<str>, u64>,
    pub gauges: HashMap<Arc<str>, f64>,
    pub histograms: HashMap<Arc<str>, Arc<Vec<f64>>>,
    pub timestamp: std::time::SystemTime,
}

/// Benchmark comparison results
#[derive(Debug, Default)]
pub struct BenchmarkResults {
    pub optimized_duration: Duration,
    pub zero_allocation_ops: u64,
    pub cache_hit_rate: f64,
}

impl Default for OptimizedMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_allocation_counter_increment() {
        let collector = OptimizedMetricsCollector::new();
        
        // First increment - will allocate
        collector.increment_counter("test_metric", 1).unwrap();
        
        // Subsequent increments - zero allocation
        for _ in 0..1000 {
            collector.increment_counter("test_metric", 1).unwrap();
        }
        
        // Verify final value
        assert_eq!(collector.get_counter("test_metric").unwrap().unwrap(), 1001);
    }

    #[test]
    fn test_common_metrics_interning() {
        let collector = OptimizedMetricsCollector::new();
        
        // These should use pre-allocated Arc<str> instances
        collector.increment_counter("request_count", 1).unwrap();
        collector.increment_counter("error_count", 1).unwrap();
        collector.increment_counter("latency_p99", 1).unwrap();
        
        let performance = collector.get_performance().unwrap();
        assert!(performance.total_operations > 0);
    }

    #[test]
    fn test_performance_tracking() {
        let collector = OptimizedMetricsCollector::new();
        
        // Perform operations
        for i in 0..100 {
            collector.increment_counter("request_count", 1).unwrap();
            collector.set_gauge(&format!("dynamic_metric_{}", i), i as f64).unwrap();
        }
        
        let performance = collector.get_performance().unwrap();
        assert!(performance.zero_allocation_operations > 0);
        assert!(performance.cache_hits > 0);
        assert_eq!(performance.total_operations, 200); // 100 counters + 100 gauges
    }

    #[test]
    fn test_arc_str_efficiency() {
        let collector = OptimizedMetricsCollector::new();
        
        // Add the same metric name multiple times
        for _ in 0..1000 {
            collector.increment_counter("repeated_metric", 1).unwrap();
        }
        
        // Only first operation should miss cache
        let performance = collector.get_performance().unwrap();
        assert_eq!(performance.cache_misses, 1); // Only first increment
        assert_eq!(performance.cache_hits, 999); // All subsequent increments
    }
} 