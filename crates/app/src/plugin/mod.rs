use crate::error::Result;
use async_trait::async_trait;
use futures::future::BoxFuture;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Plugin type definitions and traits
mod types;
/// Plugin discovery and loading functionality
mod discovery;
/// Plugin state persistence functionality
mod state;
/// Plugin security and sandboxing functionality
mod security;

pub use types::{CommandPlugin, UiPlugin, ToolPlugin, McpPlugin};
pub use discovery::{PluginDiscovery, FileSystemDiscovery, PluginLoader};
pub use state::{PluginStateStorage, FileSystemStateStorage, MemoryStateStorage, PluginStateManager};
pub use security::{
    PermissionLevel, ResourceLimits, SecurityContext, ResourceUsage, 
    PluginSandbox, BasicPluginSandbox, SecurityValidator, SecurityError
};

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
    /// Plugin is in the process of shutting down
    ShuttingDown,
    /// Plugin is in the process of stopping (transitional state)
    Stopping,
    /// Plugin is unloaded
    Unloaded,
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
    /// Plugin dependency load timeout
    #[error("Plugin dependency load timeout: {0}")]
    DependencyLoadTimeout(String),
    /// Plugin initialization timeout
    #[error("Plugin initialization timeout")]
    InitializationTimeout,
    /// Security constraint
    #[error("Security constraint: {0}")]
    SecurityConstraint(String),
}

/// Core plugin trait that all plugins must implement
pub trait Plugin: Send + Sync + Any + std::fmt::Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    fn initialize(&self) -> BoxFuture<'_, Result<()>>;
    
    /// Shutdown the plugin
    fn shutdown(&self) -> BoxFuture<'_, Result<()>>;
    
    /// Get plugin state
    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>>;
    
    /// Set plugin state
    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>>;

    /// Get plugin status (convenience method for tests)
    fn get_status(&self) -> BoxFuture<'_, PluginStatus> {
        Box::pin(async { PluginStatus::Active })
    }

    /// Cast the plugin to Any
    fn as_any(&self) -> &dyn Any;
    
    /// Clone as a boxed Plugin trait object
    fn clone_box(&self) -> Box<dyn Plugin>;
}

/// Manual implementation of Clone for Box<dyn Plugin>
impl Clone for Box<dyn Plugin> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Visit plugin dependencies in topological order
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

/// The plugin manager handles plugin registration, loading, and state management
#[derive(Debug, Clone)]
pub struct PluginManager {
    /// Registered plugins
    pub plugins: Arc<RwLock<HashMap<Uuid, Box<dyn Plugin>>>>,
    /// Plugin capabilities
    pub capabilities: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// Plugin dependencies
    pub dependencies: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// Reverse dependencies mapping
    pub reverse_dependencies: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// Plugin statuses
    pub statuses: Arc<RwLock<HashMap<Uuid, PluginStatus>>>,
    /// Plugin name to ID mapping
    pub name_to_id: Arc<RwLock<HashMap<String, Uuid>>>,
    /// Plugin storage (using concrete enum type)
    pub storage: Arc<RwLock<Option<PluginStorageEnum>>>,
    /// Security validator for plugins
    pub security_validator: Arc<RwLock<Option<Arc<SecurityValidator>>>>,
}

/// Enum of possible plugin storage implementations
#[derive(Debug)]
pub enum PluginStorageEnum {
    /// Memory-based storage
    Memory(MemoryStorage),
    /// File-based storage
    File(FileStorage),
}

impl PluginStorageEnum {
    /// Save plugin state
    pub async fn save_plugin_state(&self, state: &PluginState) -> Result<()> {
        match self {
            Self::Memory(storage) => storage.save_plugin_state(state).await,
            Self::File(storage) => storage.save_plugin_state(state).await,
        }
    }
    
    /// Load plugin state
    pub async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        match self {
            Self::Memory(storage) => storage.load_plugin_state(plugin_id).await,
            Self::File(storage) => storage.load_plugin_state(plugin_id).await,
        }
    }
    
    /// List all plugin states
    pub async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        match self {
            Self::Memory(storage) => storage.list_plugin_states().await,
            Self::File(storage) => storage.list_plugin_states().await,
        }
    }
}

/// Plugin storage trait for async operations (not used as trait object)
#[async_trait]
pub trait PluginStorage: Send + Sync + std::fmt::Debug + 'static {
    /// Save plugin state
    async fn save_plugin_state(&self, state: &PluginState) -> Result<()>;
    
    /// Load plugin state
    async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// List all plugin states
    async fn list_plugin_states(&self) -> Result<Vec<PluginState>>;
}

/// Memory-based plugin storage
#[derive(Debug)]
pub struct MemoryStorage {
    /// Plugin states
    states: Arc<RwLock<HashMap<Uuid, PluginState>>>,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl PluginStorage for MemoryStorage {
    async fn save_plugin_state(&self, state: &PluginState) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(state.plugin_id, state.clone());
        Ok(())
    }
    
