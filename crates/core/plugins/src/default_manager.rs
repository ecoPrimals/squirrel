// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Default plugin manager implementation
//!
//! This module provides the default implementation of the plugin manager.

use crate::Plugin;
use crate::dependency_resolver::DependencyResolver;
use crate::discovery::{DefaultPluginDiscovery, create_placeholder_plugin};
use crate::errors::{PluginError, Result};
use crate::metrics::{PluginManagerMetrics, PluginManagerStatus};
use crate::plugin::PluginMetadata;
use crate::registry::PluginRegistry;
use crate::state::{MemoryStateManager, PluginStateManager};
use crate::traits::PluginManagerTrait;
use crate::types::PluginStatus;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Default plugin manager implementation
pub struct DefaultPluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>,
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    /// Dependency resolver for initialization order (reserved for dependency resolution system)
    #[allow(dead_code)]
    dependency_resolver: Arc<RwLock<DependencyResolver>>,
    /// State manager for plugin state persistence (reserved for state persistence system)
    #[allow(dead_code)]
    state_manager: Arc<dyn PluginStateManager>,
    /// Discovery service for plugin loading (reserved for plugin discovery system)
    #[allow(dead_code)]
    discovery: Arc<DefaultPluginDiscovery>,
    /// Performance metrics
    metrics: Arc<RwLock<PluginManagerMetrics>>,
}

impl DefaultPluginManager {
    /// Create a new default plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            dependency_resolver: Arc::new(RwLock::new(DependencyResolver::new())),
            state_manager: Arc::new(MemoryStateManager::new()) as Arc<dyn PluginStateManager>,
            discovery: Arc::new(DefaultPluginDiscovery::new()),
            metrics: Arc::new(RwLock::new(PluginManagerMetrics::new())),
        }
    }

    /// Get plugin manager status
    pub async fn get_status(&self) -> PluginManagerStatus {
        let plugins = self.plugins.read().await;
        let statuses = self.statuses.read().await;

        let total = plugins.len();
        let active = statuses
            .values()
            .filter(|s| **s == PluginStatus::Running)
            .count();
        let failed = statuses
            .values()
            .filter(|s| **s == PluginStatus::Failed)
            .count();

        PluginManagerStatus::new(total, active, failed)
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PluginManagerMetrics {
        let metrics = self.metrics.read().await;
        PluginManagerMetrics {
            load_time_ms: metrics.load_time_ms,
            memory_usage_kb: metrics.memory_usage_kb,
        }
    }
}

impl fmt::Debug for DefaultPluginManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultPluginManager")
            .field("plugins", &"<plugin map>")
            .field("name_to_id", &"<name mapping>")
            .field("dependency_resolver", &"<dependency resolver>")
            .field("state_manager", &"<state manager>")
            .field("discovery", &"<discovery service>")
            .field("metrics", &"<metrics>")
            .finish()
    }
}

impl Default for DefaultPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Manifest format for plugin.toml (TOML with [plugin] section)
#[derive(serde::Deserialize)]
struct PluginManifestToml {
    plugin: PluginManifestSection,
}

#[derive(serde::Deserialize)]
struct PluginManifestSection {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    author: String,
    #[allow(dead_code)] // Required for serde Deserialize - fields parsed from manifest but not used
    #[serde(default)]
    capabilities: Option<toml::Value>,
    #[allow(dead_code)] // Required for serde Deserialize - fields parsed from manifest but not used
    #[serde(default)]
    dependencies: Vec<toml::Value>,
}

/// Manifest format for plugin.json (flat structure)
#[derive(serde::Deserialize)]
struct PluginManifestJson {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    author: String,
    #[allow(dead_code)] // Required for serde Deserialize - fields parsed from manifest but not used
    #[serde(default)]
    capabilities: Option<serde_json::Value>,
    #[allow(dead_code)] // Required for serde Deserialize - fields parsed from manifest but not used
    #[serde(default)]
    dependencies: Vec<serde_json::Value>,
}

