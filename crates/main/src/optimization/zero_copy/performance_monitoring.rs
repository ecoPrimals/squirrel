// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Performance monitoring and metrics for zero-copy optimizations

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Zero-copy performance metrics tracker
///
/// Tracks various optimization metrics to measure the effectiveness
/// of zero-copy patterns and identify areas for improvement.
#[derive(Debug)]
pub struct ZeroCopyMetrics {
    pub(crate) allocations_saved: AtomicUsize,
    pub(crate) bytes_saved: AtomicU64,
    pub(crate) clone_operations_avoided: AtomicUsize,
    pub(crate) string_interning_hits: AtomicUsize,
    pub(crate) total_operations: AtomicUsize,
}

impl ZeroCopyMetrics {
    /// Create new metrics tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocations_saved: AtomicUsize::new(0),
            bytes_saved: AtomicU64::new(0),
            clone_operations_avoided: AtomicUsize::new(0),
            string_interning_hits: AtomicUsize::new(0),
            total_operations: AtomicUsize::new(0),
        }
    }

    /// Record an allocation that was saved
    pub fn record_allocation_saved(&self, bytes: u64) {
        self.allocations_saved.fetch_add(1, Ordering::Relaxed);
        self.bytes_saved.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a clone operation that was avoided
    pub fn record_clone_avoided(&self) {
        self.clone_operations_avoided
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a string interning cache hit
    pub fn record_string_interning_hit(&self) {
        self.string_interning_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a general operation
    pub fn record_operation(&self) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            allocations_saved: self.allocations_saved.load(Ordering::Relaxed),
            bytes_saved: self.bytes_saved.load(Ordering::Relaxed),
            clone_operations_avoided: self.clone_operations_avoided.load(Ordering::Relaxed),
            string_interning_hits: self.string_interning_hits.load(Ordering::Relaxed),
            total_operations: self.total_operations.load(Ordering::Relaxed),
        }
    }

    /// Get efficiency score (0.0 - 1.0)
    pub fn get_efficiency_score(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        let optimizations = self.clone_operations_avoided.load(Ordering::Relaxed)
            + self.string_interning_hits.load(Ordering::Relaxed);

        optimizations as f64 / total as f64
    }

    /// Get operations count
    pub fn get_operations_count(&self) -> usize {
        self.total_operations.load(Ordering::Relaxed)
    }

    /// Get clones avoided count
    pub fn get_clones_avoided(&self) -> usize {
        self.clone_operations_avoided.load(Ordering::Relaxed)
    }
}

impl Default for ZeroCopyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of zero-copy metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub allocations_saved: usize,
    pub bytes_saved: u64,
    pub clone_operations_avoided: usize,
    pub string_interning_hits: usize,
    pub total_operations: usize,
}

impl MetricsSnapshot {
    /// Calculate efficiency percentage
    #[must_use]
    pub fn efficiency_percentage(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }

