//! Plugin manager
//!
//! This module provides functionality for managing plugin lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::debug;
use log::{info, warn, error};

use async_trait::async_trait;

use crate::plugin::{Plugin, PluginMetadata};
use crate::PluginStatus;
use crate::discovery::{PluginDiscovery, DefaultPluginDiscovery};
use crate::state::PluginStateManager;
use crate::PluginError;
use crate::security::{SecurityManager, SecurityManagerAdapter};
#[cfg(feature = "mcp")]
use squirrel_mcp::security::manager::SecurityManagerImpl;
#[cfg(feature = "mcp")]
use squirrel_mcp::security::crypto::DefaultCryptoProvider;
#[cfg(feature = "mcp")]
use squirrel_mcp::security::auth::DefaultTokenManager;
#[cfg(feature = "mcp")]
use squirrel_mcp::security::identity::DefaultIdentityManager;
#[cfg(feature = "mcp")]
use squirrel_mcp::security::rbac::BasicRBACManager;
#[cfg(feature = "mcp")]
use squirrel_mcp::security::audit::DefaultAuditService;
// Removing unused imports from signature
// use crate::security::signature::{SignatureVerifier, PluginSignature};

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
    security_manager: Arc<SecurityManagerAdapter>,
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins", &"<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>")
            .field("security_manager", &"<Arc<SecurityManagerAdapter>>")
            .finish()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    #[must_use] pub fn new() -> Self {
        // Initialize the security manager
        #[cfg(feature = "mcp")]
        let security_manager = {
            // Create necessary components for the security manager
            let key_storage = Arc::new(InMemoryKeyStorage::new());
            let crypto_provider = Arc::new(DefaultCryptoProvider::new());
            let token_manager = Arc::new(DefaultTokenManager::new(
                key_storage.clone(),
                crypto_provider.clone(),
            ));
            let identity_manager = Arc::new(DefaultIdentityManager::new());
            let rbac_manager = Arc::new(BasicRBACManager::new());
            let audit_service = Arc::new(DefaultAuditService::new());
            
            // Create the MCP security manager with these components
            let mcp_security_manager = Arc::new(SecurityManagerImpl::new(
                crypto_provider,
                token_manager,
                identity_manager,
                rbac_manager,
                audit_service,
            )) as Arc<dyn SecurityManager>;
            
            Arc::new(SecurityManagerAdapter::new(mcp_security_manager))
        };
        
        #[cfg(not(feature = "mcp"))]
        let security_manager = Arc::new(SecurityManagerAdapter::default());
        
        Self {
            plugins: RwLock::new(HashMap::new()),
            security_manager,
        }
    }
    
    /// Get the security manager
    pub async fn get_security_manager(&self) -> Result<Arc<SecurityManagerAdapter>> {
        Ok(self.security_manager.clone())
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
    
    /// Register a plugin with metadata, implementation, and optional signature
    pub async fn register_plugin_with_signature(
        &self, 
        plugin: Arc<dyn Plugin>, 
        signature: Option<Vec<u8>>
    ) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        // If signature is provided, verify it
        if let Some(sig_bytes) = signature {
            debug!("Verifying signature for plugin {}", metadata.name);
            
            // Verify the signature using the security manager
            let verification_result = self.security_manager.verify_signature(metadata, &sig_bytes).await?;
            if !verification_result {
                error!("Signature verification failed for plugin {}", metadata.name);
                return Err(PluginError::SecurityError(
                    format!("Signature verification failed for plugin {}", metadata.name)
                ).into());
            }
            
            debug!("Signature verification succeeded for plugin {}", metadata.name);
        } else {
            // Check if signatures are required by getting the configuration from the signature verifier
            // This is a more complex approach that would require accessing the SignatureVerifier directly
            // For now, we'll just log a warning if no signature is provided
            warn!("No signature provided for plugin {}", metadata.name);
        }
        
        // Perform standard security verification
        if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
            return Err(PluginError::SecurityError(format!("Security verification failed: {}", e)).into());
        }
        
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&id) {
            return Err(anyhow::anyhow!("Plugin already registered: {}", id));
        }
        
        info!("Registered plugin {} ({})", metadata.name, id);
        plugins.insert(id, plugin);
        Ok(())
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        // Register plugin without signature verification
        self.register_plugin_with_signature(plugin, None).await
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
    
    /// Load plugins from a directory with signature verification
    pub async fn load_plugins_with_signatures(&self, directory: &str, verify_signatures: bool) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();
        
        let mut plugin_count = 0;
        for plugin in plugin_paths {
            let metadata = plugin.metadata();
            let id = metadata.id;
            let name = metadata.name.clone(); // Clone the name to use later
            
            // If signature verification is enabled, look for a signature file
            if verify_signatures {
                let sig_path = std::path::Path::new(directory)
                    .join(format!("{}.sig", name));
                
                if sig_path.exists() {
                    // Load the signature
                    let sig_bytes = std::fs::read(&sig_path)?;
                    
                    // Verify signature using security manager
                    if let Ok(verification_result) = self.security_manager.verify_signature(metadata, &sig_bytes).await {
                        if verification_result {
                            // Verify security
                            if let Ok(()) = self.security_manager.verify_plugin(plugin.as_ref()).await {
                                // Register with standard method
                                let arc_plugin = Arc::from(plugin);
                                if let Ok(()) = self.register_plugin(arc_plugin).await {
                                    ids.push(id);
                                    plugin_count += 1;
                                }
                            } else {
                                error!("Security verification failed for plugin {}", name);
                            }
                        } else {
                            error!("Signature verification failed for plugin {}", name);
                        }
                    } else {
                        error!("Signature verification error for plugin {}", name);
                    }
                } else {
                    warn!("No signature file found for plugin {}, registering without signature", name);
                    // Register without signature verification
                    let arc_plugin = Arc::from(plugin);
                    if let Ok(()) = self.register_plugin(arc_plugin).await {
                        ids.push(id);
                        plugin_count += 1;
                    }
                }
            } else {
                // Register without signature verification
                let arc_plugin = Arc::from(plugin);
                if let Ok(()) = self.register_plugin(arc_plugin).await {
                    ids.push(id);
                    plugin_count += 1;
                }
            }
        }
        
        info!("Loaded {} plugins from {}", plugin_count, directory);
        Ok(ids)
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        self.load_plugins_with_signatures(directory, false).await
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
    
    /// Security manager
    security_manager: Arc<SecurityManagerAdapter>,
}

