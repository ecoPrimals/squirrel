//! Default plugin manager implementation
//!
//! This module provides the default implementation of the plugin manager.

use crate::dependency_resolver::DependencyResolver;
use crate::discovery::DefaultPluginDiscovery;
use crate::metrics::{PluginManagerMetrics, PluginManagerStatus};
use crate::state::{MemoryStateManager, PluginStateManager};
use crate::types::PluginStatus;
use crate::Plugin;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
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
    /// Dependency resolver
    dependency_resolver: Arc<RwLock<DependencyResolver>>,
    /// State manager
    state_manager: Arc<dyn PluginStateManager>,
    /// Discovery service
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

use crate::errors::Result;
use crate::registry::PluginRegistry;
use crate::traits::PluginManagerTrait;
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
            .ok_or_else(|| crate::errors::PluginError::NotFound(id).into())
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
            .ok_or_else(|| crate::errors::PluginError::NotFound(id).into())
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

    async fn load_plugins(&self, _directory: &str) -> Result<Vec<Uuid>> {
        // TODO: Implement plugin loading from directory
        Ok(vec![])
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
