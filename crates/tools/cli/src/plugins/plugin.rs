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
