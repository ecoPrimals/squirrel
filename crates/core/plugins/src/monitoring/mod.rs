// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring plugin module
//!
//! This module provides functionality for monitoring plugins.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// Monitoring plugin trait
#[async_trait]
pub trait MonitoringPlugin: Plugin {
    /// Collect metrics
    async fn collect_metrics(&self) -> Result<Value>;

    /// Get monitoring targets
    fn get_monitoring_targets(&self) -> Vec<String>;

    /// Check if the plugin supports a specific monitoring target
    fn supports_monitoring_target(&self, target: &str) -> bool {
        self.get_monitoring_targets().contains(&target.to_string())
    }

    /// Handle alerts
    async fn handle_alert(&self, alert: Value) -> Result<()>;

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}
