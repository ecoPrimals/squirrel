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
use super::config::{DashboardConfig, ComponentConfig};
use super::security::{AuthManager, RateLimiter, OriginVerifier, DataMaskingManager, AuditLogger};
use super::secure_server::{create_secure_server, SecureServerState, BroadcastMessage};

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
    pub config: ComponentConfig,
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
        let addr: SocketAddr = format!("{}:{}", self.config.server.host, self.config.server.port)
            .parse()
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
    pub async fn register_component(&self, component: Component) -> Result<()> {
        if self.components.read().await.contains_key(&component.id) {
            return Err(SquirrelError::generic(format!("Component with ID {} already exists", component.id)));
        }
        
        let id = component.id.clone();
        let mut components = self.components.write().await;
        components.insert(id.clone(), component);
        
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
        let auth_manager = if matches!(self.config.security.auth.auth_type, super::security::AuthType::None) {
            None
        } else {
            let jwt_secret = std::env::var("DASHBOARD_JWT_SECRET")
                .unwrap_or_else(|_| "dashboard_default_secret_key_change_me_in_production".to_string());
            
            Some(Arc::new(AuthManager::new(
                self.config.security.auth.clone(),
                jwt_secret
            )))
        };
        
        // Rate limiter
        let rate_limiter = Some(Arc::new(RateLimiter::new(
            self.config.security.rate_limit.clone()
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
        let data_masking_manager = if self.config.security.masking_rules.is_empty() {
            None
        } else {
            match DataMaskingManager::new(self.config.security.masking_rules.clone()) {
                Ok(manager) => Some(Arc::new(manager)),
                Err(e) => {
                    error!("Failed to initialize data masking manager: {}", e);
                    None
                }
            }
        };
        
        // Audit logger
        let audit_logger = if let Some(audit_config) = &self.config.security.audit {
            if audit_config.enabled {
                Some(Arc::new(AuditLogger::new(audit_config.clone())))
            } else {
                None
            }
        } else {
            None
        };
        
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