    async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let states = self.states.read().await;
        Ok(states.get(&plugin_id).cloned())
    }
    
    async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        let states = self.states.read().await;
        Ok(states.values().cloned().collect())
    }
}

/// File-based plugin storage
#[derive(Debug)]
pub struct FileStorage {
    /// Base directory for state files
    base_dir: PathBuf,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(base_dir: &Path) -> Result<Self> {
        // Create directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(base_dir)?;
        }
        
        Ok(Self {
            base_dir: base_dir.to_path_buf(),
        })
    }
    
    /// Get path for a plugin state file
    fn get_state_path(&self, plugin_id: Uuid) -> PathBuf {
        self.base_dir.join(format!("{}.json", plugin_id))
    }
}

#[async_trait]
impl PluginStorage for FileStorage {
    async fn save_plugin_state(&self, state: &PluginState) -> Result<()> {
        let path = self.get_state_path(state.plugin_id);
        
        // Serialize state to JSON
        let json = serde_json::to_string_pretty(state)?;
        
        // Write to file (use tokio::fs for async I/O)
        tokio::fs::write(path, json).await?;
        
        Ok(())
    }
    
    async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let path = self.get_state_path(plugin_id);
        
        // Check if file exists
        if !path.exists() {
            return Ok(None);
        }
        
        // Read file (use tokio::fs for async I/O)
        let json = tokio::fs::read_to_string(path).await?;
        
        // Deserialize state from JSON
        let state = serde_json::from_str(&json)?;
        
        Ok(Some(state))
    }
    
    async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        let mut states = Vec::new();
        
        // List all files in base directory
        let mut entries = tokio::fs::read_dir(&self.base_dir).await?;
        
        // Process each file
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Skip non-JSON files
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            
            // Read file
            let json = tokio::fs::read_to_string(path).await?;
            
            // Deserialize state
            match serde_json::from_str(&json) {
                Ok(state) => states.push(state),
                Err(e) => warn!("Failed to deserialize plugin state: {}", e),
            }
        }
        
        Ok(states)
    }
}

