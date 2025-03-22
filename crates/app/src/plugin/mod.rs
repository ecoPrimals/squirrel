use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{debug, error, info, warn};
use crate::error::Result;

/// Plugin type definitions and traits
mod types;
/// Plugin discovery and loading functionality
mod discovery;
/// Plugin state persistence functionality
mod state;

pub use types::{CommandPlugin, UiPlugin, ToolPlugin, McpPlugin};
pub use discovery::{PluginDiscovery, FileSystemDiscovery, PluginLoader};
pub use state::{PluginStateStorage, FileSystemStateStorage, MemoryStateStorage, PluginStateManager};

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
    /// Plugin is initializing
    Initializing,
    /// Plugin is shutting down
    ShuttingDown,
}

/// Plugin initialization error
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(Uuid),
    /// Plugin already registered
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(Uuid),
    /// Plugin dependency not found
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),
    /// Plugin dependency cycle detected
    #[error("Plugin dependency cycle detected: {0}")]
    DependencyCycle(Uuid),
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    /// Plugin shutdown failed
    #[error("Plugin shutdown failed: {0}")]
    ShutdownFailed(String),
    /// Plugin state error
    #[error("Plugin state error: {0}")]
    StateError(String),
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
    /// Plugin state manager
    state_manager: PluginStateManager,
    /// Plugin name to ID mapping
    name_to_id: Arc<RwLock<HashMap<String, Uuid>>>,
}

/// Visits a plugin and its dependencies in topological order
/// 
/// # Errors
/// 
/// Returns an error if a dependency cycle is detected or a dependency is not found
async fn visit_dependency(
    id: Uuid,
    plugins: &HashMap<Uuid, Box<dyn Plugin>>,
    name_to_id: &HashMap<String, Uuid>,
    visited: &mut HashSet<Uuid>,
    temp: &mut HashSet<Uuid>,
    order: &mut Vec<Uuid>,
) -> Result<()> {
    // Check for cycle
    if temp.contains(&id) {
        return Err(PluginError::DependencyCycle(id).into());
    }
    
    // If already visited, skip
    if visited.contains(&id) {
        return Ok(());
    }
    
    // Mark as temporarily visited
    temp.insert(id);
    
    // Get plugin dependencies
    let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
    let dependencies = &plugin.metadata().dependencies;
    
    // Visit all dependencies
    for dep_name in dependencies {
        let dep_id = name_to_id.get(dep_name)
            .ok_or_else(|| PluginError::DependencyNotFound(dep_name.clone()))?;
        
        // Box the future to avoid infinitely sized futures due to recursion
        Box::pin(visit_dependency(*dep_id, plugins, name_to_id, visited, temp, order)).await?;
    }
    
    // Mark as visited
    temp.remove(&id);
    visited.insert(id);
    
    // Add to order
    order.push(id);
    
    Ok(())
}

