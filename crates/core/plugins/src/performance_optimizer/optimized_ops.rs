// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! High-performance utilities for plugin operations.

use std::sync::Arc;

use crate::zero_copy::{ZeroCopyPlugin, ZeroCopyPluginEntry, ZeroCopyPluginRegistry};

use super::optimizer::get_global_optimizer;
use super::types::OptimizerMetrics;

/// Perform an optimized plugin lookup
pub async fn fast_plugin_lookup(
    plugin_name: &str,
    registry: &ZeroCopyPluginRegistry,
) -> Option<Arc<ZeroCopyPluginEntry>> {
    get_global_optimizer()
        .optimized_plugin_lookup(plugin_name, registry)
        .await
}

/// Perform an optimized capability query
pub async fn fast_capability_query(
    capability: &str,
    registry: &ZeroCopyPluginRegistry,
) -> Vec<Arc<ZeroCopyPluginEntry>> {
    get_global_optimizer()
        .optimized_capability_query(capability, registry)
        .await
}

/// Batch load multiple plugins efficiently
pub async fn batch_load(
    plugin_entries: Vec<Arc<ZeroCopyPluginEntry>>,
) -> Vec<crate::errors::Result<Arc<dyn ZeroCopyPlugin>>> {
    get_global_optimizer()
        .batch_load_plugins(plugin_entries)
        .await
}

/// Get current optimization metrics
pub async fn get_performance_metrics() -> OptimizerMetrics {
    get_global_optimizer().get_optimization_metrics().await
}