/// Load plugins from a directory by scanning subdirectories for plugin.toml or plugin.json.
async fn load_plugins_from_directory<M: PluginRegistry + PluginManagerTrait>(
    manager: &M,
    directory: &str,
) -> Result<Vec<Uuid>> {
    let path = Path::new(directory);

    // Nonexistent directory: return Ok with empty vec (graceful handling per tests)
    if !path.exists() {
        return Ok(vec![]);
    }

    // Path is a file, not a directory: return error
    if path.is_file() {
        let err = std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {directory}"),
        );
        return Err(PluginError::IoError(err));
    }

    let mut plugin_ids = Vec::new();
    let mut entries = fs::read_dir(path).await.map_err(PluginError::IoError)?;

    while let Some(entry) = entries.next_entry().await.map_err(PluginError::IoError)? {
        let subdir = entry.path();
        if !subdir.is_dir() {
            continue;
        }

        // Try plugin.toml first, then plugin.json
        let manifest_path_toml = subdir.join("plugin.toml");
        let manifest_path_json = subdir.join("plugin.json");

        let metadata = if manifest_path_toml.exists() {
            let content = match fs::read_to_string(&manifest_path_toml).await {
                Ok(c) => c,
                Err(_) => continue, // Skip on read error
            };
            let manifest: PluginManifestToml = match toml::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue, // Skip invalid manifests (logged as warnings by caller)
            };
            PluginMetadata {
                id: manifest.plugin.id,
                name: manifest.plugin.name,
                version: manifest.plugin.version,
                description: manifest.plugin.description,
                author: manifest.plugin.author,
                capabilities: Vec::new(),
                dependencies: Vec::new(),
            }
        } else if manifest_path_json.exists() {
            let content = match fs::read_to_string(&manifest_path_json).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let manifest: PluginManifestJson = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue, // Skip invalid manifests
            };
            PluginMetadata {
                id: manifest.id,
                name: manifest.name,
                version: manifest.version,
                description: manifest.description,
                author: manifest.author,
                capabilities: Vec::new(),
                dependencies: Vec::new(),
            }
        } else {
            // No manifest in this subdirectory, skip
            continue;
        };

        let plugin = create_placeholder_plugin(metadata);
        let id = plugin.id();
        PluginRegistry::register_plugin(manager, plugin).await?;
        plugin_ids.push(id);
    }

    Ok(plugin_ids)
}

use async_trait::async_trait;

#[async_trait]
impl PluginRegistry for DefaultPluginManager {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let id = plugin.id();
        let name = plugin.metadata().name.clone();

        let mut plugins = self.plugins.write().await;
        let mut statuses = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;

        plugins.insert(id, plugin);
        statuses.insert(id, PluginStatus::Registered);
        name_to_id.insert(name, id);

        Ok(())
    }

    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let mut statuses = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;

        if let Some(plugin) = plugins.remove(&id) {
            let name = plugin.metadata().name.clone();
            statuses.remove(&id);
            name_to_id.remove(&name);
        }

        Ok(())
    }

    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins
            .get(&id)
            .cloned()
            .ok_or_else(|| crate::errors::PluginError::NotFound(id))
    }

    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        let name_to_id = self.name_to_id.read().await;
        let id = name_to_id
            .get(name)
            .ok_or_else(|| crate::errors::PluginError::PluginNotFound(name.to_string()))?;
        PluginRegistry::get_plugin(self, *id).await
    }

    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.values().cloned().collect())
    }

    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        statuses
            .get(&id)
            .cloned()
            .ok_or_else(|| crate::errors::PluginError::NotFound(id))
    }

    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
        Ok(())
    }

    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.list_plugins().await
    }
}

