// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin trait and related types
//!
//! This module defines the core plugin trait and related types.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::fmt::Debug;
use uuid::Uuid;

/// Legacy Plugin metadata - DEPRECATED
///
/// Use `squirrel_interfaces::plugins::PluginMetadata` instead.
/// This will be removed in a future version.
#[deprecated(
    since = "2.0.0",
    note = "Use squirrel_interfaces::plugins::PluginMetadata instead"
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: Uuid,

    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<Uuid>,
}

impl PluginMetadata {
    /// Create a new plugin metadata
    #[must_use]
    pub fn new(name: &str, version: &str, description: &str, author: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add a capability to the plugin
    #[must_use]
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }

    /// Add a dependency to the plugin
    #[must_use]
    pub fn with_dependency(mut self, dependency: Uuid) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Default Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Default plugin implementation".to_string(),
            author: "System".to_string(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

/// Plugin status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Copy)]
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

impl std::fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registered => write!(f, "registered"),
            Self::Initialized => write!(f, "initialized"),
            Self::Unloaded => write!(f, "unloaded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl Default for PluginStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy Plugin trait, will be deprecated in favor of `IPlugin`
///
/// NOTE: This trait uses `async_trait` because it is used as a trait object (`dyn Plugin`)
/// throughout the codebase. Native async traits are not compatible with trait objects.
/// This is a legitimate use case for keeping `async_trait` - see Phase 4 migration notes.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin ID
    fn id(&self) -> Uuid {
        self.metadata().id
    }

    /// Get the plugin metadata
    // Backward compatibility: PluginMetadata during migration to squirrel_interfaces
    #[allow(
        deprecated,
        reason = "backward compat: PluginMetadata during migration to squirrel_interfaces"
    )]
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;

    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;

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

/// Web plugin extension trait
#[expect(dead_code, reason = "Reserved for future web plugin system")]
#[async_trait]
pub trait WebPluginExt: Plugin {
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
    #![allow(deprecated)]

    use super::*;
    use uuid::Uuid;

    #[test]
    fn plugin_metadata_new_default_with_capability_dependency() {
        let m = PluginMetadata::new("n", "1.0.0", "d", "a")
            .with_capability("web")
            .with_dependency(Uuid::nil());
        assert_eq!(m.name, "n");
        assert_eq!(m.capabilities, vec!["web"]);
        assert_eq!(m.dependencies.len(), 1);

        let def = PluginMetadata::default();
        assert_eq!(def.name, "Default Plugin");
        assert!(def.capabilities.is_empty());
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
