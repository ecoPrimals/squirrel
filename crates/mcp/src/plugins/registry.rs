use crate::plugins::interfaces::Plugin;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;

use crate::plugins::interfaces::{PluginMetadata};
use crate::plugins::types::PluginId;
use crate::error::{Result, PluginError};

/// A registry for managing plugins
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin with the registry
    async fn register(&self, plugin: Box<dyn Plugin>) -> Result<()>;
    
    /// Unregister a plugin from the registry
    async fn unregister(&self, id: &PluginId) -> Result<()>;
    
    /// Get a plugin by ID
    async fn get(&self, id: &PluginId) -> Result<Arc<Box<dyn Plugin>>>;
    
    /// Check if a plugin is registered
    async fn is_registered(&self, id: &PluginId) -> bool;
    
    /// Get all registered plugins
    async fn get_all(&self) -> Result<Vec<Arc<Box<dyn Plugin>>>>;
    
    /// Get metadata for all registered plugins
    async fn get_all_metadata(&self) -> Result<Vec<PluginMetadata>>;
}

/// Default implementation of PluginRegistry
pub struct DefaultPluginRegistry {
    plugins: RwLock<HashMap<PluginId, Arc<Box<dyn Plugin>>>>,
}

impl DefaultPluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginRegistry {
    async fn register(&self, plugin: Box<dyn Plugin>) -> Result<()> {
        let id = plugin.metadata().id.clone();
        let plugin_arc = Arc::new(plugin);
        
        let mut plugins = self.plugins.write().unwrap();
        if plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id.clone()).into());
        }
        
        plugins.insert(id.clone(), plugin_arc);
        Ok(())
    }
    
    async fn unregister(&self, id: &PluginId) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if !plugins.contains_key(id) {
            return Err(PluginError::NotFound(id.clone()).into());
        }
        
        plugins.remove(id);
        Ok(())
    }
    
    async fn get(&self, id: &PluginId) -> Result<Arc<Box<dyn Plugin>>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(id)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(id.clone()).into())
    }
    
    async fn is_registered(&self, id: &PluginId) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(id)
    }
    
    async fn get_all(&self) -> Result<Vec<Arc<Box<dyn Plugin>>>> {
        let plugins = self.plugins.read().unwrap();
        Ok(plugins.values().cloned().collect())
    }
    
    async fn get_all_metadata(&self) -> Result<Vec<PluginMetadata>> {
        let plugins = self.plugins.read().unwrap();
        Ok(plugins.values()
            .map(|plugin| plugin.metadata().clone())
            .collect())
    }
}

// ... existing code ... 