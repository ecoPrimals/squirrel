use crate::plugins::interfaces::Plugin;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
// Phase 4: Removed async_trait - using native async fn in traits
use std::future::Future;

use crate::plugins::interfaces::{PluginMetadata};
use crate::plugins::types::PluginId;
use crate::error::{Result};
use crate::error::plugin::PluginError;

/// A registry for managing plugins
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin with the registry
    fn register(&self, plugin: Box<dyn Plugin>) -> impl Future<Output = Result<()>> + Send;
    
    /// Unregister a plugin from the registry
    fn unregister(&self, id: &PluginId) -> impl Future<Output = Result<()>> + Send;
    
    /// Get a plugin by ID
    fn get(&self, id: &PluginId) -> impl Future<Output = Result<Arc<Box<dyn Plugin>>>> + Send;
    
    /// Check if a plugin is registered
    fn is_registered(&self, id: &PluginId) -> impl Future<Output = bool> + Send;
    
    /// Get all registered plugins
    fn get_all(&self) -> impl Future<Output = Result<Vec<Arc<Box<dyn Plugin>>>>> + Send;
    
    /// Get metadata for all registered plugins
    fn get_all_metadata(&self) -> impl Future<Output = Result<Vec<PluginMetadata>>> + Send;
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

impl PluginRegistry for DefaultPluginRegistry {
    fn register(&self, plugin: Box<dyn Plugin>) -> impl Future<Output = Result<()>> + Send {
        let id = plugin.metadata().id.clone();
        let plugin_arc = Arc::new(plugin);
        
        // Capture self reference for async block
        let plugins_ref = &self.plugins;
        
        async move {
            let mut plugins = plugins_ref.write().map_err(|_| {
                PluginError::InternalError("Failed to acquire write lock for plugin registry".to_string())
            })?;
            
            if plugins.contains_key(&id) {
                return Err(PluginError::AlreadyRegistered(id.clone()).into());
            }
            
            plugins.insert(id.clone(), plugin_arc);
            Ok(())
        }
    }
    
    fn unregister(&self, id: &PluginId) -> impl Future<Output = Result<()>> + Send {
        let id = id.clone();
        let plugins_ref = &self.plugins;
        
        async move {
            let mut plugins = plugins_ref.write().map_err(|_| {
                PluginError::InternalError("Failed to acquire write lock for plugin registry".to_string())
            })?;
            
            if !plugins.contains_key(&id) {
                return Err(PluginError::NotFound(id.clone()).into());
            }
            
            plugins.remove(&id);
            Ok(())
        }
    }
    
    fn get(&self, id: &PluginId) -> impl Future<Output = Result<Arc<Box<dyn Plugin>>>> + Send {
        let id = id.clone();
        let plugins_ref = &self.plugins;
        
        async move {
            let plugins = plugins_ref.read().map_err(|_| {
                PluginError::InternalError("Failed to acquire read lock for plugin registry".to_string())
            })?;
            
            plugins.get(&id)
                .cloned()
                .ok_or_else(|| PluginError::NotFound(id.clone()).into())
        }
    }
    
    fn is_registered(&self, id: &PluginId) -> impl Future<Output = bool> + Send {
        let id = id.clone();
        let plugins_ref = &self.plugins;
        
        async move {
            match plugins_ref.read() {
                Ok(plugins) => plugins.contains_key(&id),
                Err(_) => {
                    // If mutex is poisoned, assume plugin is not registered
                    false
                }
            }
        }
    }
    
    fn get_all(&self) -> impl Future<Output = Result<Vec<Arc<Box<dyn Plugin>>>>> + Send {
        let plugins_ref = &self.plugins;
        
        async move {
            let plugins = plugins_ref.read().map_err(|_| {
                PluginError::InternalError("Failed to acquire read lock for plugin registry".to_string())
            })?;
            
            Ok(plugins.values().cloned().collect())
        }
    }
    
    fn get_all_metadata(&self) -> impl Future<Output = Result<Vec<PluginMetadata>>> + Send {
        let plugins_ref = &self.plugins;
        
        async move {
            let plugins = plugins_ref.read().map_err(|_| {
                PluginError::InternalError("Failed to acquire read lock for plugin registry".to_string())
            })?;
            
            Ok(plugins.values()
                .map(|plugin| plugin.metadata().clone())
                .collect())
        }
    }
}

// ... existing code ... 