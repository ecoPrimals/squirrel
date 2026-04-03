// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Main plugin performance optimizer implementation.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::errors::{PluginError, Result};
use crate::zero_copy::{ZeroCopyPlugin, ZeroCopyPluginEntry, ZeroCopyPluginRegistry};

use super::batch_processor::BatchProcessor;
use super::config::PerformanceOptimizerConfig;
use super::hot_path_cache::HotPathCache;
use super::memory_optimizer::MemoryOptimizer;
use super::predictive_loader::PredictiveLoader;
use super::types::{
    CacheStatistics, CachedCapabilityQuery, CachedPluginLookup, OptimizationRecommendation,
    OptimizerMetrics, RecommendationSeverity,
};

/// Rolling samples for `record_runtime_sample` (response times in ms).
const OBSERVED_RESPONSE_WINDOW: usize = 256;

/// Runtime samples used to drive suggestions (response time, errors, throughput).
#[derive(Debug)]
struct ObservedRuntimeMetrics {
    response_times_ms: VecDeque<f64>,
    success_count: u64,
    error_count: u64,
    session_start: Instant,
    total_session_ops: u64,
}

impl ObservedRuntimeMetrics {
    fn new() -> Self {
        Self {
            response_times_ms: VecDeque::with_capacity(OBSERVED_RESPONSE_WINDOW),
            success_count: 0,
            error_count: 0,
            session_start: Instant::now(),
            total_session_ops: 0,
        }
    }

    fn record(&mut self, duration_ms: f64, success: bool) {
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
        if self.response_times_ms.len() >= OBSERVED_RESPONSE_WINDOW {
            self.response_times_ms.pop_front();
        }
        self.response_times_ms.push_back(duration_ms);
        self.total_session_ops += 1;
    }

    fn avg_response_ms(&self) -> f64 {
        if self.response_times_ms.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.response_times_ms.iter().sum();
        sum / self.response_times_ms.len() as f64
    }

    fn error_rate(&self) -> f64 {
        let total = self.success_count.saturating_add(self.error_count);
        if total == 0 {
            return 0.0;
        }
        self.error_count as f64 / total as f64
    }

    fn throughput_ops_per_sec(&self) -> f64 {
        let elapsed = self.session_start.elapsed().as_secs_f64().max(1e-9);
        self.total_session_ops as f64 / elapsed
    }

    const fn recorded_sample_count(&self) -> u64 {
        self.success_count.saturating_add(self.error_count)
    }
}

/// Global plugin performance optimizer
static GLOBAL_OPTIMIZER: std::sync::OnceLock<Arc<PluginPerformanceOptimizer>> =
    std::sync::OnceLock::new();

/// Get or initialize the global plugin performance optimizer
pub fn get_global_optimizer() -> Arc<PluginPerformanceOptimizer> {
    GLOBAL_OPTIMIZER
        .get_or_init(|| {
            Arc::new(PluginPerformanceOptimizer::new(
                PerformanceOptimizerConfig::production(),
            ))
        })
        .clone()
}

/// Plugin performance optimizer
#[derive(Debug)]
pub struct PluginPerformanceOptimizer {
    /// Hot path cache for frequent operations
    hot_path_cache: Arc<HotPathCache>,

    /// Batch processor for bulk operations
    batch_processor: Arc<BatchProcessor>,

    /// Predictive loader
    predictive_loader: Arc<PredictiveLoader>,

    /// Memory optimizer
    memory_optimizer: Arc<MemoryOptimizer>,

    /// Observed response times and outcomes for suggestion generation
    observed_runtime: Arc<RwLock<ObservedRuntimeMetrics>>,

    /// Configuration
    config: PerformanceOptimizerConfig,
}

