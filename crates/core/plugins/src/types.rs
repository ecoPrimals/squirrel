// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin types
//!
//! This module defines the various plugin types supported by the system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use universal_constants::limits;

/// Plugin type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Built-in plugin
    Builtin,
    /// Native (shared library) plugin
    Native,
    /// WebAssembly plugin
    WebAssembly,
    /// Script plugin
    Script,
}

/// Plugin resource usage information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginResources {
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Number of open file handles
    pub file_handles: u32,
    /// Number of network connections
    pub network_connections: u32,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Configuration settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Plugin-specific environment variables
    pub environment: HashMap<String, String>,
    /// Resource limits
    pub limits: ResourceLimits,
}

/// Resource limits for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(
    clippy::struct_field_names,
    reason = "Domain naming convention: plugin_id, plugin_name"
)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES),
            max_cpu_percent: Some(limits::DEFAULT_PLUGIN_MAX_CPU_PERCENT),
            max_execution_time_secs: Some(limits::DEFAULT_PLUGIN_MAX_EXECUTION_TIME_SECS),
        }
    }
}

/// Plugin status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is inactive
    Inactive,
    /// Plugin is registered but not loaded
    Registered,
    /// Plugin is loaded but not running
    Loaded,
    /// Plugin is initialized and ready
    Initialized,
    /// Plugin is running
    Running,
    /// Plugin is stopped
    Stopped,
    /// Plugin failed to start
    Failed,
    /// Plugin is stopping
    Stopping,
    /// Plugin is unloaded
    Unloaded,
}

// PluginMetadata removed - use squirrel_interfaces::plugins::PluginMetadata instead
// This was duplicate/unused code. The canonical version is in squirrel-interfaces crate.

#[cfg(test)]
mod tests {
    use super::*;
    use universal_constants::limits;

    fn serde_roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let json = serde_json::to_string(value).expect("should succeed");
        let decoded: T = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(value, &decoded);
    }

    #[test]
    fn test_plugin_type_serde() {
        serde_roundtrip(&PluginType::Builtin);
        serde_roundtrip(&PluginType::Native);
        serde_roundtrip(&PluginType::WebAssembly);
        serde_roundtrip(&PluginType::Script);
    }

    #[test]
    fn test_plugin_status_serde() {
        serde_roundtrip(&PluginStatus::Inactive);
        serde_roundtrip(&PluginStatus::Registered);
        serde_roundtrip(&PluginStatus::Loaded);
        serde_roundtrip(&PluginStatus::Initialized);
        serde_roundtrip(&PluginStatus::Running);
        serde_roundtrip(&PluginStatus::Stopped);
        serde_roundtrip(&PluginStatus::Failed);
        serde_roundtrip(&PluginStatus::Stopping);
        serde_roundtrip(&PluginStatus::Unloaded);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(
            limits.max_memory_bytes,
            Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES)
        );
        assert_eq!(
            limits.max_cpu_percent,
            Some(limits::DEFAULT_PLUGIN_MAX_CPU_PERCENT)
        );
        assert_eq!(
            limits.max_execution_time_secs,
            Some(limits::DEFAULT_PLUGIN_MAX_EXECUTION_TIME_SECS)
        );
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.settings.is_empty());
        assert!(config.environment.is_empty());
        assert_eq!(
            config.limits.max_memory_bytes,
            Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES)
        );
    }

    #[test]
    fn test_plugin_resources_default() {
        let resources = PluginResources::default();
        assert_eq!(resources.memory_usage, 0);
        assert!((resources.cpu_usage - 0.0).abs() < f64::EPSILON);
        assert_eq!(resources.file_handles, 0);
        assert_eq!(resources.network_connections, 0);
    }
}
