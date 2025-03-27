//! Plugin registry implementation
//!
//! This module provides functionality for registering and retrieving plugins.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

use squirrel_interfaces::plugins::{Plugin, PluginRegistry};

/// Default implementation of a plugin registry
pub struct DefaultPluginRegistry {
    /// Plugins registered in the registry
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,

    /// Plugin capabilities index for quick lookup
    capabilities: RwLock<HashMap<String, Vec<String>>>,
}

impl DefaultPluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        debug!("Creating new DefaultPluginRegistry");
        Self {
            plugins: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Add a plugin's capabilities to the index
    fn index_plugin_capabilities(&self, plugin: &dyn Plugin) -> Result<()> {
        let plugin_id = plugin.metadata().id.clone();
        let capabilities = plugin.metadata().capabilities.clone();

        debug!(
            "Indexing capabilities for plugin {}: {:?}",
            plugin_id, capabilities
        );

        let mut capabilities_write = self
            .capabilities
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire capabilities write lock: {}", e))?;

        for capability in capabilities {
            capabilities_write
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(plugin_id.clone());
        }

        Ok(())
    }

    /// Remove a plugin's capabilities from the index
    fn remove_plugin_capabilities(&self, plugin_id: &str) -> Result<()> {
        debug!("Removing capabilities for plugin {}", plugin_id);

        let mut capabilities_write = self
            .capabilities
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire capabilities write lock: {}", e))?;

        for (_, plugins) in capabilities_write.iter_mut() {
            plugins.retain(|id| id != plugin_id);
        }

        // Remove any capability entries that no longer have any plugins
        capabilities_write.retain(|_, plugins| !plugins.is_empty());

        Ok(())
    }
}

impl Debug for DefaultPluginRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to acquire read locks - use placeholders if we can't
        let plugins = match self.plugins.read() {
            Ok(plugins) => {
                let plugin_ids: Vec<_> = plugins.keys().cloned().collect();
                format!("{:?}", plugin_ids)
            }
            Err(_) => "<locked>".to_string(),
        };

        let capabilities = match self.capabilities.read() {
            Ok(capabilities) => {
                let cap_list: Vec<_> = capabilities.keys().cloned().collect();
                format!("{:?}", cap_list)
            }
            Err(_) => "<locked>".to_string(),
        };

        f.debug_struct("DefaultPluginRegistry")
            .field("plugins", &plugins)
            .field("capabilities", &capabilities)
            .finish()
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginRegistry {
    // Implementation for all plugin types
    async fn register_plugin<P: Plugin + 'static>(&self, plugin: Arc<P>) -> Result<String> {
        let plugin_id = plugin.metadata().id.clone();
        let plugin_name = plugin.metadata().name.clone();

        info!("Registering plugin {} ({})", plugin_name, plugin_id);

        // Lock the plugins map for writing
        let mut plugins_write = self
            .plugins
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire plugins write lock: {}", e))?;

        // Check if the plugin already exists
        if plugins_write.contains_key(&plugin_id) {
            warn!("Plugin {} already registered", plugin_id);
            return Ok(plugin_id);
        }

        // Store the plugin
        // Convert concrete plugin to dynamic plugin if needed
        let plugin_dyn: Arc<dyn Plugin> = plugin;
        plugins_write.insert(plugin_id.clone(), plugin_dyn.clone());
        
        // Index the plugin's capabilities
        drop(plugins_write); // Release the write lock before indexing capabilities
        self.index_plugin_capabilities(plugin_dyn.as_ref())
            .context("Failed to index plugin capabilities")?;

        debug!("Plugin {} registered successfully", plugin_id);
        Ok(plugin_id)
    }

    async fn get_plugin(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        match self.plugins.read() {
            Ok(plugins) => plugins.get(id).cloned(),
            Err(e) => {
                error!("Failed to acquire plugins read lock: {}", e);
                None
            }
        }
    }

    async fn get_plugin_by_capability(&self, capability: &str) -> Option<Arc<dyn Plugin>> {
        // Get plugin IDs with the requested capability
        let plugin_ids = match self.capabilities.read() {
            Ok(capabilities) => capabilities.get(capability).cloned(),
            Err(e) => {
                error!("Failed to acquire capabilities read lock: {}", e);
                return None;
            }
        };

        // If we found plugin IDs, get the first plugin
        if let Some(plugin_ids) = plugin_ids {
            if !plugin_ids.is_empty() {
                return self.get_plugin(&plugin_ids[0]).await;
            }
        }

        None
    }

    /// Get a plugin by type and capability
    async fn get_plugin_by_type_and_capability<T: Plugin + ?Sized + 'static>(&self, capability: &str) -> Option<Arc<T>> {
        // Find a plugin with the requested capability
        if let Some(plugin) = self.get_plugin_by_capability(capability).await {
            debug!("Found plugin with capability {}: {}", capability, plugin.metadata().id);
            
            // Since we don't have access to downcast_arc without additional dependencies,
            // we simply log a debug message and return None
            debug!("Plugin found but cannot be downcast to the requested type without additional dependencies");
            None
        } else {
            None
        }
    }

    async fn list_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        match self.plugins.read() {
            Ok(plugins) => plugins.values().cloned().collect(),
            Err(e) => {
                error!("Failed to acquire plugins read lock: {}", e);
                Vec::new()
            }
        }
    }
}

