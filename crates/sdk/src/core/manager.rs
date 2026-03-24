// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    use super::super::plugin::{
        PluginCapabilities, PluginInfo, PluginStats, PluginStatus, WasmPlugin,
    };
    use super::PluginManager;
    use super::utils;
    use crate::config::PluginConfig;
    use crate::error::PluginError;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use wasm_bindgen::prelude::*;

    fn sample_info(id: &str, state: PluginStatus) -> PluginInfo {
        PluginInfo {
            id: id.to_string(),
            name: "n".to_string(),
            version: "1.0.0".to_string(),
            state,
            config: PluginConfig::default(),
            stats: PluginStats::default(),
            capabilities: vec![],
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            repository: None,
            keywords: vec![],
            metadata: serde_json::json!({}),
        }
    }

    struct MockPlugin {
        info: PluginInfo,
        stop_fail: Arc<AtomicBool>,
    }

    impl MockPlugin {
        fn new(id: &str, state: PluginStatus, stop_fail: Arc<AtomicBool>) -> Self {
            Self {
                info: sample_info(id, state),
                stop_fail,
            }
        }
    }

    impl WasmPlugin for MockPlugin {
        fn get_info(&self) -> PluginInfo {
            self.info.clone()
        }

        fn initialize(&mut self, _config: JsValue) -> Result<(), JsValue> {
            Ok(())
        }

        fn start(&mut self) -> Result<(), JsValue> {
            Ok(())
        }

        fn stop(&mut self) -> Result<(), JsValue> {
            if self.stop_fail.load(Ordering::SeqCst) {
                Err(JsValue::from_str("stop failed"))
            } else {
                Ok(())
            }
        }

        fn pause(&mut self) -> Result<(), JsValue> {
            Ok(())
        }

        fn resume(&mut self) -> Result<(), JsValue> {
            Ok(())
        }

        fn handle_command(&self, _command: &str, _params: JsValue) -> Result<JsValue, JsValue> {
            Ok(JsValue::NULL)
        }

        fn handle_event(&self, _event: JsValue) -> Result<(), JsValue> {
            Ok(())
        }

        fn get_stats(&self) -> PluginStats {
            PluginStats::default()
        }

        fn get_capabilities(&self) -> PluginCapabilities {
            PluginCapabilities::default()
        }

        fn shutdown(&mut self) -> Result<(), JsValue> {
            Ok(())
        }

        fn is_initialized(&self) -> bool {
            true
        }

        fn get_status(&self) -> PluginStatus {
            self.info.state.clone()
        }
    }

    #[test]
    fn plugin_manager_default_new_register_list() {
        let _: PluginManager = PluginManager::default();
        let mgr = PluginManager::new();
        let fail = Arc::new(AtomicBool::new(false));
        mgr.register_plugin(
            "p1".to_string(),
            Box::new(MockPlugin::new("p1", PluginStatus::Active, fail.clone())),
        )
        .expect("should succeed");
        assert!(mgr.has_plugin("p1").expect("should succeed"));
        assert_eq!(mgr.list_plugins().expect("should succeed").len(), 1);

        let err = mgr.register_plugin(
            "p1".to_string(),
            Box::new(MockPlugin::new("p1", PluginStatus::Active, fail)),
        );
        assert!(matches!(err, Err(PluginError::PluginAlreadyExists { .. })));
    }

    #[test]
    fn plugin_manager_start_all_rejects_uninitialized() {
        let mgr = PluginManager::new();
        let fail = Arc::new(AtomicBool::new(false));
        mgr.register_plugin(
            "u".to_string(),
            Box::new(MockPlugin::new("u", PluginStatus::Uninitialized, fail)),
        )
        .expect("should succeed");
        let mut mgr = mgr;
        let err = mgr.start_all().unwrap_err();
        assert!(matches!(err, PluginError::InitializationError { .. }));
    }

    #[test]
    fn plugin_manager_stop_all_and_unregister() {
        let mgr = PluginManager::new();
        let fail = Arc::new(AtomicBool::new(false));
        mgr.register_plugin(
            "a".to_string(),
            Box::new(MockPlugin::new("a", PluginStatus::Active, fail)),
        )
        .expect("should succeed");
        let mut mgr = mgr;
        mgr.stop_all().expect("should succeed");
        mgr.unregister_plugin("a").expect("should succeed");
        assert!(!mgr.has_plugin("a").expect("should succeed"));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn plugin_manager_unregister_stop_failure() {
        let mgr = PluginManager::new();
        let fail = Arc::new(AtomicBool::new(true));
        mgr.register_plugin(
            "bad".to_string(),
            Box::new(MockPlugin::new("bad", PluginStatus::Active, fail.clone())),
        )
        .expect("should succeed");
        let err = mgr.unregister_plugin("bad").unwrap_err();
        assert!(matches!(err, PluginError::InternalError { .. }));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn plugin_manager_stop_all_failure() {
        let mgr = PluginManager::new();
        let fail = Arc::new(AtomicBool::new(true));
        mgr.register_plugin(
            "bad".to_string(),
            Box::new(MockPlugin::new("bad", PluginStatus::Active, fail.clone())),
        )
        .expect("should succeed");
        let mut mgr = mgr;
        let err = mgr.stop_all().unwrap_err();
        assert!(matches!(err, PluginError::InternalError { .. }));
    }

    #[test]
    fn utils_get_version_and_create_context() {
        assert_eq!(utils::get_version(), crate::SDK_VERSION);
        let ctx = utils::create_default_context("pid".to_string());
        assert_eq!(ctx.plugin_id, "pid");
        assert!(utils::init_sdk("x".into()).is_ok());
    }

    #[test]
    fn utils_validate_plugin_info_ok() {
        let info = sample_info("id", PluginStatus::Active);
        utils::validate_plugin_info(&info).expect("should succeed");
    }

    #[test]
    fn utils_validate_plugin_info_errors() {
        let mut info = sample_info("id", PluginStatus::Active);
        info.id = String::new();
        assert!(utils::validate_plugin_info(&info).is_err());

        let mut info = sample_info("id", PluginStatus::Active);
        info.name = String::new();
        assert!(utils::validate_plugin_info(&info).is_err());

        let mut info = sample_info("id", PluginStatus::Active);
        info.version = String::new();
        assert!(utils::validate_plugin_info(&info).is_err());

        let mut info = sample_info("id", PluginStatus::Active);
        info.version = "not-semver".to_string();
        assert!(utils::validate_plugin_info(&info).is_err());
    }
}
