// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration and state types for the zero-copy plugin system.
//!
//! Extracted from [`super::zero_copy`] for module size management.
//!
//! Fields are self-documenting DTO structs; see the parent module for usage docs.
#![expect(missing_docs, reason = "DTO fields — documented at usage site")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::PluginStatus;

/// Zero-copy plugin configuration
#[derive(Debug, Clone)]
pub struct ZeroCopyPluginConfig {
    pub plugin_id: Uuid,
    pub config_data: Arc<HashMap<String, serde_json::Value>>,
    pub environment: Arc<HashMap<String, String>>,
    pub resource_limits: Arc<ResourceLimits>,
    pub security_settings: Arc<SecuritySettings>,
}

/// Resource limits for plugin execution (memory, CPU, disk, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_disk_mb: Option<u64>,
    pub max_network_mbps: Option<f64>,
    pub max_open_files: Option<u32>,
}

/// Security constraints for plugin sandboxing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub sandboxed: bool,
    pub allowed_paths: Vec<String>,
    pub allowed_hosts: Vec<String>,
    pub required_permissions: Vec<String>,
}

impl ZeroCopyPluginConfig {
    #[must_use]
    pub fn new(plugin_id: Uuid) -> Self {
        Self {
            plugin_id,
            config_data: Arc::new(HashMap::new()),
            environment: Arc::new(HashMap::new()),
            resource_limits: Arc::new(ResourceLimits {
                max_memory_mb: Some(512),
                max_cpu_percent: Some(10.0),
                max_disk_mb: Some(100),
                max_network_mbps: Some(10.0),
                max_open_files: Some(64),
            }),
            security_settings: Arc::new(SecuritySettings {
                sandboxed: true,
                allowed_paths: vec!["/tmp".to_string()],
                allowed_hosts: {
                    let localhost = universal_constants::network::DEFAULT_LOCALHOST.to_string();
                    vec![
                        std::env::var("MCP_HOST").unwrap_or_else(|_| localhost.clone()),
                        std::env::var("SECURITY_HOST").unwrap_or_else(|_| localhost.clone()),
                        localhost,
                    ]
                },
                required_permissions: vec![],
            }),
        }
    }

    #[must_use]
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config_data.get(key)
    }

    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(std::string::String::as_str)
    }
}

/// Zero-copy plugin state with Arc for shared data
#[derive(Debug, Clone)]
pub struct ZeroCopyPluginState {
    pub plugin_id: Uuid,
    pub status: PluginStatus,
    pub state_data: Arc<HashMap<String, serde_json::Value>>,
    pub last_updated: std::time::SystemTime,
    pub state_history: Arc<Vec<StateTransition>>,
    pub metrics: Arc<PluginMetrics>,
}

/// Record of a plugin status change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: PluginStatus,
    pub to: PluginStatus,
    pub timestamp: std::time::SystemTime,
    pub reason: Option<String>,
}

/// Performance metrics for a plugin instance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginMetrics {
    pub total_executions: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub error_count: u64,
    pub last_execution: Option<std::time::SystemTime>,
}

impl ZeroCopyPluginState {
    #[must_use]
    pub fn new(plugin_id: Uuid, status: PluginStatus) -> Self {
        Self {
            plugin_id,
            status,
            state_data: Arc::new(HashMap::new()),
            last_updated: std::time::SystemTime::now(),
            state_history: Arc::new(Vec::new()),
            metrics: Arc::new(PluginMetrics::default()),
        }
    }

    pub fn apply_status(&mut self, new_status: PluginStatus, reason: Option<String>) {
        let transition = StateTransition {
            from: self.status,
            to: new_status,
            timestamp: std::time::SystemTime::now(),
            reason,
        };

        let history = Arc::make_mut(&mut self.state_history);
        history.push(transition);

        self.status = new_status;
        self.last_updated = std::time::SystemTime::now();
    }

    #[must_use]
    pub fn with_status(mut self, new_status: PluginStatus, reason: Option<String>) -> Self {
        self.apply_status(new_status, reason);
        self
    }

    #[must_use]
    pub fn get_state(&self, key: &str) -> Option<&serde_json::Value> {
        self.state_data.get(key)
    }

    #[must_use]
    pub fn metrics(&self) -> &PluginMetrics {
        &self.metrics
    }
}
