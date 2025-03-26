// Plugin Management Module
//
// This module provides functionality for managing plugins, including registration,
// lookup, and lifecycle management.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::lifecycle::{LifecycleState, PluginLifecycle, PluginLifecycleManager};
use crate::plugins::Result;
use crate::plugins::PluginManagerConfig;
use crate::plugins::discovery::PluginDiscovery;
use crate::plugins::security::{PluginSecurityManager, PluginSecurityValidator};

use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus};

/// Plugin registry
///
/// This trait defines the interface for registering and looking up plugins.
#[async_trait]
pub trait PluginRegistry: Send + Sync + Debug {
    /// Register a plugin
    async fn register(&self, plugin: Arc<dyn Plugin>) -> Result<Uuid>;
    
    /// Unregister a plugin
    async fn unregister(&self, id: Uuid) -> Result<()>;
    
    /// Get a plugin by ID
    async fn get(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;
    
    /// Get all registered plugins
    async fn get_all(&self) -> Result<Vec<Arc<dyn Plugin>>>;
    
    /// Get plugins by capability
    async fn get_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Plugin>>>;
    
    /// Get plugins by tag
    async fn get_by_tag(&self, tag: &str) -> Result<Vec<Arc<dyn Plugin>>>;
    
    /// Check if a plugin is registered
    async fn is_registered(&self, id: Uuid) -> bool;
    
    /// Get plugin metadata
    async fn get_metadata(&self, id: Uuid) -> Result<PluginMetadata>;
    
    /// Get all plugin metadata
    async fn get_all_metadata(&self) -> Result<Vec<PluginMetadata>>;
}

/// Plugin registry implementation
#[derive(Debug)]
pub struct PluginRegistryImpl {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
    
    /// Plugin metadata cache
    metadata_cache: RwLock<HashMap<Uuid, PluginMetadata>>,
    
    /// Plugin capabilities
    capabilities: RwLock<HashMap<String, HashSet<Uuid>>>,
    
    /// Plugin tags
    tags: RwLock<HashMap<String, HashSet<Uuid>>>,
}

impl PluginRegistryImpl {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            metadata_cache: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
            tags: RwLock::new(HashMap::new()),
        }
    }
    
    /// Index a plugin's capabilities and tags
    async fn index_plugin(&self, plugin: &Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        // Index capabilities
        let mut capabilities = self.capabilities.write().await;
        for capability in &metadata.capabilities {
            capabilities
                .entry(capability.clone())
                .or_insert_with(HashSet::new)
                .insert(id);
        }
        
        // Index tags
        let mut tags = self.tags.write().await;
        for tag in &metadata.tags {
            tags
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(id);
        }
        
        // Cache metadata
        let mut metadata_cache = self.metadata_cache.write().await;
        metadata_cache.insert(id, metadata.clone());
        
        Ok(())
    }
    
    /// Remove a plugin from the index
    async fn remove_from_index(&self, id: Uuid) -> Result<()> {
        // Get metadata
        let metadata = {
            let metadata_cache = self.metadata_cache.read().await;
            match metadata_cache.get(&id).cloned() {
                Some(metadata) => metadata,
                None => return Err(PluginError::NotFound(id)),
            }
        };
        
        // Remove from capabilities
        let mut capabilities = self.capabilities.write().await;
        for capability in &metadata.capabilities {
            if let Some(plugins) = capabilities.get_mut(capability) {
                plugins.remove(&id);
            }
        }
        
        // Remove from tags
        let mut tags = self.tags.write().await;
        for tag in &metadata.tags {
            if let Some(plugins) = tags.get_mut(tag) {
                plugins.remove(&id);
            }
        }
        
        // Remove from metadata cache
        let mut metadata_cache = self.metadata_cache.write().await;
        metadata_cache.remove(&id);
        
        Ok(())
    }
}

