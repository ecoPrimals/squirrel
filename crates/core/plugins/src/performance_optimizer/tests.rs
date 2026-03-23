// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the plugin performance optimizer.

use std::sync::Arc;

use super::config::PerformanceOptimizerConfig;
use super::optimized_ops;
use super::optimizer::{PluginPerformanceOptimizer, get_global_optimizer};
use super::types::OptimizerMetrics;

#[test]
fn test_performance_optimizer_config_production() {
    let config = PerformanceOptimizerConfig::production();
    assert_eq!(config.hot_path_cache.max_cached_operations, 10000);
    assert_eq!(config.hot_path_cache.min_access_count, 3);
    assert!(config.hot_path_cache.enable_warming);
    assert_eq!(config.batch_processing.max_batch_size, 100);
    assert!(config.batch_processing.dynamic_batching);
    assert!(config.predictive_loading.enabled);
    assert!((config.predictive_loading.confidence_threshold - 0.7).abs() < 1e-9);
    assert!(config.memory_optimization.zero_copy_enabled);
    assert!(config.memory_optimization.enable_compaction);
}

#[test]
fn test_performance_optimizer_config_development() {
    let config = PerformanceOptimizerConfig::development();
    assert_eq!(config.hot_path_cache.max_cached_operations, 1000);
    assert!(!config.hot_path_cache.enable_warming);
    assert_eq!(config.batch_processing.max_batch_size, 20);
    assert!(!config.batch_processing.dynamic_batching);
    assert!(!config.predictive_loading.enabled);
    assert!(!config.memory_optimization.enable_compaction);
}

#[tokio::test]
async fn test_get_global_optimizer() {
    let optimizer = get_global_optimizer();
    assert!(Arc::strong_count(&optimizer) >= 2);
}

#[test]
fn test_optimizer_metrics_default() {
    let metrics = OptimizerMetrics::default();
    assert!((metrics.cache_efficiency - 0.0).abs() < f64::EPSILON);
    assert!((metrics.batch_efficiency - 0.0).abs() < f64::EPSILON);
    assert!((metrics.prediction_accuracy - 0.0).abs() < f64::EPSILON);
    assert_eq!(metrics.memory_saved_bytes, 0);
    assert_eq!(metrics.operations_optimized, 0);
}

#[test]
fn test_optimizer_metrics_clone() {
    let metrics = OptimizerMetrics {
        cache_efficiency: 0.7,
        batch_efficiency: 0.5,
        prediction_accuracy: 0.8,
        memory_saved_bytes: 1024,
        operations_optimized: 100,
        total_time_saved_ms: 500,
    };
    let cloned = metrics.clone();
    assert!((metrics.cache_efficiency - cloned.cache_efficiency).abs() < f64::EPSILON);
    assert_eq!(metrics.operations_optimized, cloned.operations_optimized);
}

#[tokio::test]
async fn test_plugin_performance_optimizer_new() {
    let config = PerformanceOptimizerConfig::development();
    let _optimizer = PluginPerformanceOptimizer::new(config);
}

#[tokio::test]
async fn test_get_performance_metrics() {
    let metrics = optimized_ops::get_performance_metrics().await;
    assert!(metrics.cache_efficiency >= 0.0 && metrics.cache_efficiency <= 1.0);
}

#[tokio::test]
async fn hot_path_cache_plugin_and_capability_inserts() {
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use uuid::Uuid;

    use super::config::HotPathCacheConfig;
    use super::hot_path_cache::HotPathCache;
    use super::types::{CachedCapabilityQuery, CachedPluginLookup};

    let cfg = HotPathCacheConfig {
        max_cached_operations: 10,
        cache_ttl: Duration::from_secs(60),
        min_access_count: 0,
        enable_warming: false,
    };
    let cache = HotPathCache::new(cfg);
    let pid = Uuid::new_v4();
    let lookup = CachedPluginLookup {
        plugin_id: Some(pid),
        cached_at: SystemTime::now(),
        access_count: std::sync::atomic::AtomicU64::new(0),
        hit_rate: 1.0,
    };
    cache.cache_plugin_lookup("lookup:key".into(), lookup).await;
    assert!(cache.get_plugin_lookup("lookup:key").await.is_some());

    let cq = CachedCapabilityQuery {
        matching_plugins: Arc::new(vec![pid]),
        cached_at: SystemTime::now(),
        access_count: std::sync::atomic::AtomicU64::new(0),
        query_time: Duration::from_millis(2),
    };
    cache.cache_capability_query("cap:key".into(), cq).await;
    assert!(cache.get_capability_query("cap:key").await.is_some());

    let _ = cache.get_statistics().await;
    cache.start_cache_warming().await;
}