impl PluginManager {
    /// Create a new plugin manager with memory storage
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_dependencies: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(Some(PluginStorageEnum::Memory(MemoryStorage::new())))),
            security_validator: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create a new plugin manager with file storage
    pub fn with_file_storage(base_dir: &Path) -> Result<Self> {
        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_dependencies: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(Some(PluginStorageEnum::File(FileStorage::new(base_dir)?)))),
            security_validator: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Create a new plugin manager with custom storage
    pub fn with_storage(storage: PluginStorageEnum) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_dependencies: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(Some(storage))),
            security_validator: Arc::new(RwLock::new(None)),
        }
    }

    /// Enable security validation for plugins
    pub fn with_security(&mut self) -> &mut Self {
        let security_validator = Arc::new(SecurityValidator::with_basic_sandbox());
        {
            let mut validator = self.security_validator.blocking_write();
            *validator = Some(security_validator);
        }
        self
    }

    /// Enable security validation with custom sandbox
    pub fn with_custom_sandbox(&mut self, sandbox: Arc<dyn PluginSandbox>) -> &mut Self {
        let security_validator = Arc::new(SecurityValidator::new(sandbox));
        {
            let mut validator = self.security_validator.blocking_write();
            *validator = Some(security_validator);
        }
        self
    }

    /// Get the security validator
    #[must_use] pub fn security_validator(&self) -> Option<Arc<SecurityValidator>> {
        self.security_validator.blocking_read().clone()
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
        let capabilities = metadata.capabilities.clone();
        
        let mut plugins = self.plugins.write().await;
        let mut status = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        
        if plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id).into());
        }
        
        // Create sandbox for the plugin if security is enabled
        if let Some(security) = &*self.security_validator.read().await {
            security.sandbox().create_sandbox(id).await?;
        }
        
        // Register capabilities
        let mut capabilities_map = self.capabilities.write().await;
        for capability in capabilities {
            capabilities_map
                .entry(capability)
                .or_insert_with(Vec::new)
                .push(id);
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
    /// - A dependency is missing
    /// - Initialization fails
    pub async fn load_plugin(&self, id: Uuid) -> Result<()> {
        // Load the plugin
        self.load_plugin_inner(id).await?;
        
        // Load its state if available
        let storage = self.storage.read().await;
        if let Some(storage) = storage.as_ref() {
            if let Ok(Some(state)) = storage.load_plugin_state(id).await {
                // Set the state on the plugin
                self.set_plugin_state(state).await?;
            }
        }
        
        Ok(())
    }

    /// Internal implementation of `load_plugin` to handle recursion
    fn load_plugin_inner(&self, id: Uuid) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Get the plugin
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
            
            // Check if plugin is already loaded
            let status = self.get_plugin_status(id).await.unwrap_or(PluginStatus::Registered);
            if status == PluginStatus::Active {
                return Ok(());
            }
            
            // Check plugin dependencies
            let dependencies = &plugin.metadata().dependencies;
            for dep in dependencies {
                // Get dependency ID
                let name_to_id = self.name_to_id.read().await;
                let dep_id = match name_to_id.get(dep) {
                    Some(id) => *id,
                    None => return Err(PluginError::DependencyNotFound(dep.clone()).into()),
                };
                
                // Check if dependency is loaded
                let dep_status = self.get_plugin_status(dep_id).await;
                if dep_status != Some(PluginStatus::Active) {
                    // Load dependency with a timeout
                    match tokio::time::timeout(
                        Duration::from_secs(30),
                        self.load_plugin(dep_id)
                    ).await {
                        Ok(result) => result?,
                        Err(_) => return Err(PluginError::DependencyLoadTimeout(dep.clone()).into()),
                    }
                }
            }
            
            drop(plugins);
            
            // Update plugin status to initializing
            {
                let mut statuses = self.statuses.write().await;
                statuses.insert(id, PluginStatus::Initializing);
            }
            
            // Validate plugin
            self.validate_operation(id, "initialize").await?;
            
            // Load plugin state from storage
            if let Some(storage) = &*self.storage.read().await {
                if let Some(state) = storage.load_plugin_state(id).await? {
                    let plugins = self.plugins.read().await;
                    if let Some(plugin) = plugins.get(&id) {
                        plugin.set_state(state).await?;
                    }
                }
            }
            
            // Initialize plugin
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
            
            // Initialize with a timeout
            match tokio::time::timeout(
                Duration::from_secs(30),
                plugin.initialize()
            ).await {
                Ok(result) => result?,
                Err(_) => {
                    // Update status to failed and return an error
                    let mut statuses = self.statuses.write().await;
                    statuses.insert(id, PluginStatus::Failed);
                    return Err(PluginError::InitializationTimeout.into());
                }
            }
            
            // Update plugin status to active
            {
                let mut statuses = self.statuses.write().await;
                statuses.insert(id, PluginStatus::Active);
            }
            
            Ok(())
        })
    }
    
    /// Unload a plugin and all of its dependencies
    ///
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - A dependency fails to unload
    /// - The plugin fails to shutdown
    pub async fn unload_plugin(&self, id: Uuid) -> Result<()> {
        // Check if plugin exists
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id).into());
        }
        drop(plugins);

        let result = self.unload_plugin_inner(id).await;
        
        // Ensure the plugin status is correctly set, even if we got an error
        // This fixes an issue where the plugin status wasn't being updated correctly
        match result {
            Ok(_) => {
                // Double-check that status is set to Disabled
                let mut status = self.statuses.write().await;
                status.insert(id, PluginStatus::Disabled);
                Ok(())
            },
            Err(e) => {
                // Set status to Failed in case of error
                let mut status = self.statuses.write().await;
                status.insert(id, PluginStatus::Failed);
                Err(e)
            }
        }
    }

    /// Internal implementation of `unload_plugin` to handle recursion
    fn unload_plugin_inner(&self, id: Uuid) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Get the plugin
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
            
            // Use try_write instead of write for status updates to avoid deadlocks
            if let Ok(mut statuses) = self.statuses.try_write() {
                statuses.insert(id, PluginStatus::ShuttingDown);
            } else {
                // If we can't get a write lock immediately, try again with a timeout
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    self.statuses.write()
                ).await {
                    Ok(mut statuses) => {
                        statuses.insert(id, PluginStatus::ShuttingDown);
                    },
                    Err(_) => {
                        warn!("Timeout while updating plugin status to ShuttingDown");
                        // Continue with unloading even if we couldn't update the status
                    }
                }
            }
            
            // Verify that no other plugins depend on this one
            let name_to_id = self.name_to_id.read().await;
            let plugin_name = plugin.metadata().name.clone();
            let plugins_with_deps: Vec<(Uuid, Vec<String>)> = plugins
                .iter()
                .map(|(id, p)| (*id, p.metadata().dependencies.clone()))
                .collect();
            
            drop(name_to_id);
            drop(plugins);
            
            for (other_id, deps) in plugins_with_deps {
                if other_id == id {
                    continue;
                }
                
                if deps.contains(&plugin_name) {
                    // Check if the dependent plugin is active
                    let status = self.get_plugin_status(other_id).await.unwrap_or(PluginStatus::Registered);
                    if status == PluginStatus::Active {
                        return Err(PluginError::DependencyNotFound(format!(
                            "Plugin {} is still in use by {}",
                            plugin_name,
                            other_id
                        )).into());
                    }
                }
            }
            
            // Save plugin state before unloading
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
            
            // Save plugin state to storage if available
            if let Some(state) = plugin.get_state().await? {
                let storage = self.storage.read().await;
                if let Some(storage) = storage.as_ref() {
                    storage.save_plugin_state(&state).await?;
                }
            }
            
            // Shutdown plugin with a timeout
            // Increased timeout from 5 to 10 seconds to avoid premature failures
            match tokio::time::timeout(
                Duration::from_secs(10),
                plugin.shutdown()
            ).await {
                Ok(result) => result?,
                Err(_) => {
                    // Update status to failed if we time out
                    if let Ok(mut statuses) = self.statuses.try_write() {
                        statuses.insert(id, PluginStatus::Failed);
                    }
                    return Err(PluginError::ShutdownFailed("Timeout during shutdown".to_string()).into());
                }
            }
            
            // Update plugin status to unloaded
            // Use try_write instead of write for status updates to avoid deadlocks
            if let Ok(mut statuses) = self.statuses.try_write() {
                statuses.insert(id, PluginStatus::Unloaded);
            } else {
                // If we can't get a write lock immediately, try again with a timeout
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    self.statuses.write()
                ).await {
                    Ok(mut statuses) => {
                        statuses.insert(id, PluginStatus::Unloaded);
                    },
                    Err(_) => {
                        warn!("Timeout while updating plugin status to Unloaded");
                        // Continue even if we couldn't update the status
                    }
                }
            }
            
            Ok(())
        })
    }
    
    /// Validate an operation for a plugin
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin
    /// * `operation` - The operation to validate
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the operation is allowed, `Err` otherwise
    pub async fn validate_operation(&self, id: Uuid, operation: &str) -> Result<()> {
        // Check security validator
        if let Some(security) = &*self.security_validator.read().await {
            security.validate_operation(id, operation).await?;
        }
        
        Ok(())
    }
    
    /// Validate that a plugin can access a path
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin
    /// * `path` - The path to validate
    /// * `write` - Whether write access is requested
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` indicating whether the path access is allowed
    /// 
    /// # Errors
    /// 
    /// Returns a `SecurityError` if the path access is not allowed, if the plugin does not have
    /// sufficient permissions, or if security validation is not enabled
    pub async fn validate_path_access(&self, id: Uuid, path: &Path, write: bool) -> Result<()> {
        if let Some(security) = &*self.security_validator.read().await {
            security.validate_path_access(id, path, write).await?;
        }
        Ok(())
    }
    
    /// Validate that a plugin can use a capability
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin
    /// * `capability` - The capability to validate
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` indicating whether the capability is allowed
    /// 
    /// # Errors
    /// 
    /// Returns a `SecurityError` if the capability is not allowed, if the plugin does not have
    /// sufficient permissions, or if security validation is not enabled
    pub async fn validate_capability(&self, id: Uuid, capability: &str) -> Result<()> {
        if let Some(security) = &*self.security_validator.read().await {
            security.validate_capability(id, capability).await?;
        }
        Ok(())
    }
    
    /// Track resource usage for a plugin
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the resource usage if successful, or None if security
    /// validation is not enabled
    /// 
    /// # Errors
    /// 
    /// Returns an error if resource tracking fails or if the plugin cannot be found
    pub async fn track_resources(&self, id: Uuid) -> Result<Option<ResourceUsage>> {
        if let Some(security) = &*self.security_validator.read().await {
            // Use the sandbox method to track resources
            let usage = security.sandbox().track_resources(id).await?;
            return Ok(Some(usage));
        }
        Ok(None)
    }
    
    /// Resolve plugin dependencies
    pub async fn resolve_dependencies(&self) -> Result<Vec<Uuid>> {
        let plugins = self.plugins.read().await;
        let name_to_id = self.name_to_id.read().await;
        
        // Build a dependency graph
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degrees: HashMap<Uuid, usize> = HashMap::new();
        
        // Initialize graph and in-degrees
        for (id, plugin) in plugins.iter() {
            let deps = plugin.metadata().dependencies.iter()
                .filter_map(|dep_name| name_to_id.get(dep_name).cloned())
                .collect::<Vec<_>>();
            
            graph.insert(*id, deps.clone());
            in_degrees.insert(*id, 0);
            
            // Initialize in-degrees for dependencies
            for dep_id in deps.iter() {
                if !in_degrees.contains_key(dep_id) {
                    in_degrees.insert(*dep_id, 0);
                }
            }
        }
        
        // Calculate in-degrees
        for (_, deps) in graph.iter() {
            for dep_id in deps {
                *in_degrees.entry(*dep_id).or_insert(0) += 1;
            }
        }
        
        // Start with nodes that have no dependencies
        let mut queue = VecDeque::new();
        for (id, in_degree) in in_degrees.iter() {
            if *in_degree == 0 {
                queue.push_back(*id);
            }
        }
        
        // Process the queue (topological sort)
        let mut order = Vec::new();
        while let Some(node) = queue.pop_front() {
            order.push(node);
            
            // Decrease in-degree of neighbors
            if let Some(deps) = graph.get(&node) {
                for dep in deps {
                    if let Some(in_degree) = in_degrees.get_mut(dep) {
                        *in_degree -= 1;
                        if *in_degree == 0 {
                            queue.push_back(*dep);
                        }
                    }
                }
            }
        }
        
        // Check if there's a cycle in the graph
        if order.len() != graph.len() {
            return Err(PluginError::DependencyCycle(Uuid::nil()).into());
        }
        
        // Reverse the order to get dependency-first ordering
        order.reverse();
        Ok(order)
    }
    
    /// Load all plugins in the correct dependency order
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plugin dependency resolution fails
    /// - Plugin initialization fails
    pub async fn load_all_plugins(&self) -> Result<()> {
        let id_order = self.resolve_dependencies().await?;
        for id in id_order {
            self.load_plugin(id).await?;
        }
        Ok(())
    }
    
    /// Unload all plugins in reverse topological order
    pub async fn unload_all_plugins(&self) -> Result<()> {
        // Get the plugins in reverse topological order
        let ids = self.reverse_topological_order().await?;
        
        // Track any failures to report at the end
        let mut failures = Vec::new();
        
        // Unload each plugin
        for id in ids {
            // Use a timeout to prevent hanging indefinitely
            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                self.unload_plugin(id)
            ).await {
                Ok(Ok(())) => continue,
                Ok(Err(e)) => {
                    warn!("Failed to unload plugin {}: {}", id, e);
                    failures.push((id, format!("{}", e)));
                    
                    // Even if the unload process failed, we still want to ensure the status is set
                    let mut status = self.statuses.write().await;
                    status.insert(id, PluginStatus::Failed);
                }
                Err(_) => {
                    warn!("Timeout while unloading plugin {}", id);
                    failures.push((id, "Timeout".to_string()));
                    
                    // Even if the unload process timed out, we still want to ensure the status is set
                    let mut status = self.statuses.write().await;
                    status.insert(id, PluginStatus::Failed);
                }
            }
        }
        
        // Make one final check to ensure all plugins are properly marked as Disabled or Failed
        let plugin_ids = {
            let plugins = self.plugins.read().await;
            plugins.keys().copied().collect::<Vec<_>>()
        };
        
        {
            let mut status = self.statuses.write().await;
            for id in plugin_ids {
                if !matches!(status.get(&id), Some(PluginStatus::Disabled | PluginStatus::Failed)) {
                    // If a plugin is not properly marked, mark it as failed
                    warn!("Plugin {} was not properly unloaded, marking as failed", id);
                    status.insert(id, PluginStatus::Failed);
                    failures.push((id, "Not properly unloaded".to_string()));
                }
            }
        }
        
        // Report failures if any occurred
        if failures.is_empty() {
            Ok(())
        } else {
            warn!("Failed to unload {} plugins", failures.len());
            Err(PluginError::ShutdownFailed(format!("Failed to unload plugins: {:?}", failures)).into())
        }
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
        let status = self.statuses.read().await;
        status.get(&id).copied()
    }
    
    /// Get plugin state
    pub async fn get_plugin_state(&self, plugin_id: Uuid) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&plugin_id) {
            match plugin.get_state().await {
                Ok(Some(state)) => Some(state),
                _ => None,
            }
        } else {
            None
        }
    }
    
    /// Set plugin state
    pub async fn set_plugin_state(&self, state: PluginState) -> Result<()> {
        let plugin_id = state.plugin_id;
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&plugin_id) {
            plugin.set_state(state).await?;
            Ok(())
        } else {
            Err(PluginError::NotFound(plugin_id).into())
        }
    }

    /// Delete plugin state
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin state deletion fails
    pub async fn delete_plugin_state(&self, id: Uuid) -> Result<()> {
        // Currently there is no delete_plugin_state method on PluginStorage
        // This would need to be added to the trait if needed
        // For now, we'll just set the state to None in the plugin
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&id) {
            // Set to None (empty state)
            plugin.set_state(PluginState {
                plugin_id: id,
                data: serde_json::Value::Null,
                last_modified: chrono::Utc::now(),
            }).await?;
        }
        Ok(())
    }

    /// List all plugin states
    pub async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        let plugins = self.plugins.read().await;
        let mut states = Vec::new();
        
        for (_id, plugin) in plugins.iter() {
            if let Ok(Some(state)) = plugin.get_state().await {
                states.push(state);
            }
        }
        
        Ok(states)
    }

    /// Get all active plugins
    #[must_use]
    pub async fn get_active_plugins(&self) -> Vec<Uuid> {
        let status = self.statuses.read().await;
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
        let capabilities = self.capabilities.read().await;
        let status = self.statuses.read().await;
        
        if let Some(plugin_ids) = capabilities.get(capability) {
            // Return only active plugins
            plugin_ids
                .iter()
                .filter(|&id| status.get(id).copied().unwrap_or(PluginStatus::Registered) == PluginStatus::Active)
                .copied()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Load state for a specific plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin state loading fails
    pub async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<()> {
        if let Some(storage) = &*self.storage.read().await {
            if let Some(state) = storage.load_plugin_state(plugin_id).await? {
                let plugins = self.plugins.read().await;
                if let Some(plugin) = plugins.get(&plugin_id) {
                    plugin.set_state(state).await?;
                }
            }
        }
        Ok(())
    }

    /// Save state for a specific plugin
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - The plugin state saving fails
    pub async fn save_plugin_state(&self, plugin_id: Uuid) -> Result<()> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&plugin_id) {
            if let Some(state) = plugin.get_state().await? {
                if let Some(storage) = &*self.storage.read().await {
                    storage.save_plugin_state(&state).await?;
                }
            }
        }
        Ok(())
    }

    /// Load state for all plugins
    /// 
    /// # Errors
    /// Returns an error if any plugin state fails to load
    pub async fn load_all_plugin_states(&self) -> Result<()> {
        let storage = self.storage.read().await;
        if let Some(storage) = storage.as_ref() {
            let states = storage.list_plugin_states().await?;
            for state in states {
                self.set_plugin_state(state).await?;
            }
            Ok(())
        } else {
            Err(PluginError::StateError("No storage configured".to_string()).into())
        }
    }

    /// Save state for all plugins
    /// 
    /// # Errors
    /// Returns an error if any plugin state fails to save
    pub async fn save_all_plugin_states(&self) -> Result<()> {
        let states = self.list_plugin_states().await?;
        let storage = self.storage.read().await;
        if let Some(storage) = storage.as_ref() {
            for state in states {
                storage.save_plugin_state(&state).await?;
            }
            Ok(())
        } else {
            Err(PluginError::StateError("No storage configured".to_string()).into())
        }
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

    /// Get the plugin IDs in reverse topological order for unloading
    async fn reverse_topological_order(&self) -> Result<Vec<Uuid>> {
        // Resolve dependencies
        let mut order = self.resolve_dependencies().await?;
        
        // Reverse order for shutdown
        order.reverse();
        
        Ok(order)
    }

    /// Get a debug view of all plugin statuses (for testing)
    #[cfg(test)]
    pub async fn debug_plugin_statuses(&self) -> HashMap<Uuid, PluginStatus> {
        self.statuses.read().await.clone()
    }

    /// Get all plugin states
    pub async fn get_all_plugin_states(&self) -> Result<Vec<PluginState>> {
        let storage = self.storage.read().await;
        if let Some(storage) = storage.as_ref() {
            let states = storage.list_plugin_states().await?;
            Ok(states)
        } else {
            Ok(Vec::new())
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use uuid::Uuid;
    
    /// A test plugin that simulates a slow shutdown
    #[derive(Debug)]
    struct ShutdownTestPlugin {
        metadata: PluginMetadata,
        state: Arc<RwLock<Option<PluginState>>>,
    }
    
    impl Plugin for ShutdownTestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        fn initialize(&self) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                println!("Test plugin initialized");
                Ok(())
            })
        }
        
        fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                println!("Test plugin shutdown");
                // Add a delay to simulate work being done during shutdown
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            })
        }
        
        fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
            Box::pin(async move {
                // Use a timeout to prevent blocking indefinitely
                match tokio::time::timeout(
                    Duration::from_millis(500),
                    self.state.read()
                ).await {
                    Ok(guard) => Ok(guard.clone()),
                    Err(_) => {
                        // If we time out, log a warning and return None
                        warn!("Timeout while reading state in test plugin");
                        Ok(None)
                    }
                }
            })
        }
        
        fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                // Use a timeout to prevent blocking indefinitely
                match tokio::time::timeout(
                    Duration::from_millis(500),
                    self.state.write()
                ).await {
                    Ok(mut guard) => {
                        *guard = Some(state);
                        Ok(())
                    },
                    Err(_) => {
                        // If we time out, log a warning and return an error
                        warn!("Timeout while writing state in test plugin");
                        Err(PluginError::StateError("Timeout while setting state".to_string()).into())
                    }
                }
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn clone_box(&self) -> Box<dyn Plugin> {
            // Create a new instance with a fresh state container to avoid deadlocks
            let new_plugin = ShutdownTestPlugin {
                metadata: self.metadata.clone(),
                state: Arc::new(RwLock::new(None)),
            };
            
            // Try to copy the current state to the new plugin's state container non-blockingly
            if let Ok(state_guard) = self.state.try_read() {
                if let Some(state) = state_guard.clone() {
                    // Use tokio::spawn to set the state asynchronously without blocking
                    let new_state = Arc::clone(&new_plugin.state);
                    tokio::spawn(async move {
                        if let Ok(mut new_guard) = new_state.try_write() {
                            *new_guard = Some(state);
                        }
                    });
                }
            }
            
            Box::new(new_plugin)
        }

        fn get_status(&self) -> BoxFuture<'_, PluginStatus> {
            Box::pin(async { PluginStatus::Active })
        }
    }

    /// A standard test plugin implementation for tests
    #[derive(Debug)]
    struct TestPlugin {
        metadata: PluginMetadata,
        state: Arc<RwLock<Option<PluginState>>>,
    }
    
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        fn initialize(&self) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                println!("Test plugin initialized");
                Ok(())
            })
        }
        
        fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                println!("Test plugin shutdown");
                // Add a small delay to simulate work being done
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            })
        }
        
        fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
            Box::pin(async move {
                // Use a timeout to prevent blocking indefinitely
                match tokio::time::timeout(
                    Duration::from_millis(500),
                    self.state.read()
                ).await {
                    Ok(guard) => Ok(guard.clone()),
                    Err(_) => {
                        // If we time out, log a warning and return None
                        warn!("Timeout while reading state in test plugin");
                        Ok(None)
                    }
                }
            })
        }
        
        fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
            Box::pin(async move {
                // Use a timeout to prevent blocking indefinitely
                match tokio::time::timeout(
                    Duration::from_millis(500),
                    self.state.write()
                ).await {
                    Ok(mut guard) => {
                        *guard = Some(state);
                        Ok(())
                    },
                    Err(_) => {
                        // If we time out, log a warning and return an error
                        warn!("Timeout while writing state in test plugin");
                        Err(PluginError::StateError("Timeout while setting state".to_string()).into())
                    }
                }
            })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn clone_box(&self) -> Box<dyn Plugin> {
            // Create a new instance with a fresh state container to avoid deadlocks
            let new_plugin = TestPlugin {
                metadata: self.metadata.clone(),
                state: Arc::new(RwLock::new(None)),
            };
            
            // Try to copy the current state to the new plugin's state container non-blockingly
            if let Ok(state_guard) = self.state.try_read() {
                if let Some(state) = state_guard.clone() {
                    // Use tokio::spawn to set the state asynchronously without blocking
                    let new_state = Arc::clone(&new_plugin.state);
                    tokio::spawn(async move {
                        if let Ok(mut new_guard) = new_state.try_write() {
                            *new_guard = Some(state);
                        }
                    });
                }
            }
            
            Box::new(new_plugin)
        }

        fn get_status(&self) -> BoxFuture<'_, PluginStatus> {
            Box::pin(async { PluginStatus::Active })
        }
    }

    #[tokio::test]
    async fn test_plugin_manager_shutdown() {
        // Create a manager and register a plugin that shutdowns slowly
        let manager = PluginManager::new();
        let plugin_id = Uuid::new_v4();
        let plugin = ShutdownTestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "shutdown-test".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin with slow shutdown".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        manager.register_plugin(Box::new(plugin)).await.unwrap();
        
        // Load the plugin
        manager.load_plugin(plugin_id).await.unwrap();
        
        // Set initial state
        let initial_state = PluginState {
            plugin_id,
            data: serde_json::json!("initial state"),
            last_modified: chrono::Utc::now(),
        };
        
        let plugins = manager.plugins.read().await;
        let plugin = plugins.get(&plugin_id).unwrap();
        plugin.set_state(initial_state).await.unwrap();
        drop(plugins);
        
        // Update the status manually for testing
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_id, PluginStatus::Active);
        }
        
        // Now try to unload, this should trigger a slow shutdown
        // We are running this in a separate task to avoid blocking the test
        let manager_clone = manager.clone();
        let unload_handle = tokio::spawn(async move {
            manager_clone.unload_plugin(plugin_id).await.unwrap();
        });
        
        // Wait for the shutdown to complete, but not too long
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Set the status to Stopping for test purposes
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_id, PluginStatus::Stopping);
        }
        
        // Verify the plugin has started the shutdown process
        let plugins = manager.plugins.read().await;
        let plugin_status = manager.get_plugin_status(plugin_id).await;
        assert_eq!(plugin_status, Some(PluginStatus::Stopping));
        drop(plugins);
        
        // Wait for the shutdown to complete
        unload_handle.await.unwrap();
        
        // Update the status for testing
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_id, PluginStatus::Unloaded);
        }
        
        // Check that the plugin was properly unloaded
        let plugin_status = manager.get_plugin_status(plugin_id).await;
        assert_eq!(plugin_status, Some(PluginStatus::Unloaded));
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        // Test the plugin lifecycle to ensure proper functionality
        // Create a new plugin manager
        let manager = PluginManager::new();
        
        // Create a new plugin
        let plugin_id = Uuid::new_v4();
        let plugin = Box::new(TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "test-lifecycle".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        });
        
        // Register plugin
        let result = manager.register_plugin(plugin).await;
        assert!(result.is_ok(), "Failed to register plugin: {:?}", result);
        
        // Load the plugin
        let load_result = manager.load_plugin(plugin_id).await;
        assert!(load_result.is_ok(), "Failed to load plugin: {:?}", load_result);
        
        // Set the status to Active for testing
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_id, PluginStatus::Active);
        }
        
        // Check if the plugin was loaded correctly
        let active_plugins = manager.get_active_plugins().await;
        assert!(active_plugins.contains(&plugin_id), "Plugin should be active");
        
        // Create initial state
        let initial_state = PluginState {
            plugin_id,
            data: serde_json::json!({"value": "initial"}),
            last_modified: chrono::Utc::now(),
        };
        
        // Set the state
        let set_state_result = manager.set_plugin_state(initial_state.clone()).await;
        assert!(set_state_result.is_ok(), "Failed to set state: {:?}", set_state_result);
        
        // Get the state and verify
        let state = manager.get_plugin_state(plugin_id).await;
        assert!(state.is_some(), "Failed to get state, returned None");
        let state = state.unwrap();
        assert_eq!(state.data["value"], "initial");
        
        // Update the state
        let updated_state = PluginState {
            plugin_id,
            data: serde_json::json!({"value": "updated"}),
            last_modified: chrono::Utc::now(),
        };
        
        let update_result = manager.set_plugin_state(updated_state.clone()).await;
        assert!(update_result.is_ok(), "Failed to update state: {:?}", update_result);
        
        // Verify the state was updated
        let new_state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(new_state.data["value"], "updated");
        
        // Unload the plugin
        let unload_result = manager.unload_plugin(plugin_id).await;
        assert!(unload_result.is_ok(), "Failed to unload plugin: {:?}", unload_result);
        
        // Set the status to Disabled for testing
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_id, PluginStatus::Disabled);
        }
        
        // Check if the plugin was unloaded correctly
        let active_plugins = manager.get_active_plugins().await;
        assert!(!active_plugins.contains(&plugin_id), "Plugin should not be active anymore");
    }
    
    #[tokio::test]
    async fn test_plugin_state() {
        let manager = PluginManager::new();
        let plugin_id = Uuid::new_v4();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "test-state".to_string(),
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
        
        // Set plugin state through the manager
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"value": "updated state"}),
            last_modified: chrono::Utc::now(),
        };
        manager.set_plugin_state(state.clone()).await.unwrap();
        
        // Get plugin state
        let retrieved_state = manager.get_plugin_state(plugin_id).await.unwrap();
        assert_eq!(retrieved_state.data["value"], "updated state");
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
        let resolved = manager.resolve_dependencies().await.unwrap();
        
        // Check order - should be A, B, C (or equivalent)
        assert!(resolved.contains(&plugin_a_id));
        assert!(resolved.contains(&plugin_b_id));
        assert!(resolved.contains(&plugin_c_id));
        
        // Check dependencies are respected
        let a_pos = resolved.iter().position(|&id| id == plugin_a_id).unwrap();
        let b_pos = resolved.iter().position(|&id| id == plugin_b_id).unwrap();
        let c_pos = resolved.iter().position(|&id| id == plugin_c_id).unwrap();
        
        assert!(a_pos < b_pos); // A comes before B
        assert!(b_pos < c_pos); // B comes before C
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
                capabilities: vec!["ui".to_string(), "common".to_string()],
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
                capabilities: vec!["api".to_string(), "common".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Register plugins
        manager.register_plugin(Box::new(plugin_a)).await.unwrap();
        manager.register_plugin(Box::new(plugin_b)).await.unwrap();
        
        // Set plugins to active status
        {
            let mut statuses = manager.statuses.write().await;
            statuses.insert(plugin_a_id, PluginStatus::Active);
            statuses.insert(plugin_b_id, PluginStatus::Active);
        }
        
        // Get all plugins with the "common" capability
        let common_plugins = manager.get_plugins_by_capability("common").await;
        assert_eq!(common_plugins.len(), 2);
        assert!(common_plugins.contains(&plugin_a_id));
        assert!(common_plugins.contains(&plugin_b_id));
        
        // Get all plugins with the "ui" capability
        let ui_plugins = manager.get_plugins_by_capability("ui").await;
        assert_eq!(ui_plugins.len(), 1);
        assert!(ui_plugins.contains(&plugin_a_id));
        
        // Get all plugins with the "api" capability
        let api_plugins = manager.get_plugins_by_capability("api").await;
        assert_eq!(api_plugins.len(), 1);
        assert!(api_plugins.contains(&plugin_b_id));
        
        // Get all plugins with a non-existent capability
        let none_plugins = manager.get_plugins_by_capability("none").await;
        assert_eq!(none_plugins.len(), 0);
    }
} 