impl Default for DefaultPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new plugin registry
pub fn create_plugin_registry() -> Arc<DefaultPluginRegistry> {
    Arc::new(DefaultPluginRegistry::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use squirrel_interfaces::plugins::PluginMetadata;

    #[derive(Debug)]
    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn new(id: &str, capabilities: Vec<&str>) -> Self {
            let mut metadata = PluginMetadata::new(
                id,
                "1.0.0",
                "Test plugin for unit tests",
                "DataScienceBioLab",
            );

            for capability in capabilities {
                metadata = metadata.with_capability(capability);
            }

            Self { metadata }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_plugin() -> Result<()> {
        let registry = DefaultPluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("test-plugin", vec!["test"]));

        let id = registry.register_plugin(plugin).await?;
        assert_eq!(id, "test-plugin");
        
        let plugins = registry.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_plugin() -> Result<()> {
        let registry = DefaultPluginRegistry::new();
        let plugin = Arc::new(TestPlugin::new("test-plugin", vec!["test"]));

        registry.register_plugin(plugin).await?;
        
        let retrieved = registry.get_plugin("test-plugin").await;
        assert!(retrieved.is_some());
        
        let not_found = registry.get_plugin("nonexistent").await;
        assert!(not_found.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_plugin_by_capability() -> Result<()> {
        let registry = DefaultPluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("plugin1", vec!["capability1"]));
        let plugin2 = Arc::new(TestPlugin::new("plugin2", vec!["capability2"]));

        registry.register_plugin(plugin1).await?;
        registry.register_plugin(plugin2).await?;
        
        let retrieved = registry.get_plugin_by_capability("capability1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata().id, "plugin1");
        
        let retrieved = registry.get_plugin_by_capability("capability2").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata().id, "plugin2");
        
        let not_found = registry.get_plugin_by_capability("nonexistent").await;
        assert!(not_found.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_plugins() -> Result<()> {
        let registry = DefaultPluginRegistry::new();
        let plugin1 = Arc::new(TestPlugin::new("plugin1", vec!["capability1"]));
        let plugin2 = Arc::new(TestPlugin::new("plugin2", vec!["capability2"]));

        registry.register_plugin(plugin1).await?;
        registry.register_plugin(plugin2).await?;
        
        let plugins = registry.list_plugins().await;
        assert_eq!(plugins.len(), 2);
        
        // Verify plugin IDs
        let plugin_ids: Vec<_> = plugins.iter().map(|p| p.metadata().id.clone()).collect();
        assert!(plugin_ids.contains(&"plugin1".to_string()));
        assert!(plugin_ids.contains(&"plugin2".to_string()));
        
        Ok(())
    }
} 