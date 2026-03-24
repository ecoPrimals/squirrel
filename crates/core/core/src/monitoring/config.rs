// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration for monitoring delegation and fallbacks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for monitoring delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring delegation
    pub enabled: bool,

    /// Require at least one provider to be available
    pub require_provider: bool,

    /// Songbird-specific configuration
    pub songbird_config: Option<SongbirdConfig>,

    /// Generic monitoring provider configurations
    pub provider_configs: HashMap<String, serde_json::Value>,

    /// Fallback configuration
    pub fallback_config: FallbackConfig,
}

/// Songbird monitoring service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    /// Songbird endpoint URL.
    pub endpoint: String,
    /// Service name for identification.
    pub service_name: String,
    /// Optional auth token.
    pub auth_token: Option<String>,
    /// Batch size for metrics.
    pub batch_size: usize,
    /// Flush interval for batching.
    pub flush_interval: std::time::Duration,
}

/// Fallback logging configuration when no provider is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Log level (debug, info, warn, error).
    pub log_level: String,
    /// Whether to include metrics in fallback output.
    pub include_metrics: bool,
    /// Whether to include health in fallback output.
    pub include_health: bool,
    /// Whether to include performance in fallback output.
    pub include_performance: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_provider: false,
            songbird_config: None,
            provider_configs: HashMap::new(),
            fallback_config: FallbackConfig::default(),
        }
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        }
    }
}
