// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin system interfaces
//!
//! This module defines the shared interfaces for the plugin system.
//! These interfaces are used by multiple components in the Squirrel ecosystem.
//!
//! [`Plugin`] uses `impl Future<Output = _> + Send` for async methods so futures are
//! `Send` and can be forwarded from object-safe [`DynPlugin`].

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
pub trait Plugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    fn initialize(&self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Shutdown the plugin
    fn shutdown(&self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Check if plugin has a capability
    fn has_capability(&self, capability: &str) -> bool {
        self.metadata()
            .capabilities
            .contains(&capability.to_string())
    }
}

/// Object-safe projection of [`Plugin`] for heterogeneous registries.
#[async_trait]
pub trait DynPlugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;

    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
}

#[async_trait]
impl<T: Plugin + Send + Sync> DynPlugin for T {
    fn metadata(&self) -> &PluginMetadata {
        Plugin::metadata(self)
    }

    async fn initialize(&self) -> Result<()> {
        Plugin::initialize(self).await
    }

    async fn shutdown(&self) -> Result<()> {
        Plugin::shutdown(self).await
    }
}

/// Commands plugin interface
pub trait CommandsPlugin: Plugin {
    /// Get available commands
    fn get_available_commands(&self) -> Vec<CommandMetadata>;

    /// Get metadata for a specific command
    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata>;

    /// Execute a command
    fn execute_command(
        &self,
        command_id: &str,
        input: Value,
    ) -> impl std::future::Future<Output = Result<Value>> + Send;

    /// Get help text for a command
    fn get_command_help(&self, command_id: &str) -> Option<String>;
}

/// Plugin registry interface
///
/// The associated type [`PluginRegistry::PluginHandle`] is defined by each
/// implementation (for example an enum in `squirrel-plugins`).
pub trait PluginRegistry: Send + Sync {
    /// Concrete handle type used by this registry (often an enum for dispatch).
    type PluginHandle: Clone + Send + Sync + Debug;

    /// Register a plugin with the registry
    fn register_plugin<P: Plugin + 'static>(
        &self,
        plugin: Arc<P>,
    ) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Get a plugin by ID
    fn get_plugin(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Option<Self::PluginHandle>> + Send;

    /// Get a plugin by capability
    fn get_plugin_by_capability(
        &self,
        capability: &str,
    ) -> impl std::future::Future<Output = Option<Self::PluginHandle>> + Send;

    /// List all plugins
    fn list_plugins(&self) -> impl std::future::Future<Output = Vec<Self::PluginHandle>> + Send;
}

/// Plugin factory interface
///
/// This trait defines a factory for creating plugins
pub trait PluginFactory: Send + Sync {
    /// Concrete plugin type produced by this factory
    type Output: Plugin + 'static;

    /// Create a plugin
    ///
    /// # Errors
    ///
    /// Returns an error if plugin creation fails due to initialization issues,
    /// missing dependencies, or invalid configuration.
    fn create_plugin(&self) -> Result<Arc<Self::Output>>;

    /// Get the ID of the plugin that this factory creates
    fn plugin_id(&self) -> &str;
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Serde test round-trips use expect on known-good values"
)]
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
