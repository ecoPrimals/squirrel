// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Plugin discovery mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin discovery
//!
//! This module provides the plugin discovery and loading architecture:
//!
//! - **`PluginManifest`**: Deserializable manifest format (manifest.json) describing plugin metadata,
//!   entry point, capabilities, and dependencies.
//! - **`PluginLoader`**: Async trait for loading a plugin from a manifest and path. Implementations
//!   may perform dynamic loading (e.g. via `libloading`) or other strategies.
//! - **`PluginDiscovery`**: Async trait for discovering plugins in a directory (e.g. scanning for
//!   manifest.json files).
//! - **`FilePluginDiscovery`**: Discovers plugins by scanning directories for manifest.json and
//!   delegates loading to a `PluginLoader`.
//! - **`DefaultPluginLoader`** / **`DefaultPluginDiscovery`**: Default implementations. Native
//!   dynamic library loading is not compiled in here; load attempts return structured
//!   [`PluginError::LoadError`] with discovery hints (other primals expose plugins via IPC).

// Backward compatibility: Uses deprecated plugin::PluginMetadata during migration to squirrel_interfaces
#![allow(deprecated)]

use std::path::Path;
use std::sync::Arc;

use std::any::Any;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;
use tracing::{debug, info};
#[cfg(any(test, feature = "testing"))]
use uuid::Uuid;

use crate::PluginError;
use crate::plugin::{Plugin, PluginMetadata};

/// Plugin manifest format
#[derive(Debug, Deserialize)]
#[allow(
    dead_code,
    reason = "Serde-populated manifest schema; fields read in tests and to_metadata (testing feature)"
)]
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
    /// Convert to plugin metadata (used by tests and testing feature)
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    #[expect(
        deprecated,
        reason = "backward compat: PluginMetadata during migration"
    )]
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
#[derive(Debug)]
pub struct FilePluginDiscovery<L> {
    /// Plugin loader
    loader: L,
}

impl<L: PluginLoader> FilePluginDiscovery<L> {
    /// Create new file-based plugin discovery
    #[expect(dead_code, reason = "Constructor for FilePluginDiscovery")]
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
                format!("Plugin directory does not exist: {}", directory.display()),
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

/// Build a no-op plugin from manifest metadata (null object pattern).
///
/// Used when a manifest is present but this primal does not host native plugin code: the plugin
/// entry is a documented no-op; real implementations are discovered on other primals via IPC.
#[expect(
    deprecated,
    reason = "backward compat: PluginMetadata during migration"
)]
#[must_use]
pub fn create_noop_plugin(metadata: PluginMetadata) -> Arc<dyn Plugin> {
    Arc::new(NoOpPlugin { metadata })
}

/// No-op plugin: satisfies the [`Plugin`] contract without side effects; logs lifecycle for observability.
#[expect(
    deprecated,
    reason = "backward compat: PluginMetadata during migration"
)]
#[derive(Debug, Clone)]
pub struct NoOpPlugin {
    metadata: PluginMetadata,
}

#[async_trait]
#[expect(
    deprecated,
    reason = "backward compat: PluginMetadata during migration"
)]
impl Plugin for NoOpPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        info!(
            plugin_id = %self.metadata.id,
            plugin_name = %self.metadata.name,
            "NoOpPlugin initialize (no-op); no native code loaded on this primal"
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!(
            plugin_id = %self.metadata.id,
            plugin_name = %self.metadata.name,
            "NoOpPlugin shutdown (no-op)"
        );
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
                format!("Plugin directory does not exist: {}", directory.display()),
            );
            return Err(PluginError::IoError(err).into());
        }

        debug!(
            directory = %directory.display(),
            "DefaultPluginDiscovery: directory scan returns empty; discovered plugins from other primals arrive via IPC capability registry"
        );
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
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] when native dynamic loading is unavailable for the given path.
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub async fn load_plugin(&self, path: &Path) -> Result<Arc<dyn Plugin>> {
        Err(PluginError::LoadError(format!(
            "Native dynamic loading unavailable on this primal for path {}. Discover plugin providers via IPC or use manifest-only no-op registration.",
            path.display()
        ))
        .into())
    }
}

/// Default plugin loader implementation (kept for trait impl / future use)
#[derive(Debug, Copy, Clone)]
#[allow(
    dead_code,
    reason = "Default PluginLoader impl; referenced only from unit tests in non-test library builds"
)]
pub struct DefaultPluginLoader;

#[async_trait]
impl PluginLoader for DefaultPluginLoader {
    /// Load a plugin from a manifest
    async fn load_plugin(
        &self,
        manifest: &PluginManifest,
        _path: &Path,
    ) -> Result<Arc<dyn Plugin>> {
        Err(PluginError::LoadError(format!(
            "Native loader not available for manifest `{}` on this primal; load from peer primals via IPC discovery.",
            manifest.name
        ))
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manifest_deserialization() {
        let json = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author",
            "entry_point": "libplugin.so",
            "plugin_type": "native",
            "dependencies": ["dep1"],
            "capabilities": ["cap1", "cap2"]
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, "A test plugin");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.entry_point, "libplugin.so");
        assert_eq!(manifest.plugin_type, "native");
        assert_eq!(manifest.dependencies, vec!["dep1"]);
        assert_eq!(manifest.capabilities, vec!["cap1", "cap2"]);
    }

    #[test]
    fn test_plugin_manifest_minimal() {
        let json = r#"{
            "name": "minimal",
            "version": "0.1.0",
            "description": "",
            "author": "",
            "entry_point": "",
            "plugin_type": ""
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "minimal");
        assert!(manifest.dependencies.is_empty());
        assert!(manifest.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_manifest_to_metadata() {
        let json = r#"{
            "name": "meta-test",
            "version": "1.0.0",
            "description": "d",
            "author": "a",
            "entry_point": "lib.so",
            "plugin_type": "native",
            "dependencies": [],
            "capabilities": ["cap-a"]
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        let metadata = manifest.to_metadata();
        assert_eq!(metadata.name, "meta-test");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_create_noop_plugin() {
        let metadata = PluginMetadata::new("noop-test", "1.0", "desc", "author");
        let plugin = create_noop_plugin(metadata);
        assert_eq!(plugin.metadata().name, "noop-test");
        assert_eq!(plugin.metadata().version, "1.0");
        plugin.initialize().await.unwrap();
        plugin.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_default_discovery_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = DefaultPluginDiscovery::new();
        let plugins = discovery.discover_plugins(temp_dir.path()).await.unwrap();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_default_discovery_nonexistent_dir() {
        let discovery = DefaultPluginDiscovery::new();
        let result = discovery
            .discover_plugins(Path::new("/nonexistent/path/12345"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_default_plugin_loader_returns_error() {
        let loader = DefaultPluginLoader;
        let manifest = PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: String::new(),
            author: String::new(),
            entry_point: String::new(),
            plugin_type: String::new(),
            dependencies: vec![],
            capabilities: vec![],
        };
        let result = loader.load_plugin(&manifest, Path::new("/tmp")).await;
        match result {
            Ok(_) => panic!("expected load_plugin to fail"),
            Err(e) => assert!(e.to_string().contains("Native loader not available")),
        }
    }
}
