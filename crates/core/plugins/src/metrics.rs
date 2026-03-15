// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin manager metrics and status structures
//!
//! This module provides structures for tracking plugin manager performance and status.

use serde::{Deserialize, Serialize};

/// Plugin manager status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerStatus {
    /// Total number of registered plugins.
    pub total_plugins: usize,
    /// Number of plugins currently active and running.
    pub active_plugins: usize,
    /// Number of plugins that failed to load or crashed.
    pub failed_plugins: usize,
}

impl PluginManagerStatus {
    /// Creates status from plugin counts.
    pub fn new(total: usize, active: usize, failed: usize) -> Self {
        Self {
            total_plugins: total,
            active_plugins: active,
            failed_plugins: failed,
        }
    }
}

/// Plugin manager performance metrics
#[derive(Debug, Default)]
pub struct PluginManagerMetrics {
    /// Total time spent loading plugins in milliseconds.
    pub load_time_ms: u64,
    /// Memory used by the plugin manager in kilobytes.
    pub memory_usage_kb: u64,
}

impl PluginManagerMetrics {
    /// Creates default metrics.
    pub fn new() -> Self {
        Self::default()
    }
}
