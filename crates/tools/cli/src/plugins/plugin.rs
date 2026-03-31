// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin types and structures
//!
//! This module defines the core types for the plugin system.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::commands::registry::CommandRegistry;
use crate::plugins::error::PluginError;
use squirrel_commands::Command;

/// Metadata for a plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// The name of the plugin
    pub name: String,
    /// The version of the plugin
    pub version: String,
    /// Optional description of the plugin
    pub description: Option<String>,
    /// Optional author of the plugin
    pub author: Option<String>,
    /// Optional homepage URL
    pub homepage: Option<String>,
    /// Declared capabilities (from `plugin.toml`, empty if absent)
    pub capabilities: Vec<String>,
}

/// Status of a plugin
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is installed but not loaded
    Installed,
    /// Plugin is enabled and active
    Enabled,
    /// Plugin is disabled
    Disabled,
    /// Plugin failed to load with an error
    Failed(String),
    /// Other status with custom data (string only)
    #[cfg(test)]
    Custom(String),
}

/// Represents a plugin in the system
#[derive(Debug, Clone)]
pub struct PluginItem {
    /// Metadata about the plugin
    metadata: PluginMetadata,
    /// Path to the plugin file or directory
    path: PathBuf,
    /// Current status of the plugin
    status: PluginStatus,
}

impl PluginItem {
    /// Create a new plugin instance
    pub fn new(metadata: PluginMetadata, path: PathBuf, status: PluginStatus) -> Self {
        Self {
            metadata,
            path,
            status,
        }
    }

    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get the plugin path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the plugin status
    pub fn status(&self) -> &PluginStatus {
        &self.status
    }

    /// Set the plugin status
    pub fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

/// Plugin trait defining the interface for CLI plugins
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Get the plugin description
    fn description(&self) -> Option<&str>;

    /// Initialize the plugin
    ///
    /// This method is called when the plugin is loaded.
    /// It should perform any necessary setup.
    ///
    /// Expected state transition: Created -> Initialized
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization succeeds, or an error otherwise
    async fn initialize(&self) -> Result<(), PluginError>;

    /// Start the plugin
    ///
    /// This method is called after initialization to start the plugin.
    /// It should begin any background processing or services.
    ///
    /// Expected state transition: Initialized -> Started
    ///
    /// # Returns
    ///
    /// `Ok(())` if starting succeeds, or an error otherwise
    async fn start(&self) -> Result<(), PluginError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Register commands with the command registry
    ///
    /// This method is called after initialization to register
    /// any commands provided by the plugin.
    ///
    /// # Arguments
    ///
    /// * `registry` - The command registry to register commands with
    ///
    /// # Returns
    ///
    /// `Ok(())` if registration succeeds, or an error otherwise
    fn register_commands(&self, registry: &CommandRegistry) -> Result<(), PluginError>;

    /// Get the commands provided by this plugin
    ///
    /// # Returns
    ///
    /// A vector of command instances provided by this plugin
    fn commands(&self) -> Vec<Arc<dyn Command>>;

    /// Execute plugin-specific functionality
    ///
    /// This method allows plugins to perform custom operations
    /// beyond just registering commands.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments for the plugin operation
    ///
    /// # Returns
    ///
    /// `Ok(String)` with the result if execution succeeds, or an error otherwise
    async fn execute(&self, args: &[String]) -> Result<String, PluginError>;

    /// Stop the plugin
    ///
    /// This method is called to stop the plugin's operations.
    /// It should stop any background processing or services.
    ///
    /// Expected state transition: Started -> Stopped
    ///
    /// # Returns
    ///
    /// `Ok(())` if stopping succeeds, or an error otherwise
    async fn stop(&self) -> Result<(), PluginError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Clean up plugin resources
    ///
    /// This method is called when the plugin is being unloaded.
    /// It should clean up any resources allocated by the plugin.
    ///
    /// Expected state transition: Stopped -> Cleaned
    ///
    /// # Returns
    ///
    /// `Ok(())` if cleanup succeeds, or an error otherwise
    async fn cleanup(&self) -> Result<(), PluginError>;
}

