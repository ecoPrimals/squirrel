//! Plugin manager
//!
//! This module provides functionality for managing plugin lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::debug;

use async_trait::async_trait;

use crate::plugin::{Plugin, PluginMetadata};
use crate::PluginStatus;
use crate::discovery::{PluginDiscovery, DefaultPluginDiscovery};
use crate::state::PluginStateManager;
use crate::PluginError;

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
}

/// Plugin manager trait
#[async_trait]
pub trait PluginManagerTrait: PluginRegistry {
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;
    
    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;
    
    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;
    
    /// Load plugins from a directory
    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>>;
    
    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()>;
    
    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()>;
}

/// Plugin manager implementation
pub struct PluginManager {
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins", &"<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>")
            .finish()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    #[must_use] pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
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
        // Use a placeholder plugin instead of HelloWorldPlugin
        use crate::discovery::create_placeholder_plugin;
        let placeholder_metadata = PluginMetadata::new(
            "system-placeholder", 
            "1.0.0", 
            "System placeholder plugin", 
            "Squirrel System"
        );
        let placeholder = create_placeholder_plugin(placeholder_metadata);
        self.register_plugin(placeholder).await?;
        
        Ok(())
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&id) {
            return Err(anyhow::anyhow!("Plugin already registered: {}", id));
        }
        
        plugins.insert(id, plugin);
        Ok(())
    }
    
    /// Get a plugin by ID
    pub async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        match plugins.get(&id) {
            Some(plugin) => Ok(plugin.clone()),
            None => Err(anyhow::anyhow!("Plugin not found: {}", id)),
        }
    }
    
    /// Get all plugins
    pub async fn get_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        // Use the correct import for PluginDiscovery
        use crate::discovery::PluginDiscovery;
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();
        
        let mut _plugin_count = 0;
        for plugin in plugin_paths {
            // Register the plugin directly
            if let Ok(()) = self.register_plugin(plugin.clone()).await {
                let id = plugin.metadata().id;
                ids.push(id);
                _plugin_count += 1;
            }
        }
        
        Ok(ids)
    }
    
    /// Initialize all plugins
    pub async fn initialize(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        for plugin in plugins {
            if let Err(e) = plugin.initialize().await {
                eprintln!("Failed to initialize plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    pub async fn shutdown(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        for plugin in plugins {
            if let Err(e) = plugin.shutdown().await {
                eprintln!("Failed to shutdown plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }
        
        Ok(())
    }
    
    /// Get all components of a specific type
    pub fn get_components<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation,
        // we would iterate through plugins and retrieve components of the specified type
        Ok(Vec::new())
    }
    
    /// Get all endpoints of a specific type
    pub fn get_endpoints<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation,
        // we would iterate through plugins and retrieve endpoints of the specified type
        Ok(Vec::new())
    }

    /// Get a plugin by ID
    pub async fn get_plugin_by_id(&self, id: &Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(id).ok_or(PluginError::NotFound(*id))?;
        Ok(plugin.clone())
    }

    /// Get all plugins
    pub async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }

    /// Execute initialization for all plugins
    pub async fn initialize_all(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        for plugin in plugins.values() {
            if let Err(e) = plugin.initialize().await {
                eprintln!("Failed to initialize plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }
        
        Ok(())
    }

    /// Execute shutdown for all plugins
    pub async fn shutdown_all(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        for plugin in plugins.values() {
            if let Err(e) = plugin.shutdown().await {
                eprintln!("Failed to shutdown plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }
        
        Ok(())
    }

    /// Load plugins from a directory using the given `PluginDiscovery` 
    pub async fn load_plugins_with_discovery<D: PluginDiscovery + Send + Sync>(
        &self,
        discovery: &D,
        directory: &str,
    ) -> Result<usize> {
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut count = 0;
        
        // Since we don't have a load_plugin method in PluginDiscovery trait,
        // we'll directly use the discovered plugins
        // Assuming discover_plugins returns Vec<Box<dyn Plugin>> as per trait definition
        for plugin in plugin_paths {
            // Convert Box<dyn Plugin> to Arc<dyn Plugin>
            let arc_plugin = Arc::from(plugin);
            self.register_plugin(arc_plugin).await?;
            count += 1;
        }
        
        Ok(count)
    }

    /// Initialize plugins with default configuration
    async fn initialize_plugins_default(&self) -> Result<()> {
        // Create and register a placeholder plugin
        use crate::discovery::create_placeholder_plugin;
        let placeholder_metadata = PluginMetadata::new(
            "system-placeholder", 
            "1.0.0", 
            "System placeholder plugin", 
            "Squirrel System"
        );
        let placeholder = create_placeholder_plugin(placeholder_metadata);
        self.register_plugin(placeholder).await?;
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default plugin manager implementation
pub struct DefaultPluginManager {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
    
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    
    /// Plugin state manager
    state_manager: Arc<dyn PluginStateManager + Send + Sync>,
}

impl std::fmt::Debug for DefaultPluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultPluginManager")
            .field("plugins", &"<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>")
            .field("statuses", &"<RwLock<HashMap<Uuid, PluginStatus>>>")
            .field("name_to_id", &"<RwLock<HashMap<String, Uuid>>>")
            .field("state_manager", &"<Arc<dyn PluginStateManager>>")
            .finish()
    }
}

impl DefaultPluginManager {
    /// Create a new plugin manager
    pub fn new(state_manager: Arc<dyn PluginStateManager>) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            state_manager,
        }
    }
    
    /// Check if a plugin has dependencies
    async fn check_dependencies(&self, plugin: &Arc<dyn Plugin>) -> Result<()> {
        let plugins = self.plugins.read().await;
        let metadata = plugin.metadata();
        
        for dependency_id in &metadata.dependencies {
            let mut found = false;
            
            for (plugin_id, _) in plugins.iter() {
                if plugin_id == dependency_id {
                    found = true;
                    break;
                }
            }
            
            if !found {
                return Err(PluginError::DependencyNotFound(dependency_id.to_string()).into());
            }
        }
        
        Ok(())
    }
    
    /// Check for dependency cycles
    async fn check_dependency_cycles(&self, plugin_id: Uuid, path: &mut Vec<Uuid>) -> Result<()> {
        if path.contains(&plugin_id) {
            return Err(PluginError::DependencyCycle(plugin_id).into());
        }
        
        path.push(plugin_id);
        
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&plugin_id) {
            let metadata = plugin.metadata();
            
            for dependency_id in &metadata.dependencies {
                // Use Box::pin to handle recursive async calls
                let future = self.check_dependency_cycles(*dependency_id, path);
                Box::pin(future).await?;
            }
        }
        
        path.pop();
        
        Ok(())
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginManager {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        let name = metadata.name.clone();
        
        let mut plugins = self.plugins.write().await;
        
        if plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id).into());
        }
        
        plugins.insert(id, plugin);
        
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Registered);
        
        let mut name_to_id = self.name_to_id.write().await;
        name_to_id.insert(name, id);
        
        Ok(())
    }
    
    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let mut statuses = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        let plugin = plugins.remove(&id).ok_or(PluginError::NotFound(id))?;
        let name = plugin.metadata().name.clone();
        
        statuses.remove(&id);
        name_to_id.remove(&name);
        
        debug!("Unregistered plugin: {}", id);
        
        Ok(())
    }
    
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(plugin.clone())
    }
    
    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        let name_to_id = self.name_to_id.read().await;
        let id = name_to_id.get(name).ok_or_else(|| PluginError::PluginNotFound(name.to_string()))?;
        PluginRegistry::get_plugin(self, *id).await
    }
    
    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }
    
    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        let status = statuses.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(*status)
    }
    
    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id).into());
        }
        
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
        Ok(())
    }
    
    /// Get all registered plugins
    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }
}

