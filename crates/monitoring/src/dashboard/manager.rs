//! Dashboard manager implementation
//!
//! This module provides the core Dashboard Manager implementation
//! which coordinates dashboard components, WebSocket server, and security features.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::sync::{RwLock, Mutex, broadcast};
use tokio::task::JoinHandle;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tracing::{info, error, debug};
use async_trait::async_trait;

use squirrel_core::error::{Result, SquirrelError};
use crate::dashboard::config::{ComponentSettings, DashboardConfig};
use super::security::{AuthManager, RateLimiter, OriginVerifier, DataMaskingManager, AuditLogger};
use super::secure_server::{create_secure_server, SecureServerState, BroadcastMessage};
use crate::dashboard::plugins::{
    DashboardPlugin, create_dashboard_plugin_registry, DashboardPluginRegistryImpl, ExamplePlugin, DashboardPluginRegistry
};
use crate::dashboard::DashboardError;
use crate::dashboard::DashboardComponent;

/// Dashboard manager
#[derive(Debug)]
pub struct DashboardManager {
    /// Dashboard configuration
    config: DashboardConfig,
    /// Server running state
    is_running: AtomicBool,
    /// Server task handle
    server_task: Mutex<Option<JoinHandle<()>>>,
    /// Components registry
    components: RwLock<HashMap<String, Component>>,
    /// Authentication manager
    auth_manager: Mutex<Option<Arc<AuthManager>>>,
    /// Rate limiter
    rate_limiter: Mutex<Option<Arc<RateLimiter>>>,
    /// Origin verifier
    origin_verifier: Mutex<Option<Arc<OriginVerifier>>>,
    /// Data masking manager
    data_masking_manager: Mutex<Option<Arc<DataMaskingManager>>>,
    /// Audit logger
    audit_logger: Mutex<Option<Arc<AuditLogger>>>,
    /// Server state
    server_state: Mutex<Option<SecureServerState>>,
    /// Plugin registry
    plugin_registry: Mutex<Option<Arc<DashboardPluginRegistryImpl>>>,
}

/// Dashboard component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component ID
    pub id: String,
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: String,
    /// Component configuration
    pub config: ComponentSettings,
    /// Component data
    pub data: Option<Value>,
    /// Last updated timestamp
    pub last_updated: Option<u64>,
}

/// Dashboard manager trait
#[async_trait]
pub trait Manager: Send + Sync + Debug {
    /// Get all components
    async fn get_components(&self) -> Vec<Component>;
    
    /// Get component data
    async fn get_component_data(&self, id: &str) -> Option<Value>;
    
    /// Get health status
    async fn get_health_status(&self) -> Value;
}

impl Default for DashboardManager {
    fn default() -> Self {
        Self::new(DashboardConfig::default())
    }
}