#[async_trait]
impl PluginRegistry for PluginRegistryImpl {
    async fn register(&self, plugin: Arc<dyn Plugin>) -> Result<Uuid> {
        let id = plugin.metadata().id;
        
        // Check if plugin with this ID is already registered
        if self.is_registered(id).await {
            return Err(PluginError::RegisterError(format!(
                "Plugin with ID {} is already registered",
                id
            )));
        }
        
        // Index the plugin
        self.index_plugin(&plugin).await?;
        
        // Register the plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(id, plugin);
        
        info!("Registered plugin with ID {}", id);
        
        Ok(id)
    }
    
    async fn unregister(&self, id: Uuid) -> Result<()> {
        // Check if plugin is registered
        if !self.is_registered(id).await {
            return Err(PluginError::NotFound(id));
        }
        
        // Remove from index
        self.remove_from_index(id).await?;
        
        // Remove from registry
        let mut plugins = self.plugins.write().await;
        plugins.remove(&id);
        
        info!("Unregistered plugin with ID {}", id);
        
        Ok(())
    }
    
    async fn get(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        match plugins.get(&id) {
            Some(plugin) => Ok(plugin.clone()),
            None => Err(PluginError::NotFound(id)),
        }
    }
    
    async fn get_all(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let mut result = Vec::with_capacity(plugins.len());
        
        for plugin in plugins.values() {
            result.push(plugin.clone());
        }
        
        Ok(result)
    }
    
    async fn get_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Plugin>>> {
        let capabilities = self.capabilities.read().await;
        let plugins = self.plugins.read().await;
        
        match capabilities.get(capability) {
            Some(plugin_ids) => {
                let mut result = Vec::with_capacity(plugin_ids.len());
                
                for id in plugin_ids {
                    if let Some(plugin) = plugins.get(id) {
                        result.push(plugin.clone());
                    }
                }
                
                Ok(result)
            }
            None => Ok(Vec::new()),
        }
    }
    
    async fn get_by_tag(&self, tag: &str) -> Result<Vec<Arc<dyn Plugin>>> {
        let tags = self.tags.read().await;
        let plugins = self.plugins.read().await;
        
        match tags.get(tag) {
            Some(plugin_ids) => {
                let mut result = Vec::with_capacity(plugin_ids.len());
                
                for id in plugin_ids {
                    if let Some(plugin) = plugins.get(id) {
                        result.push(plugin.clone());
                    }
                }
                
                Ok(result)
            }
            None => Ok(Vec::new()),
        }
    }
    
    async fn is_registered(&self, id: Uuid) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(&id)
    }
    
    async fn get_metadata(&self, id: Uuid) -> Result<PluginMetadata> {
        let metadata_cache = self.metadata_cache.read().await;
        match metadata_cache.get(&id) {
            Some(metadata) => Ok(metadata.clone()),
            None => {
                // Try to get from plugin
                let plugins = self.plugins.read().await;
                match plugins.get(&id) {
                    Some(plugin) => Ok(plugin.metadata().clone()),
                    None => Err(PluginError::NotFound(id)),
                }
            }
        }
    }
    
    async fn get_all_metadata(&self) -> Result<Vec<PluginMetadata>> {
        let metadata_cache = self.metadata_cache.read().await;
        let mut result = Vec::with_capacity(metadata_cache.len());
        
        for metadata in metadata_cache.values() {
            result.push(metadata.clone());
        }
        
        Ok(result)
    }
}

/// Plugin manager
///
/// This struct provides high-level functionality for managing plugins.
#[derive(Debug)]
pub struct PluginManager {
    /// Plugin registry
    registry: Arc<dyn PluginRegistry>,
    
    /// Plugin lifecycle manager
    lifecycle: Arc<dyn PluginLifecycle>,
    
    /// Plugin discovery
    discovery: Option<Arc<dyn PluginDiscovery>>,
    
    /// Plugin security manager
    security: Arc<dyn PluginSecurityManager>,
    
    /// Configuration
    config: PluginManagerConfig,
}

impl PluginManager {
    /// Create a new plugin manager with default configuration
    pub fn new() -> Self {
        Self::with_config(PluginManagerConfig::default())
    }
    
