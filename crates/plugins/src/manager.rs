//! Plugin manager
//!
//! This module provides functionality for managing plugin lifecycle.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use anyhow::Result;

use crate::core::{Plugin, PluginMetadata, PluginStatus, HelloWorldPlugin};
use crate::discovery::{PluginDiscovery, PluginLoader};
use crate::state::{PluginState, PluginStateManager};
use crate::plugin::Plugin as PluginTrait;

/// Plugin registry trait
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<()>;
    
    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Box<dyn Plugin>>;
    
    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Box<dyn Plugin>>;
    
    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Box<dyn Plugin>>>;
    
    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;
    
    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;
}

/// Plugin manager trait
#[async_trait]
pub trait PluginManager: PluginRegistry {
    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Load plugins from a directory
    async fn load_plugins<D: PluginDiscovery + Send + Sync>(
        &self,
        discovery: &D,
        directory: &Path,
    ) -> Result<Vec<Uuid>>;
    
    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()>;
    
    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()>;
}

/// Plugin manager for managing the plugin lifecycle
pub struct PluginManager {
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: RwLock::new(HashMap::new()),
        };
        
        // Register built-in plugins
        if let Err(e) = manager.register_built_in_plugins() {
            eprintln!("Failed to register built-in plugins: {}", e);
        }
        
        manager
    }
    
    /// Register built-in plugins
    fn register_built_in_plugins(&mut self) -> Result<()> {
        // Register the Hello World plugin
        let hello_world = Arc::new(HelloWorldPlugin::new());
        self.register_plugin(hello_world)?;
        
        Ok(())
    }
    
    /// Register a plugin
    pub fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        let mut plugins = self.plugins.write().unwrap();
        if plugins.contains_key(&id) {
            return Err(anyhow::anyhow!("Plugin already registered: {}", id));
        }
        
        plugins.insert(id, plugin);
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get_plugin(&self, id: Uuid) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(&id).cloned()
    }
    
    /// Get all plugins
    pub fn get_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins<P: AsRef<Path>>(&self, dir: P) -> Result<usize> {
        let discovery = PluginDiscovery::new();
        let plugins = discovery.discover_plugins(dir).await?;
        
        let mut count = 0;
        for plugin in plugins {
            self.register_plugin(plugin)?;
            count += 1;
        }
        
        Ok(count)
    }
    
    /// Initialize all plugins
    pub async fn initialize(&self) -> Result<()> {
        let plugins = self.get_plugins();
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
        let plugins = self.get_plugins();
        for plugin in plugins {
            if let Err(e) = plugin.shutdown().await {
                eprintln!("Failed to shutdown plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }
        
        Ok(())
    }
    
    /// Get components of a specific type
    pub async fn get_components<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation, 
        // we would iterate through plugins and retrieve components of the specified type
        Ok(Vec::new())
    }
    
    /// Get endpoints of a specific type
    pub async fn get_endpoints<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation,
        // we would iterate through plugins and retrieve endpoints of the specified type
        Ok(Vec::new())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default plugin manager implementation
#[derive(Debug)]
pub struct DefaultPluginManager {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Box<dyn Plugin>>>,
    
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    
    /// Plugin state manager
    state_manager: Arc<dyn PluginStateManager>,
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
    async fn check_dependencies(&self, plugin: &Box<dyn Plugin>) -> Result<()> {
        let plugins = self.plugins.read().await;
        let metadata = plugin.metadata();
        
        for dependency in &metadata.dependencies {
            let mut found = false;
            
            for other_plugin in plugins.values() {
                let other_metadata = other_plugin.metadata();
                
                if other_metadata.capabilities.contains(dependency) {
                    found = true;
                    break;
                }
            }
            
            if !found {
                return Err(PluginError::DependencyNotFound(dependency.clone()));
            }
        }
        
        Ok(())
    }
    
    /// Visit plugins in dependency order
    async fn visit_dependencies<F>(
        &self,
        plugin_id: Uuid,
        visited: &mut HashSet<Uuid>,
        visiting: &mut HashSet<Uuid>,
        result: &mut Vec<Uuid>,
        f: &F,
    ) -> Result<()>
    where
        F: Fn(&Box<dyn Plugin>) -> Result<bool> + Send + Sync,
    {
        if visited.contains(&plugin_id) {
            return Ok(());
        }
        
        if visiting.contains(&plugin_id) {
            return Err(PluginError::DependencyCycle(plugin_id));
        }
        
        visiting.insert(plugin_id);
        
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&plugin_id).ok_or(PluginError::NotFound(plugin_id))?;
        
        let metadata = plugin.metadata();
        
        // Process dependencies
        for dependency in &metadata.dependencies {
            // Find plugins that provide this capability
            for (other_id, other_plugin) in plugins.iter() {
                let other_metadata = other_plugin.metadata();
                
                if other_metadata.capabilities.contains(dependency) {
                    self.visit_dependencies(*other_id, visited, visiting, result, f).await?;
                }
            }
        }
        
        // Process this plugin
        if f(plugin)? {
            result.push(plugin_id);
        }
        
        visiting.remove(&plugin_id);
        visited.insert(plugin_id);
        
        Ok(())
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginManager {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        let name = metadata.name.clone();
        
        let mut plugins = self.plugins.write().await;
        let mut statuses = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        if plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id));
        }
        
        plugins.insert(id, plugin);
        statuses.insert(id, PluginStatus::Registered);
        name_to_id.insert(name, id);
        
        debug!("Registered plugin: {}", id);
        
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
    async fn get_plugin(&self, id: Uuid) -> Result<Box<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(plugin.clone())
    }
    
    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Box<dyn Plugin>> {
        let name_to_id = self.name_to_id.read().await;
        let id = name_to_id.get(name).ok_or_else(|| {
            PluginError::NotFound(Uuid::nil()) // Using nil UUID for name lookup failure
        })?;
        
        self.get_plugin(*id).await
    }
    
    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Box<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result = plugins.values().cloned().collect();
        Ok(result)
    }
    
    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        let status = statuses.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(status.clone())
    }
    
    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        if !statuses.contains_key(&id) {
            return Err(PluginError::NotFound(id));
        }
        
        statuses.insert(id, status);
        Ok(())
    }
}