#[async_trait]
impl PluginManagerTrait for DefaultPluginManager {
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(plugin.clone())
    }
    
    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        let status = statuses.get(&id).cloned();

        // Check if plugin is already active
        if let Some(status) = status {
            if status == PluginStatus::Initialized {
                return Ok(());
            }
        }

        // Get plugin using explicit trait qualification
        let plugin = PluginRegistry::get_plugin(self, id).await.or_else(|_| Err(anyhow::anyhow!("Plugin not found")))?;

        // Initialize plugin
        match plugin.initialize().await {
            Ok(_) => {
                statuses.insert(id, PluginStatus::Initialized);
                Ok(())
            }
            Err(err) => {
                statuses.insert(id, PluginStatus::Failed);
                Err(err)
            }
        }
    }
    
    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        let status = statuses.get(&id).cloned();

        // Check if plugin is not active
        if let Some(status) = status {
            if status != PluginStatus::Initialized {
                return Ok(());
            }
        }

        // Get plugin using explicit trait qualification
        let plugin = PluginRegistry::get_plugin(self, id).await.or_else(|_| Err(anyhow::anyhow!("Plugin not found")))?;

        // Shutdown plugin
        match plugin.shutdown().await {
            Ok(_) => {
                statuses.insert(id, PluginStatus::Unloaded);
                Ok(())
            }
            Err(err) => {
                // Keep plugin active on error
                Err(err)
            }
        }
    }
    
    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        let status = statuses.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(*status)
    }
    
    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id).into());
        }
        
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
        Ok(())
    }
    
    /// Load plugins from a directory
    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        // Use the correct import for PluginDiscovery
        use crate::discovery::PluginDiscovery;
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();
        
        let mut _plugin_count = 0;
        for plugin in plugin_paths {
            // Register the plugin directly
            if let Ok(()) = self.register_plugin(plugin.clone()).await {
                let id = plugin.metadata().id;
                ids.push(id);
                _plugin_count += 1;
            }
        }
        
        Ok(ids)
    }
    
    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let ids: Vec<Uuid> = plugins.keys().copied().collect();
        drop(plugins);
        
        for id in ids {
            self.initialize_plugin(id).await?;
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let ids: Vec<Uuid> = plugins.keys().copied().collect();
        drop(plugins);
        
        for id in ids {
            // Try to shutdown but don't fail if one plugin fails
            let _ = self.shutdown_plugin(id).await;
        }
        
        Ok(())
    }
} 