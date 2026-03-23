// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin registry traits and implementations
//!
//! This module provides the core registry functionality for plugin management.

use crate::Plugin;
use crate::errors::Result;
use crate::types::PluginStatus;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Plugin registry trait
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;

    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;

    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;

    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>>;

    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;

    /// Get all registered plugins
    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;

    /// Get all plugins (alias for `get_all_plugins`)
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.get_all_plugins().await
    }
}

#[cfg(test)]
mod tests {
    use super::PluginRegistry;
    use crate::plugin::{Plugin, PluginMetadata};
    use crate::types::PluginStatus;
    use crate::{PluginError, Result};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct MockRegistry {
        plugins: Mutex<HashMap<Uuid, Arc<dyn Plugin>>>,
        statuses: Mutex<HashMap<Uuid, PluginStatus>>,
    }

    impl MockRegistry {
        fn new() -> Self {
            Self {
                plugins: Mutex::new(HashMap::new()),
                statuses: Mutex::new(HashMap::new()),
            }
        }
    }

    fn test_plugin(id: Uuid, name: &str) -> Arc<dyn Plugin> {
        let mut meta = PluginMetadata::new(name, "1.0.0", "desc", "author");
        meta.id = id;
        crate::discovery::create_noop_plugin(meta)
    }

    #[async_trait]
    impl PluginRegistry for MockRegistry {
        async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
            let id = plugin.id();
            let mut g = self.plugins.lock().unwrap();
            if g.contains_key(&id) {
                return Err(PluginError::PluginAlreadyExists(id.to_string()));
            }
            g.insert(id, plugin);
            Ok(())
        }

        async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
            self.plugins.lock().unwrap().remove(&id);
            self.statuses.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
            self.plugins
                .lock()
                .unwrap()
                .get(&id)
                .cloned()
                .ok_or_else(|| PluginError::NotFound(id))
        }

        async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
            self.plugins
                .lock()
                .unwrap()
                .values()
                .find(|p| p.metadata().name == name)
                .cloned()
                .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))
        }

        async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
            Ok(self.plugins.lock().unwrap().values().cloned().collect())
        }

        async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
            self.statuses
                .lock()
                .unwrap()
                .get(&id)
                .copied()
                .ok_or_else(|| PluginError::NotFound(id))
        }

        async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
            if !self.plugins.lock().unwrap().contains_key(&id) {
                return Err(PluginError::NotFound(id));
            }
            self.statuses.lock().unwrap().insert(id, status);
            Ok(())
        }

        async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
            Ok(self.plugins.lock().unwrap().values().cloned().collect())
        }
    }

    #[tokio::test]
    async fn get_plugins_delegates_to_get_all_plugins() {
        let reg = MockRegistry::new();
        let id = Uuid::new_v4();
        let p = test_plugin(id, "alpha");
        reg.register_plugin(p).await.unwrap();
        let via_get_plugins = reg.get_plugins().await.unwrap();
        let via_get_all = reg.get_all_plugins().await.unwrap();
        assert_eq!(via_get_plugins.len(), via_get_all.len());
        assert_eq!(via_get_plugins[0].id(), via_get_all[0].id());
    }

    #[tokio::test]
    async fn mock_registry_register_get_list_status_roundtrip() {
        let reg = MockRegistry::new();
        let id = Uuid::new_v4();
        let p = test_plugin(id, "p1");
        reg.register_plugin(p).await.unwrap();
        assert_eq!(reg.get_plugin(id).await.unwrap().metadata().name, "p1");
        assert_eq!(reg.get_plugin_by_name("p1").await.unwrap().id(), id);
        assert_eq!(reg.list_plugins().await.unwrap().len(), 1);
        reg.set_plugin_status(id, PluginStatus::Initialized)
            .await
            .unwrap();
        assert_eq!(
            reg.get_plugin_status(id).await.unwrap(),
            PluginStatus::Initialized
        );
        reg.unregister_plugin(id).await.unwrap();
        assert!(reg.get_plugin(id).await.is_err());
    }
}