impl std::fmt::Debug for DefaultPluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultPluginManager")
            .field("plugins", &"<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>")
            .field("statuses", &"<RwLock<HashMap<Uuid, PluginStatus>>>")
            .field("name_to_id", &"<RwLock<HashMap<String, Uuid>>>")
            .field("state_manager", &"<Arc<dyn PluginStateManager>>")
            .field("security_manager", &"<Arc<SecurityManagerAdapter>>")
            .finish()
    }
}

impl DefaultPluginManager {
    /// Create a new plugin manager with the specified state manager
    pub fn new(state_manager: Arc<dyn PluginStateManager>, 
               security_manager: Option<Arc<SecurityManagerAdapter>>) -> Self {
        let security_manager = security_manager.unwrap_or_else(|| {
            #[cfg(feature = "mcp")]
            let security_adapter = {
                // Create all the necessary components for SecurityManagerImpl
                let crypto_provider = Arc::new(DefaultCryptoProvider::new());
                let token_manager = Arc::new(DefaultTokenManager::new());
                let identity_manager = Arc::new(DefaultIdentityManager::new());
                let rbac_manager = Arc::new(BasicRBACManager::new());
                let audit_service = Arc::new(DefaultAuditService::new());
                
                // Create the MCP security manager with these components
                let mcp_security_manager = Arc::new(SecurityManagerImpl::new(
                    crypto_provider,
                    token_manager,
                    identity_manager,
                    rbac_manager,
                    audit_service,
                ));
                
                Arc::new(SecurityManagerAdapter::new(mcp_security_manager))
            };
            
            #[cfg(not(feature = "mcp"))]
            let security_adapter = Arc::new(SecurityManagerAdapter::default());
            
            security_adapter
        });
        
        Self {
            plugins: RwLock::new(HashMap::new()),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            state_manager,
            security_manager,
        }
    }
    
    /// Load plugins from a directory with signature verification
    pub async fn load_plugins_with_signatures(&self, directory: &str, verify_signatures: bool) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();
        