#[async_trait]
impl PluginManager for DefaultPluginManager {
    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        
        // Check dependencies
        self.check_dependencies(plugin).await?;
        
        // Update status
        self.set_plugin_status(id, PluginStatus::Initializing).await?;
        
        // Initialize plugin
        match plugin.initialize().await {
            Ok(()) => {
                self.set_plugin_status(id, PluginStatus::Active).await?;
                debug!("Initialized plugin: {}", id);
                Ok(())
            }
            Err(e) => {
                self.set_plugin_status(id, PluginStatus::Failed).await?;
                error!("Failed to initialize plugin {}: {}", id, e);
                Err(PluginError::InitializationFailed(e.to_string()))
            }
        }
    }
    
    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        
        // Update status
        self.set_plugin_status(id, PluginStatus::ShuttingDown).await?;
        
        // Shutdown plugin
        match plugin.shutdown().await {
            Ok(()) => {
                self.set_plugin_status(id, PluginStatus::Unloaded).await?;
                debug!("Shutdown plugin: {}", id);
                Ok(())
            }
            Err(e) => {
                self.set_plugin_status(id, PluginStatus::Failed).await?;
                error!("Failed to shutdown plugin {}: {}", id, e);
                Err(PluginError::ShutdownFailed(e.to_string()))
            }
        }
    }
    
    /// Load plugins from a directory
    async fn load_plugins<D: PluginDiscovery + Send + Sync>(
        &self,
        discovery: &D,
        directory: &Path,
    ) -> Result<Vec<Uuid>> {
        let plugins = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();
        
        for plugin in plugins {
            let id = plugin.metadata().id;
            self.register_plugin(plugin).await?;
            ids.push(id);
        }
        
        Ok(ids)
    }
    
    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        
        let plugins = self.plugins.read().await;
        
        // Sort plugins by dependencies
        for id in plugins.keys() {
            let mut visiting = HashSet::new();
            self.visit_dependencies(
                *id,
                &mut visited,
                &mut visiting,
                &mut result,
                &|_| Ok(true),
            )
            .await?;
        }
        
        // Initialize plugins in dependency order
        for id in result {
            self.initialize_plugin(id).await?;
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        
        let plugins = self.plugins.read().await;
        
        // Sort plugins by reverse dependencies
        for id in plugins.keys() {
            let mut visiting = HashSet::new();
            self.visit_dependencies(
                *id,
                &mut visited,
                &mut visiting,
                &mut result,
                &|_| Ok(true),
            )
            .await?;
        }
        
        // Shutdown plugins in reverse dependency order
        for id in result.into_iter().rev() {
            self.shutdown_plugin(id).await?;
        }
        
        Ok(())
    }
} 