//! Core plugin functionality
//!
//! This module defines the core plugin traits and structures that are
//! used across all plugin types.

use std::fmt::Debug;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Status of a plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is registered but not initialized
    Registered,
    /// Plugin is initializing
    Initializing,
    /// Plugin is initialized and ready
    Ready,
    /// Plugin has encountered an error
    Error,
    /// Plugin is shutting down
    ShuttingDown,
    /// Plugin is disabled
    Disabled,
}

/// Plugin metadata
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
    pub capabilities: Vec<String>,
    
    /// Plugin dependencies (IDs of plugins this plugin depends on)
    pub dependencies: Vec<Uuid>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add a capability to the plugin
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add a dependency to the plugin
    pub fn with_dependency(mut self, dependency: Uuid) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Base plugin trait
///
/// All plugins must implement this trait.
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Get the current plugin status
    async fn status(&self) -> PluginStatus;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Plugin feature check
    fn has_feature(&self, feature: &str) -> bool {
        self.metadata().capabilities.contains(&feature.to_string())
    }
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T>;

/// Type for plugin reference
pub type PluginRef<T> = Arc<T>;

/// Helper struct for plugin state
#[derive(Debug)]
pub struct PluginState {
    /// Plugin status
    pub status: RwLock<PluginStatus>,
}

impl PluginState {
    /// Create a new plugin state
    pub fn new() -> Self {
        Self {
            status: RwLock::new(PluginStatus::Registered),
        }
    }

    /// Get the current status
    pub async fn status(&self) -> PluginStatus {
        *self.status.read().await
    }

    /// Set the plugin status
    pub async fn set_status(&self, status: PluginStatus) {
        let mut status_lock = self.status.write().await;
        *status_lock = status;
    }
}

impl Default for PluginState {
    fn default() -> Self {
        Self::new()
    }
} 