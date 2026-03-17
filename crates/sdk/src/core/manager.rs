// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin lifecycle manager — registration, startup, shutdown of WASM plugins.

use super::plugin::{PluginStatus, WasmPlugin};
use crate::error::{PluginError, PluginResult};
use crate::utils::safe_lock;

/// Manager for registered WASM plugins.
pub struct PluginManager {
    plugins: std::sync::Mutex<std::collections::HashMap<String, Box<dyn WasmPlugin>>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&self, id: String, plugin: Box<dyn WasmPlugin>) -> PluginResult<()> {
        let mut plugins = safe_lock(&self.plugins, "plugins")?;

        if plugins.contains_key(&id) {
            return Err(PluginError::PluginAlreadyExists { plugin_id: id });
        }

        plugins.insert(id, plugin);
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&self, id: &str) -> PluginResult<()> {
        let mut plugins = safe_lock(&self.plugins, "plugins")?;

        if let Some(mut plugin) = plugins.remove(id)
            && let Err(e) = plugin.stop()
        {
            return Err(PluginError::InternalError {
                message: format!("Failed to stop plugin: {e:?}"),
            });
        }

        Ok(())
    }

    /// Check if a plugin exists by ID
    pub fn has_plugin(&self, id: &str) -> PluginResult<bool> {
        let plugins = safe_lock(&self.plugins, "plugins")?;
        Ok(plugins.contains_key(id))
    }

    /// List all plugin IDs
    pub fn list_plugins(&self) -> PluginResult<Vec<String>> {
        let plugins = safe_lock(&self.plugins, "plugins")?;
        Ok(plugins.keys().cloned().collect())
    }

    /// Start all plugins
    pub fn start_all(&mut self) -> PluginResult<()> {
        let plugins = safe_lock(&self.plugins, "plugins")?;

        for (id, plugin) in plugins.iter() {
            let info = plugin.get_info();
            if matches!(info.state, PluginStatus::Uninitialized) {
                return Err(PluginError::InitializationError {
                    reason: format!("Plugin {id} is not initialized"),
                });
            }
        }

        Ok(())
    }

    /// Stop all plugins
    pub fn stop_all(&mut self) -> PluginResult<()> {
        let mut plugins = safe_lock(&self.plugins, "plugins")?;

        for plugin in plugins.values_mut() {
            if let Err(e) = plugin.stop() {
                return Err(PluginError::InternalError {
                    message: format!("Failed to stop plugin: {e:?}"),
                });
            }
        }

        Ok(())
    }
}

/// Plugin factory trait for creating plugin instances
pub trait PluginFactory {
    /// Create a new plugin instance
    fn create_plugin() -> Box<dyn WasmPlugin>;
}

/// Macro to simplify plugin registration
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[wasm_bindgen]
        pub fn create_plugin() -> $plugin_type {
            <$plugin_type>::new()
        }

        #[wasm_bindgen]
        pub fn get_plugin_info() -> PluginInfo {
            let plugin = <$plugin_type>::new();
            plugin.get_info()
        }
    };
}

/// Utility functions for plugin development
pub mod utils {
    use super::super::plugin::{PluginContext, PluginInfo};
    use crate::error::PluginResult;

    /// Initialize the SDK for the current plugin
    pub fn init_sdk(plugin_id: String) -> PluginResult<()> {
        crate::internal::init_plugin_environment(&plugin_id)?;
        Ok(())
    }

    /// Get SDK version
    pub fn get_version() -> String {
        crate::SDK_VERSION.to_string()
    }

    /// Create a plugin context with default values
    pub fn create_default_context(plugin_id: String) -> PluginContext {
        PluginContext::new(plugin_id)
    }

    /// Validate plugin info
    pub fn validate_plugin_info(info: &PluginInfo) -> PluginResult<()> {
        if info.id.is_empty() {
            return Err(crate::error::PluginError::InvalidConfiguration {
                message: "Plugin ID cannot be empty".to_string(),
            });
        }

        if info.name.is_empty() {
            return Err(crate::error::PluginError::InvalidConfiguration {
                message: "Plugin name cannot be empty".to_string(),
            });
        }

        if info.version.is_empty() {
            return Err(crate::error::PluginError::InvalidConfiguration {
                message: "Plugin version cannot be empty".to_string(),
            });
        }

        if semver::Version::parse(&info.version).is_err() {
            return Err(crate::error::PluginError::InvalidConfiguration {
                message: "Invalid version format".to_string(),
            });
        }

        Ok(())
    }
}
