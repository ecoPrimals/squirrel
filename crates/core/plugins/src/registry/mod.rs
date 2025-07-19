//! Plugin registry module
//!
//! This module provides a registry for plugins.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugin::{Plugin, PluginMetadata};
use crate::PluginError;

/// Plugin dependency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency ID
    pub id: Uuid,

    /// Dependency name
    pub name: String,

    /// Minimum version required
    pub min_version: String,

    /// Maximum version supported
    pub max_version: Option<String>,

    /// Whether the dependency is optional
    pub optional: bool,
}

/// Plugin registry trait
#[async_trait::async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin with the registry
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;

    /// Unregister a plugin from the registry
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;

    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Option<Arc<dyn Plugin>>;

    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Option<Arc<dyn Plugin>>;

    /// Get all plugins
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;
}

/// Default plugin registry implementation
pub struct DefaultPluginRegistry {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,

    /// Plugin metadata
    metadata: RwLock<HashMap<Uuid, PluginMetadata>>,

    /// Plugin index by name
    index: RwLock<BTreeMap<String, Vec<Uuid>>>,

    /// Plugin dependencies
    dependencies: RwLock<HashMap<Uuid, Vec<PluginDependency>>>,
}

impl std::fmt::Debug for DefaultPluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultPluginRegistry")
            .field("plugins", &"<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>")
            .field("metadata", &"<RwLock<HashMap<Uuid, PluginMetadata>>>")
            .field("index", &"<RwLock<BTreeMap<String, Vec<Uuid>>>>")
            .field(
                "dependencies",
                &"<RwLock<HashMap<Uuid, Vec<PluginDependency>>>>",
            )
            .finish()
    }
}

impl DefaultPluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            index: RwLock::new(BTreeMap::new()),
            dependencies: RwLock::new(HashMap::new()),
        }
    }

    /// Register plugin dependencies
    pub async fn register_dependencies(
        &self,
        id: Uuid,
        dependencies: Vec<PluginDependency>,
    ) -> Result<()> {
        let mut deps = self.dependencies.write().await;
        deps.insert(id, dependencies);
        Ok(())
    }

    /// Resolve plugin dependencies
    pub async fn resolve_dependencies(&self, id: Uuid) -> Result<Vec<Arc<dyn Plugin>>> {
        let deps = self.dependencies.read().await;
        let plugins = self.plugins.read().await;

        let dependencies = match deps.get(&id) {
            Some(deps) => deps,
            None => return Ok(Vec::new()),
        };

        let mut result = Vec::new();

        for dep in dependencies {
            match plugins.get(&dep.id) {
                Some(plugin) => result.push(plugin.clone()),
                None => {
                    if !dep.optional {
                        return Err(PluginError::DependencyNotFound(dep.name.clone()).into());
                    }
                }
            }
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl PluginRegistry for DefaultPluginRegistry {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata().clone();
        let id = metadata.id;
        let name = metadata.name.clone();

        // Add to plugins
        let mut plugins = self.plugins.write().await;
        plugins.insert(id, plugin);

        // Add to metadata
        let mut meta = self.metadata.write().await;
        meta.insert(id, metadata);

        // Add to index
        let mut idx = self.index.write().await;
        if let Some(ids) = idx.get_mut(&name) {
            ids.push(id);
        } else {
            idx.insert(name, vec![id]);
        }

        Ok(())
    }

    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        // Get metadata
        let meta = self.metadata.read().await;
        let name = match meta.get(&id) {
            Some(metadata) => metadata.name.clone(),
            None => return Err(PluginError::PluginNotFound(id.to_string()).into()),
        };
        drop(meta);

        // Remove from plugins
        let mut plugins = self.plugins.write().await;
        if plugins.remove(&id).is_none() {
            return Err(PluginError::PluginNotFound(id.to_string()).into());
        }

        // Remove from metadata
        let mut meta = self.metadata.write().await;
        meta.remove(&id);

        // Remove from index
        let mut idx = self.index.write().await;
        if let Some(ids) = idx.get_mut(&name) {
            ids.retain(|&i| i != id);
            if ids.is_empty() {
                idx.remove(&name);
            }
        }

        // Remove from dependencies
        let mut deps = self.dependencies.write().await;
        deps.remove(&id);

        Ok(())
    }

    async fn get_plugin(&self, id: Uuid) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(&id).cloned()
    }

    async fn get_plugin_by_name(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let idx = self.index.read().await;
        let ids = idx.get(name)?;

        if ids.is_empty() {
            return None;
        }

        let plugins = self.plugins.read().await;
        plugins.get(&ids[0]).cloned()
    }

    async fn get_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.values().cloned().collect())
    }
}

impl Default for DefaultPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// In-memory plugin registry implementation
/// This is a simplified registry implementation for testing purposes
pub struct InMemoryPluginRegistry {
    registry: DefaultPluginRegistry,
}

impl std::fmt::Debug for InMemoryPluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryPluginRegistry")
            .field("registry", &self.registry)
            .finish()
    }
}

impl Default for InMemoryPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPluginRegistry {
    /// Create a new in-memory plugin registry
    pub fn new() -> Self {
        Self {
            registry: DefaultPluginRegistry::new(),
        }
    }
}

#[async_trait::async_trait]
impl PluginRegistry for InMemoryPluginRegistry {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        self.registry.register_plugin(plugin).await
    }

    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        self.registry.unregister_plugin(id).await
    }

    async fn get_plugin(&self, id: Uuid) -> Option<Arc<dyn Plugin>> {
        self.registry.get_plugin(id).await
    }

    async fn get_plugin_by_name(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.registry.get_plugin_by_name(name).await
    }

    async fn get_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.registry.get_plugins().await
    }
}