impl PluginManager {
    /// Create a new plugin manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            state_manager: PluginStateManager::with_memory_storage(),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new plugin manager with file system state storage
    /// 
    /// # Errors
    /// 
    /// Returns an error if the base directory for state storage cannot be created or accessed.
    pub fn with_file_storage(base_dir: std::path::PathBuf) -> Result<Self> {
        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            state_manager: PluginStateManager::with_file_storage(base_dir)?,
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new plugin manager with custom state storage
    #[must_use]
    pub fn with_state_storage(storage: Box<dyn PluginStateStorage>) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            state_manager: PluginStateManager::new(storage),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
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
        let name = metadata.name.clone();
        
        let mut plugins = self.plugins.write().await;
        let mut status = self.status.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        if plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id).into());
        }
        
        plugins.insert(id, plugin);
        status.insert(id, PluginStatus::Registered);
        name_to_id.insert(name, id);
        
        debug!("Registered plugin: {}", id);
        
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
            let current_status = status.get(&id).copied().unwrap_or(PluginStatus::Registered);
            
            if current_status == PluginStatus::Active {
                debug!("Plugin already active: {}", id);
                return Ok(());
            }
            
            if current_status == PluginStatus::Initializing {
                warn!("Plugin already initializing: {}", id);
                return Ok(());
            }
            
            // Mark as initializing
            status.insert(id, PluginStatus::Initializing);
            
            // Try to load state first
            if let Err(e) = self.state_manager.load_state(plugin.as_ref()).await {
                warn!("Failed to load state for plugin {}: {}", id, e);
                // Continue with initialization even if state loading fails
            }
            
            debug!("Initializing plugin: {}", id);
            match plugin.initialize().await {
                Ok(()) => {
                    status.insert(id, PluginStatus::Active);
                    info!("Plugin activated: {}", id);
                    Ok(())
                }
                Err(e) => {
                    status.insert(id, PluginStatus::Failed);
                    error!("Plugin initialization failed: {}: {}", id, e);
                    Err(PluginError::InitializationFailed(e.to_string()).into())
                }
            }
        } else {
            Err(PluginError::NotFound(id).into())
        }
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
            // Check current status
            let current_status = status.get(&id).copied().unwrap_or(PluginStatus::Registered);
            
            if current_status == PluginStatus::Disabled {
                debug!("Plugin already disabled: {}", id);
                return Ok(());
            }
            
            if current_status == PluginStatus::ShuttingDown {
                warn!("Plugin already shutting down: {}", id);
                return Ok(());
            }
            
            // Mark as shutting down
            status.insert(id, PluginStatus::ShuttingDown);
            
            // Save plugin state before shutdown
            if let Err(e) = self.state_manager.save_state(plugin.as_ref()).await {
                warn!("Failed to save state for plugin {}: {}", id, e);
                // Continue with shutdown even if state saving fails
            }
            
            debug!("Shutting down plugin: {}", id);
            match plugin.shutdown().await {
                Ok(()) => {
                    status.insert(id, PluginStatus::Disabled);
                    info!("Plugin disabled: {}", id);
                    Ok(())
                }
                Err(e) => {
                    status.insert(id, PluginStatus::Failed);
                    error!("Plugin shutdown failed: {}: {}", id, e);
                    Err(PluginError::ShutdownFailed(e.to_string()).into())
                }
            }
        } else {
            Err(PluginError::NotFound(id).into())
        }
    }
    
    /// Resolve plugin dependencies and return load order
    /// 
    /// # Errors
    /// Returns an error if:
    /// - A dependency is not found
    /// - A dependency cycle is detected
    pub async fn resolve_dependencies(&self) -> Result<Vec<Uuid>> {
        let plugins = self.plugins.read().await;
        let name_to_id = self.name_to_id.read().await;
        
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();
        let mut order = Vec::new();
        
        // Visit all plugins
        for (&id, _) in plugins.iter() {
            if !visited.contains(&id) {
                visit_dependency(
                    id, 
                    &plugins, 
                    &name_to_id, 
                    &mut visited, 
                    &mut temp, 
                    &mut order
                ).await?;
            }
        }
        
        Ok(order)
    }
    
    /// Load all plugins in the correct dependency order
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plugin dependency resolution fails
    /// - Plugin initialization fails
    pub async fn load_all_plugins(&self) -> Result<()> {
        // Resolve dependencies
        let order = self.resolve_dependencies().await?;
        
        // Load plugins in order
        for id in order {
            self.load_plugin(id).await?;
        }
        
        Ok(())
    }
    
    /// Unload all plugins in reverse dependency order
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plugin dependency resolution fails
    /// - Plugin shutdown fails
    pub async fn unload_all_plugins(&self) -> Result<()> {
        // Resolve dependencies
        let mut order = self.resolve_dependencies().await?;
        
        // Reverse order for shutdown
        order.reverse();
        
        // Unload plugins in reverse order
        for id in order {
            self.unload_plugin(id).await?;
        }
        
        Ok(())
    }
    
    /// Get plugin by ID
    ///
    /// # Errors
    /// Returns an error if the plugin is not found
    pub async fn get_plugin_by_id<F, R>(&self, id: Uuid, f: F) -> Result<R>
    where
        F: FnOnce(&Box<dyn Plugin>) -> R,
    {
        self.with_plugin(id, f).await
    }
    
    /// Execute a function with a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin is not found
    pub async fn with_plugin<F, R>(&self, id: Uuid, f: F) -> Result<R>
    where
        F: FnOnce(&Box<dyn Plugin>) -> R,
    {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(f(plugin))
    }
    
    /// Get plugin by name
    ///
    /// # Errors
    /// Returns an error if the plugin is not found
    pub async fn get_plugin_by_name<F, R>(&self, name: &str, f: F) -> Result<R>
    where
        F: FnOnce(&Box<dyn Plugin>) -> R,
    {
        let name_to_id = self.name_to_id.read().await;
        let id = name_to_id.get(name).ok_or_else(|| PluginError::DependencyNotFound(name.to_string()))?;
        self.with_plugin(*id, f).await
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
        match self.state_manager.load_plugin_state(id).await {
            Ok(state) => state,
            Err(e) => {
                error!("Failed to load plugin state: {}: {}", id, e);
                None
            }
        }
    }
    
    /// Set plugin state
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin state update fails
    /// - The plugin is not found
    pub async fn set_plugin_state(&self, state: PluginState) -> Result<()> {
        self.state_manager.save_plugin_state(&state).await
    }

    /// Delete plugin state
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin state deletion fails
    pub async fn delete_plugin_state(&self, id: Uuid) -> Result<()> {
        self.state_manager.delete_plugin_state(id).await
    }

    /// List all plugin states
    /// 
    /// # Errors
    /// Returns an error if the state listing fails
    pub async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        self.state_manager.list_plugin_states().await
    }

    /// Get all active plugins
    #[must_use]
    pub async fn get_active_plugins(&self) -> Vec<Uuid> {
        let status = self.status.read().await;
        status
            .iter()
            .filter(|(_, &s)| s == PluginStatus::Active)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get plugin capabilities
    #[must_use]
    pub async fn get_plugin_capabilities(&self, id: Uuid) -> Option<Vec<String>> {
        let plugins = self.plugins.read().await;
        plugins.get(&id).map(|plugin| plugin.metadata().capabilities.clone())
    }

    /// Get plugins by capability
    #[must_use]
    pub async fn get_plugins_by_capability(&self, capability: &str) -> Vec<Uuid> {
        let plugins = self.plugins.read().await;
        let status = self.status.read().await;
        
        plugins
            .iter()
            .filter(|(&id, plugin)| {
                // Check if active and has the capability
                status.get(&id).copied().unwrap_or(PluginStatus::Registered) == PluginStatus::Active
                && plugin.metadata().capabilities.contains(&capability.to_string())
            })
            .map(|(&id, _)| id)
            .collect()
    }

    /// Load state for a specific plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin state loading fails
    pub async fn load_plugin_state(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        if let Some(plugin) = plugins.get(&id) {
            self.state_manager.load_state(plugin.as_ref()).await?;
            debug!("Loaded state for plugin: {}", id);
            Ok(())
        } else {
            Err(PluginError::NotFound(id).into())
        }
    }

    /// Save state for a specific plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin state saving fails
    pub async fn save_plugin_state(&self, id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        if let Some(plugin) = plugins.get(&id) {
            self.state_manager.save_state(plugin.as_ref()).await?;
            debug!("Saved state for plugin: {}", id);
            Ok(())
        } else {
            Err(PluginError::NotFound(id).into())
        }
    }

    /// Load state for all plugins
    /// 
    /// # Errors
    /// Returns an error if any plugin state fails to load
    pub async fn load_all_plugin_states(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin_refs: Vec<&dyn Plugin> = plugins.values()
            .map(std::convert::AsRef::as_ref)
            .collect();
        
        for plugin in plugin_refs {
            if let Err(e) = self.state_manager.load_state(plugin).await {
                warn!("Failed to load state for plugin {}: {}", plugin.metadata().id, e);
            }
        }
        
        info!("Loaded states for all plugins");
        Ok(())
    }

    /// Save state for all plugins
    /// 
    /// # Errors
    /// Returns an error if any plugin state fails to save
    pub async fn save_all_plugin_states(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let plugin_refs: Vec<&dyn Plugin> = plugins.values()
            .map(std::convert::AsRef::as_ref)
            .collect();
        
        for plugin in plugin_refs {
            if let Err(e) = self.state_manager.save_state(plugin).await {
                warn!("Failed to save state for plugin {}: {}", plugin.metadata().id, e);
            }
        }
        
        info!("Saved states for all plugins");
        Ok(())
    }

    /// Safely shut down the plugin manager, saving all plugin states
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Plugin dependency resolution fails
    /// - Plugin shutdown fails
    pub async fn shutdown(&self) -> Result<()> {
        // First save all plugin states
        info!("Saving plugin states before shutdown");
        if let Err(e) = self.save_all_plugin_states().await {
            warn!("Error saving plugin states: {}", e);
            // Continue with shutdown
        }
        
        // Then unload all plugins in reverse dependency order
        info!("Unloading all plugins");
        self.unload_all_plugins().await?;
        
        info!("Plugin manager shutdown complete");
        Ok(())
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
        let plugin_id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(None));
        
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: state.clone(),
        };
        
        // Register plugin
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        
        // Set initial state
        let initial_state = PluginState {
            plugin_id,
            data: serde_json::json!({"initialized": false}),
            last_modified: chrono::Utc::now(),
        };
        
        // Save initial state to storage
        manager.set_plugin_state(initial_state.clone()).await.unwrap();
        
        // Get plugin status
        let id = manager.name_to_id.read().await.get("test").unwrap().clone();
        assert_eq!(manager.get_plugin_status(id).await, Some(PluginStatus::Registered));
        
        // Load plugin (should load state)
        manager.load_plugin(id).await.unwrap();
        assert_eq!(manager.get_plugin_status(id).await, Some(PluginStatus::Active));
        
        // Verify state was loaded
        let loaded_state = state.read().await.clone().expect("State should be loaded");
        assert_eq!(loaded_state.plugin_id, initial_state.plugin_id);
        assert_eq!(loaded_state.data, initial_state.data);
        
        // Update state
        let updated_state = PluginState {
            plugin_id,
            data: serde_json::json!({"initialized": true}),
            last_modified: chrono::Utc::now(),
        };
        
        {
            let mut plugin_state = state.write().await;
            *plugin_state = Some(updated_state.clone());
        }
        
        // Unload plugin (should save state)
        manager.unload_plugin(id).await.unwrap();
        assert_eq!(manager.get_plugin_status(id).await, Some(PluginStatus::Disabled));
        
        // Verify state was saved
        let saved_state = manager.get_plugin_state(id).await.expect("State should be saved");
        assert_eq!(saved_state.data, updated_state.data);
    }
    
    #[tokio::test]
    async fn test_plugin_state() {
        let manager = PluginManager::new();
        let plugin_id = Uuid::new_v4();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugin
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        
        // Set plugin state
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"key": "value"}),
            last_modified: chrono::Utc::now(),
        };
        manager.set_plugin_state(state.clone()).await.unwrap();
        
        // Get plugin state
        let retrieved_state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(retrieved_state.plugin_id, state.plugin_id);
        assert_eq!(retrieved_state.data, state.data);
    }
    
    #[tokio::test]
    async fn test_dependency_resolution() {
        let manager = PluginManager::new();
        
        // Create plugins with dependencies
        let plugin_a_id = Uuid::new_v4();
        let plugin_a = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_a_id,
                name: "plugin-a".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin A".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["a".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_b_id = Uuid::new_v4();
        let plugin_b = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_b_id,
                name: "plugin-b".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin B".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-a".to_string()],
                capabilities: vec!["b".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_c_id = Uuid::new_v4();
        let plugin_c = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_c_id,
                name: "plugin-c".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin C".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-b".to_string()],
                capabilities: vec!["c".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugins
        manager.register_plugin(Box::new(plugin_a)).await.unwrap();
        manager.register_plugin(Box::new(plugin_b)).await.unwrap();
        manager.register_plugin(Box::new(plugin_c)).await.unwrap();
        
        // Resolve dependencies
        let order = manager.resolve_dependencies().await.unwrap();
        
        // Verify order (A -> B -> C)
        assert_eq!(order.len(), 3);
        
        // The plugin with no dependencies should be first
        assert_eq!(order[0], plugin_a_id);
        
        // The plugin that depends on A should be second
        assert_eq!(order[1], plugin_b_id);
        
        // The plugin that depends on B should be last
        assert_eq!(order[2], plugin_c_id);
        
        // Test loading all plugins
        manager.load_all_plugins().await.unwrap();
        
        // Verify all plugins are active
        assert_eq!(manager.get_plugin_status(plugin_a_id).await, Some(PluginStatus::Active));
        assert_eq!(manager.get_plugin_status(plugin_b_id).await, Some(PluginStatus::Active));
        assert_eq!(manager.get_plugin_status(plugin_c_id).await, Some(PluginStatus::Active));
    }
    
    #[tokio::test]
    async fn test_dependency_cycle() {
        let manager = PluginManager::new();
        
        // Create plugins with a dependency cycle
        let plugin_a_id = Uuid::new_v4();
        let plugin_a = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_a_id,
                name: "plugin-a".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin A".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-c".to_string()],
                capabilities: vec!["a".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_b_id = Uuid::new_v4();
        let plugin_b = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_b_id,
                name: "plugin-b".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin B".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-a".to_string()],
                capabilities: vec!["b".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_c_id = Uuid::new_v4();
        let plugin_c = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_c_id,
                name: "plugin-c".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin C".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-b".to_string()],
                capabilities: vec!["c".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugins
        manager.register_plugin(Box::new(plugin_a)).await.unwrap();
        manager.register_plugin(Box::new(plugin_b)).await.unwrap();
        manager.register_plugin(Box::new(plugin_c)).await.unwrap();
        
        // Resolve dependencies should fail due to cycle
        let result = manager.resolve_dependencies().await;
        assert!(result.is_err());
        
        // Should be a dependency cycle error
        match result {
            Err(e) => {
                // Convert to string to check the error type
                let err_str = e.to_string();
                assert!(err_str.contains("dependency cycle"));
            },
            Ok(_) => panic!("Expected dependency cycle error"),
        }
    }
    
    #[tokio::test]
    async fn test_get_plugins_by_capability() {
        let manager = PluginManager::new();
        
        // Create plugins with different capabilities
        let plugin_a_id = Uuid::new_v4();
        let plugin_a = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_a_id,
                name: "plugin-a".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin A".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_b_id = Uuid::new_v4();
        let plugin_b = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_b_id,
                name: "plugin-b".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin B".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["cap2".to_string(), "cap3".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register and activate plugins
        manager.register_plugin(Box::new(plugin_a)).await.unwrap();
        manager.register_plugin(Box::new(plugin_b)).await.unwrap();
        manager.load_plugin(plugin_a_id).await.unwrap();
        manager.load_plugin(plugin_b_id).await.unwrap();
        
        // Get plugins by capability
        let cap1_plugins = manager.get_plugins_by_capability("cap1").await;
        let cap2_plugins = manager.get_plugins_by_capability("cap2").await;
        let cap3_plugins = manager.get_plugins_by_capability("cap3").await;
        
        // Verify correct plugins found
        assert_eq!(cap1_plugins.len(), 1);
        assert_eq!(cap1_plugins[0], plugin_a_id);
        
        assert_eq!(cap2_plugins.len(), 2);
        assert!(cap2_plugins.contains(&plugin_a_id));
        assert!(cap2_plugins.contains(&plugin_b_id));
        
        assert_eq!(cap3_plugins.len(), 1);
        assert_eq!(cap3_plugins[0], plugin_b_id);
    }
    
    #[tokio::test]
    async fn test_plugin_manager_state_persistence() {
        // Create a plugin manager with memory storage
        let manager = PluginManager::new();
        let plugin_id = Uuid::new_v4();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "stateful-plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin with state".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugin
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        
        // Create initial state
        let initial_state = PluginState {
            plugin_id,
            data: serde_json::json!({"counter": 1, "name": "initial"}),
            last_modified: chrono::Utc::now(),
        };
        
        // Set plugin state through plugin instance
        let plugins = manager.plugins.read().await;
        let plugin = plugins.get(&plugin_id).unwrap();
        plugin.set_state(initial_state.clone()).await.unwrap();
        
        // Save plugin state using manager
        manager.save_plugin_state(plugin_id).await.unwrap();
        
        // Clear plugin state to verify loading works
        plugin.set_state(PluginState {
            plugin_id,
            data: serde_json::json!({"counter": 0, "name": "cleared"}),
            last_modified: chrono::Utc::now(),
        }).await.unwrap();
        
        // Load plugin state using manager
        manager.load_plugin_state(plugin_id).await.unwrap();
        
        // Verify state was restored
        let loaded_state = plugin.get_state().await.unwrap().unwrap();
        assert_eq!(loaded_state.plugin_id, initial_state.plugin_id);
        assert_eq!(loaded_state.data, initial_state.data);
        
        // Test direct state access
        let state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(state.data, initial_state.data);
        
        // Test state update through manager
        let updated_state = PluginState {
            plugin_id,
            data: serde_json::json!({"counter": 2, "name": "updated"}),
            last_modified: chrono::Utc::now(),
        };
        manager.set_plugin_state(updated_state.clone()).await.unwrap();
        
        // Verify state was updated
        let final_state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(final_state.data, updated_state.data);
        
        // Test save/load all states
        manager.save_all_plugin_states().await.unwrap();
        manager.load_all_plugin_states().await.unwrap();
    }

    #[tokio::test]
    async fn test_plugin_manager_shutdown() {
        let manager = PluginManager::new();
        
        // Create multiple plugins
        let plugin_a_id = Uuid::new_v4();
        let plugin_a = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_a_id,
                name: "plugin-a".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin A".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["a".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        let plugin_b_id = Uuid::new_v4();
        let plugin_b = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_b_id,
                name: "plugin-b".to_string(),
                version: "0.1.0".to_string(),
                description: "Plugin B".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec!["plugin-a".to_string()],
                capabilities: vec!["b".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugins
        manager.register_plugin(Box::new(plugin_a)).await.unwrap();
        manager.register_plugin(Box::new(plugin_b)).await.unwrap();
        
        // Load all plugins
        manager.load_all_plugins().await.unwrap();
        
        // Verify all plugins are active
        assert_eq!(manager.get_plugin_status(plugin_a_id).await, Some(PluginStatus::Active));
        assert_eq!(manager.get_plugin_status(plugin_b_id).await, Some(PluginStatus::Active));
        
        // Set plugin states
        manager.set_plugin_state(PluginState {
            plugin_id: plugin_a_id,
            data: serde_json::json!({"value": "a-data"}),
            last_modified: chrono::Utc::now(),
        }).await.unwrap();
        
        manager.set_plugin_state(PluginState {
            plugin_id: plugin_b_id,
            data: serde_json::json!({"value": "b-data"}),
            last_modified: chrono::Utc::now(),
        }).await.unwrap();
        
        // Shut down the plugin manager
        manager.shutdown().await.unwrap();
        
        // Verify all plugins are disabled
        assert_eq!(manager.get_plugin_status(plugin_a_id).await, Some(PluginStatus::Disabled));
        assert_eq!(manager.get_plugin_status(plugin_b_id).await, Some(PluginStatus::Disabled));
        
        // Verify states exist
        let states = manager.list_plugin_states().await.unwrap();
        assert_eq!(states.len(), 2);
        
        // Verify specific state values
        let a_state = manager.get_plugin_state(plugin_a_id).await.unwrap();
        let b_state = manager.get_plugin_state(plugin_b_id).await.unwrap();
        
        assert_eq!(a_state.data["value"], "a-data");
        assert_eq!(b_state.data["value"], "b-data");
    }
} 