impl PluginPerformanceOptimizer {
    /// Create a new plugin performance optimizer
    #[must_use]
    pub fn new(config: PerformanceOptimizerConfig) -> Self {
        let hot_path_cache = Arc::new(HotPathCache::new(config.hot_path_cache.clone()));
        let batch_processor = Arc::new(BatchProcessor::new(config.batch_processing.clone()));
        let predictive_loader = Arc::new(PredictiveLoader::new(config.predictive_loading.clone()));
        let memory_optimizer = Arc::new(MemoryOptimizer::new(config.memory_optimization.clone()));
        let observed_runtime = Arc::new(RwLock::new(ObservedRuntimeMetrics::new()));

        let optimizer = Self {
            hot_path_cache,
            batch_processor,
            predictive_loader,
            memory_optimizer,
            observed_runtime,
            config,
        };

        // Start background optimization tasks
        optimizer.start_optimization_tasks();

        optimizer
    }

    /// Optimize plugin lookup with caching
    #[instrument(skip(self, registry))]
    pub async fn optimized_plugin_lookup(
        &self,
        plugin_name: &str,
        registry: &ZeroCopyPluginRegistry,
    ) -> Option<Arc<ZeroCopyPluginEntry>> {
        let cache_key = format!("lookup:{plugin_name}");

        // Check hot path cache first
        if let Some(cached) = self.hot_path_cache.get_plugin_lookup(&cache_key).await {
            if let Some(plugin_id) = cached.plugin_id {
                if let Some(plugin) = registry.get_plugin(plugin_id).await {
                    self.update_cache_hit_metrics("plugin_lookup").await;
                    return Some(plugin);
                }
            } else {
                // Cached negative result
                self.update_cache_hit_metrics("plugin_lookup").await;
                return None;
            }
        }

        // Perform actual lookup
        let start = Instant::now();
        let result = registry.get_plugin_by_name(plugin_name).await;
        let _lookup_time = start.elapsed();

        // Cache the result
        let cached_lookup = CachedPluginLookup {
            plugin_id: result.as_ref().map(|p| p.id()),
            cached_at: SystemTime::now(),
            access_count: std::sync::atomic::AtomicU64::new(1),
            hit_rate: 1.0,
        };

        self.hot_path_cache
            .cache_plugin_lookup(cache_key, cached_lookup)
            .await;
        self.update_cache_miss_metrics("plugin_lookup").await;

        result
    }

    /// Optimize capability query with caching and batching
    #[instrument(skip(self, registry))]
    pub async fn optimized_capability_query(
        &self,
        capability: &str,
        registry: &ZeroCopyPluginRegistry,
    ) -> Vec<Arc<ZeroCopyPluginEntry>> {
        let cache_key = format!("capability:{capability}");

        // Check hot path cache
        if let Some(cached) = self.hot_path_cache.get_capability_query(&cache_key).await {
            let mut results = Vec::new();
            for plugin_id in cached.matching_plugins.iter() {
                if let Some(plugin) = registry.get_plugin(*plugin_id).await {
                    results.push(plugin);
                }
            }
            if !results.is_empty() {
                self.update_cache_hit_metrics("capability_query").await;
                return results;
            }
        }

        // Perform actual query
        let start = Instant::now();
        let results = registry.find_plugins_by_capability(capability).await;
        let query_time = start.elapsed();

        // Cache the result
        let plugin_ids: Vec<Uuid> = results.iter().map(|p| p.id()).collect();
        let cached_query = CachedCapabilityQuery {
            matching_plugins: Arc::new(plugin_ids),
            cached_at: SystemTime::now(),
            access_count: std::sync::atomic::AtomicU64::new(1),
            query_time,
        };

        self.hot_path_cache
            .cache_capability_query(cache_key, cached_query)
            .await;
        self.update_cache_miss_metrics("capability_query").await;

        results
    }

    /// Batch load plugins for better performance
    pub async fn batch_load_plugins(
        &self,
        plugin_entries: Vec<Arc<ZeroCopyPluginEntry>>,
    ) -> Vec<Result<Arc<dyn ZeroCopyPlugin>>> {
        self.batch_processor
            .batch_load_plugins(plugin_entries)
            .await
    }

    /// Record one observed operation (e.g. registry lookup or RPC) to feed suggestions and metrics.
    pub async fn record_runtime_sample(&self, duration: Duration, success: bool) {
        let ms = duration.as_millis() as f64;
        let mut o = self.observed_runtime.write().await;
        o.record(ms, success);
    }