    /// Create a new plugin manager with custom configuration
    pub fn with_config(config: PluginManagerConfig) -> Self {
        let registry = Arc::new(PluginRegistryImpl::new());
        let lifecycle = Arc::new(PluginLifecycleManager::new());
        let security = Arc::new(PluginSecurityValidator::new(config.security_level.clone(), config.resource_limits.clone()));
        
        // Create plugin discovery if enabled
        let discovery = if config.enable_discovery {
            None // Will be created in initialize
        } else {
            None
        };
        
        Self {
            registry,
            lifecycle,
            discovery,
            security,
            config,
        }
    }
    
    /// Initialize the plugin manager
    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize discovery if enabled
        if self.config.enable_discovery {
            // Will be created in a future update
        }
        
        // Initialize security manager
        
        Ok(())
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<Uuid> {
        // Validate plugin security
        self.security.validate_plugin(&plugin).await?;
        
        // Register plugin
        let id = self.registry.register(plugin.clone()).await?;
        
        Ok(id)
    }
    
    /// Unregister a plugin
    pub async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.registry.get(id).await?;
        
        // Stop plugin if started
        if let Ok(state) = self.lifecycle.get_state(id).await {
            if state == LifecycleState::Started {
                self.lifecycle.stop_plugin(plugin.clone()).await?;
            }
        }
        
        // Unregister plugin
        self.registry.unregister(id).await?;
        
        Ok(())
    }
    
    /// Initialize a plugin
    pub async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.registry.get(id).await?;
        
        // Initialize plugin
        self.lifecycle.initialize_plugin(plugin).await?;
        
        Ok(())
    }
    
    /// Start a plugin
    pub async fn start_plugin(&self, id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.registry.get(id).await?;
        
        // Start plugin
        self.lifecycle.start_plugin(plugin).await?;
        
        Ok(())
    }
    
    /// Stop a plugin
    pub async fn stop_plugin(&self, id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.registry.get(id).await?;
        
        // Stop plugin
        self.lifecycle.stop_plugin(plugin).await?;
        
        Ok(())
    }
    
    /// Get a plugin by ID
    pub async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        self.registry.get(id).await
    }
    
    /// Get all plugins
    pub async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.registry.get_all().await
    }
    
    /// Get plugins by capability
    pub async fn get_plugins_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Plugin>>> {
        self.registry.get_by_capability(capability).await
    }
    
    /// Get plugins by tag
    pub async fn get_plugins_by_tag(&self, tag: &str) -> Result<Vec<Arc<dyn Plugin>>> {
        self.registry.get_by_tag(tag).await
    }
    
    /// Get plugin metadata
    pub async fn get_plugin_metadata(&self, id: Uuid) -> Result<PluginMetadata> {
        self.registry.get_metadata(id).await
    }
    
    /// Get all plugin metadata
    pub async fn get_all_plugin_metadata(&self) -> Result<Vec<PluginMetadata>> {
        self.registry.get_all_metadata().await
    }
    
    /// Get plugin status
    pub async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        // Get plugin state
        let state = self.lifecycle.get_state(id).await?;
        
        // Map state to status
        let status = match state {
            LifecycleState::Created => PluginStatus::Created,
            LifecycleState::Loading => PluginStatus::Loading,
            LifecycleState::Loaded => PluginStatus::Loaded,
            LifecycleState::Initializing => PluginStatus::Initializing,
            LifecycleState::Initialized => PluginStatus::Initialized,
            LifecycleState::Starting => PluginStatus::Starting,
            LifecycleState::Started => PluginStatus::Started,
            LifecycleState::Stopping => PluginStatus::Stopping,
            LifecycleState::Stopped => PluginStatus::Stopped,
            LifecycleState::Unloading => PluginStatus::Unloading,
            LifecycleState::Unloaded => PluginStatus::Unloaded,
            LifecycleState::Failed => PluginStatus::Failed,
        };
        
        Ok(status)
    }
    
    /// Get plugin lifecycle history
    pub async fn get_plugin_history(&self, id: Uuid) -> Result<Vec<crate::plugins::lifecycle::LifecycleEvent>> {
        self.lifecycle.get_history(id).await
    }
    
    /// Check if a plugin is registered
    pub async fn is_plugin_registered(&self, id: Uuid) -> bool {
        self.registry.is_registered(id).await
    }
} 