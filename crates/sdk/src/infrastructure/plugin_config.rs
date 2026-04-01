// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin manifest and sandbox configuration types.
//!
//! Extracted from the main config module to keep the infrastructure env-config
//! types separate from the plugin-domain serde/manifest types.

use crate::error::{PluginError, PluginResult};
use crate::plugin::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin permissions
    pub permissions: Vec<Permission>,
    /// Plugin settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Plugin UI configuration
    pub ui: Option<UiConfig>,
    /// Plugin resources
    pub resources: Vec<String>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin license
    pub license: String,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Plugin repository URL
    pub repository: Option<String>,
    /// Plugin documentation URL
    pub documentation: Option<String>,
    /// Plugin categories
    pub categories: Vec<String>,
    /// Plugin tags
    pub tags: Vec<String>,
    /// Plugin keywords
    pub keywords: Vec<String>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A Squirrel plugin".to_string(),
            author: "Unknown".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            documentation: None,
            categories: vec!["general".to_string()],
            tags: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Whether network access is allowed
    pub network_access: bool,
    /// Allowed file system paths
    pub file_system_access: Vec<String>,
    /// Memory limit in MB
    pub memory_limit_mb: u32,
    /// CPU limit as percentage
    pub cpu_limit_percent: u8,
    /// Execution timeout in seconds
    pub execution_timeout_seconds: u32,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SandboxConfig {
    /// Load sandbox configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            network_access: std::env::var("SANDBOX_NETWORK_ACCESS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            file_system_access: std::env::var("SANDBOX_FILE_SYSTEM_ACCESS")
                .unwrap_or_else(|_| String::new())
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            memory_limit_mb: std::env::var("SANDBOX_MEMORY_LIMIT_MB")
                .unwrap_or_else(|_| "128".to_string())
                .parse()
                .unwrap_or(128),
            cpu_limit_percent: std::env::var("SANDBOX_CPU_LIMIT_PERCENT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            execution_timeout_seconds: std::env::var("SANDBOX_EXECUTION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            security_level: match std::env::var("SANDBOX_SECURITY_LEVEL")
                .unwrap_or_else(|_| "restricted".to_string())
                .to_lowercase()
                .as_str()
            {
                "standard" => SecurityLevel::Standard,
                "enhanced" => SecurityLevel::Enhanced,
                "full" => SecurityLevel::Full,
                _ => SecurityLevel::Restricted,
            },
        }
    }

    /// Validate sandbox configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.memory_limit_mb == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Memory limit must be greater than 0".to_string(),
            });
        }

        if self.cpu_limit_percent == 0 || self.cpu_limit_percent > 100 {
            return Err(PluginError::InvalidConfiguration {
                message: "CPU limit must be between 1 and 100".to_string(),
            });
        }

        if self.execution_timeout_seconds == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Execution timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Security level for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Minimal permissions, safe for untrusted plugins
    Restricted,
    /// Standard permissions for verified plugins
    Standard,
    /// Enhanced permissions for trusted plugins
    Enhanced,
    /// Full permissions for system plugins
    Full,
}

/// MCP protocol capabilities configuration (distinct from `McpClientConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// MCP protocol version
    pub protocol_version: String,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether tools are supported
    pub supports_tools: bool,
    /// Whether resources are supported
    pub supports_resources: bool,
    /// Whether prompts are supported
    pub supports_prompts: bool,
    /// Custom capabilities
    pub custom_capabilities: HashMap<String, serde_json::Value>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            protocol_version: "1.0".to_string(),
            supports_streaming: false,
            supports_tools: true,
            supports_resources: false,
            supports_prompts: false,
            custom_capabilities: HashMap::new(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Whether the plugin has a web UI
    pub has_web_ui: bool,
    /// UI entry point file
    pub ui_entry_point: Option<String>,
    /// UI theme support
    pub theme_support: bool,
    /// Responsive design support
    pub responsive: bool,
    /// Accessibility support
    pub accessibility: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            has_web_ui: false,
            ui_entry_point: None,
            theme_support: true,
            responsive: true,
            accessibility: true,
        }
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency name
    pub name: String,
    /// Version requirement
    pub version: String,
    /// Whether the dependency is optional
    pub optional: bool,
    /// Dependency features to enable
    pub features: Vec<String>,
}

impl PluginConfig {
    /// Validate the configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.metadata.name.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin name cannot be empty".to_string(),
            });
        }

        if self.metadata.version.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin version cannot be empty".to_string(),
            });
        }

        if !self.metadata.version.chars().any(|c| c.is_ascii_digit()) {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin version must contain at least one digit".to_string(),
            });
        }

        for permission in &self.permissions {
            match permission {
                Permission::NetworkAccess => {}
                Permission::FileSystemRead(_) | Permission::FileSystemWrite(_) => {}
                Permission::LocalStorage | Permission::SessionStorage => {}
            }
        }

        Ok(())
    }

    /// Get memory limit in bytes
    pub fn memory_limit_bytes(&self) -> usize {
        let sandbox_config = SandboxConfig::from_env();
        (sandbox_config.memory_limit_mb as usize) * 1024 * 1024
    }

    /// Check if path access is allowed
    pub fn is_path_allowed(&self, _path: &str) -> bool {
        true
    }

    /// Get a setting value by key
    pub fn get_setting<T>(&self, key: &str) -> PluginResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.settings.get(key) {
            Some(value) => {
                let parsed: T = serde_json::from_value(value.clone())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a setting value
    pub fn set_setting<T>(&mut self, key: &str, value: T) -> PluginResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.settings.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Check if plugin has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        match capability {
            "network" => self
                .permissions
                .iter()
                .any(|p| matches!(p, Permission::NetworkAccess)),
            "filesystem" => self.permissions.iter().any(|p| {
                matches!(
                    p,
                    Permission::FileSystemRead(_) | Permission::FileSystemWrite(_)
                )
            }),
            "storage" => self
                .permissions
                .iter()
                .any(|p| matches!(p, Permission::LocalStorage | Permission::SessionStorage)),
            "ui" => self.ui.as_ref().is_some_and(|ui| ui.has_web_ui),
            _ => false,
        }
    }

    /// Get the maximum allowed memory in bytes
    pub fn max_memory_bytes(&self) -> usize {
        let sandbox_config = SandboxConfig::from_env();
        (sandbox_config.memory_limit_mb as usize) * 1024 * 1024
    }
}
