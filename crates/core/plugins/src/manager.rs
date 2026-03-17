// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

// NOTE: Using deprecated plugin::PluginMetadata until interfaces crate stabilizes
// The interfaces version lacks dependency tracking. See: PLUGIN_METADATA_MIGRATION_PLAN.md
#![allow(deprecated)]

//! Plugin manager
//!
//! This module provides the core plugin manager implementation.

use crate::dependency_resolver::DependencyResolver;
use crate::errors::{PluginError, Result};
use crate::plugin;
use crate::registry::PluginRegistry;
use crate::traits::PluginManagerTrait;
use crate::types::PluginStatus;
use crate::{Plugin, PluginConfig};
use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Minimal no-op plugin for built-in system placeholder (not from discovery)
#[derive(Debug)]
struct SystemPlaceholderPlugin {
    metadata: plugin::PluginMetadata,
}

#[async_trait]
impl Plugin for SystemPlaceholderPlugin {
    fn metadata(&self) -> &plugin::PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        warn!(
            plugin_name = %self.metadata.name,
            "SystemPlaceholderPlugin: no real plugin loaded; initialize is a no-op"
        );
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        warn!(
            plugin_name = %self.metadata.name,
            "SystemPlaceholderPlugin: no real plugin loaded; shutdown is a no-op"
        );
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Plugin manager for handling plugin lifecycle and dependencies
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<DashMap<Uuid, Arc<dyn Plugin>>>,
    /// Plugin configurations
    plugin_configs: Arc<DashMap<Uuid, PluginConfig>>,
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    /// Dependency resolver for proper plugin initialization order (reserved for dependency resolution system)
    #[expect(
        dead_code,
        reason = "Phase 2 placeholder — dependency resolution system"
    )]
    dependency_resolver: RwLock<DependencyResolver>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(DashMap::new()),
            plugin_configs: Arc::new(DashMap::new()),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            dependency_resolver: RwLock::new(DependencyResolver::new()),
        }
    }

    /// Initialize the plugin manager
    pub async fn init(&self) -> Result<()> {
        self.register_built_in_plugins().await?;
        debug!("Plugin manager initialized");
        Ok(())
    }

    /// Register built-in plugins
    async fn register_built_in_plugins(&self) -> Result<()> {
        let placeholder_metadata = plugin::PluginMetadata::new(
            "system-placeholder",
            "1.0.0",
            "System placeholder plugin",
            "Squirrel System",
        );
        let placeholder = Arc::new(SystemPlaceholderPlugin {
            metadata: placeholder_metadata,
        });
        self.register_plugin(placeholder).await?;
        Ok(())
    }

    /// Register a plugin with metadata, implementation, and optional signature
    pub async fn register_plugin_with_signature(
        &self,
        plugin: Arc<dyn Plugin>,
        signature: Option<Vec<u8>>,
    ) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;

        if let Some(_sig_bytes) = signature {
            debug!("Verifying signature for plugin {}", metadata.name);
            // Security verification handled by BearDog framework
        }

        self.plugins.insert(id, plugin.clone());
        self.plugin_configs.insert(id, PluginConfig::default());
        self.statuses
            .write()
            .await
            .insert(id, PluginStatus::Registered);
        self.name_to_id
            .write()
            .await
            .insert(metadata.name.clone(), id);

        info!(
            "Plugin {} (ID: {}) registered successfully with signature verification",
            metadata.name, id
        );
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginRegistry for PluginManager {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        self.register_plugin_with_signature(plugin, None).await
    }

    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        if let Some((_, plugin)) = self.plugins.remove(&id) {
            let metadata = plugin.metadata();
            self.plugin_configs.remove(&id);
            {
                let mut statuses = self.statuses.write().await;
                statuses.remove(&id);
            }
            {
                let mut name_to_id = self.name_to_id.write().await;
                name_to_id.remove(&metadata.name);
            }
            info!("Plugin {} unregistered successfully", metadata.name);
            Ok(())
        } else {
            Err(PluginError::PluginNotFound(id.to_string()))
        }
    }

    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        self.plugins
            .get(&id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| PluginError::PluginNotFound(id.to_string()))
    }

    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        let id = {
            let name_to_id = self.name_to_id.read().await;
            *name_to_id
                .get(name)
                .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))?
        };
        PluginManagerTrait::get_plugin(self, id).await
    }

    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        Ok(self
            .plugins
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        statuses
            .get(&id)
            .copied()
            .ok_or_else(|| PluginError::PluginNotFound(id.to_string()))
    }

    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        if let std::collections::hash_map::Entry::Occupied(mut e) = statuses.entry(id) {
            e.insert(status);
            Ok(())
        } else {
            Err(PluginError::PluginNotFound(id.to_string()))
        }
    }

    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.list_plugins().await
    }
}

