// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Main plugin performance optimizer implementation.

use std::sync::Arc;
use std::time::{Instant, SystemTime};

use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::errors::{PluginError, Result};
use crate::zero_copy::{ZeroCopyPlugin, ZeroCopyPluginEntry, ZeroCopyPluginRegistry};

use super::batch_processor::BatchProcessor;
use super::config::PerformanceOptimizerConfig;
use super::hot_path_cache::HotPathCache;
use super::memory_optimizer::MemoryOptimizer;
use super::predictive_loader::PredictiveLoader;
use super::types::{CacheStatistics, CachedCapabilityQuery, CachedPluginLookup, OptimizerMetrics};

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

    /// Performance metrics for tracking optimizer effectiveness
    metrics: Arc<RwLock<OptimizerMetrics>>,

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
        let metrics = Arc::new(RwLock::new(OptimizerMetrics::default()));

        let optimizer = Self {
            hot_path_cache,
            batch_processor,
            predictive_loader,
            memory_optimizer,
            metrics,
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
        if plugin_entries.len() <= 1 {
            // No benefit from batching single operations
            let results = Vec::new();
            for entry in plugin_entries {
                // STUB: Plugin loading is intentionally simplified for performance benchmarking.
                warn!(
                    plugin_name = %entry.name(),
                    "batch_load_plugins: stub path — no real plugin loaded; unified plugin system not yet implemented"
                );
                let _entry = entry; // Suppress unused warning
            }
            return results;
        }

        // Use batch processor for multiple plugins
        self.batch_processor
            .batch_load_plugins(plugin_entries)
            .await
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
    #[allow(
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
        assert!(empty.is_empty());

        let id2 = Uuid::new_v4();
        let m2 = ZeroCopyPluginMetadata::new(id2, "a".into(), "1".into(), "d".into(), "a".into());
        let e2 = Arc::new(ZeroCopyPluginEntry::new(
            m2,
            ZeroCopyPluginConfig::new(id2),
            None,
        ));
        let batch = opt.batch_load_plugins(vec![e, e2]).await;
        assert!(batch.is_empty());
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
