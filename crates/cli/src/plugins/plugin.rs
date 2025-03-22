//! Plugin types and structures
//!
//! This module defines the core types for the plugin system.

use std::path::{Path, PathBuf};

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