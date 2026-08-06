// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin trait and related types
//!
//! This module defines the core plugin trait and related types.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
pub use squirrel_interfaces::plugins::PluginMetadata;
use strum::Display;

/// Plugin status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PluginStatus {
    /// Plugin is registered but not initialized
    Registered,

    /// Plugin is initialized and ready for use
    Initialized,

    /// Plugin is unloaded
    Unloaded,

    /// Plugin failed to initialize
    Failed,
}

impl PluginStatus {
    /// Create a new registered status
    #[must_use]
    pub const fn new() -> Self {
        Self::Registered
    }
}

impl Default for PluginStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy plugin trait (object-safe via boxed futures).
///
/// Async methods return `Pin<Box<dyn Future<...>>>` so `dyn Plugin` works without the
/// `async_trait` crate (native `async fn` in traits is not object-safe).
pub trait Plugin: Send + Sync {
    /// Get the plugin ID
    fn id(&self) -> &str {
        self.metadata().id.as_str()
    }

    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    fn initialize(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Shutdown the plugin
    fn shutdown(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Convert the plugin to Any
    fn as_any(&self) -> &dyn Any;
}

/// A simplified web plugin endpoint for use with the trait below
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Path to the endpoint
    pub path: String,

    /// HTTP method
    pub method: String,

    /// Required permissions
    pub permissions: Vec<String>,
}

/// Web plugin extension trait (Phase 2 — WASM sandbox)
#[expect(dead_code, reason = "Phase 2 web plugin system; HelloWorldPlugin implements it for testing")]
pub trait WebPluginExt: Plugin + Send + Sync {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;

    /// Handle web endpoint request
    async fn handle_web_endpoint(
        &self,
        endpoint: &WebEndpoint,
        data: Option<Value>,
    ) -> Result<Value>;
}

// Re-export the CommandsPlugin trait from interfaces for convenience

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_metadata_new_default_with_capability_dependency() {
        let m = PluginMetadata::new("plugin-id", "1.0.0", "d", "a")
            .with_name("n")
            .with_capability("web")
            .with_dependency("dep-id");
        assert_eq!(m.id, "plugin-id");
        assert_eq!(m.name, "n");
        assert_eq!(m.capabilities, vec!["web"]);
        assert_eq!(m.dependencies, vec!["dep-id"]);

        let unnamed = PluginMetadata::new("default-plugin", "0.1.0", "desc", "System");
        assert_eq!(unnamed.name, "default-plugin");
        assert!(unnamed.capabilities.is_empty());
    }

    #[test]
    fn plugin_metadata_serde_roundtrip() {
        let m = PluginMetadata::new("x", "0.1.0", "desc", "auth");
        let j = serde_json::to_string(&m).expect("should succeed");
        let back: PluginMetadata = serde_json::from_str(&j).expect("should succeed");
        assert_eq!(back.name, m.name);
        assert_eq!(back.id, m.id);
    }

    #[test]
    fn plugin_status_display_default() {
        assert_eq!(PluginStatus::Registered.to_string(), "registered");
        assert_eq!(PluginStatus::Initialized.to_string(), "initialized");
        assert_eq!(PluginStatus::Unloaded.to_string(), "unloaded");
        assert_eq!(PluginStatus::Failed.to_string(), "failed");
        assert_eq!(PluginStatus::default(), PluginStatus::Registered);
        assert_eq!(PluginStatus::new(), PluginStatus::Registered);
    }

    #[test]
    fn plugin_status_serde_roundtrip() {
        for s in [
            PluginStatus::Registered,
            PluginStatus::Initialized,
            PluginStatus::Unloaded,
            PluginStatus::Failed,
        ] {
            let j = serde_json::to_string(&s).expect("should succeed");
            let back: PluginStatus = serde_json::from_str(&j).expect("should succeed");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn legacy_web_endpoint_serde_roundtrip() {
        let e = WebEndpoint {
            path: "/a".to_string(),
            method: "GET".to_string(),
            permissions: vec!["p".to_string()],
        };
        let j = serde_json::to_string(&e).expect("should succeed");
        let back: WebEndpoint = serde_json::from_str(&j).expect("should succeed");
        assert_eq!(back.path, e.path);
        assert_eq!(back.method, e.method);
    }
}
