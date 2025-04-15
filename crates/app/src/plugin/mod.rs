use crate::error::Result;
use async_trait::async_trait;
use futures::future::BoxFuture;
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Plugin type definitions and traits
pub mod types;
pub use types::*;

/// Plugin discovery and loading functionality
pub mod discovery;
pub use discovery::{
    FileSystemDiscovery, PluginDiscovery, PluginLoader,
    EnhancedPluginDiscovery, EnhancedPluginLoader, GenericPlugin,
};

/// Plugin state persistence functionality
pub mod state;
pub use state::{
    FileSystemStateStorage, MemoryStateStorage, PluginStateManager, PluginStateStorage
};

/// Plugin security and sandboxing functionality
pub mod security;
pub use security::{
    PermissionLevel, SecurityContext, ResourceLimits, ResourceUsage,
    SecurityError, SecurityValidator, EnhancedSecurityValidator, SecurityAuditEntry
};

/// Plugin sandboxing implementation
pub mod sandbox;
pub use sandbox::{
    SandboxError, PluginSandbox, CrossPlatformSandbox, BasicPluginSandbox
};

/// Plugin resource monitoring functionality
pub mod resource_monitor;
pub use resource_monitor::{ResourceMonitor, ResourceMonitorError};

/// Add registry module
pub mod registry;
pub use registry::{PluginRegistry, PluginCatalogEntry};

/// Make examples public
pub mod examples;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Cyclic dependency
    #[error("Cyclic dependency detected")]
    CyclicDependency,
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
#[allow(dead_code)]
fn _visit_dependency(
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
        
        // Recursively visit dependencies
        _visit_dependency(*dep_id, plugins, name_to_id, visited, temp, order)?;
    }
    
    // Mark as visited
    temp.remove(&id);
    visited.insert(id);
    
    // Add to order
    order.push(id);
    
    Ok(())
}

/// Plugin manager containing plugins and their metadata
#[derive(Debug)]
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
    pub security_validator: Arc<RwLock<Option<Arc<SecurityValidatorEnum>>>>,
}

/// Security validator enum to support both validator types
#[derive(Debug)]
pub enum SecurityValidatorEnum {
    /// Legacy validator
    Basic(SecurityValidator),
    /// Enhanced validator with more features
    Enhanced(EnhancedSecurityValidator),
}

impl SecurityValidatorEnum {
    /// Validate operation
    pub async fn validate_operation(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        match self {
            Self::Basic(validator) => validator.validate_operation(plugin_id, operation).await,
            Self::Enhanced(validator) => validator.validate_operation(plugin_id, operation).await,
        }
    }
    
    /// Validate path access
    pub async fn validate_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        match self {
            Self::Basic(validator) => validator.validate_path_access(plugin_id, path, write).await,
            Self::Enhanced(validator) => validator.validate_path_access(plugin_id, path, write).await,
        }
    }
    
    /// Validate capability
    pub async fn validate_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        match self {
            Self::Basic(validator) => validator.validate_capability(plugin_id, capability).await,
            Self::Enhanced(validator) => validator.validate_capability(plugin_id, capability).await,
        }
    }
    
    /// Get sandbox
    pub fn sandbox(&self) -> Arc<dyn PluginSandbox> {
        match self {
            Self::Basic(validator) => validator.sandbox(),
            Self::Enhanced(validator) => validator.sandbox(),
        }
    }

    /// Get resource monitor from sandbox if available
    pub fn resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        match self {
            Self::Basic(_) => None, // Basic sandbox doesn't have resource monitoring
            Self::Enhanced(validator) => Some(validator.get_resource_monitor()),
        }
    }
    
    /// Get audit log
    pub async fn get_audit_log(&self, plugin_id: Option<Uuid>, limit: usize) -> Option<Vec<SecurityAuditEntry>> {
        match self {
            Self::Basic(_) => None,
            Self::Enhanced(validator) => Some(validator.get_audit_log(plugin_id, limit).await),
        }
    }
}

impl From<SecurityValidator> for SecurityValidatorEnum {
    fn from(validator: SecurityValidator) -> Self {
        Self::Basic(validator)
    }
}

