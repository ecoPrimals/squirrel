// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// NOTE: Using deprecated plugin::PluginMetadata until interfaces crate stabilizes
// The interfaces version lacks dependency tracking. See: PLUGIN_METADATA_MIGRATION_PLAN.md
#![allow(
    deprecated,
    reason = "Uses deprecated plugin::PluginMetadata until interfaces crate stabilizes (see module note)"
)]

//! `PluginV2` trait with improved thread safety
//!
//! This module provides a new version of the Plugin trait that uses callbacks
//! instead of direct adapter references to avoid potential Send/Sync issues.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;
use uuid::Uuid;

use crate::plugin::{Plugin, PluginMetadata, WebEndpoint};

// Note: async_trait still needed for Plugin trait (used as dyn Plugin),
// but PluginV2 and WebPluginExtV2 use native async (no trait objects)

/// Callback for logging
pub type LogCallback = Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>;
/// Callback for getting plugin by ID
pub type GetPluginCallback = Box<dyn Fn(Uuid) -> Result<Arc<dyn Plugin>> + Send + Sync>;
/// Callback for getting plugin by name
pub type GetPluginByNameCallback = Box<dyn Fn(&str) -> Result<Arc<dyn Plugin>> + Send + Sync>;
/// Callback for listing plugins
pub type ListPluginsCallback = Box<dyn Fn() -> Result<Vec<Arc<dyn Plugin>>> + Send + Sync>;
/// Callback for getting config
pub type GetConfigCallback = Box<dyn Fn(&str) -> Result<Value> + Send + Sync>;
/// Callback for setting config
pub type SetConfigCallback = Box<dyn Fn(&str, Value) -> Result<()> + Send + Sync>;
/// Callback for persisting state
pub type PersistStateCallback = Box<dyn Fn(Uuid, &str, Value) -> Result<()> + Send + Sync>;
/// Callback for loading state
pub type LoadStateCallback = Box<dyn Fn(Uuid, &str) -> Result<Value> + Send + Sync>;
/// Callback for permission check
pub type CheckPermissionCallback = Box<dyn Fn(&str, Uuid) -> Result<bool> + Send + Sync>;

/// Callbacks for `PluginV2`
#[derive(Default)]
pub struct PluginCallbacks {
    /// Log a message
    pub log: Option<LogCallback>,
    /// Access the plugin registry
    pub get_plugin: Option<GetPluginCallback>,
    /// Get plugin by name
    pub get_plugin_by_name: Option<GetPluginByNameCallback>,
    /// List all plugins
    pub list_plugins: Option<ListPluginsCallback>,
    /// Get configuration value
    pub get_config: Option<GetConfigCallback>,
    /// Set configuration value
    pub set_config: Option<SetConfigCallback>,
    /// Persist plugin state
    pub persist_state: Option<PersistStateCallback>,
    /// Load plugin state
    pub load_state: Option<LoadStateCallback>,
    /// Security check
    pub check_permission: Option<CheckPermissionCallback>,
}

/// V2 version of the Plugin trait with improved thread safety
///
/// This version provides explicit Send + Sync bounds and uses callbacks
/// instead of direct adapter references to avoid potential Send/Sync issues.
pub trait PluginV2: Send + Sync + std::fmt::Debug {
    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    fn initialize(&self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Shutdown the plugin
    fn shutdown(&self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Convert the plugin to Any
    fn as_any(&self) -> &dyn Any;

    /// Register callbacks for plugin interaction with manager
    fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
        // Default empty implementation
        let _ = callbacks; // Suppress unused variable warning
    }
}

/// Web plugin extension trait for V2 plugins
#[expect(dead_code, reason = "Reserved for future web plugin V2 system")]
pub trait WebPluginExtV2: PluginV2 {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;

    /// Handle web endpoint request
    fn handle_web_endpoint(
        &self,
        endpoint: &WebEndpoint,
        data: Option<Value>,
    ) -> impl std::future::Future<Output = Result<Value>> + Send;
}

/// Helper struct to adapt `PluginV2` to Plugin for backward compatibility
#[derive(Debug)]
pub struct PluginWrapper<T: PluginV2> {
    inner: T,
}

impl<T: PluginV2> PluginWrapper<T> {
    /// Create a new `PluginWrapper` with the given `PluginV2` implementation
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T: PluginV2 + 'static> Plugin for PluginWrapper<T> {
    fn metadata(&self) -> &PluginMetadata {
        self.inner.metadata()
    }

    async fn initialize(&self) -> Result<()> {
        self.inner.initialize().await
    }

    async fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().await
    }

    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
}

/// Helper function to adapt a `PluginV2` to Plugin (used in tests)
#[allow(dead_code)]
pub fn adapt_plugin_v2<T: PluginV2 + 'static>(plugin: T) -> Arc<dyn Plugin> {
    Arc::new(PluginWrapper::new(plugin))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple example implementation of the PluginV2 trait
    struct ExamplePluginV2 {
        metadata: PluginMetadata,
        log: Option<LogCallback>,
    }

    impl std::fmt::Debug for ExamplePluginV2 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ExamplePluginV2")
                .field("metadata", &self.metadata)
                .field(
                    "log",
                    &if self.log.is_some() {
                        "Some(log_fn)"
                    } else {
                        "None"
                    },
                )
                .finish()
        }
    }

    impl ExamplePluginV2 {
        fn new(name: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(name, "1.0.0", "Example plugin", "Test Author"),
                log: None,
            }
        }

        fn log(&self, level: &str, message: &str) {
            if let Some(log) = &self.log {
                let _ = log(level, message);
            }
        }
    }

    impl PluginV2 for ExamplePluginV2 {
        #[expect(
            deprecated,
            reason = "Tests deprecated path for backward compatibility"
        )]
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&self) -> impl std::future::Future<Output = Result<()>> + Send {
            self.log("info", "ExamplePluginV2 initialized");
            async move { Ok(()) }
        }

        fn shutdown(&self) -> impl std::future::Future<Output = Result<()>> + Send {
            self.log("info", "ExamplePluginV2 shutdown");
            async move { Ok(()) }
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
            self.log = callbacks.log;
        }
    }

    #[tokio::test]
    async fn test_plugin_v2_adapter() {
        // Create a V2 plugin
        let mut plugin_v2 = ExamplePluginV2::new("example-plugin");

        // Set up callbacks
        let callbacks = PluginCallbacks {
            log: Some(Box::new(|level, message| {
                println!("[{level}] {message}");
                Ok(())
            })),
            ..Default::default()
        };

        // Register callbacks
        plugin_v2.register_callbacks(callbacks);

        // Adapt to Plugin trait
        let plugin: Arc<dyn Plugin> = adapt_plugin_v2(plugin_v2);

        // Test metadata
        assert_eq!(plugin.metadata().name, "example-plugin");

        // Test methods
        assert!(plugin.initialize().await.is_ok());
        assert!(plugin.shutdown().await.is_ok());
    }
}
