// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Plugin manager metrics and status structures
//!
//! This module provides structures for tracking plugin manager performance and status.

use serde::{Deserialize, Serialize};

/// Plugin manager status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerStatus {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub failed_plugins: usize,
}

impl PluginManagerStatus {
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
    pub load_time_ms: u64,
    pub memory_usage_kb: u64,
}

impl PluginManagerMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
