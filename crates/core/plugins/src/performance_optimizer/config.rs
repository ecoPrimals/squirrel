// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration types for the plugin performance optimizer.

use std::time::Duration;

/// Configuration for plugin performance optimization
#[derive(Debug, Clone)]
pub struct PerformanceOptimizerConfig {
    /// Hot path cache configuration
    pub hot_path_cache: HotPathCacheConfig,

    /// Batch processing configuration
    pub batch_processing: BatchProcessingConfig,

    /// Predictive loading configuration
    pub predictive_loading: PredictiveLoadingConfig,

    /// Memory optimization configuration
    pub memory_optimization: MemoryOptimizationConfig,
}

/// Hot path cache configuration
#[derive(Debug, Clone)]
pub struct HotPathCacheConfig {
    /// Maximum cached operations
    pub max_cached_operations: usize,

    /// Cache TTL
    pub cache_ttl: Duration,

    /// Minimum access count to cache
    pub min_access_count: u64,

    /// Enable cache warming
    pub enable_warming: bool,
}

/// Batch processing configuration
#[derive(Debug, Clone)]
pub struct BatchProcessingConfig {
    /// Maximum batch size
    pub max_batch_size: usize,

    /// Batch timeout
    pub batch_timeout: Duration,

    /// Enable dynamic batching
    pub dynamic_batching: bool,
}

/// Predictive loading configuration
#[derive(Debug, Clone)]
pub struct PredictiveLoadingConfig {
    /// Enable predictive loading
    pub enabled: bool,

    /// Prediction window
    pub prediction_window: Duration,

    /// Minimum confidence threshold
    pub confidence_threshold: f64,

    /// Maximum predictive loads
    pub max_predictive_loads: usize,
}

/// Memory optimization configuration
#[derive(Debug, Clone)]
pub struct MemoryOptimizationConfig {
    /// Enable zero-copy optimizations
    pub zero_copy_enabled: bool,

    /// Memory pool integration
    pub memory_pool_integration: bool,

    /// Lazy loading threshold
    pub lazy_loading_threshold: usize,

    /// Enable memory compaction
    pub enable_compaction: bool,
}

impl PerformanceOptimizerConfig {
    /// Production-optimized configuration
    pub const fn production() -> Self {
        Self {
            hot_path_cache: HotPathCacheConfig {
                max_cached_operations: 10000,
                cache_ttl: Duration::from_secs(300), // 5 minutes
                min_access_count: 3,
                enable_warming: true,
            },
            batch_processing: BatchProcessingConfig {
                max_batch_size: 100,
                batch_timeout: Duration::from_millis(50),
                dynamic_batching: true,
            },
            predictive_loading: PredictiveLoadingConfig {
                enabled: true,
                prediction_window: Duration::from_secs(60),
                confidence_threshold: 0.7,
                max_predictive_loads: 50,
            },
            memory_optimization: MemoryOptimizationConfig {
                zero_copy_enabled: true,
                memory_pool_integration: true,
                lazy_loading_threshold: 1024 * 1024, // 1MB
                enable_compaction: true,
            },
        }
    }

    /// Development configuration with reduced overhead
    pub const fn development() -> Self {
        Self {
            hot_path_cache: HotPathCacheConfig {
                max_cached_operations: 1000,
                cache_ttl: Duration::from_secs(60),
                min_access_count: 2,
                enable_warming: false,
            },
            batch_processing: BatchProcessingConfig {
                max_batch_size: 20,
                batch_timeout: Duration::from_millis(100),
                dynamic_batching: false,
            },
            predictive_loading: PredictiveLoadingConfig {
                enabled: false,
                prediction_window: Duration::from_secs(30),
                confidence_threshold: 0.8,
                max_predictive_loads: 10,
            },
            memory_optimization: MemoryOptimizationConfig {
                zero_copy_enabled: true,
                memory_pool_integration: false,
                lazy_loading_threshold: 512 * 1024, // 512KB
                enable_compaction: false,
            },
        }
    }
}
