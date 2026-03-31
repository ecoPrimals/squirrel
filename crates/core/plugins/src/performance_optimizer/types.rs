// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Type definitions for the plugin performance optimizer.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;
use std::time::SystemTime;

use uuid::Uuid;

use crate::zero_copy::ZeroCopyPluginEntry;

/// Cached plugin lookup result
#[derive(Debug)]
pub struct CachedPluginLookup {
    pub plugin_id: Option<Uuid>,
    pub cached_at: SystemTime,
    pub access_count: AtomicU64,
    pub hit_rate: f64,
}

impl Clone for CachedPluginLookup {
    fn clone(&self) -> Self {
        Self {
            plugin_id: self.plugin_id,
            cached_at: self.cached_at,
            access_count: AtomicU64::new(
                self.access_count.load(std::sync::atomic::Ordering::SeqCst),
            ),
            hit_rate: self.hit_rate,
        }
    }
}

/// Cached capability query result
#[derive(Debug)]
pub struct CachedCapabilityQuery {
    pub matching_plugins: std::sync::Arc<Vec<Uuid>>,
    pub cached_at: SystemTime,
    pub access_count: AtomicU64,
    pub query_time: std::time::Duration,
}

impl Clone for CachedCapabilityQuery {
    fn clone(&self) -> Self {
        Self {
            matching_plugins: self.matching_plugins.clone(),
            cached_at: self.cached_at,
            access_count: AtomicU64::new(
                self.access_count.load(std::sync::atomic::Ordering::SeqCst),
            ),
            query_time: self.query_time,
        }
    }
}

/// Cached execution result
#[derive(Debug)]
pub struct CachedExecutionResult {
    /// Cached result data
    pub result: std::sync::Arc<String>,
    /// When this result was cached
    pub cached_at: SystemTime,
    /// Number of times this cache entry was accessed
    pub access_count: AtomicU64,
    /// Original execution time (for comparison)
    pub execution_time: std::time::Duration,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    pub lookup_hits: u64,
    pub lookup_misses: u64,
    pub capability_hits: u64,
    pub capability_misses: u64,
    pub execution_hits: u64,
    pub execution_misses: u64,
    pub total_memory_saved: u64,
}

/// Batch operation types
#[derive(Debug)]
pub enum BatchOperation {
    /// Load a plugin in batch
    PluginLoad {
        /// Plugin to load
        plugin_id: Uuid,
        /// Plugin entry data
        entry: std::sync::Arc<ZeroCopyPluginEntry>,
    },
    /// Query capabilities in batch
    CapabilityQuery {
        /// Capability to query
        capability: String,
    },
    /// Update metadata in batch
    MetadataUpdate {
        /// Plugin to update
        plugin_id: Uuid,
        /// New metadata
        metadata: std::sync::Arc<crate::zero_copy::ZeroCopyPluginMetadata>,
    },
}

/// Batch processing statistics
#[derive(Debug, Default, Clone)]
pub struct BatchStatistics {
    pub batches_processed: u64,
    pub operations_batched: u64,
    pub average_batch_size: f64,
    pub time_saved_ms: u64,
}

/// Usage pattern for a plugin or operation
#[derive(Debug, Clone)]
pub struct UsagePattern {
    /// Access times for pattern analysis
    pub access_times: VecDeque<SystemTime>,
    /// Frequency of access
    pub access_frequency: f64,
    /// Peak hours of usage (0-23)
    pub peak_hours: Vec<u8>,
    /// Correlation with other patterns
    pub correlation_patterns: HashMap<String, f64>,
}

/// Prediction model for anticipating needs
#[derive(Debug, Default)]
pub struct PredictionModel {
    pub confidence_scores: HashMap<String, f64>,
    pub prediction_accuracy: f64,
    pub total_predictions: u64,
    pub correct_predictions: u64,
}

/// Predictive load operation
#[derive(Debug, Clone)]
pub struct PredictiveLoad {
    /// Plugin to predictively load
    pub plugin_id: Uuid,
    /// Confidence in prediction
    pub confidence: f64,
    /// When to load the plugin
    pub predicted_load_time: SystemTime,
    /// Reason for this prediction
    pub reason: PredictionReason,
}

/// Reason for predictive loading
#[derive(Debug, Clone)]
pub enum PredictionReason {
    TemporalPattern,
    UsageCorrelation,
    HistoricalTrend,
    ExplicitDependency,
}