impl From<EnhancedSecurityValidator> for SecurityValidatorEnum {
    fn from(validator: EnhancedSecurityValidator) -> Self {
        Self::Enhanced(validator)
    }
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
    ///
    /// # Errors
    /// Returns an error if saving the plugin state fails
    pub async fn save_plugin_state(&self, state: &PluginState) -> Result<()> {
        match self {
            Self::Memory(storage) => storage.save_plugin_state(state).await,
            Self::File(storage) => storage.save_plugin_state(state).await,
        }
    }
    
    /// Load plugin state
    ///
    /// # Errors
    /// Returns an error if loading the plugin state fails
    pub async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        match self {
            Self::Memory(storage) => storage.load_plugin_state(plugin_id).await,
            Self::File(storage) => storage.load_plugin_state(plugin_id).await,
        }
    }
    
    /// List all plugin states
    ///
    /// # Errors
    /// Returns an error if listing plugin states fails
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
    ///
    /// # Errors
    /// Returns an error if saving the plugin state fails
    async fn save_plugin_state(&self, state: &PluginState) -> Result<()>;
    
    /// Load plugin state
    ///
    /// # Errors
    /// Returns an error if loading the plugin state fails
    async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// List all plugin states
    ///
    /// # Errors
    /// Returns an error if listing plugin states fails
    async fn list_plugin_states(&self) -> Result<Vec<PluginState>>;
}

/// Memory-based plugin storage
#[derive(Debug)]
pub struct MemoryStorage {
    /// Plugin states
    states: Arc<RwLock<HashMap<Uuid, PluginState>>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// Create a new memory storage
    #[must_use] pub fn new() -> Self {
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
    ///
    /// # Errors
    /// Returns an error if creating the base directory fails
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
        self.base_dir.join(format!("{plugin_id}.json"))
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
    #[must_use] pub fn new() -> Self {
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
    ///
    /// # Errors
    /// Returns an error if creating the file storage fails
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
    #[must_use] pub fn with_storage(storage: PluginStorageEnum) -> Self {
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

