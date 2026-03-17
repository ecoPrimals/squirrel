// SPDX-License-Identifier: AGPL-3.0-only
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
}

/// Internal memory info for optimizer
#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub total_allocated: u64,
    pub total_saved: u64,
    pub pools_active: usize,
}
