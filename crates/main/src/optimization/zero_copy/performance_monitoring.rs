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