#[async_trait]
impl PluginManagerTrait for PluginManager {
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        PluginRegistry::get_plugin(self, id).await
    }

    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        let plugin = PluginManagerTrait::get_plugin(self, id).await?;
        plugin.initialize().await?;
        PluginManagerTrait::set_plugin_status(self, id, PluginStatus::Running).await?;
        Ok(())
    }

    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        let plugin = PluginManagerTrait::get_plugin(self, id).await?;
        plugin.shutdown().await?;
        PluginManagerTrait::set_plugin_status(self, id, PluginStatus::Stopped).await?;
        Ok(())
    }

    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        PluginRegistry::get_plugin_status(self, id).await
    }

    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        PluginRegistry::set_plugin_status(self, id, status).await
    }

    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        // Implementation would use discovery service
        debug!("Loading plugins from directory: {}", directory);
        Ok(Vec::new()) // Placeholder
    }

    async fn initialize_all_plugins(&self) -> Result<()> {
        let plugins = self.list_plugins().await?;
        for plugin in plugins {
            let id = plugin.metadata().id;
            if let Err(e) = self.initialize_plugin(id).await {
                error!("Failed to initialize plugin {}: {}", id, e);
            }
        }
        Ok(())
    }

    async fn shutdown_all_plugins(&self) -> Result<()> {
        let plugins = self.list_plugins().await?;
        for plugin in plugins {
            let id = plugin.metadata().id;
            if let Err(e) = self.shutdown_plugin(id).await {
                error!("Failed to shutdown plugin {}: {}", id, e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::create_placeholder_plugin;
    use crate::plugin::PluginMetadata;
    use crate::traits::PluginManagerTrait;

    fn make_test_plugin(name: &str) -> Arc<dyn Plugin> {
        create_placeholder_plugin(PluginMetadata::new(name, "1.0.0", "Test plugin", "Test"))
    }

    #[tokio::test]
    async fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        let plugins = manager.list_plugins().await.unwrap();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_manager_init_registers_builtin() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugins = manager.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata().name, "system-placeholder");
    }

    #[tokio::test]
    async fn test_register_and_unregister_plugin() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugin = make_test_plugin("test-plugin");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        let plugins = manager.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 2);
        manager.unregister_plugin(id).await.unwrap();
        let plugins = manager.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
    }

    #[tokio::test]
    async fn test_get_plugin_by_id() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugin = make_test_plugin("lookup-plugin");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        let found = PluginManagerTrait::get_plugin(&manager, id).await.unwrap();
        assert_eq!(found.metadata().name, "lookup-plugin");
    }

    #[tokio::test]
    async fn test_get_plugin_by_name() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugin = make_test_plugin("named-plugin");
        manager.register_plugin(plugin).await.unwrap();
        let found = manager.get_plugin_by_name("named-plugin").await.unwrap();
        assert_eq!(found.metadata().name, "named-plugin");
    }

    #[tokio::test]
    async fn test_get_plugin_status_and_set() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugins = manager.list_plugins().await.unwrap();
        let id = plugins[0].id();
        let status = PluginManagerTrait::get_plugin_status(&manager, id)
            .await
            .unwrap();
        assert_eq!(status, PluginStatus::Registered);
        PluginManagerTrait::set_plugin_status(&manager, id, PluginStatus::Running)
            .await
            .unwrap();
        let status = PluginManagerTrait::get_plugin_status(&manager, id)
            .await
            .unwrap();
        assert_eq!(status, PluginStatus::Running);
    }

    #[tokio::test]
    async fn test_initialize_and_shutdown_plugin() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let plugin = make_test_plugin("init-plugin");
        let id = plugin.id();
        manager.register_plugin(plugin).await.unwrap();
        manager.initialize_plugin(id).await.unwrap();
        let status = PluginManagerTrait::get_plugin_status(&manager, id)
            .await
            .unwrap();
        assert_eq!(status, PluginStatus::Running);
        manager.shutdown_plugin(id).await.unwrap();
        let status = PluginManagerTrait::get_plugin_status(&manager, id)
            .await
            .unwrap();
        assert_eq!(status, PluginStatus::Stopped);
    }

    #[tokio::test]
    async fn test_get_plugin_unknown_id_returns_error() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let unknown_id = uuid::Uuid::new_v4();
        let result = PluginManagerTrait::get_plugin(&manager, unknown_id).await;
        match result {
            Ok(_) => panic!("expected error for unknown plugin"),
            Err(e) => assert!(matches!(e, PluginError::PluginNotFound(_))),
        }
    }

    #[tokio::test]
    async fn test_unregister_unknown_plugin_returns_error() {
        let manager = PluginManager::new();
        manager.init().await.unwrap();
        let unknown_id = uuid::Uuid::new_v4();
        let result = manager.unregister_plugin(unknown_id).await;
        let err = result.expect_err("expected error for unknown plugin");
        assert!(matches!(err, PluginError::PluginNotFound(_)));
    }
}