        let optimizations = self.clone_operations_avoided + self.string_interning_hits;
        (optimizations as f64 / self.total_operations as f64) * 100.0
    }

    /// Calculate average bytes saved per allocation
    #[must_use]
    pub fn average_bytes_saved(&self) -> f64 {
        if self.allocations_saved == 0 {
            return 0.0;
        }

        self.bytes_saved as f64 / self.allocations_saved as f64
    }

    /// Calculate string interning hit rate
    #[must_use]
    pub fn string_interning_hit_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }

        (self.string_interning_hits as f64 / self.total_operations as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_metrics_new() {
        let metrics = ZeroCopyMetrics::new();
        assert_eq!(metrics.allocations_saved.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.bytes_saved.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.clone_operations_avoided.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.string_interning_hits.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_operations.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_zero_copy_metrics_default() {
        let metrics = ZeroCopyMetrics::default();
        assert_eq!(metrics.get_operations_count(), 0);
    }

    #[test]
    fn test_record_allocation_saved() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_allocation_saved(1024);
        assert_eq!(metrics.allocations_saved.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bytes_saved.load(Ordering::Relaxed), 1024);

        metrics.record_allocation_saved(2048);
        assert_eq!(metrics.allocations_saved.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.bytes_saved.load(Ordering::Relaxed), 3072);
    }

    #[test]
    fn test_record_clone_avoided() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_clone_avoided();
        assert_eq!(metrics.clone_operations_avoided.load(Ordering::Relaxed), 1);

        metrics.record_clone_avoided();
        assert_eq!(metrics.clone_operations_avoided.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_string_interning_hit() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_string_interning_hit();
        assert_eq!(metrics.string_interning_hits.load(Ordering::Relaxed), 1);

        metrics.record_string_interning_hit();
        assert_eq!(metrics.string_interning_hits.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_operation() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_operation();
        assert_eq!(metrics.total_operations.load(Ordering::Relaxed), 1);

        metrics.record_operation();
        assert_eq!(metrics.total_operations.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_get_metrics() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_allocation_saved(1024);
        metrics.record_clone_avoided();
        metrics.record_string_interning_hit();
        metrics.record_operation();

        let snapshot = metrics.get_metrics();
        assert_eq!(snapshot.allocations_saved, 1);
        assert_eq!(snapshot.bytes_saved, 1024);
        assert_eq!(snapshot.clone_operations_avoided, 1);
        assert_eq!(snapshot.string_interning_hits, 1);
        assert_eq!(snapshot.total_operations, 1);
    }

    #[test]
    fn test_get_efficiency_score_zero_operations() {
        let metrics = ZeroCopyMetrics::new();
        assert_eq!(metrics.get_efficiency_score(), 0.0);
    }

    #[test]
    fn test_get_efficiency_score_with_optimizations() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_operation();
        metrics.record_operation();
        metrics.record_operation();
        metrics.record_operation();
        metrics.record_clone_avoided();
        metrics.record_string_interning_hit();

        // 2 optimizations out of 4 operations = 0.5
        assert_eq!(metrics.get_efficiency_score(), 0.5);
    }

    #[test]
    fn test_get_efficiency_score_perfect() {
        let metrics = ZeroCopyMetrics::new();
        metrics.record_operation();
        metrics.record_operation();
        metrics.record_clone_avoided();
        metrics.record_clone_avoided();

        // 2 optimizations out of 2 operations = 1.0
        assert_eq!(metrics.get_efficiency_score(), 1.0);
    }

    #[test]
    fn test_get_operations_count() {
        let metrics = ZeroCopyMetrics::new();
        assert_eq!(metrics.get_operations_count(), 0);

        metrics.record_operation();
        assert_eq!(metrics.get_operations_count(), 1);

        metrics.record_operation();
        metrics.record_operation();
        assert_eq!(metrics.get_operations_count(), 3);
    }

    #[test]
    fn test_get_clones_avoided() {
        let metrics = ZeroCopyMetrics::new();
        assert_eq!(metrics.get_clones_avoided(), 0);

        metrics.record_clone_avoided();
        assert_eq!(metrics.get_clones_avoided(), 1);

        metrics.record_clone_avoided();
        metrics.record_clone_avoided();
        assert_eq!(metrics.get_clones_avoided(), 3);
    }

    #[test]
    fn test_metrics_snapshot_efficiency_percentage_zero() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 0,
            bytes_saved: 0,
            clone_operations_avoided: 0,
            string_interning_hits: 0,
            total_operations: 0,
        };
        assert_eq!(snapshot.efficiency_percentage(), 0.0);
    }

    #[test]
    fn test_metrics_snapshot_efficiency_percentage() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 10,
            bytes_saved: 10240,
            clone_operations_avoided: 30,
            string_interning_hits: 20,
            total_operations: 100,
        };
        // (30 + 20) / 100 * 100 = 50%
        assert_eq!(snapshot.efficiency_percentage(), 50.0);
    }

    #[test]
    fn test_metrics_snapshot_efficiency_percentage_perfect() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 10,
            bytes_saved: 10240,
            clone_operations_avoided: 50,
            string_interning_hits: 50,
            total_operations: 100,
        };
        // (50 + 50) / 100 * 100 = 100%
        assert_eq!(snapshot.efficiency_percentage(), 100.0);
    }

    #[test]
    fn test_metrics_snapshot_average_bytes_saved_zero() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 0,
            bytes_saved: 0,
            clone_operations_avoided: 0,
            string_interning_hits: 0,
            total_operations: 0,
        };
        assert_eq!(snapshot.average_bytes_saved(), 0.0);
    }

    #[test]
    fn test_metrics_snapshot_average_bytes_saved() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 10,
            bytes_saved: 10240,
            clone_operations_avoided: 0,
            string_interning_hits: 0,
            total_operations: 10,
        };
        // 10240 / 10 = 1024.0
        assert_eq!(snapshot.average_bytes_saved(), 1024.0);
    }

    #[test]
    fn test_metrics_snapshot_string_interning_hit_rate_zero() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 0,
            bytes_saved: 0,
            clone_operations_avoided: 0,
            string_interning_hits: 0,
            total_operations: 0,
        };
        assert_eq!(snapshot.string_interning_hit_rate(), 0.0);
    }

    #[test]
    fn test_metrics_snapshot_string_interning_hit_rate() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 10,
            bytes_saved: 10240,
            clone_operations_avoided: 20,
            string_interning_hits: 25,
            total_operations: 100,
        };
        // 25 / 100 * 100 = 25%
        assert_eq!(snapshot.string_interning_hit_rate(), 25.0);
    }

    #[test]
    fn test_metrics_snapshot_string_interning_hit_rate_perfect() {
        let snapshot = MetricsSnapshot {
            allocations_saved: 10,
            bytes_saved: 10240,
            clone_operations_avoided: 0,
            string_interning_hits: 100,
            total_operations: 100,
        };
        // 100 / 100 * 100 = 100%
        assert_eq!(snapshot.string_interning_hit_rate(), 100.0);
    }

    #[test]
    fn test_concurrent_metric_updates() {
        use std::sync::Arc as StdArc;
        use std::thread;

        let metrics = StdArc::new(ZeroCopyMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing metrics 100 times
        for _ in 0..10 {
            let metrics_clone = StdArc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.record_operation();
                    metrics_clone.record_clone_avoided();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify totals: 10 threads * 100 operations = 1000
        assert_eq!(metrics.get_operations_count(), 1000);
        assert_eq!(metrics.get_clones_avoided(), 1000);
    }
}
