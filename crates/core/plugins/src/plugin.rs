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

/// Legacy Plugin metadata, will be deprecated in favor of IPluginMetadata
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
    pub fn new() -> Self {
        Self::Registered
    }

    /// Convert the status to a string
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            Self::Registered => "registered".to_string(),
            Self::Initialized => "initialized".to_string(),
            Self::Unloaded => "unloaded".to_string(),
            Self::Failed => "failed".to_string(),
        }
    }
}

impl Default for PluginStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy Plugin trait, will be deprecated in favor of IPlugin
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin ID
    fn id(&self) -> Uuid {
        self.metadata().id
    }

    /// Get the plugin metadata
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
