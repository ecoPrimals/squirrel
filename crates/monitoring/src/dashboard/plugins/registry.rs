//! Dashboard plugin registry implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use squirrel_core::error::{Result, SquirrelError};

use super::types::{DashboardPlugin, DashboardPluginRegistry, DashboardPluginType};
use crate::dashboard::manager::DashboardManager;

/// Dashboard plugin registry implementation
#[derive(Debug)]
pub struct DashboardPluginRegistryImpl {
    /// Map of plugins by ID
    plugins: Mutex<HashMap<String, Arc<dyn DashboardPlugin>>>,
}

impl DashboardPluginRegistryImpl {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for DashboardPluginRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new dashboard plugin registry
pub fn create_dashboard_plugin_registry() -> Arc<DashboardPluginRegistryImpl> {
    Arc::new(DashboardPluginRegistryImpl::new())
}

/// Register a plugin with a dashboard manager
pub async fn register_plugin_with_manager(
    manager: &Arc<DashboardManager>,
    plugin: Arc<dyn DashboardPlugin>,
) -> Result<()> {
    info!("Registering plugin with dashboard manager");
    
    let plugin_registry = manager.get_plugin_registry().await?;
    plugin_registry.register_plugin(plugin).await?;
    
    info!("Plugin registered successfully");
    Ok(())
}

#[async_trait]
impl DashboardPluginRegistry for DashboardPluginRegistryImpl {
    async fn register_plugin(&self, plugin: Arc<dyn DashboardPlugin>) -> Result<()> {
        let id_str = plugin.metadata().id.clone();
        let name = plugin.metadata().name.clone();
        
        // Initialize the plugin
        plugin.initialize().await.map_err(|e| {
            error!("Failed to initialize plugin {}: {}", name, e);
            SquirrelError::dashboard(format!("Failed to initialize plugin {}: {}", name, e))
        })?;
        
        // Check if plugin with this ID already exists
        let mut plugins = self.plugins.lock().await;
        if plugins.contains_key(&id_str) {
            return Err(SquirrelError::dashboard(format!("Plugin with ID {} already registered", id_str)));
        }
        
        // Register the plugin
        plugins.insert(id_str, plugin);
        info!("Plugin '{}' registered successfully", name);
        
        Ok(())
    }
    
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn DashboardPlugin>>> {
        let plugins = self.plugins.lock().await;
        Ok(plugins.values().cloned().collect())
    }
    
    async fn get_plugin(&self, id: &str) -> Result<Option<Arc<dyn DashboardPlugin>>> {
        let plugins = self.plugins.lock().await;
        Ok(plugins.get(id).cloned())
    }
    
    async fn get_plugins_by_type(&self, plugin_type: DashboardPluginType) -> Result<Vec<Arc<dyn DashboardPlugin>>> {
        let plugins = self.plugins.lock().await;
        let filtered = plugins.values()
            .filter(|p| p.plugin_type() == plugin_type)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn remove_plugin(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.lock().await;
        
        if plugins.contains_key(id) {
            // Get the plugin and call shutdown
            if let Some(plugin) = plugins.get(id) {
                let name = plugin.metadata().name.clone();
                
                match plugin.shutdown().await {
                    Ok(_) => info!("Plugin '{}' shutdown successfully", name),
                    Err(e) => error!("Failed to cleanly shutdown plugin '{}': {}", name, e),
                }
            }
            
            // Remove from registry
            plugins.remove(id);
            info!("Plugin removed successfully");
            Ok(())
        } else {
            Err(SquirrelError::dashboard(format!("Plugin with ID {} not found", id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::plugins::example::ExamplePlugin;
    
    #[tokio::test]
    async fn test_register_plugin() {
        let registry = DashboardPluginRegistryImpl::new();
        let plugin = Arc::new(ExamplePlugin::new());
        
        let result = registry.register_plugin(plugin.clone()).await;
        assert!(result.is_ok());
        
        let plugins = registry.get_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
    }
    
    #[tokio::test]
    async fn test_get_plugin() {
        let registry = DashboardPluginRegistryImpl::new();
        let plugin = Arc::new(ExamplePlugin::new());
        
        let plugin_id = plugin.metadata().id.to_string();
        registry.register_plugin(plugin).await.unwrap();
        
        let retrieved = registry.get_plugin(&plugin_id).await;
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_some());
    }
    
    #[tokio::test]
    async fn test_remove_plugin() {
        let registry = DashboardPluginRegistryImpl::new();
        let plugin = Arc::new(ExamplePlugin::new());
        
        let plugin_id = plugin.metadata().id.to_string();
        registry.register_plugin(plugin).await.unwrap();
        
        let result = registry.remove_plugin(&plugin_id).await;
        assert!(result.is_ok());
        
        let plugins = registry.get_plugins().await.unwrap();
        assert_eq!(plugins.len(), 0);
    }
} 