/// A factory for creating plugins
pub trait PluginFactory: Send + Sync + 'static {
    /// Create a new plugin instance
    ///
    /// # Returns
    ///
    /// A boxed plugin instance
    fn create(&self) -> Result<Arc<dyn Plugin>, PluginError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_metadata() -> PluginMetadata {
        PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: Some("Test Author".to_string()),
            homepage: Some("https://example.com".to_string()),
            capabilities: vec![],
        }
    }

    // --- PluginMetadata tests ---
    #[test]
    fn test_plugin_metadata_creation() {
        let meta = sample_metadata();
        assert_eq!(meta.name, "test-plugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.description.as_deref(), Some("A test plugin"));
        assert_eq!(meta.author.as_deref(), Some("Test Author"));
        assert_eq!(meta.homepage.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_plugin_metadata_minimal() {
        let meta = PluginMetadata {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        assert_eq!(meta.name, "minimal");
        assert!(meta.description.is_none());
        assert!(meta.author.is_none());
    }

    #[test]
    fn test_plugin_metadata_clone() {
        let meta = sample_metadata();
        let cloned = meta.clone();
        assert_eq!(cloned.name, meta.name);
        assert_eq!(cloned.version, meta.version);
    }

    // --- PluginStatus tests ---
    #[test]
    fn test_plugin_status_installed() {
        let status = PluginStatus::Installed;
        assert_eq!(status, PluginStatus::Installed);
    }

    #[test]
    fn test_plugin_status_enabled() {
        let status = PluginStatus::Enabled;
        assert_eq!(status, PluginStatus::Enabled);
    }

    #[test]
    fn test_plugin_status_disabled() {
        let status = PluginStatus::Disabled;
        assert_eq!(status, PluginStatus::Disabled);
    }

    #[test]
    fn test_plugin_status_failed() {
        let status = PluginStatus::Failed("load error".to_string());
        assert_eq!(status, PluginStatus::Failed("load error".to_string()));
        assert_ne!(status, PluginStatus::Enabled);
    }

    #[test]
    fn test_plugin_status_custom() {
        let status = PluginStatus::Custom("testing".to_string());
        assert_eq!(status, PluginStatus::Custom("testing".to_string()));
    }

    #[test]
    fn test_plugin_status_clone() {
        let status = PluginStatus::Failed("err".to_string());
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    // --- PluginItem tests ---
    #[test]
    fn test_plugin_item_new() {
        let meta = sample_metadata();
        let path = PathBuf::from("/plugins/test.so");
        let item = PluginItem::new(meta, path, PluginStatus::Installed);

        assert_eq!(item.metadata().name, "test-plugin");
        assert_eq!(item.path(), Path::new("/plugins/test.so"));
        assert_eq!(item.status(), &PluginStatus::Installed);
    }

    #[test]
    fn test_plugin_item_set_status() {
        let meta = sample_metadata();
        let path = PathBuf::from("/plugins/test.so");
        let mut item = PluginItem::new(meta, path, PluginStatus::Installed);

        assert_eq!(item.status(), &PluginStatus::Installed);
        item.set_status(PluginStatus::Enabled);
        assert_eq!(item.status(), &PluginStatus::Enabled);
    }

    #[test]
    fn test_plugin_item_status_transitions() {
        let meta = sample_metadata();
        let path = PathBuf::from("/plugins/test.so");
        let mut item = PluginItem::new(meta, path, PluginStatus::Installed);

        // Installed -> Enabled -> Disabled -> Failed
        item.set_status(PluginStatus::Enabled);
        assert_eq!(item.status(), &PluginStatus::Enabled);

        item.set_status(PluginStatus::Disabled);
        assert_eq!(item.status(), &PluginStatus::Disabled);

        item.set_status(PluginStatus::Failed("crash".to_string()));
        assert_eq!(item.status(), &PluginStatus::Failed("crash".to_string()));
    }

    #[test]
    fn test_plugin_item_clone() {
        let meta = sample_metadata();
        let path = PathBuf::from("/plugins/test.so");
        let item = PluginItem::new(meta, path, PluginStatus::Enabled);
        let cloned = item.clone();

        assert_eq!(cloned.metadata().name, item.metadata().name);
        assert_eq!(cloned.path(), item.path());
        assert_eq!(cloned.status(), item.status());
    }

    #[test]
    fn test_plugin_item_metadata_accessor() {
        let meta = PluginMetadata {
            name: "my-plugin".to_string(),
            version: "2.0.0".to_string(),
            description: Some("My awesome plugin".to_string()),
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let item = PluginItem::new(meta, PathBuf::from("/opt/plugins"), PluginStatus::Installed);

        let m = item.metadata();
        assert_eq!(m.name, "my-plugin");
        assert_eq!(m.version, "2.0.0");
        assert!(m.description.is_some());
    }
}