impl DashboardManager {
    /// Creates a new dashboard manager with the given configuration
    #[must_use]
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            is_running: AtomicBool::new(false),
            server_task: Mutex::new(None),
            components: RwLock::new(HashMap::new()),
            auth_manager: Mutex::new(None),
            rate_limiter: Mutex::new(None),
            origin_verifier: Mutex::new(None),
            data_masking_manager: Mutex::new(None),
            audit_logger: Mutex::new(None),
            server_state: Mutex::new(None),
            plugin_registry: Mutex::new(None),
        }
    }
    
    /// Start the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to start
    pub async fn start(&self) -> Result<()> {
        // Check if already running
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        // Initialize security components
        self.initialize_security().await?;
        
        // Create server
        let server = create_secure_server(self.config.clone());
        
        // Get server state
        let _server_state = if let Some(state) = &*self.server_state.lock().await {
            state.clone()
        } else {
            return Err(SquirrelError::dashboard("Failed to initialize server state".to_string()));
        };
        
        // Get server address
        let addr: SocketAddr = format!("{}:{}", 
            self.config.server.as_ref().map_or("127.0.0.1", |s| &s.host), 
            self.config.server.as_ref().map_or(8765, |s| s.port)
        ).parse()
        .map_err(|e| SquirrelError::dashboard(format!("Invalid server address: {}", e)))?;
        
        // Start server
        let server_task = tokio::spawn(async move {
            // Log server start
            info!("Starting dashboard server on {}", addr);
            
            // Start server
            axum::serve(
                tokio::net::TcpListener::bind(addr).await.expect("Failed to bind server"),
                server
            )
            .await
            .expect("Server failed");
        });
        
        // Update state
        {
            let mut task_guard = self.server_task.lock().await;
            *task_guard = Some(server_task);
        }
        
        // Mark as running
        self.is_running.store(true, Ordering::SeqCst);
        
        // Log startup
        info!("Dashboard manager started");
        
        Ok(())
    }
    
    /// Stop the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to stop
    pub async fn stop(&self) -> Result<()> {
        // Check if running
        if !self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        // Get server task
        let task = {
            let mut task_guard = self.server_task.lock().await;
            task_guard.take()
        };
        
        // Abort server task if running
        if let Some(task) = task {
            task.abort();
        }
        
        // Mark as not running
        self.is_running.store(false, Ordering::SeqCst);
        
        // Log shutdown
        info!("Dashboard manager stopped");
        
        Ok(())
    }
    
    /// Check if the dashboard manager is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
    
    /// Register a component
    ///
    /// # Errors
    /// Returns an error if the component could not be registered
    pub async fn register_component(&self, component: Arc<dyn DashboardComponent>) -> Result<()> {
        let component_id = component.id();
        if self.components.read().await.contains_key(component_id) {
            return Err(SquirrelError::generic(format!("Component with ID {} already exists", component_id)));
        }
        
        // Create a Component struct from the DashboardComponent trait object
        let component_data = component.get_data().await.ok();
        let component_struct = Component {
            id: component_id.to_string(),
            name: component_id.to_string(), // Use ID as name if not specified
            component_type: "plugin".to_string(),
            config: ComponentSettings::default(),
            data: component_data,
            last_updated: component.last_update().await.map(|dt| dt.timestamp_millis() as u64),
        };
        
        // Start the component
        component.start().await?;
        
        // Store the component
        let id = component_struct.id.clone();
        let mut components = self.components.write().await;
        components.insert(id.clone(), component_struct);
        
        debug!("Component registered: {}", id);
        Ok(())
    }
    
    /// Update a component's data
    ///
    /// # Errors
    /// Returns an error if the component could not be updated
    pub async fn update_component(&self, component_id: &str, data: Value) -> Result<()> {
        // Get components
        let mut components = self.components.write().await;
        
        // Check if component exists
        let component = components.get_mut(component_id).ok_or_else(|| {
            SquirrelError::dashboard(format!("Component not found: {}", component_id))
        })?;
        
        // Update component
        component.data = Some(data.clone());
        component.last_updated = Some(chrono::Utc::now().timestamp_millis() as u64);
        
        // Broadcast update
        if let Some(state) = &*self.server_state.lock().await {
            let broadcast_message = BroadcastMessage {
                message_type: "update".to_string(),
                component_id: Some(component_id.to_string()),
                payload: data,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                compressed: false,
            };
            
            // Send broadcast message
            let _ = state.tx.send(broadcast_message);
        }
        
        Ok(())
    }
    
    /// Remove a component
    ///
    /// # Errors
    /// Returns an error if the component could not be removed
    pub async fn remove_component(&self, component_id: &str) -> Result<()> {
        // Get components
        let mut components = self.components.write().await;
        
        // Check if component exists
        if !components.contains_key(component_id) {
            return Err(SquirrelError::dashboard(format!("Component not found: {}", component_id)));
        }
        
        // Remove component
        components.remove(component_id);
        
        // Log removal
        debug!("Component removed: {}", component_id);
        
        Ok(())
    }
    
    /// Get all components
    #[must_use]
    pub async fn get_components(&self) -> Vec<Component> {
        let components = self.components.read().await;
        components.values().cloned().collect()
    }
    
    /// Get a specific component
    #[must_use]
    pub async fn get_component(&self, component_id: &str) -> Option<Component> {
        let components = self.components.read().await;
        components.get(component_id).cloned()
    }
    
    /// Get server statistics
    #[must_use]
    pub async fn get_stats(&self) -> DashboardStats {
        // Get client count
        let client_count = if let Some(state) = &*self.server_state.lock().await {
            let clients = state.clients.read().await;
            clients.len()
        } else {
            0
        };
        
        // Get component count
        let component_count = {
            let components = self.components.read().await;
            components.len()
        };
        
        // Create stats
        DashboardStats {
            is_running: self.is_running(),
            client_count,
            component_count,
            uptime: 0, // TODO: Track uptime
            security_enabled: self.is_security_enabled().await,
        }
    }
    
    /// Initialize security components
    async fn initialize_security(&self) -> Result<()> {
        // Configure security components based on configuration
        
        // Authentication manager
        let auth_manager = if self.config.security.auth.as_ref().is_some_and(|auth| 
            matches!(auth.auth_type, super::security::AuthType::Basic)) 
        {
            let jwt_secret = std::env::var("DASHBOARD_JWT_SECRET")
                .unwrap_or_else(|_| "dashboard_default_secret_key_change_me_in_production".to_string());
            
            Some(Arc::new(AuthManager::new(
                self.config.security.auth.clone().unwrap(),
                jwt_secret
            )))
        } else {
            None
        };
        
        // Rate limiter
        let rate_limiter = self.config.security.rate_limit.as_ref().map(|rate_limit_config| Arc::new(RateLimiter::new(
                rate_limit_config.clone()
            )));
        
        // Origin verifier
        let origin_verifier = if self.config.security.allowed_origins.is_empty() {
            None
        } else {
            Some(Arc::new(OriginVerifier::new(
                self.config.security.allowed_origins.clone()
            )))
        };
        
        // Data masking manager
        let data_masking_manager = if self.config.security.data_masking.is_empty() {
            None
        } else {
            match DataMaskingManager::new(self.config.security.data_masking.clone()) {
                Ok(manager) => Some(Arc::new(manager)),
                Err(e) => {
                    error!("Failed to create data masking manager: {}", e);
                    None
                }
            }
        };
        
        // Audit logger
        let audit_logger = self.config.security.audit.as_ref().map(|audit_config| Arc::new(AuditLogger::new(audit_config.clone())));
        
        // Create server state
        let (tx, _) = broadcast::channel(1000);
        
        let server_state = SecureServerState {
            config: self.config.clone(),
            auth_manager: auth_manager.clone(),
            rate_limiter: rate_limiter.clone(),
            origin_verifier: origin_verifier.clone(),
            data_masking_manager: data_masking_manager.clone(),
            audit_logger: audit_logger.clone(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            tx,
        };
        
        // Store components
        {
            let mut auth_guard = self.auth_manager.lock().await;
            *auth_guard = auth_manager;
        }
        
        {
            let mut rate_guard = self.rate_limiter.lock().await;
            *rate_guard = rate_limiter;
        }
        
        {
            let mut origin_guard = self.origin_verifier.lock().await;
            *origin_guard = origin_verifier;
        }
        
        {
            let mut masking_guard = self.data_masking_manager.lock().await;
            *masking_guard = data_masking_manager;
        }
        
        {
            let mut audit_guard = self.audit_logger.lock().await;
            *audit_guard = audit_logger;
        }
        
        {
            let mut state_guard = self.server_state.lock().await;
            *state_guard = Some(server_state);
        }
        
        Ok(())
    }
    
    /// Check if security is enabled
    async fn is_security_enabled(&self) -> bool {
        let auth_enabled = self.auth_manager.lock().await.is_some();
        let tls_enabled = self.config.security.tls.is_some();
        
        auth_enabled || tls_enabled
    }
    
    /// Get the number of connected clients
    #[must_use]
    pub async fn get_client_count(&self) -> usize {
        if let Some(state) = &*self.server_state.lock().await {
            let clients = state.clients.read().await;
            clients.len()
        } else {
            0
        }
    }
    
    /// Get a list of connected clients
    #[must_use]
    pub async fn get_clients(&self) -> Vec<ClientInfo> {
        let state = self.server_state.lock().await;
        if let Some(state) = &*state {
            let clients = state.clients.read().await;
            clients.values().cloned().map(ClientInfo::from).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Broadcast a message to all clients
    ///
    /// # Errors
    /// Returns an error if the message could not be broadcast
    pub async fn broadcast(&self, message_type: &str, payload: Value) -> Result<()> {
        if let Some(state) = &*self.server_state.lock().await {
            let broadcast_message = BroadcastMessage {
                message_type: message_type.to_string(),
                component_id: None,
                payload,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                compressed: false,
            };
            
            // Send broadcast message
            let _ = state.tx.send(broadcast_message);
            Ok(())
        } else {
            Err(SquirrelError::dashboard("Server not initialized".to_string()))
        }
    }
    
    /// Register a plugin
    ///
    /// # Errors
    /// Returns an error if the plugin could not be registered
    pub async fn register_plugin<T>(&self, plugin: Arc<T>) -> Result<()>
    where
        T: DashboardPlugin + Send + Sync + 'static,
    {
        // Initialize plugin registry if needed
        self.initialize_plugin_registry().await?;
        
        // Get plugin registry
        let registry = match &*self.plugin_registry.lock().await {
            Some(registry) => registry.clone(),
            None => return Err(SquirrelError::dashboard("Plugin registry not initialized".to_string())),
        };
        
        // Register plugin with registry
        registry.register_plugin(plugin.clone()).await
            .map_err(|e| SquirrelError::dashboard(format!("Failed to register plugin with registry: {}", e)))?;
        
        // Register the plugin as a component with the dashboard
        self.register_component(plugin).await
            .map_err(|e| SquirrelError::dashboard(format!("Failed to register plugin with dashboard: {}", e)))?;
        
        Ok(())
    }
    
    /// Initialize the plugin registry
    ///
    /// # Errors
    /// Returns an error if the plugin registry fails to initialize
    async fn initialize_plugin_registry(&self) -> Result<()> {
        let mut registry_guard = self.plugin_registry.lock().await;
        
        if registry_guard.is_none() {
            // Create registry
            let registry = create_dashboard_plugin_registry();
            *registry_guard = Some(registry);
        }
        
        Ok(())
    }
    
    /// Get all registered plugins
    ///
    /// # Errors
    /// Returns an error if the plugin registry is not initialized
    pub async fn get_plugins(&self) -> Result<Vec<Arc<dyn DashboardPlugin>>> {
        // Initialize plugin registry if needed
        self.initialize_plugin_registry().await?;
        
        // Get plugin registry
        let registry = match &*self.plugin_registry.lock().await {
            Some(registry) => registry.clone(),
            None => return Err(SquirrelError::dashboard("Plugin registry not initialized".to_string())),
        };
        
        // Get plugins
        registry.get_plugins().await
            .map_err(|e| SquirrelError::dashboard(format!("Failed to get plugins: {}", e)))
    }
    
    /// Discover plugins from a directory
    ///
    /// # Arguments
    /// * `plugin_dir` - Directory to search for plugins
    ///
    /// # Returns
    /// * `Result<usize>` - Number of plugins discovered
    ///
    /// # Errors
    /// Returns an error if plugin discovery fails
    pub async fn discover_plugins(&self, plugin_dir: &str) -> Result<usize> {
        info!("Discovering plugins from directory: {}", plugin_dir);
        
        // Initialize plugin registry if needed
        self.initialize_plugin_registry().await?;
        
        // Check if directory exists
        if !std::path::Path::new(plugin_dir).exists() {
            return Err(SquirrelError::dashboard(format!("Plugin directory not found: {}", plugin_dir)));
        }
        
        // TODO: Implement actual plugin discovery from files
        // For now, just register the built-in example plugins
        
        // Create example plugins
        let visualization_plugin = Arc::new(crate::dashboard::plugins::ExamplePlugin::new());
        
        // Register plugins
        self.register_plugin(visualization_plugin).await?;
        
        // Return number of plugins registered
        Ok(1)
    }
    
    /// Update a plugin's data
    ///
    /// # Errors
    /// Returns an error if the plugin update fails
    pub async fn update_plugin_data(&self, plugin_id: &str, data: Value) -> Result<()> {
        // Initialize plugin registry if needed
        self.initialize_plugin_registry().await?;
        
        // Get plugin registry
        let registry = match &*self.plugin_registry.lock().await {
            Some(registry) => registry.clone(),
            None => return Err(SquirrelError::dashboard("Plugin registry not initialized".to_string())),
        };
        
        // Parse plugin ID
        let plugin_id = plugin_id.to_string();
        
        // Get plugin
        let plugin = match registry.get_plugin(&plugin_id).await
            .map_err(|e| SquirrelError::dashboard(format!("Failed to get plugin: {}", e)))? {
            Some(plugin) => plugin,
            None => return Err(SquirrelError::dashboard(format!("Plugin not found: {}", plugin_id))),
        };
        
        // Update plugin
        plugin.update(data.clone()).await
            .map_err(|e| SquirrelError::dashboard(format!("Failed to update plugin: {}", e)))?;
        
        // Update component data
        self.update_component(plugin_id.as_str(), data).await
    }
    
    /// Get the plugin registry
    pub async fn get_plugin_registry(&self) -> Result<Arc<dyn DashboardPluginRegistry>> {
        let registry = self.plugin_registry.lock().await;
        match &*registry {
            Some(registry) => Ok(registry.clone()),
            None => {
                error!("Plugin registry not initialized");
                Err(DashboardError::PluginError("Plugin registry not initialized".to_string()).into())
            }
        }
    }
    
    /// Initialize example plugins for demonstration purposes
    pub async fn initialize_example_plugins(&self) -> Result<()> {
        info!("Initializing example plugins");
        
        let registry = self.get_plugin_registry().await?;
        
        // Create example visualization plugin
        let visualization_plugin = Arc::new(ExamplePlugin::new());
        
        // Register plugins
        registry.register_plugin(visualization_plugin.clone()).await?;
        
        info!("Example plugins initialized successfully");
        Ok(())
    }
    
    /// Set the plugin registry (for testing)
    #[cfg(test)]
    pub async fn set_test_plugin_registry(&self) -> Result<()> {
        let mut registry_guard = self.plugin_registry.lock().await;
        
        if registry_guard.is_none() {
            // Create registry
            let registry = crate::dashboard::plugins::create_dashboard_plugin_registry();
            *registry_guard = Some(registry);
        }
        
        Ok(())
    }
}

/// Dashboard server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Whether the server is running
    pub is_running: bool,
    /// Number of connected clients
    pub client_count: usize,
    /// Number of registered components
    pub component_count: usize,
    /// Server uptime in seconds
    pub uptime: u64,
    /// Whether security features are enabled
    pub security_enabled: bool,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client ID
    pub id: String,
    /// Client IP address
    pub ip: String,
    /// Client user agent
    pub user_agent: String,
    /// Client username (if authenticated)
    pub username: Option<String>,
    /// Client connection time
    pub connected_at: u64,
    /// Client subscriptions
    pub subscriptions: Vec<String>,
    /// Message count
    pub message_count: usize,
}

// Implementation to convert secure_server::ClientInfo to manager::ClientInfo
impl From<super::secure_server::ClientInfo> for ClientInfo {
    fn from(client: super::secure_server::ClientInfo) -> Self {
        Self {
            id: client.id,
            ip: client.ip,
            user_agent: client.user_agent,
            username: client.username,
            connected_at: client.connected_at,
            subscriptions: client.subscriptions,
            message_count: client.message_count,
        }
    }
}

pub struct ComponentState {
    pub id: String,
    pub config: ComponentSettings,
    pub state: serde_json::Value,
} 