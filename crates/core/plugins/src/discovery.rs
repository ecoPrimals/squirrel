//! Plugin discovery
//!
//! This module provides functionality for discovering and loading plugins.

#![allow(deprecated)] // Allow use of plugin::PluginMetadata during migration to squirrel_interfaces

use std::any::Any;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;
use uuid::Uuid;

use crate::plugin::{Plugin, PluginMetadata};
use crate::PluginError;

/// Plugin manifest format
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Manifest structure for plugin discovery system
pub struct PluginManifest {
    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin entry point
    pub entry_point: String,

    /// Plugin type
    pub plugin_type: String,

    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Plugin capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl PluginManifest {
    /// Convert to plugin metadata
    #[must_use]
    #[allow(deprecated)]
    pub fn to_metadata(&self) -> PluginMetadata {
        let mut metadata =
            PluginMetadata::new(&self.name, &self.version, &self.description, &self.author);

        // Add capabilities
        for capability in &self.capabilities {
            metadata = metadata.with_capability(capability);
        }

        // Add dependencies - in a real implementation, we'd resolve
        // names to UUIDs, but for now we'll just create dummy UUIDs
        for _ in &self.dependencies {
            // Create a random UUID for demonstration
            let dependency_id = Uuid::new_v4();
            metadata = metadata.with_dependency(dependency_id);
        }

        metadata
    }
}

/// Plugin loader trait for loading plugins
#[async_trait]
pub trait PluginLoader: Send + Sync {
    /// Load a plugin from a manifest
    async fn load_plugin(&self, manifest: &PluginManifest, path: &Path) -> Result<Arc<dyn Plugin>>;
}

/// Plugin discovery trait
#[async_trait]
pub trait PluginDiscovery: Send + Sync {
    /// Discover plugins in a directory
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<Arc<dyn Plugin>>>;
}

/// File-based plugin discovery
#[allow(dead_code)] // Infrastructure for file-based plugin discovery
#[derive(Debug)]
pub struct FilePluginDiscovery<L> {
    /// Plugin loader
    loader: L,
}

impl<L: PluginLoader> FilePluginDiscovery<L> {
    /// Create new file-based plugin discovery
    #[allow(dead_code)] // Constructor for FilePluginDiscovery
    pub const fn new(loader: L) -> Self {
        Self { loader }
    }
}

#[async_trait]
impl<L: PluginLoader + Send + Sync> PluginDiscovery for FilePluginDiscovery<L> {
    /// Discover plugins in a directory
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<Arc<dyn Plugin>>> {
        // Ensure directory exists
        if !directory.exists() {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Plugin directory does not exist: {directory:?}"),
            );
            return Err(PluginError::IoError(err).into());
        }

        let mut plugins = Vec::new();

        // Read directory entries
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip non-directories
            if !path.is_dir() {
                continue;
            }

            // Check for manifest file
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            // Read and parse manifest
            let manifest_content = fs::read_to_string(&manifest_path).await?;
            let manifest: PluginManifest =
                serde_json::from_str(&manifest_content).map_err(PluginError::SerializationError)?;

            // Load plugin
            let plugin = self.loader.load_plugin(&manifest, &path).await?;
            plugins.push(plugin);
        }

        Ok(plugins)
    }
}

/// Create a placeholder plugin
#[allow(deprecated)] // Uses deprecated plugin::PluginMetadata during migration
pub fn create_placeholder_plugin(metadata: PluginMetadata) -> Arc<dyn Plugin> {
    Arc::new(PlaceholderPlugin { metadata })
}

/// A placeholder plugin implementation
#[allow(dead_code)] // Placeholder for plugin template system
#[allow(deprecated)] // Uses deprecated plugin::PluginMetadata during migration
#[derive(Debug, Clone)]
struct PlaceholderPlugin {
    metadata: PluginMetadata,
}

#[async_trait]
#[allow(deprecated)] // Uses deprecated plugin::PluginMetadata during migration
impl Plugin for PlaceholderPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Default implementation of plugin discovery
#[derive(Debug, Clone)]
pub struct DefaultPluginDiscovery {
    /// Plugin type
    pub plugin_type: String,
    /// Plugin author
    pub author: String,
}

#[async_trait]
impl PluginDiscovery for DefaultPluginDiscovery {
    /// Discover plugins in a directory
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<Arc<dyn Plugin>>> {
        if !directory.exists() {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Plugin directory does not exist: {directory:?}"),
            );
            return Err(PluginError::IoError(err).into());
        }

        // In a real implementation, this would load plugins from the directory
        // For now, just return an empty vector
        Ok(Vec::new())
    }
}

impl Default for DefaultPluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPluginDiscovery {
    /// Create a new plugin discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugin_type: String::from("default"),
            author: String::from("system"),
        }
    }

    /// Load a plugin from a path
    #[allow(deprecated)] // Uses deprecated plugin::PluginMetadata during migration
    pub async fn load_plugin(&self, _path: &Path) -> Result<Arc<dyn Plugin>> {
        // In a real implementation, this would load the plugin from the path
        // For now, just return a placeholder plugin
        let _metadata =
            PluginMetadata::new("Plugin at path", "0.1.0", "A placeholder plugin", "System");

        // Just return the placeholder plugin
        Ok(create_placeholder_plugin(_metadata))
    }
}

/// Default plugin loader implementation
#[allow(dead_code)] // Infrastructure for default plugin loading
#[derive(Debug, Copy, Clone)]
pub struct DefaultPluginLoader;

#[async_trait]
#[allow(deprecated)] // Uses deprecated plugin::PluginMetadata during migration
impl PluginLoader for DefaultPluginLoader {
    /// Load a plugin from a manifest
    async fn load_plugin(
        &self,
        manifest: &PluginManifest,
        _path: &Path,
    ) -> Result<Arc<dyn Plugin>> {
        // In a real implementation, this would load the plugin from the manifest and path
        // For now, just return a placeholder plugin with metadata from the manifest
        let metadata = manifest.to_metadata();
        Ok(create_placeholder_plugin(metadata))
    }
}
