//! Core plugin infrastructure
//!
//! This module defines the core traits and types for the plugin system.

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::PluginState;
use crate::Result;

/// Plugin metadata containing identification and capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: Uuid,
    
    /// Plugin name (unique across all plugins)
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: String,
    
    /// Plugin capabilities (used for discovery and dependency resolution)
    pub capabilities: Vec<String>,
    
    /// Plugin dependencies (capabilities required by this plugin)
    pub dependencies: Vec<String>,
    
    /// Plugin type identifier
    pub plugin_type: String,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
        plugin_type: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            plugin_type: plugin_type.into(),
        }
    }
    
    /// Add a capability to the plugin
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    /// Add a dependency to the plugin
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }
}

/// Plugin status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is registered but not loaded
    Registered,
    
    /// Plugin is loaded and initialized
    Active,
    
    /// Plugin is temporarily disabled
    Disabled,
    
    /// Plugin has failed and needs attention
    Failed,
    
    /// Plugin is initializing
    Initializing,
    
    /// Plugin is in the process of shutting down
    ShuttingDown,
    
    /// Plugin is in the process of stopping (transitional state)
    Stopping,
    
    /// Plugin is unloaded
    Unloaded,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync + Any + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    fn initialize(&self) -> BoxFuture<'_, Result<()>>;
    
    /// Shutdown the plugin
    fn shutdown(&self) -> BoxFuture<'_, Result<()>>;
    
    /// Get plugin state
    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>>;
    
    /// Set plugin state
    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>>;
    
    /// Get plugin status (convenience method for tests)
    fn get_status(&self) -> BoxFuture<'_, PluginStatus> {
        Box::pin(async { PluginStatus::Active })
    }
    
    /// Cast the plugin to Any
    fn as_any(&self) -> &dyn Any;
    
    /// Clone as a boxed Plugin trait object
    fn clone_box(&self) -> Box<dyn Plugin>;
}

/// Manual implementation of Clone for Box<dyn Plugin>
impl Clone for Box<dyn Plugin> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
} 