/// Plugin memory usage information
#[derive(Debug, Clone)]
pub struct PluginMemoryInfo {
    pub allocated_bytes: u64,
    pub peak_usage: u64,
    pub last_compaction: SystemTime,
    pub compaction_savings: u64,
}

/// Zero-copy pool trait
pub trait ZeroCopyPool: Send + Sync + std::fmt::Debug {
    fn get_object(&self) -> Box<dyn std::any::Any + Send>;
    fn return_object(&self, object: Box<dyn std::any::Any + Send>);
    fn pool_size(&self) -> usize;
    fn memory_usage(&self) -> u64;
}

/// Lazy loading information
#[derive(Debug, Clone)]
pub struct LazyLoadInfo {
    pub plugin_id: Uuid,
    pub data_size: u64,
    pub load_threshold: u64,
    pub last_access: SystemTime,
}

/// Overall optimizer metrics
#[derive(Debug, Default, Clone)]
pub struct OptimizerMetrics {
    /// Cache hit rate (0.0 to 1.0).
    pub cache_efficiency: f64,
    /// Batch processing efficiency (0.0 to 1.0).
    pub batch_efficiency: f64,
    /// Accuracy of predictive loading (0.0 to 1.0).
    pub prediction_accuracy: f64,
    /// Memory saved through optimization in bytes.
    pub memory_saved_bytes: u64,
    /// Number of operations optimized.
    pub operations_optimized: u64,
    /// Total time saved by optimizations in milliseconds.
    pub total_time_saved_ms: u64,
    /// Rolling average observed response time in milliseconds (from `record_runtime_sample`).
    pub avg_response_time_ms: f64,
    /// Observed error rate (0.0 to 1.0) from recorded samples.
    pub error_rate: f64,
    /// Observed throughput (operations per second) over the optimizer session.
    pub throughput_ops_per_sec: f64,
}

/// Severity for an optimization recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecommendationSeverity {
    /// Informational tuning hint.
    Info,
    /// Worth addressing soon.
    Warning,
    /// Likely user-visible impact.
    Critical,
}

/// Actionable optimization suggestion derived from live metrics.
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Relative importance.
    pub severity: RecommendationSeverity,
    /// Short title for dashboards or logs.
    pub summary: String,
    /// Concrete guidance (may include measured values).
    pub detail: String,
}

/// Internal memory info for optimizer
#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub total_allocated: u64,
    pub total_saved: u64,
    pub pools_active: usize,
}

