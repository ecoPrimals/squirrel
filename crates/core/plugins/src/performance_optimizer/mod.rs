// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Performance Optimizer
#![allow(
    dead_code,
    reason = "Performance optimizer infrastructure awaiting activation"
)]
//!
//! This module provides advanced performance optimizations for plugin operations including:
//! - Hot path caching for frequent operations
//! - Batch processing for bulk operations
//! - Predictive loading based on usage patterns
//! - Memory pool integration
//! - Zero-copy plugin data structures

mod batch_processor;
mod config;
mod hot_path_cache;
mod memory_optimizer;
pub mod optimized_ops;
mod optimizer;
mod predictive_loader;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use config::{
    BatchProcessingConfig, HotPathCacheConfig, MemoryOptimizationConfig,
    PerformanceOptimizerConfig, PredictiveLoadingConfig,
};
pub use optimizer::{PluginPerformanceOptimizer, get_global_optimizer, init_global_optimizer};
pub use types::OptimizerMetrics;
