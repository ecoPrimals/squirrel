// Plugin system interfaces
//
// This module defines the interfaces for the plugin system integration

use std::fmt::Debug;
use std::sync::Arc;
use std::any::Any;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::versioning::VersionRequirement;

/// Status of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is registered but not initialized
    Registered,
    /// Plugin is in the process of initializing
    Initializing,
    /// Plugin is running
    Running,
    /// Plugin is in the process of shutting down
    ShuttingDown,
    /// Plugin is shut down
    ShutDown,
    /// Plugin encountered an error
    Error,
}

/// Metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin ID
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin status
    pub status: PluginStatus,
}

/// Interface for a plugin
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    /// Get the plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shut down the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Returns a reference to self as a Any trait object for downcasting
    fn as_any(&self) -> &dyn Any {
        &()  // Default implementation returns an empty tuple reference
    }
}

/// Interface for an MCP plugin
#[async_trait]
pub trait McpPlugin: Plugin {
    /// Handle a message from MCP
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Validate that a message conforms to the expected schema
    fn validate_message_schema(&self, message: &serde_json::Value) -> Result<()>;
    
    /// Get the protocol version requirements for this plugin
    /// 
    /// This allows the plugin to specify what protocol versions it's compatible with.
    /// Returns a set of requirements such as ">=1.0.0, <2.0.0" using SemVer syntax.
    fn protocol_version_requirements(&self) -> VersionRequirement {
        // Default to requiring version 1.0.0 or later, but below 2.0.0
        VersionRequirement::new(">=1.0.0, <2.0.0")
    }
}

/// Interface for the plugin manager
#[async_trait]
pub trait PluginManagerInterface: Send + Sync + Debug {
    /// Register a plugin with the manager
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
    
    /// Get a plugin by ID
    async fn get_plugin_by_id(&self, plugin_id: Uuid) -> Result<Option<Arc<dyn Plugin>>>;
    
    /// Execute a plugin with the given message
    async fn execute_mcp_plugin(&self, plugin_id: Uuid, message: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Update the status of a plugin
    async fn update_plugin_status(&self, plugin_id: Uuid, status: PluginStatus) -> Result<()>;
} 