#[async_trait]
impl PluginManagerTrait for DefaultPluginManager {
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        PluginRegistry::get_plugin(self, id).await
    }

    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        let plugin = PluginManagerTrait::get_plugin(self, id).await?;
        plugin.initialize().await?;
        PluginRegistry::set_plugin_status(self, id, PluginStatus::Running).await?;
        Ok(())
    }

    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        let plugin = PluginManagerTrait::get_plugin(self, id).await?;
        plugin.shutdown().await?;
        PluginRegistry::set_plugin_status(self, id, PluginStatus::Inactive).await?;
        Ok(())
    }

    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        PluginRegistry::get_plugin_status(self, id).await
    }

    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        PluginRegistry::set_plugin_status(self, id, status).await
    }

    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        load_plugins_from_directory(self, directory).await
    }

    async fn initialize_all_plugins(&self) -> Result<()> {
        let plugins = self.list_plugins().await?;
        for plugin in plugins {
            self.initialize_plugin(plugin.id()).await?;
        }
        Ok(())
    }

    async fn shutdown_all_plugins(&self) -> Result<()> {
        let plugins = self.list_plugins().await?;
        for plugin in plugins {
            self.shutdown_plugin(plugin.id()).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::create_placeholder_plugin;
    use crate::traits::PluginManagerTrait;

    fn make_test_plugin(name: &str) -> Arc<dyn Plugin> {
        create_placeholder_plugin(PluginMetadata::new(name, "1.0.0", "Test plugin", "Test"))
    }

    #[tokio::test]
    async fn test_default_plugin_manager_new() {
        let manager = DefaultPluginManager::new();
        let plugins = manager.list_plugins().await.unwrap();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_get_status_empty() {
        let manager = DefaultPluginManager::new();
        let status = manager.get_status().await;
        assert_eq!(status.total_plugins, 0);
        assert_eq!(status.active_plugins, 0);
        assert_eq!(status.failed_plugins, 0);
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let manager = DefaultPluginManager::new();
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.load_time_ms, 0);
        assert_eq!(metrics.memory_usage_kb, 0);
    }

    #[tokio::test]
    async fn test_registry_register_get_list() {
        let manager = DefaultPluginManager::new();
        let plugin = make_test_plugin("reg-test");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        let found = PluginManagerTrait::get_plugin(&manager, id).await.unwrap();
        assert_eq!(found.metadata().name, "reg-test");
        let plugins = manager.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_get_by_name() {
        let manager = DefaultPluginManager::new();
        let plugin = make_test_plugin("named");
        manager.register_plugin(plugin).await.unwrap();
        let found = PluginRegistry::get_plugin_by_name(&manager, "named").await.unwrap();
        assert_eq!(found.metadata().name, "named");
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let manager = DefaultPluginManager::new();
        let plugin = make_test_plugin("unreg");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        manager.unregister_plugin(id).await.unwrap();
        let plugins = manager.list_plugins().await.unwrap();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_shutdown_single_plugin() {
        let manager = DefaultPluginManager::new();
        let plugin = make_test_plugin("init-shutdown");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        manager.initialize_plugin(id).await.unwrap();
        let status = PluginManagerTrait::get_plugin_status(&manager, id).await.unwrap();
        assert_eq!(status, PluginStatus::Running);
        manager.shutdown_plugin(id).await.unwrap();
        let status = PluginManagerTrait::get_plugin_status(&manager, id).await.unwrap();
        assert_eq!(status, PluginStatus::Inactive);
    }

    #[tokio::test]
    async fn test_initialize_shutdown_all_plugins() {
        let manager = DefaultPluginManager::new();
        let p1 = make_test_plugin("all-1");
        let p2 = make_test_plugin("all-2");
        manager.register_plugin(p1).await.unwrap();
        manager.register_plugin(p2).await.unwrap();
        manager.initialize_all_plugins().await.unwrap();
        let status = manager.get_status().await;
        assert_eq!(status.active_plugins, 2);
        manager.shutdown_all_plugins().await.unwrap();
        let status = manager.get_status().await;
        assert_eq!(status.active_plugins, 0);
    }

    #[tokio::test]
    async fn test_get_plugin_unknown_returns_error() {
        let manager = DefaultPluginManager::new();
        let unknown_id = Uuid::new_v4();
        let result = PluginManagerTrait::get_plugin(&manager, unknown_id).await;
        match result {
            Ok(_) => panic!("expected error for unknown plugin"),
            Err(PluginError::NotFound(id)) => assert_eq!(id, unknown_id),
            Err(e) => panic!("expected NotFound, got {:?}", e),
        }
    }
}
