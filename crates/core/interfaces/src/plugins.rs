// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Plugin system interfaces
//!
//! This module defines the shared interfaces for the plugin system.
//! These interfaces are used by multiple components in the Squirrel ecosystem.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

/// Metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version string
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl PluginMetadata {
    /// Create a new plugin metadata
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            name: id_str,
            version: version.into(),
            description: description.into(),
            author: author.into(),
            capabilities: Vec::new(),
        }
    }

    /// Add a capability to the plugin
    #[must_use]
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
}

/// Metadata about a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Unique command identifier
    pub id: String,

    /// Human-readable command name
    pub name: String,

    /// Command description
    pub description: String,

    /// JSON schema describing expected input
    pub input_schema: Value,

    /// JSON schema describing expected output
    pub output_schema: Value,

    /// Permissions required to execute this command
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Execution context for plugin operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginExecutionContext {
    /// User ID if authenticated
    pub user_id: Option<String>,

    /// User roles if applicable
    pub roles: Vec<String>,

    /// Request context data
    pub context: HashMap<String, Value>,
}

/// Base trait for all plugins
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Check if plugin has a capability
    fn has_capability(&self, capability: &str) -> bool {
        self.metadata()
            .capabilities
            .contains(&capability.to_string())
    }
}

/// Commands plugin interface
#[async_trait]
pub trait CommandsPlugin: Plugin {
    /// Get available commands
    fn get_available_commands(&self) -> Vec<CommandMetadata>;

    /// Get metadata for a specific command
    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata>;

    /// Execute a command
    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value>;

    /// Get help text for a command
    fn get_command_help(&self, command_id: &str) -> Option<String>;
}

/// Plugin registry interface
///
/// This trait defines the core interface for a plugin registry,
/// allowing plugins to be registered and retrieved without creating
/// circular dependencies between crates.
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin with the registry
    async fn register_plugin<P: Plugin + 'static>(&self, plugin: Arc<P>) -> Result<String>;

    /// Get a plugin by ID
    async fn get_plugin(&self, id: &str) -> Option<Arc<dyn Plugin>>;

    /// Get a plugin by capability
    async fn get_plugin_by_capability(&self, capability: &str) -> Option<Arc<dyn Plugin>>;

    /// Get a plugin by type and capability
    async fn get_plugin_by_type_and_capability<T: Plugin + ?Sized + 'static>(
        &self,
        capability: &str,
    ) -> Option<Arc<T>>;

    /// List all plugins
    async fn list_plugins(&self) -> Vec<Arc<dyn Plugin>>;
}

/// Plugin factory interface
///
/// This trait defines a factory for creating plugins
pub trait PluginFactory: Send + Sync {
    /// Create a plugin
    ///
    /// # Errors
    ///
    /// Returns an error if plugin creation fails due to initialization issues,
    /// missing dependencies, or invalid configuration.
    fn create_plugin(&self) -> Result<Arc<dyn Plugin>>;

    /// Get the ID of the plugin that this factory creates
    fn plugin_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_new() {
        let meta = PluginMetadata::new("test-plugin", "1.0.0", "A test plugin", "TestAuthor");
        assert_eq!(meta.id, "test-plugin");
        assert_eq!(meta.name, "test-plugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.description, "A test plugin");
        assert_eq!(meta.author, "TestAuthor");
        assert!(meta.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_metadata_with_capability() {
        let meta = PluginMetadata::new("test", "1.0", "desc", "auth")
            .with_capability("ai.inference")
            .with_capability("storage.read");
        assert_eq!(meta.capabilities.len(), 2);
        assert_eq!(meta.capabilities[0], "ai.inference");
        assert_eq!(meta.capabilities[1], "storage.read");
    }

    #[test]
    fn test_plugin_metadata_serde() {
        let meta = PluginMetadata::new("test", "1.0.0", "desc", "author").with_capability("cap1");
        let json = serde_json::to_string(&meta).expect("serialize");
        let deser: PluginMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "test");
        assert_eq!(deser.capabilities, vec!["cap1"]);
    }

    #[test]
    fn test_command_metadata_serde() {
        let meta = CommandMetadata {
            id: "cmd1".to_string(),
            name: "Test Command".to_string(),
            description: "A test command".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "string"}),
            permissions: vec!["read".to_string()],
        };
        let json = serde_json::to_string(&meta).expect("serialize");
        let deser: CommandMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "cmd1");
        assert_eq!(deser.permissions, vec!["read"]);
    }

    #[test]
    fn test_plugin_execution_context_default() {
        let ctx = PluginExecutionContext::default();
        assert!(ctx.user_id.is_none());
        assert!(ctx.roles.is_empty());
        assert!(ctx.context.is_empty());
    }

    #[test]
    fn test_plugin_execution_context_serde() {
        let ctx = PluginExecutionContext {
            user_id: Some("user1".to_string()),
            roles: vec!["admin".to_string()],
            context: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        };
        let json = serde_json::to_string(&ctx).expect("serialize");
        let deser: PluginExecutionContext = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.user_id, Some("user1".to_string()));
        assert_eq!(deser.roles, vec!["admin"]);
    }
}