#[cfg(test)]
mod tests {
    use super::{
        BatchOperation, BatchStatistics, CacheStatistics, CachedCapabilityQuery,
        CachedExecutionResult, CachedPluginLookup, LazyLoadInfo, MemoryInfo, OptimizerMetrics,
        PluginMemoryInfo, PredictionModel, PredictionReason, PredictiveLoad, UsagePattern,
        ZeroCopyPool,
    };
    use crate::zero_copy::{ZeroCopyPluginConfig, ZeroCopyPluginEntry, ZeroCopyPluginMetadata};
    use std::any::Any;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, SystemTime};
    use uuid::Uuid;

    #[derive(Debug)]
    struct DummyPool;

    impl ZeroCopyPool for DummyPool {
        fn get_object(&self) -> Box<dyn Any + Send> {
            Box::new(0u32)
        }

        fn return_object(&self, _object: Box<dyn Any + Send>) {}

        fn pool_size(&self) -> usize {
            7
        }

        fn memory_usage(&self) -> u64 {
            42
        }
    }

    #[test]
    fn cached_plugin_lookup_clone_preserves_atomic() {
        let c = CachedPluginLookup {
            plugin_id: Some(Uuid::nil()),
            cached_at: SystemTime::UNIX_EPOCH,
            access_count: std::sync::atomic::AtomicU64::new(5),
            hit_rate: 0.5,
        };
        c.access_count.fetch_add(1, Ordering::SeqCst);
        let c2 = c.clone();
        assert_eq!(
            c2.access_count.load(Ordering::SeqCst),
            c.access_count.load(Ordering::SeqCst)
        );
        let _ = format!("{c:?}");
    }

    #[test]
    fn cached_capability_query_clone_roundtrip() {
        let q = CachedCapabilityQuery {
            matching_plugins: Arc::new(vec![Uuid::new_v4()]),
            cached_at: SystemTime::UNIX_EPOCH,
            access_count: std::sync::atomic::AtomicU64::new(1),
            query_time: Duration::from_millis(10),
        };
        assert_eq!(q.matching_plugins.len(), 1);
    }

    #[test]
    fn cache_batch_optimizer_metrics_defaults_clone_and_debug() {
        let cs = CacheStatistics::default();
        let _ = format!("{cs:?}");
        let bs = BatchStatistics::default();
        let _ = format!("{bs:?}");
        let om = OptimizerMetrics::default();
        let om2 = om.clone();
        assert!((om2.cache_efficiency - om.cache_efficiency).abs() < f64::EPSILON);
        assert!((om2.avg_response_time_ms - om.avg_response_time_ms).abs() < f64::EPSILON);
        let mi = MemoryInfo::default();
        let _ = format!("{mi:?}");
    }

    #[test]
    fn batch_operation_variants_construct() {
        let id = Uuid::new_v4();
        let meta = ZeroCopyPluginMetadata::new(id, "n".into(), "v".into(), "d".into(), "a".into());
        let cfg = ZeroCopyPluginConfig::new(id);
        let entry = Arc::new(ZeroCopyPluginEntry::new(meta, cfg, None));
        let m = BatchOperation::PluginLoad {
            plugin_id: id,
            entry: Arc::clone(&entry),
        };
        let _ = format!("{m:?}");
        let cq = BatchOperation::CapabilityQuery {
            capability: "cap".into(),
        };
        let _ = format!("{cq:?}");
        let md =
            ZeroCopyPluginMetadata::new(id, "n2".into(), "v2".into(), "d2".into(), "a2".into());
        let mu = BatchOperation::MetadataUpdate {
            plugin_id: id,
            metadata: Arc::new(md),
        };
        let _ = format!("{mu:?}");
    }

    #[test]
    fn cached_execution_result_debug() {
        let r = CachedExecutionResult {
            result: Arc::new("ok".into()),
            cached_at: SystemTime::UNIX_EPOCH,
            access_count: std::sync::atomic::AtomicU64::new(2),
            execution_time: Duration::from_nanos(1),
        };
        let _ = format!("{r:?}");
    }

    #[test]
    fn usage_pattern_prediction_and_memory_types() {
        let mut times = VecDeque::new();
        times.push_back(SystemTime::UNIX_EPOCH);
        let up = UsagePattern {
            access_times: times,
            access_frequency: 1.0,
            peak_hours: vec![9, 17],
            correlation_patterns: HashMap::from([("a".into(), 0.1)]),
        };
        let _ = format!("{up:?}");
        assert_eq!(up.peak_hours, vec![9, 17]);

        let mut pm = PredictionModel::default();
        pm.confidence_scores.insert("k".into(), 0.9);
        pm.prediction_accuracy = 0.8;
        pm.total_predictions = 10;
        pm.correct_predictions = 8;
        let _ = format!("{pm:?}");

        let pl = PredictiveLoad {
            plugin_id: Uuid::new_v4(),
            confidence: 0.95,
            predicted_load_time: SystemTime::UNIX_EPOCH,
            reason: PredictionReason::TemporalPattern,
        };
        let reasons = [
            PredictionReason::TemporalPattern,
            PredictionReason::UsageCorrelation,
            PredictionReason::HistoricalTrend,
            PredictionReason::ExplicitDependency,
        ];
        for r in reasons {
            let _ = format!("{r:?}");
            let x = PredictiveLoad {
                reason: r,
                ..pl.clone()
            };
            let _ = format!("{x:?}");
        }

        let pmi = PluginMemoryInfo {
            allocated_bytes: 1,
            peak_usage: 2,
            last_compaction: SystemTime::UNIX_EPOCH,
            compaction_savings: 3,
        };
        let _ = format!("{pmi:?}");

        let lli = LazyLoadInfo {
            plugin_id: Uuid::new_v4(),
            data_size: 100,
            load_threshold: 50,
            last_access: SystemTime::UNIX_EPOCH,
        };
        let _ = format!("{lli:?}");
    }

    #[test]
    fn zero_copy_pool_trait_object() {
        let p: &dyn ZeroCopyPool = &DummyPool;
        assert_eq!(p.pool_size(), 7);
        assert_eq!(p.memory_usage(), 42);
        let _ = format!("{p:?}");
    }
}
