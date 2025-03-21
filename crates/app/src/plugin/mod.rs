use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::error::Result;

/// Plugin type definitions and traits
mod types;
/// Plugin discovery and loading functionality
mod discovery;

pub use types::{CommandPlugin, UiPlugin, ToolPlugin, McpPlugin};
pub use discovery::{PluginDiscovery, FileSystemDiscovery, PluginLoader};

/// Plugin metadata containing information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
}

/// Plugin state that can be persisted and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// Plugin ID
    pub plugin_id: Uuid,
    /// State data
    pub data: serde_json::Value,
    /// Last modified timestamp
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// Plugin lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is registered but not loaded
    Registered,
    /// Plugin is loaded and initialized
    Active,
    /// Plugin is temporarily disabled
    Disabled,
    /// Plugin has failed and needs attention
    Failed,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync + Any + std::fmt::Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Get plugin state
    async fn get_state(&self) -> Result<Option<PluginState>>;
    
    /// Set plugin state
    async fn set_state(&self, state: PluginState) -> Result<()>;

    /// Cast the plugin to Any
    fn as_any(&self) -> &dyn Any where Self: 'static, Self: Sized {
        self
    }
}

/// Plugin manager that handles plugin lifecycle and state
#[derive(Debug)]
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<Uuid, Box<dyn Plugin>>>>,
    /// Plugin status
    status: Arc<RwLock<HashMap<Uuid, PluginStatus>>>,
    /// Plugin state
    state: Arc<RwLock<HashMap<Uuid, PluginState>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin registration fails
    /// - The plugin ID is already registered
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        let mut plugins = self.plugins.write().await;
        let mut status = self.status.write().await;
        
        plugins.insert(id, plugin);
        status.insert(id, PluginStatus::Registered);
        
        Ok(())
    }
    
    /// Load and initialize a plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin initialization fails
    pub async fn load_plugin(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        let mut status = self.status.write().await;
        
        if let Some(plugin) = plugins.get(&id) {
            plugin.initialize().await?;
            status.insert(id, PluginStatus::Active);
        }
        
        Ok(())
    }
    
    /// Unload and shutdown a plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin shutdown fails
    pub async fn unload_plugin(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        let mut status = self.status.write().await;
        
        if let Some(plugin) = plugins.get(&id) {
            plugin.shutdown().await?;
            status.insert(id, PluginStatus::Disabled);
        }
        
        Ok(())
    }
    
    /// Get plugin status
    #[must_use]
    pub async fn get_plugin_status(&self, id: Uuid) -> Option<PluginStatus> {
        let status = self.status.read().await;
        status.get(&id).copied()
    }
    
    /// Get plugin state
    #[must_use]
    pub async fn get_plugin_state(&self, id: Uuid) -> Option<PluginState> {
        let state = self.state.read().await;
        state.get(&id).cloned()
    }
    
    /// Set plugin state
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin state update fails
    /// - The plugin is not found
    pub async fn set_plugin_state(&self, state: PluginState) -> Result<()> {
        let mut states = self.state.write().await;
        states.insert(state.plugin_id, state);
        Ok(())
    }

    /// Get all active plugins
    pub async fn get_active_plugins(&self) -> Vec<Uuid> {
        let status = self.status.read().await;
        status
            .iter()
            .filter(|(_, &s)| s == PluginStatus::Active)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get plugin capabilities
    pub async fn get_plugin_capabilities(&self, id: Uuid) -> Option<Vec<String>> {
        let plugins = self.plugins.read().await;
        plugins.get(&id).map(|plugin| plugin.metadata().capabilities.clone())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[derive(Debug)]
    struct TestPlugin {
        metadata: PluginMetadata,
        state: Arc<RwLock<Option<PluginState>>>,
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
        
        async fn get_state(&self) -> Result<Option<PluginState>> {
            Ok(self.state.read().await.clone())
        }
        
        async fn set_state(&self, state: PluginState) -> Result<()> {
            *self.state.write().await = Some(state);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let manager = PluginManager::new();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "test_plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec![],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_id = plugin.metadata().id;
        
        // Register plugin
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        assert_eq!(manager.get_plugin_status(plugin_id).await, Some(PluginStatus::Registered));
        
        // Load plugin
        manager.load_plugin(plugin_id).await.unwrap();
        assert_eq!(manager.get_plugin_status(plugin_id).await, Some(PluginStatus::Active));
        
        // Unload plugin
        manager.unload_plugin(plugin_id).await.unwrap();
        assert_eq!(manager.get_plugin_status(plugin_id).await, Some(PluginStatus::Disabled));
    }
    
    #[tokio::test]
    async fn test_plugin_state() {
        let manager = PluginManager::new();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "test_plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec![],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_id = plugin.metadata().id;
        
        // Register and load plugin
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        manager.load_plugin(plugin_id).await.unwrap();
        
        // Set and get state
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"key": "value"}),
            last_modified: chrono::Utc::now(),
        };
        
        manager.set_plugin_state(state.clone()).await.unwrap();
        let retrieved_state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(retrieved_state.plugin_id, state.plugin_id);
    }
} 