    /// Analyze aggregate metrics and return concrete tuning recommendations.
    pub async fn get_optimization_suggestions(&self) -> Vec<OptimizationRecommendation> {
        let metrics = self.get_optimization_metrics().await;
        let batch_stats = self.batch_processor.get_statistics().await;
        let runtime_samples = {
            let observed = self.observed_runtime.read().await;
            observed.recorded_sample_count()
        };

        let mut suggestions = Vec::new();

        if metrics.cache_efficiency < 0.5 && metrics.cache_efficiency > f64::EPSILON {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Warning,
                summary: "Low hot-path cache hit rate".to_string(),
                detail: format!(
                    "Cache efficiency is {:.1}%. Consider enabling cache warming, increasing `max_cached_operations`, or lowering `min_access_count` so frequent lookups stay warm.",
                    metrics.cache_efficiency * 100.0
                ),
            });
        }

        if batch_stats.batches_processed > 0
            && metrics.batch_efficiency < 0.3
            && self.config.batch_processing.max_batch_size > 1
        {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Info,
                summary: "Batch utilization below target".to_string(),
                detail: format!(
                    "Batch efficiency is {:.1}% of configured max batch size. Group more plugin loads per call or enable `dynamic_batching` so work fills batches.",
                    metrics.batch_efficiency * 100.0
                ),
            });
        }

        if self.config.predictive_loading.enabled
            && metrics.prediction_accuracy > 0.0
            && metrics.prediction_accuracy < 0.5
        {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Info,
                summary: "Predictive loading accuracy is weak".to_string(),
                detail: format!(
                    "Prediction accuracy is {:.1}%. Widen the prediction window or raise `confidence_threshold` so only high-confidence preloads run.",
                    metrics.prediction_accuracy * 100.0
                ),
            });
        }

        if metrics.error_rate > 0.05 {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Critical,
                summary: "Elevated observed error rate".to_string(),
                detail: format!(
                    "Observed error rate is {:.2}% over recorded samples. Inspect failing operations, dependency health, and registry/plugin state.",
                    metrics.error_rate * 100.0
                ),
            });
        } else if metrics.error_rate > 0.01 {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Warning,
                summary: "Non-zero error rate".to_string(),
                detail: format!(
                    "Observed error rate is {:.2}%. Monitor trends; sustained errors may warrant circuit breaking or backoff.",
                    metrics.error_rate * 100.0
                ),
            });
        }

        if metrics.avg_response_time_ms > 500.0 {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Warning,
                summary: "High average response time".to_string(),
                detail: format!(
                    "Mean observed latency is {:.1} ms. Profile hot paths, verify cache hits on lookups, and consider batching to reduce round trips.",
                    metrics.avg_response_time_ms
                ),
            });
        } else if metrics.avg_response_time_ms > 100.0 && metrics.cache_efficiency < 0.7 {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Info,
                summary: "Latency and cache headroom".to_string(),
                detail: format!(
                    "Average response time is {:.1} ms with cache efficiency {:.1}%. Improving cache hit rate may reduce latency further.",
                    metrics.avg_response_time_ms,
                    metrics.cache_efficiency * 100.0
                ),
            });
        }

        if runtime_samples > 20
            && metrics.throughput_ops_per_sec < 5.0
            && metrics.operations_optimized > 50
        {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Info,
                summary: "Throughput is limited relative to optimized operations".to_string(),
                detail: format!(
                    "Session throughput is {:.2} ops/s with {} optimized operations recorded. Increase concurrency or reduce per-operation work if sustained load requires higher throughput.",
                    metrics.throughput_ops_per_sec,
                    metrics.operations_optimized
                ),
            });
        }

        if metrics.memory_saved_bytes == 0
            && metrics.operations_optimized > 10
            && self.config.memory_optimization.zero_copy_enabled
        {
            suggestions.push(OptimizationRecommendation {
                severity: RecommendationSeverity::Info,
                summary: "No memory savings reported yet".to_string(),
                detail: "Memory optimizer reports zero bytes saved. Ensure workloads exercise pooled buffers or compaction so savings can accrue.".to_string(),
            });
        }

        suggestions
    }

    /// Enable predictive loading based on usage patterns
    pub async fn enable_predictive_loading(&self, registry: &ZeroCopyPluginRegistry) {
        if !self.config.predictive_loading.enabled {
            return;
        }

        self.predictive_loader.analyze_usage_patterns().await;
        let predictions = self.predictive_loader.generate_predictions().await;

        for prediction in predictions {
            if prediction.confidence >= self.config.predictive_loading.confidence_threshold
                && let Some(entry) = registry.get_plugin(prediction.plugin_id).await
            {
                debug!(
                    "Predictively loading plugin {} with confidence {:.2}",
                    entry.name(),
                    prediction.confidence
                );

                // Trigger predictive load
                tokio::spawn(async move {
                    let _ = entry; // Simulate keeping plugin warm
                });
            }
        }
    }

    /// Get comprehensive optimization metrics
    #[expect(clippy::cast_precision_loss, reason = "Metrics scoring calculation")]
    pub async fn get_optimization_metrics(&self) -> OptimizerMetrics {
        let cache_stats = self.hot_path_cache.get_statistics().await;
        let batch_stats = self.batch_processor.get_statistics().await;
        let prediction_model = self.predictive_loader.get_prediction_model().await;
        let memory_info = self.memory_optimizer.get_memory_info().await;

        let observed = self.observed_runtime.read().await;

        OptimizerMetrics {
            cache_efficiency: self.calculate_cache_efficiency(&cache_stats),
            batch_efficiency: batch_stats.average_batch_size
                / self.config.batch_processing.max_batch_size as f64,
            prediction_accuracy: prediction_model.prediction_accuracy,
            memory_saved_bytes: memory_info.total_saved,
            operations_optimized: cache_stats.lookup_hits
                + cache_stats.capability_hits
                + batch_stats.operations_batched,
            total_time_saved_ms: cache_stats.total_memory_saved + batch_stats.time_saved_ms,
            avg_response_time_ms: observed.avg_response_ms(),
            error_rate: observed.error_rate(),
            throughput_ops_per_sec: observed.throughput_ops_per_sec(),
        }
    }

    /// Start background optimization tasks
    fn start_optimization_tasks(&self) {
        // Start cache warming task
        if self.config.hot_path_cache.enable_warming {
            tokio::spawn({
                let cache = Arc::clone(&self.hot_path_cache);
                async move {
                    cache.start_cache_warming().await;
                }
            });
        }

        // Start batch processing task
        tokio::spawn({
            let batch_processor = Arc::clone(&self.batch_processor);
            async move {
                batch_processor.start_batch_processing().await;
            }
        });

        // Start predictive loading task
        if self.config.predictive_loading.enabled {
            tokio::spawn({
                let predictive_loader = Arc::clone(&self.predictive_loader);
                async move {
                    predictive_loader.start_predictive_loading().await;
                }
            });
        }

        // Start memory optimization task
        tokio::spawn({
            let memory_optimizer = Arc::clone(&self.memory_optimizer);
            async move {
                memory_optimizer.start_memory_optimization().await;
            }
        });
    }

    /// Calculate cache efficiency from statistics
    #[expect(
        clippy::unused_self,
        clippy::cast_precision_loss,
        reason = "Trait impl; precision loss audited"
    )]
    fn calculate_cache_efficiency(&self, stats: &CacheStatistics) -> f64 {
        let total_requests = stats.lookup_hits
            + stats.lookup_misses
            + stats.capability_hits
            + stats.capability_misses
            + stats.execution_hits
            + stats.execution_misses;

        if total_requests > 0 {
            let total_hits = stats.lookup_hits + stats.capability_hits + stats.execution_hits;
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Update cache hit metrics
    async fn update_cache_hit_metrics(&self, operation_type: &str) {
        debug!("Cache hit for operation: {}", operation_type);
    }

    /// Update cache miss metrics
    async fn update_cache_miss_metrics(&self, operation_type: &str) {
        debug!("Cache miss for operation: {}", operation_type);
    }
}

/// Initialize global plugin performance optimizer
///
/// # Errors
///
/// Returns [`PluginError`] if the global optimizer was already initialized.
pub fn init_global_optimizer() -> Result<()> {
    let optimizer = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::production());
    GLOBAL_OPTIMIZER
        .set(Arc::new(optimizer))
        .map_err(|_| PluginError::StateError("Global optimizer already initialized".to_string()))?;

    info!("🚀 Plugin performance optimizer initialized with production settings");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance_optimizer::config::PerformanceOptimizerConfig;
    use crate::zero_copy::{ZeroCopyPluginConfig, ZeroCopyPluginEntry, ZeroCopyPluginMetadata};
    use uuid::Uuid;

    #[tokio::test]
    async fn optimized_plugin_lookup_miss_then_hit() {
        let opt = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::development());
        let registry = crate::zero_copy::ZeroCopyPluginRegistry::new();
        let id = Uuid::new_v4();
        let mut meta = ZeroCopyPluginMetadata::new(
            id,
            "plugin-a".into(),
            "1".into(),
            "d".into(),
            "auth".into(),
        );
        meta.capabilities = std::sync::Arc::new(vec!["cap.test".into()]);
        let entry = ZeroCopyPluginEntry::new(meta, ZeroCopyPluginConfig::new(id), None);
        registry
            .register_plugin(entry)
            .await
            .expect("should succeed");

        let first = opt
            .optimized_plugin_lookup("plugin-a", &registry)
            .await
            .expect("lookup");
        assert_eq!(first.id(), id);

        let second = opt
            .optimized_plugin_lookup("plugin-a", &registry)
            .await
            .expect("cached");
        assert_eq!(second.id(), id);
    }

    #[tokio::test]
    async fn optimized_capability_query_populates_cache() {
        let opt = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::development());
        let registry = crate::zero_copy::ZeroCopyPluginRegistry::new();
        let id = Uuid::new_v4();
        let mut meta =
            ZeroCopyPluginMetadata::new(id, "p".into(), "1".into(), "d".into(), "a".into());
        meta.capabilities = std::sync::Arc::new(vec!["search.cap".into()]);
        let entry = ZeroCopyPluginEntry::new(meta, ZeroCopyPluginConfig::new(id), None);
        registry
            .register_plugin(entry)
            .await
            .expect("should succeed");

        let r1 = opt
            .optimized_capability_query("search.cap", &registry)
            .await;
        assert_eq!(r1.len(), 1);
        let r2 = opt
            .optimized_capability_query("search.cap", &registry)
            .await;
        assert_eq!(r2.len(), 1);
    }

    #[tokio::test]
    async fn batch_load_plugins_single_vs_batch_paths() {
        let opt = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::development());
        let id = Uuid::new_v4();
        let meta =
            ZeroCopyPluginMetadata::new(id, "solo".into(), "1".into(), "d".into(), "a".into());
        let e = Arc::new(ZeroCopyPluginEntry::new(
            meta,
            ZeroCopyPluginConfig::new(id),
            None,
        ));
        let empty = opt.batch_load_plugins(vec![e.clone()]).await;
        assert_eq!(empty.len(), 1);
        assert!(empty[0].is_err());

        let id2 = Uuid::new_v4();
        let m2 = ZeroCopyPluginMetadata::new(id2, "a".into(), "1".into(), "d".into(), "a".into());
        let e2 = Arc::new(ZeroCopyPluginEntry::new(
            m2,
            ZeroCopyPluginConfig::new(id2),
            None,
        ));
        let batch = opt.batch_load_plugins(vec![e, e2]).await;
        assert_eq!(batch.len(), 2);
        assert!(batch.iter().all(Result::is_err));
    }

    #[tokio::test]
    async fn get_optimization_metrics_smoke() {
        let opt = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::development());
        let m = opt.get_optimization_metrics().await;
        assert!(m.cache_efficiency >= 0.0 && m.cache_efficiency <= 1.0);
    }

    #[tokio::test]
    async fn enable_predictive_loading_no_panic_when_disabled() {
        let opt = PluginPerformanceOptimizer::new(PerformanceOptimizerConfig::development());
        let registry = crate::zero_copy::ZeroCopyPluginRegistry::new();
        opt.enable_predictive_loading(&registry).await;
    }
}