        let mut plugin_count = 0;
        for plugin in plugin_paths {
            let metadata = plugin.metadata();
            let id = metadata.id;
            let name = metadata.name.clone(); // Clone the name to use later
            
            // If signature verification is enabled, look for a signature file
            if verify_signatures {
                let sig_path = std::path::Path::new(directory)
                    .join(format!("{}.sig", name));
                
                if sig_path.exists() {
                    // Load the signature
                    let sig_bytes = std::fs::read(&sig_path)?;
                    
                    // Verify signature using security manager
                    if let Ok(verification_result) = self.security_manager.verify_signature(metadata, &sig_bytes).await {
                        if verification_result {
                            // Verify security
                            if let Ok(()) = self.security_manager.verify_plugin(plugin.as_ref()).await {
                                // Register with standard method
                                let arc_plugin = Arc::from(plugin);
                                if let Ok(()) = self.register_plugin(arc_plugin).await {
                                    ids.push(id);
                                    plugin_count += 1;
                                }
                            } else {
                                error!("Security verification failed for plugin {}", name);
                            }
                        } else {
                            error!("Signature verification failed for plugin {}", name);
                        }
                    } else {
                        error!("Signature verification error for plugin {}", name);
                    }
                } else {
                    warn!("No signature file found for plugin {}, registering without signature", name);
                    // Register without signature verification
                    let arc_plugin = Arc::from(plugin);
                    if let Ok(()) = self.register_plugin(arc_plugin).await {
                        ids.push(id);
                        plugin_count += 1;
                    }
                }
            } else {
                // Register without signature verification
                let arc_plugin = Arc::from(plugin);
                if let Ok(()) = self.register_plugin(arc_plugin).await {
                    ids.push(id);
                    plugin_count += 1;
                }
            }
        }
        
        info!("Loaded {} plugins from {}", plugin_count, directory);
        Ok(ids)
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

    /// Get a security report for a plugin
    pub async fn get_security_report(&self, id: Uuid) -> Result<crate::security::SecurityReport> {
        // Check if the plugin exists
        if !self.plugins.read().await.contains_key(&id) {
            return Err(PluginError::NotFound(id).into());
        }
        
        // Generate and return the security report
        self.security_manager.create_security_report(id).await.map_err(|e| {
            PluginError::SecurityError(format!("Failed to create security report: {}", e)).into()
        })
    }
    
    /// Get the security manager
    pub fn security_manager(&self) -> Arc<SecurityManagerAdapter> {
        self.security_manager.clone()
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginManager {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        // Verify security
        if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
            return Err(PluginError::SecurityError(format!("Security verification failed: {}", e)).into());
        }
        
        // Set the initial status to Registered
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Registered);
        
        // Map the name to the ID
        let mut name_to_id = self.name_to_id.write().await;
        name_to_id.insert(metadata.name.clone(), id);
        
        // Register the plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(id, plugin);
        
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
        // Get the plugin
        let plugin = PluginRegistry::get_plugin(self, id).await?;
        
        // Verify dependencies
        self.check_dependencies(&plugin).await?;
        
        // Check for security issues before initializing
        if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
            // Set the status to Failed
            let mut statuses = self.statuses.write().await;
            statuses.insert(id, PluginStatus::Failed);
            
            return Err(PluginError::SecurityError(format!("Security verification failed: {}", e)).into());
        }
        
        // Set up sandbox if not already set
        if !self.security_manager.is_sandboxed(id).await? {
            // Use default sandbox config
            let config = crate::security::SandboxConfig::default();
            self.security_manager.create_sandbox(id, config).await?;
        }
        
        // Initialize the plugin
        let init_result = plugin.initialize().await;
        
        // If initialization failed, update the status
        if init_result.is_err() {
            let mut statuses = self.statuses.write().await;
            statuses.insert(id, PluginStatus::Failed);
        }
        
        // Return the result with a proper error conversion
        init_result.map_err(|e| {
            anyhow::Error::from(PluginError::InitializationError(format!("Plugin initialization failed: {}", e)))
        })?;
        
        // Set the status to Initialized
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Initialized);
        
        // Log the successful initialization
        tracing::info!("Initialized plugin: {}", plugin.metadata().name);
        
        Ok(())
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
        self.load_plugins_with_signatures(directory, false).await
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