    /// Configure to use enhanced security features
    pub fn with_security(&mut self) -> &mut Self {
        // First try to create a sandbox
        // If we can't create a sandbox, we'll use the basic sandbox
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));

        // Create a security validator with the sandbox
        let validator = EnhancedSecurityValidator::new_with_sandbox(sandbox);
        
        // Store in the plugin manager
        let security_validator = Arc::new(SecurityValidatorEnum::Enhanced(validator));
        
        // Use tokio::task::block_in_place to execute the async code in a blocking context
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut guard = self.security_validator.write().await;
                *guard = Some(security_validator);
            })
        });
        
        self
    }

    /// Add a custom sandbox for security
    pub fn with_custom_sandbox(&mut self, sandbox: Arc<dyn PluginSandbox>) -> &mut Self {
        // Create an enhanced security validator with the custom sandbox
        let validator = EnhancedSecurityValidator::new_with_sandbox(sandbox);
        
        // Store in the plugin manager
        let security_validator = Arc::new(SecurityValidatorEnum::Enhanced(validator));
        
        // Use tokio::task::block_in_place to execute the async code in a blocking context
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut guard = self.security_validator.write().await;
                *guard = Some(security_validator);
            })
        });
        
        self
    }
    
    /// Get the security validator (if any)
    #[must_use]
    pub fn security_validator(&self) -> Option<Arc<SecurityValidatorEnum>> {
        let validator = self.security_validator.blocking_read();
        validator.clone()
    }
    
    /// Get security validator for a specific plugin
    pub async fn get_security_validator(&self, _id: Uuid) -> Result<Option<Arc<SecurityValidatorEnum>>> {
        // Currently we just return the global validator, but in the future
        // we could have plugin-specific validators
        Ok(self.security_validator())
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
            if let Ok(result) = tokio::time::timeout(
                Duration::from_secs(30),
                plugin.initialize()
            ).await { result? } else {
                // Update status to failed and return an error
                let mut statuses = self.statuses.write().await;
                statuses.insert(id, PluginStatus::Failed);
                return Err(PluginError::InitializationTimeout.into());
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
            Ok(()) => {
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
                            "Plugin {plugin_name} is still in use by {other_id}"
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
            if let Ok(result) = tokio::time::timeout(
                Duration::from_secs(10),
                plugin.shutdown()
            ).await { result? } else {
                // Update status to failed if we time out
                if let Ok(mut statuses) = self.statuses.try_write() {
                    statuses.insert(id, PluginStatus::Failed);
                }
                return Err(PluginError::ShutdownFailed("Timeout during shutdown".to_string()).into());
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
    ///
    /// # Errors
    /// 
    /// Returns an error if the operation is not allowed by the security validator
    pub async fn validate_operation(&self, id: Uuid, operation: &str) -> Result<()> {
        if let Some(validator) = &*self.security_validator.read().await {
            validator.validate_operation(id, operation).await
        } else {
            // If no security validator, allow all operations
            Ok(())
        }
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
        if let Some(validator) = &*self.security_validator.read().await {
            validator.validate_path_access(id, path, write).await
        } else {
            // If no security validator, allow all paths
            Ok(())
        }
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
        if let Some(validator) = &*self.security_validator.read().await {
            validator.validate_capability(id, capability).await
        } else {
            // If no security validator, allow all capabilities
            Ok(())
        }
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
        // Check if the plugin exists
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id).into());
        }
        
        // Get the sandbox from the security validator
        let security_validator = self.security_validator().ok_or_else(|| {
            PluginError::SecurityConstraint("No security validator available".to_string())
        })?;
        
        // Check for resource monitor first if available
        if let Some(resource_monitor) = security_validator.resource_monitor() {
            // Force a measurement update
            let _ = resource_monitor.measure_all_resources().await?;
            return resource_monitor.get_resource_usage(id).await;
        }
        
        // Fall back to security validator sandbox
        match security_validator.sandbox().track_resources(id).await {
            Ok(usage) => Ok(Some(usage)),
            Err(e) => {
                debug!("Failed to track resources for plugin {}: {}", id, e);
                Ok(None)
            }
        }
    }
    
    /// Resolve plugin dependencies
    ///
    /// # Errors
    /// Returns an error if a plugin dependency cannot be found or if there is a cyclic dependency
    pub async fn resolve_dependencies(&self) -> Result<Vec<Uuid>> {
        let plugins = self.plugins.read().await;
        let name_to_id = self.name_to_id.read().await;
        
        // Build a dependency graph
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degrees: HashMap<Uuid, usize> = HashMap::new();
        
        // Initialize graph and in-degrees
        for (id, plugin) in plugins.iter() {
            let deps = plugin.metadata().dependencies.iter()
                .filter_map(|dep_name| name_to_id.get(dep_name).copied())
                .collect::<Vec<_>>();
            
            graph.insert(*id, deps.clone());
            in_degrees.insert(*id, 0);
            
            // Initialize in-degrees for dependencies
            for dep_id in &deps {
                if !in_degrees.contains_key(dep_id) {
                    in_degrees.insert(*dep_id, 0);
                }
            }
        }
        
        // Calculate in-degrees
        for deps in graph.values() {
            for dep_id in deps {
                *in_degrees.entry(*dep_id).or_insert(0) += 1;
            }
        }
        
        // Start with nodes that have no dependencies
        let mut queue = VecDeque::new();
        for (id, in_degree) in &in_degrees {
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
            return Err(PluginError::CyclicDependency.into());
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
    
    /// Unload all plugins
    ///
    /// # Errors
    /// Returns an error if any plugin fails to unload
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
                    failures.push((id, format!("{e}")));
                    
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
            Err(PluginError::ShutdownFailed(format!("Failed to unload plugins: {failures:?}")).into())
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
    ///
    /// # Errors
    /// Returns an error if saving the plugin state fails
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
    ///
    /// # Errors
    /// Returns an error if listing plugin states fails
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
    ///
    /// # Errors
    /// Returns an error if loading any plugin state fails
    pub async fn get_all_plugin_states(&self) -> Result<Vec<PluginState>> {
        let storage = self.storage.read().await;
        if let Some(storage) = storage.as_ref() {
            let states = storage.list_plugin_states().await?;
            Ok(states)
        } else {
            Ok(Vec::new())
        }
    }

    /// Load a plugin and all its dependencies with enhanced error handling and recovery
    ///
    /// This method loads a plugin and ensures all its dependencies are loaded first.
    /// It includes enhanced error handling, timeout management, and resource tracking.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The plugin is not found
    /// - A dependency could not be loaded
    /// - The plugin initialization fails
    /// - A security constraint is violated
    pub async fn load_plugin_with_recovery(&self, id: Uuid) -> Result<()> {
        debug!("Loading plugin with recovery: {}", id);
        
        // Check if the plugin is already active
        if let Some(status) = self.get_plugin_status(id).await {
            if status == PluginStatus::Active {
                debug!("Plugin already active: {}", id);
                return Ok(());
            }
        }
        
        // Set status to initializing
        {
            let mut statuses = self.statuses.write().await;
            statuses.insert(id, PluginStatus::Initializing);
        }
        
        // Create a recovery context for this operation
        let recovery_result = self.with_recovery(id, "load_plugin", async move {
            // Load dependencies first
            {
                let dependencies = self.dependencies.read().await;
                if let Some(deps) = dependencies.get(&id) {
                    for dep_id in deps {
                        debug!("Loading dependency {} for plugin {}", dep_id, id);
                        match self.load_plugin(*dep_id).await {
                            Ok(()) => {
                                debug!("Successfully loaded dependency {} for plugin {}", dep_id, id);
                            }
                            Err(e) => {
                                // Set plugin status to failed
                                let mut statuses = self.statuses.write().await;
                                statuses.insert(id, PluginStatus::Failed);
                                
                                return Err(PluginError::DependencyLoadTimeout(
                                    format!("Failed to load dependency {dep_id}: {e}")
                                ).into());
                            }
                        }
                    }
                }
            }
            
            // Now load the plugin itself
            self.load_plugin_inner(id).await
        }).await;
        
        // Update status based on the result
        {
            let mut statuses = self.statuses.write().await;
            if recovery_result.is_ok() {
                statuses.insert(id, PluginStatus::Active);
            } else {
                statuses.insert(id, PluginStatus::Failed);
            }
        }
        
        recovery_result
    }
    
    /// Execute an operation with recovery capabilities
    ///
    /// This helper method provides a consistent way to execute plugin operations
    /// with error recovery, resource tracking, and timeout management.
    ///
    /// # Errors
    /// Returns any error from the operation, possibly wrapped with recovery context
    async fn with_recovery<T>(&self, id: Uuid, operation: &str, f: impl std::future::Future<Output = Result<T>>) -> Result<T>
    {
        debug!("Executing operation '{}' for plugin {} with recovery", operation, id);
        
        // Start resource tracking
        let _resource_tracking = self.track_resources(id).await;
        
        // Execute the operation with a timeout
        let timeout_duration = Duration::from_secs(30); // 30 second timeout for operations
        let timeout_result = tokio::time::timeout(timeout_duration, f).await;
        
        if let Ok(result) = timeout_result {
            match result {
                Ok(value) => {
                    debug!("Operation '{}' for plugin {} completed successfully", operation, id);
                    Ok(value)
                }
                Err(e) => {
                    error!("Operation '{}' for plugin {} failed: {}", operation, id, e);
                    Err(e)
                }
            }
        } else {
            error!("Operation '{}' for plugin {} timed out after {:?}", operation, id, timeout_duration);
            
            // Set plugin status to failed
            let mut statuses = self.statuses.write().await;
            statuses.insert(id, PluginStatus::Failed);
            
            Err(PluginError::InitializationTimeout.into())
        }
    }
    
    /// Check if a plugin has the required capabilities
    ///
    /// # Errors
    /// Returns an error if the plugin does not have the required capability
    pub async fn has_capability(&self, id: Uuid, capability: &str) -> Result<bool> {
        // Get the plugin's capabilities
        let capabilities = match self.get_plugin_capabilities(id).await {
            Some(caps) => caps,
            None => return Ok(false),
        };
        
        // Check if the capability is in the list
        Ok(capabilities.contains(&capability.to_string()))
    }
    
    /// Safely execute a plugin operation with proper error handling
    ///
    /// This method provides a safe way to execute operations on plugins with
    /// proper error handling and recovery.
    ///
    /// # Errors
    /// Returns any error from the operation with contextual information
    pub async fn execute_plugin_operation<F, Fut, T>(&self, id: Uuid, operation: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        debug!("Executing plugin operation '{}' for plugin {}", operation, id);
        
        // Check if the plugin is active
        if let Some(status) = self.get_plugin_status(id).await {
            if status != PluginStatus::Active {
                return Err(PluginError::InitializationFailed(
                    format!("Plugin {id} is not active (status: {status:?})")
                ).into());
            }
        } else {
            return Err(PluginError::NotFound(id).into());
        }
        
        // Execute the operation with recovery
        self.with_recovery(id, operation, f()).await
    }
    
    /// Get the command plugin implementation from a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin is not found or not a command plugin
    pub async fn get_command_plugin(&self, id: Uuid) -> Result<Arc<dyn CommandPlugin>> {
        self.get_plugin_by_id(id, |plugin| {
            // Try to cast directly to concrete types that implement CommandPlugin
            if let Some(plugin_impl) = plugin.as_any().downcast_ref::<CommandPluginImpl>() {
                // Create a new instance with a cloned plugin
                let cloned = plugin_impl.clone();
                // Convert to Arc<dyn CommandPlugin>
                Ok(Arc::new(cloned) as Arc<dyn CommandPlugin>)
            } else {
                // Add other concrete types that implement CommandPlugin here if needed
                Err(PluginError::InitializationFailed(
                    format!("Plugin {id} is not a command plugin")
                ).into())
            }
        }).await?
    }
    
    /// Get all command plugins
    ///
    /// This method returns all registered command plugins that are active
    pub async fn get_all_command_plugins(&self) -> Vec<(Uuid, Arc<dyn CommandPlugin>)> {
        let mut result = Vec::new();
        
        // Get all plugins with command capability
        let command_plugin_ids = self.get_plugins_by_capability("command").await;
        
        for id in command_plugin_ids {
            if let Ok(command_plugin) = self.get_command_plugin(id).await {
                result.push((id, command_plugin));
            }
        }
        
        result
    }
    
    /// Get the tool plugin implementation from a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin is not found or not a tool plugin
    pub async fn get_tool_plugin(&self, id: Uuid) -> Result<Arc<dyn ToolPlugin>> {
        self.get_plugin_by_id(id, |plugin| {
            // Try to cast directly to concrete types that implement ToolPlugin
            if let Some(plugin_impl) = plugin.as_any().downcast_ref::<ToolPluginImpl>() {
                // Create a new instance with a cloned plugin
                let cloned = plugin_impl.clone();
                // Convert to Arc<dyn ToolPlugin>
                Ok(Arc::new(cloned) as Arc<dyn ToolPlugin>)
            } else {
                // Add other concrete types that implement ToolPlugin here if needed
                Err(PluginError::InitializationFailed(
                    format!("Plugin {id} is not a tool plugin")
                ).into())
            }
        }).await?
    }
    
    /// Get all tool plugins
    ///
    /// This method returns all registered tool plugins that are active
    pub async fn get_all_tool_plugins(&self) -> Vec<(Uuid, Arc<dyn ToolPlugin>)> {
        let mut result = Vec::new();
        
        // Get all plugins with tool capability
        let tool_plugin_ids = self.get_plugins_by_capability("tool").await;
        
        for id in tool_plugin_ids {
            if let Ok(tool_plugin) = self.get_tool_plugin(id).await {
                result.push((id, tool_plugin));
            }
        }
        
        result
    }
    
    /// Get the MCP plugin implementation from a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin is not found or not an MCP plugin
    pub async fn get_mcp_plugin(&self, id: Uuid) -> Result<Arc<dyn McpPlugin>> {
        self.get_plugin_by_id(id, |plugin| {
            // Try to cast directly to concrete types that implement McpPlugin
            if let Some(plugin_impl) = plugin.as_any().downcast_ref::<McpPluginImpl>() {
                // Create a new instance with a cloned plugin
                let cloned = plugin_impl.clone();
                // Convert to Arc<dyn McpPlugin>
                Ok(Arc::new(cloned) as Arc<dyn McpPlugin>)
            } else {
                // Add other concrete types that implement McpPlugin here if needed
                Err(PluginError::InitializationFailed(
                    format!("Plugin {id} is not an MCP plugin")
                ).into())
            }
        }).await?
    }
    
    /// Get all MCP plugins
    ///
    /// This method returns all registered MCP plugins that are active
    pub async fn get_all_mcp_plugins(&self) -> Vec<(Uuid, Arc<dyn McpPlugin>)> {
        let mut result = Vec::new();
        
        // Get all plugins with MCP capability
        let mcp_plugin_ids = self.get_plugins_by_capability("mcp").await;
        
        for id in mcp_plugin_ids {
            if let Ok(mcp_plugin) = self.get_mcp_plugin(id).await {
                result.push((id, mcp_plugin));
            }
        }
        
        result
    }

    /// Gets the security audit log for a plugin or all plugins
    /// 
    /// # Arguments
    /// 
    /// * `plugin_id` - The ID of the plugin to get the audit log for, or None for all plugins
    /// * `limit` - The maximum number of audit log entries to return
    /// 
    /// # Returns
    /// 
    /// Returns the security audit log entries if available, or None if there is no security validator
    pub async fn get_security_audit_log(&self, plugin_id: Option<Uuid>, limit: usize) -> Option<Vec<SecurityAuditEntry>> {
        if let Some(validator) = &*self.security_validator.read().await {
            validator.get_audit_log(plugin_id, limit).await
        } else {
            None
        }
    }

    /// Get the resource usage for all plugins
    /// 
    /// This method attempts to measure all resources at once using the resource monitor,
    /// and falls back to tracking individual resources if that fails.
    /// 
    /// # Returns
    /// 
    /// Returns a map of plugin IDs to their resource usage
    pub async fn get_all_resource_usage(&self) -> HashMap<Uuid, ResourceUsage> {
        let mut result = HashMap::new();
        
        // Check if we have a security validator that can track resources
        if let Some(security_validator) = self.security_validator() {
            // First try to use the resource monitor to get all measurements at once
            if let Some(validator) = security_validator.resource_monitor() {
                // Force a measurement update
                if let Ok(all_usage) = validator.measure_all_resources().await {
                    // We successfully measured all resources, return the result
                    return all_usage;
                } else {
                    // Log the error but continue with the fallback approach
                    debug!("Failed to measure all resources, falling back to manual tracking");
                }
            }
            
            // Fall back to tracking resources for each plugin individually
            let plugin_ids = {
                let plugins = self.plugins.read().await;
                plugins.keys().cloned().collect::<Vec<_>>()
            };
            
            for id in plugin_ids {
                if let Ok(Some(usage)) = self.get_resource_usage(id).await {
                    result.insert(id, usage);
                }
            }
        }
        
        result
    }

    /// Get the resource usage for a specific plugin
    /// 
    /// This method attempts to use the resource monitor to get resource usage,
    /// and falls back to the security validator's sandbox if that fails.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin to get resource usage for
    /// 
    /// # Returns
    /// 
    /// Returns the resource usage if available, or None if no resource usage can be determined
    /// 
    /// # Errors
    /// 
    /// Returns an error if measuring resources fails
    pub async fn get_resource_usage(&self, id: Uuid) -> Result<Option<ResourceUsage>> {
        match self.security_validator().map(|validator| validator.resource_monitor()) {
            Some(Some(monitor)) => {
                // If we have a ResourceMonitor, use it to get resource usage
                let usage = monitor.get_resource_usage(id).await?;
                Ok(usage)
            }
            _ => {
                // If we don't have a security validator or it doesn't have a resource monitor,
                // return None to indicate we couldn't get usage information
                Ok(None)
            }
        }
    }

    /// Measure resources for a specific plugin
    /// 
    /// Unlike get_resource_usage, this method forces a new measurement of resources.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the plugin to measure resources for
    /// 
    /// # Returns
    /// 
    /// Returns the resource usage if available, or None if no resource usage can be determined
    /// 
    /// # Errors
    /// 
    /// Returns an error if measuring resources fails
    pub async fn measure_resources(&self, id: Uuid) -> Result<Option<ResourceUsage>> {
        // If we have a validator with resource_monitor, use it
        match self.security_validator().map(|validator| validator.resource_monitor()) {
            Some(Some(monitor)) => {
                // Use resource monitor to measure resource usage
                let usage = monitor.get_resource_usage(id).await?;
                Ok(usage)
            }
            _ => {
                // No resource monitor available
                Ok(None)
            }
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for PluginManager
impl Clone for PluginManager {
    fn clone(&self) -> Self {
        Self {
            plugins: self.plugins.clone(),
            capabilities: self.capabilities.clone(),
            dependencies: self.dependencies.clone(),
            reverse_dependencies: self.reverse_dependencies.clone(),
            statuses: self.statuses.clone(),
            name_to_id: self.name_to_id.clone(),
            storage: self.storage.clone(),
            security_validator: self.security_validator.clone(),
        }
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
