//! Synchronization moved to ToadStool and NestGate
//!
//! This module previously provided synchronization primitives for the MCP system.
//! Synchronization functionality has been migrated to ecosystem projects:
//! - ToadStool: Compute synchronization and task coordination
//! - NestGate: Data synchronization and distributed state management
//!
//! For synchronization capabilities, integrate with ToadStool and NestGate.

// Synchronization moved to ecosystem projects
// All exports below are placeholders for ecosystem integration

// NestGate handles data sync: Proto sync types moved to NestGate
// ToadStool handles compute sync: Task coordination moved to ToadStool

/// Placeholder for sync functionality - use ToadStool/NestGate integration
pub struct SyncManager;

/// Placeholder for context changes - use NestGate integration
pub struct ContextChange;

use crate::context_manager::Context;
use crate::monitoring::MCPMonitor;
use crate::persistence::{MCPPersistence, PersistenceConfig, PersistentState};
// use crate::sync::state::{StateSyncManager, StateChange, StateOperation}; // OLD
use crate::sync::state::{StateChange, StateSyncManager};
// Import StateOperation but give it an alias to avoid confusion with the re-export
use crate::sync::state::StateOperation as InternalStateOperation;
use crate::MCPError;
// use the generated types directly from generated module
// use crate::generated::mcp_sync::sync_service_client::SyncServiceClient;
// use crate::generated::mcp_sync::{ContextChange as ProtoContextChange, SyncRequest};
// use crate::generated::mcp_sync::context_change::OperationType as ProtoOperationType;
use chrono::{DateTime, Utc, TimeZone}; // Import TimeZone trait
// use prost::Message; // Commented out
use serde::{Deserialize, Serialize};
use squirrel_interfaces::error::{Result, SquirrelError};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use uuid::Uuid;
use tokio::time::timeout;
use tracing::{info, warn, error}; // Add warn and error macros
use tonic::transport::Channel;

/// Convert an `MCPError` to a `SquirrelError`
fn to_core_error<T>(result: std::result::Result<T, MCPError>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(SquirrelError::generic(format!("MCP error: {err}"))),
    }
}

/// State synchronization for MCP
pub mod state;
/// Server implementation for MCP synchronization
pub mod server;
// pub use state::{StateChange, StateOperation}; // Remove re-export if direct import is used everywhere

#[cfg(test)]
mod tests;

/// Configuration for MCP state synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// URL of the central synchronization server (e.g., "http://[::1]:50051")
    pub central_server_url: String,
    /// Interval in seconds between synchronization attempts
    pub sync_interval: u64,
    /// Maximum number of retries for failed sync operations
    pub max_retries: u32,
    /// Timeout in milliseconds for sync operations
    pub timeout_ms: u64,
    /// Number of days after which old sync records should be cleaned up
    pub cleanup_older_than_days: i64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            // Provide a sensible default, maybe localhost for testing
            central_server_url: "http://[::1]:50051".to_string(), 
            sync_interval: 60,
            max_retries: 3,
            timeout_ms: 5000,
            cleanup_older_than_days: 7,
        }
    }
}

/// State of the sync engine
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Whether a sync operation is currently in progress
    pub is_syncing: bool,
    /// When the last sync operation was performed
    pub last_sync: Option<DateTime<Utc>>,
    /// When the last error occurred
    pub last_error: Option<DateTime<Utc>>,
    /// The number of successful sync operations
    pub sync_count: u64,
    /// The number of errors encountered
    pub error_count: u64,
    /// The version of the last successful sync
    pub last_version: Option<u64>,
    /// Client ID for sync operations
    pub client_id: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncState {
    /// Creates a new `SyncState` with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_syncing: false,
            last_sync: None,
            last_error: None,
            sync_count: 0,
            error_count: 0,
            last_version: None,
            client_id: None,
        }
    }
}

/// Result of a synchronization operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether the sync operation was successful
    pub success: bool,
    /// The duration of the sync operation in milliseconds
    pub duration_ms: u64,
    /// Number of changes processed
    pub changes_processed: usize,
    /// Current server version after sync
    pub current_server_version: u64,
}

/// Main synchronization engine for MCP state
///
/// Responsible for coordinating state changes across distributed instances
/// and ensuring consistency of context data.
#[derive(Debug)]
pub struct MCPSync {
    /// Configuration for the sync engine
    config: Arc<RwLock<SyncConfig>>,
    /// Current state of synchronization
    state: Arc<RwLock<SyncState>>,
    /// Manager for state synchronization
    state_manager: Arc<StateSyncManager>,
    /// Persistence layer for storing sync data
    persistence: Arc<MCPPersistence>,
    /// Monitoring for sync operations
    monitor: Arc<MCPMonitor>,
    /// Mutex for synchronizing operations
    lock: Arc<Mutex<()>>,
    /// Whether the sync engine has been initialized
    initialized: Arc<RwLock<bool>>,
    /// Changes tracked by the sync engine
    changes: Arc<RwLock<Vec<StateChange>>>,
}

impl MCPSync {
    /// Creates a new `MCPSync` instance with the given configuration and dependencies
    #[must_use]
    pub fn new(
        config: SyncConfig,
        persistence: Arc<MCPPersistence>,
        monitor: Arc<MCPMonitor>,
        state_manager: Arc<StateSyncManager>,
    ) -> Self {
        let lock = Arc::new(Mutex::new(()));
        let state = SyncState {
            is_syncing: false,
            last_sync: None,
            last_error: None,
            sync_count: 0,
            error_count: 0,
            last_version: None,
            client_id: None,
        };
        
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(state)),
            state_manager,
            persistence,
            monitor,
            lock,
            initialized: Arc::new(RwLock::new(false)),
            changes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new `MCPSync` instance asynchronously
    ///
    /// # Arguments
    /// * `config` - Configuration for synchronization
    ///
    /// # Returns
    /// A Result containing the new `MCPSync` instance, or an error
    /// 
    /// # Errors
    /// 
    /// Returns an error if the monitoring system initialization fails
    pub async fn create(config: SyncConfig) -> Result<Self> {
        let start = Instant::now();

        // Create components
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::new().await.map_err(|e| SquirrelError::mcp(e.to_string()))?);
        let state_manager = Arc::new(StateSyncManager::new());

        let instance = Self {
            persistence,
            monitor,
            state_manager,
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SyncState::new())),
            initialized: Arc::new(RwLock::new(false)),
            changes: Arc::new(RwLock::new(Vec::new())),
            lock: Arc::new(Mutex::new(())),
        };

        let duration = start.elapsed();
        instance
            .monitor
            .record_message(&format!("sync_creation_time_ms_{}", duration.as_millis()))
            .await;

        Ok(instance)
    }

    /// Creates a new `MCPSync` instance synchronously
    ///
    /// This is used in cases where we need a synchronous constructor,
    /// such as in Default implementations. This method uses the synchronous
    /// monitor constructor.
    ///
    /// # Arguments
    /// * `config` - Configuration for synchronization
    ///
    /// # Returns
    /// A new `MCPSync` instance
    #[must_use] pub fn create_sync(config: SyncConfig) -> Self {
        // Create components synchronously
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::default_sync());
        let state_manager = Arc::new(StateSyncManager::new());

        Self {
            persistence,
            monitor,
            state_manager,
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SyncState::new())),
            initialized: Arc::new(RwLock::new(false)),
            changes: Arc::new(RwLock::new(Vec::new())),
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Returns a default instance of `MCPSync`
    ///
    /// This is primarily used in testing or when a fully-featured instance is not required.
    /// This uses synchronous initialization and is suitable for default implementations.
    ///
    /// # Returns
    /// A new `MCPSync` instance
    #[must_use]
    pub fn default_instance() -> Self {
        // Use synchronous constructor with default config
        Self::create_sync(SyncConfig::default())
    }

    /// Initializes the sync engine
    ///
    /// Sets up the internal state and ensures all dependencies are properly
    /// configured. This must be called before any other operations.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    pub async fn init(&mut self) -> Result<()> {
        let start = Instant::now();
        self.monitor.record_message("sync_initializing").await;

        // Result is converted from MCPError to SquirrelError
        let result = to_core_error(self.init_internal().await);
        
        // Record initialization time for monitoring
        let elapsed = start.elapsed();
        self.monitor.record_message(&format!("sync_init_time_ms_{}", elapsed.as_millis())).await;
        
        result
    }

    /// Internal implementation of init that returns `MCPError`
    async fn init_internal(&mut self) -> std::result::Result<(), MCPError> {
        // Check if already initialized
        if *self.initialized.read().await {
            return Ok(());
        }

        // Initialize persistence if needed
        let persisted = match self.persistence.load_state() {
            Ok(Some(data)) => data,
            Ok(None) => {
                // Handle the case where there's no data
                self.monitor.record_message("no_persisted_state").await;
                // Create default state
                let default_state = PersistentState {
                    contexts: vec![],
                    changes: vec![],
                    last_version: 0,
                    last_sync: Utc::now(),
                    id: uuid::Uuid::new_v4().to_string(),
                };

                if let Err(persist_err) = self.persistence.save_state(&default_state) {
                    return Err(MCPError::from(format!(
                        "Failed to save default state: {persist_err}"
                    )));
                }

                default_state
            }
            Err(err) => {
                // Handle error
                self.monitor.record_error("load_state_error").await;
                tracing::warn!("Failed to load persisted state: {}", err);

                // If we can't load the state, create a default state

                // Return the default state directly (not wrapped in Some)
                PersistentState {
                    contexts: Vec::new(),
                    changes: Vec::new(),
                    last_version: 0,
                    last_sync: Utc::now(),
                    id: Uuid::new_v4().to_string(),
                }
            }
        };

        // Initialize state with persisted data
        {
            let mut state = self.state.write().await;
            *state = SyncState {
                is_syncing: false,
                last_sync: Some(persisted.last_sync),
                last_error: None,
                sync_count: 0,
                error_count: 0,
                last_version: Some(persisted.last_version),
                client_id: None,
            };
        }

        // Set the initialized flag *before* loading persisted changes
        // to avoid circular dependency
        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        // Load persisted changes
        self.load_persisted_changes_internal().await?;

        self.monitor.record_message("sync_initialized").await;

        Ok(())
    }

    /// Ensures that the sync engine is initialized
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # Errors
    /// Returns an error if the sync engine has not been initialized
    pub async fn ensure_initialized(&self) -> Result<()> {
        to_core_error(self.ensure_initialized_internal().await)
    }

    /// Internal implementation of `ensure_initialized` that returns `MCPError`
    async fn ensure_initialized_internal(&self) -> std::result::Result<(), MCPError> {
        let initialized = *self.initialized.read().await;
        if !initialized {
            return Err(MCPError::from(
                "Sync engine not initialized".to_string(),
            ));
        }
        Ok(())
    }

    /// Loads any changes that have been persisted to storage
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # Errors
    /// Returns an error if:
    /// * The sync engine is not initialized
    /// * The persisted changes cannot be loaded from storage
    /// * There are issues applying the loaded changes
    pub async fn load_persisted_changes(&self) -> Result<()> {
        to_core_error(self.load_persisted_changes_internal().await)
    }

    /// Internal implementation of `load_persisted_changes` that returns `MCPError`
    async fn load_persisted_changes_internal(&self) -> std::result::Result<(), MCPError> {
        // Remove the ensure_initialized check to avoid circular dependency
        // The caller (init_internal) will have already set initialized to true

        // load the persisted state
        let persisted = match self.persistence.load_state() {
            Ok(state) => state,
            Err(e) => {
                self.monitor.record_error("load_state_failed").await;
                tracing::warn!("Failed to load persisted state: {}", e);

                // If we can't load the state, return None
                None
            }
        };

        // If there is persisted state, load it
        if let Some(state) = persisted {
            // Since we don't have direct access to set the state manager's state,
            // we'll need to load changes one at a time
            let contexts = state.contexts.clone(); // Clone the vector to avoid borrowing issues
            for context in contexts {
                // If the context already exists, we would update it
                // But for now, since we can't directly query, we'll just create/overwrite
                let _ = self
                    .record_context_change(&context, InternalStateOperation::Create)
                    .await;
            }

            tracing::info!(
                "Loaded {} contexts from persisted state",
                state.contexts.len()
            );
        }

        Ok(())
    }

    /// Synchronizes with the central server
    ///
    /// This method attempts to synchronize with the central server,
    /// sending any local changes and applying changes from the server.
    ///
    /// # Returns
    ///
    /// A Result containing a SyncResult or an error
    ///
    /// # Errors
    ///
    /// Returns an error if synchronization fails for any reason
    pub async fn synchronize(&self) -> Result<SyncResult> {
        info!("Starting synchronization");
        
        // Take lock to ensure we don't overlap sync operations
        let _guard = self.lock.lock().await;
        
        {
            let mut state = self.state.write().await;
            state.is_syncing = true;
        }

        let start_time = Instant::now();
        
        // Perform the actual sync and handle any errors
        let result = self.sync_internal().await;
        
        // Update state based on outcome
        let mut state = self.state.write().await;
        state.is_syncing = false;
        
        let duration_ms = u64::try_from(start_time.elapsed().as_millis())
            .unwrap_or(u64::MAX);
        
        match &result {
            Ok(sync_result) => {
                state.last_sync = Some(Utc::now());
                state.sync_count += 1;
                state.last_version = Some(sync_result.current_server_version);
                
                // Record successful sync
                self.monitor.record_sync_success(
                    0, // local_changes (we don't track this yet)
                    sync_result.changes_processed,
                    sync_result.duration_ms
                ).await;
            }
            Err(e) => {
                state.last_error = Some(Utc::now());
                state.error_count += 1;
                
                // Record sync failure
                self.monitor.record_sync_failure(e.to_string()).await;
                
                // Calculate backoff time based on error count with exponential backoff
                let config = self.config.read().await;
                if state.error_count > 1 {
                    let backoff_secs = std::cmp::min(
                        60, // Max backoff of 60 seconds
                        2_u64.pow(state.error_count.min(10) as u32 - 1) // Exponential backoff with ceiling
                    );
                    warn!("Sync failed {} times consecutively. Will retry in ~{} seconds", 
                          state.error_count, backoff_secs);
                }
            }
        }
        
        result
    }

    /// Internal synchronization logic
    ///
    /// Handles the actual communication with the sync server
    async fn sync_internal(&self) -> Result<SyncResult> {
        let start_time = Instant::now();
        
        // Get a lock to avoid multiple sync operations
        let _lock = self.lock.lock().await;
        
        // Update state to indicate sync in progress
        {
            let mut state = self.state.write().await;
            state.is_syncing = true;
        }

        // Get the current configuration
        let config = self.config.read().await.clone();
        let server_version;
        let changes_count;
        
        {
            // Get the current client state
            let state = self.state.read().await;
            server_version = state.last_version.unwrap_or(0);
        }
        
        // Get any changes from this client that need to be sent to the server
        let changes = {
            let changes = self.changes.read().await;
            changes.clone()
        };
        
        changes_count = changes.len();
        
        // Setup the client with timeout
        let timeout_duration = Duration::from_millis(config.timeout_ms);
        let client_id = Uuid::new_v4().to_string(); // Generate a new client ID if needed
        
        // Create the connection to the server
        let connection_result = connect_to_server(&config.central_server_url, timeout_duration).await;
        
        match connection_result {
            Ok(mut client) => {
                // Build the request with our changes
                let request = SyncRequest {
                    client_id: client_id.clone(),
                    last_known_version: server_version,
                    local_changes: changes.iter()
                        .map(|change| self.convert_to_proto_change(change))
                        .collect(),
                };
                
                // Send the request to the server
                match timeout(timeout_duration, client.sync(request)).await {
                    Ok(result) => match result {
                        Ok(response) => {
                            // Update client state
        {
            let mut state = self.state.write().await;
            state.is_syncing = false;
            state.last_sync = Some(Utc::now());
            state.sync_count += 1;
                                state.last_version = Some(response.get_ref().current_server_version);
                                state.client_id = Some(client_id);
                            }
                            
                            // Process remote changes
                            if !response.get_ref().success {
                                let error_msg = if !response.get_ref().error_message.is_empty() {
                                    response.get_ref().error_message.clone()
                                } else {
                                    "Unknown server error".to_string()
                                };
                                
                                return Err(SquirrelError::mcp(format!("Sync failed: {error_msg}")));
                            }
                            
                            // Apply remote changes locally
                            if !response.get_ref().remote_changes.is_empty() {
                                self.apply_remote_changes(response.get_ref().remote_changes.clone()).await?;
                            }
                            
                            // Record success in monitoring
                            let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
                            self.monitor.record_sync_success(
                                changes_count,
                                response.get_ref().remote_changes.len(),
                                duration_ms
                            ).await;
                            
                            // Return success result
        Ok(SyncResult {
            success: true,
            duration_ms,
                                changes_processed: response.get_ref().remote_changes.len(),
                                current_server_version: response.get_ref().current_server_version,
                            })
                        }
                        Err(e) => {
                            // Handle tonic error
                            let mut state = self.state.write().await;
                            state.is_syncing = false;
                            state.last_error = Some(Utc::now());
                            state.error_count += 1;
                            
                            // Record failure in monitoring
                            self.monitor.record_sync_failure(e.to_string()).await;
                            
                            Err(SquirrelError::mcp(format!("Sync request failed: {e}")))
                        }
                    },
                    Err(_) => {
                        // Handle timeout
                        let mut state = self.state.write().await;
                        state.is_syncing = false;
                        state.last_error = Some(Utc::now());
                        state.error_count += 1;
                        
                        // Record failure in monitoring
                        self.monitor.record_sync_failure("Request timed out".to_string()).await;
                        
                        Err(SquirrelError::mcp("Sync request timed out".to_string()))
                    }
                }
            }
            Err(e) => {
                // Handle connection error
                let mut state = self.state.write().await;
                state.is_syncing = false;
                state.last_error = Some(Utc::now());
                state.error_count += 1;
                
                // Record failure in monitoring
                self.monitor.record_sync_failure(format!("Connection failed: {e}")).await;
                
                Err(SquirrelError::mcp(format!("Failed to connect to sync server: {e}")))
            }
        }
    }
    
    /// Process incoming changes from the server
    async fn apply_remote_changes(&self, changes: Vec<ProtoContextChange>) -> Result<()> {
        for change in changes {
            let operation_type = change.operation_type;
            
            match ProtoOperationType::try_from(operation_type) {
                Ok(ProtoOperationType::Create) => {
                    if let Ok(context_data) = serde_json::from_slice(&change.data) {
                        self.process_create_change(change.context_id, context_data).await?;
                    }
                }
                Ok(ProtoOperationType::Update) => {
                    if let Ok(context_data) = serde_json::from_slice(&change.data) {
                        self.process_update_change(change.context_id, context_data).await?;
                    }
                }
                Ok(ProtoOperationType::Delete) => {
                    self.process_delete_change(change.context_id).await?;
                }
                _ => {
                    warn!("Unknown operation type: {}", operation_type);
                }
            }
        }
        
        Ok(())
    }

    /// Process a remote create operation
    async fn process_create_change(&self, context_id: String, context_data: Vec<u8>) -> Result<()> {
        // Deserialize context
        let context: Context = serde_json::from_slice(&context_data)
            .map_err(|e| SquirrelError::mcp(format!("Failed to deserialize context: {e}")))?;

        // Apply change to state manager
        self.state_manager.apply_change(StateChange {
            id: Uuid::new_v4(),
            context_id: Uuid::parse_str(&context_id)
                .map_err(|e| SquirrelError::mcp(format!("Invalid context ID: {e}")))?,
            timestamp: Utc::now(),
            operation: InternalStateOperation::Create,
            data: serde_json::to_value(&context)
                .map_err(|e| SquirrelError::mcp(format!("Failed to serialize context: {e}")))?,
            version: 0, // Will be set by state manager
            name: Some(context.name.clone()),
            metadata: context.metadata.clone(),
            parent_id: context.parent_id,
        }).await.map_err(|e| SquirrelError::mcp(format!("Failed to apply state change: {}", e)))?;

        info!("Applied create change for context: {}", context_id);
        Ok(())
    }
    
    /// Process a remote update operation
    async fn process_update_change(&self, context_id: String, context_data: Vec<u8>) -> Result<()> {
        // Deserialize context
        let context: Context = serde_json::from_slice(&context_data)
            .map_err(|e| SquirrelError::mcp(format!("Failed to deserialize context: {e}")))?;

        // Apply change
        self.state_manager.apply_change(StateChange {
            id: Uuid::new_v4(),
            context_id: Uuid::parse_str(&context_id)
                .map_err(|e| SquirrelError::mcp(format!("Invalid context ID: {e}")))?,
            timestamp: Utc::now(),
            operation: InternalStateOperation::Update,
            data: serde_json::to_value(&context)
                .map_err(|e| SquirrelError::mcp(format!("Failed to serialize context: {e}")))?,
            version: 0, // Will be set by state manager
            name: Some(context.name.clone()),
            metadata: context.metadata.clone(),
            parent_id: context.parent_id,
        }).await.map_err(|e| SquirrelError::mcp(format!("Failed to apply state change: {}", e)))?;

        info!("Applied update change for context: {}", context_id);
        Ok(())
    }
    
    /// Process a remote delete operation
    async fn process_delete_change(&self, context_id: String) -> Result<()> {
        // Apply change to delete the context
        self.state_manager.apply_change(StateChange {
            id: Uuid::new_v4(),
            context_id: Uuid::parse_str(&context_id)
                .map_err(|e| SquirrelError::mcp(format!("Invalid context ID: {e}")))?,
            timestamp: Utc::now(),
            operation: InternalStateOperation::Delete,
            data: serde_json::Value::Null, // No data needed for deletion
            version: 0, // Will be set by state manager
            name: None,
            metadata: None,
            parent_id: None,
        }).await.map_err(|e| SquirrelError::mcp(format!("Failed to apply state change: {}", e)))?;

        info!("Applied delete change for context: {}", context_id);
        Ok(())
    }
    
    /// Convert a state change to a protobuf change
    fn convert_to_proto_change(&self, change: &StateChange) -> ProtoContextChange {
        match state_change_to_proto(change) {
            Ok(proto) => proto,
            Err(e) => {
                error!("Failed to convert state change to proto: {}", e);
                // Create a minimal valid proto change
                ProtoContextChange {
                    operation_type: ProtoOperationType::Unspecified as i32,
                    context_id: change.context_id.to_string(),
                    name: String::new(),
                    parent_id: String::new(),
                    created_at_unix_secs: 0,
                    updated_at_unix_secs: 0,
                    data: Vec::new(),
                    metadata: Vec::new(),
                }
            }
        }
    }

    /// Records a context change operation
    ///
    /// This records both in the state manager and the monitor when a context is modified.
    ///
    /// # Arguments
    /// * `context` - The context being modified
    /// * `operation`
    pub async fn record_context_change(&self, context: &Context, operation: InternalStateOperation) -> Result<()> {
        // Ensure we're initialized
        self.ensure_initialized().await?;

        // Clone the operation for use in the async block
        let op_clone = operation.clone();

        // Record the context change to the state manager
        self.state_manager.record_change(context, op_clone).await
            .map_err(|e| SquirrelError::mcp(format!("Failed to record state change: {}", e)))?;

        // Log the change in the monitor
        self.monitor.record_message(&format!("context_change_{}_{}", operation.as_str(), context.id)).await;
        
        tracing::info!(context_id = %context.id, ?operation, "Recording context change");

        Ok(())
    }

    /// Subscribes to state changes from the underlying state manager.
    ///
    /// # Returns
    /// A Result containing a receiver channel for state changes, or an error.
    ///
    /// # Errors
    /// Returns an error if the state manager fails to subscribe.
    pub async fn subscribe_to_state_changes(&self) -> Result<tokio::sync::broadcast::Receiver<StateChange>> {
        self.ensure_initialized().await?;
        // Delegate to the state manager - subscribe_changes returns Receiver directly
        // Wrap the receiver in Ok before passing to to_core_error
        to_core_error(Ok(self.state_manager.subscribe_changes()))
    }
}

// --- Conversion functions for Protocol Buffers --- 
/// Converts a Rust StateChange object to a Protocol Buffer ContextChange object
///
/// This function handles serialization of the Rust `StateChange` object to the Protocol Buffer format
/// used for network transmission. It converts the operation type, serializes JSON data and metadata
/// to binary, and handles timestamp conversion.
///
/// # Arguments
/// * `change` - The StateChange to convert
///
/// # Returns
/// * `Ok(ProtoContextChange)` if conversion was successful
/// * `Err(MCPError::Serialization)` if JSON serialization fails
fn state_change_to_proto(change: &StateChange) -> std::result::Result<ProtoContextChange, MCPError> {
    // Map the operation type
    let operation_type = match change.operation {
        InternalStateOperation::Create => ProtoOperationType::Create as i32,
        InternalStateOperation::Update => ProtoOperationType::Update as i32,
        InternalStateOperation::Delete => ProtoOperationType::Delete as i32,
        InternalStateOperation::Sync => ProtoOperationType::Unspecified as i32,
    };
    
    // Convert data to bytes
    let data_bytes = serde_json::to_vec(&change.data)
        .map_err(|e| MCPError::InvalidArgument(format!("Failed to serialize data: {}", e)))?;
    
    // Convert metadata to bytes if available
    let metadata_bytes = match &change.metadata {
        Some(metadata) => serde_json::to_vec(metadata)
            .map_err(|e| MCPError::InvalidArgument(format!("Failed to serialize metadata: {}", e)))?,
        None => Vec::new(),
    };
    
    // Build the proto change
    Ok(ProtoContextChange {
        operation_type,
        context_id: change.context_id.to_string(),
        name: change.name.clone().unwrap_or_default(),
        parent_id: change.parent_id.map_or_else(String::new, |id| id.to_string()),
        created_at_unix_secs: change.timestamp.timestamp(), 
        updated_at_unix_secs: change.timestamp.timestamp(), 
        data: data_bytes,
        metadata: metadata_bytes,
    })
}

/// Converts a Protocol Buffer ContextChange object to a Rust StateChange object
///
/// This function handles deserialization of the Protocol Buffer format to the Rust `StateChange`
/// object used internally. It parses the operation type, deserializes JSON data and metadata.
///
/// # Arguments
/// * `proto` - The ProtoContextChange to convert
///
/// # Returns
/// * `Ok(StateChange)` if conversion was successful
/// * `Err(MCPError::Deserialization)` if parsing of any values fails
fn proto_to_state_change(proto: ProtoContextChange) -> std::result::Result<StateChange, MCPError> {
    // Parse the context ID
    let context_id = Uuid::parse_str(&proto.context_id)
        .map_err(|e| MCPError::InvalidArgument(format!("Invalid context ID: {}", e)))?;
    
    // Parse the parent ID if present
    let parent_id = if !proto.parent_id.is_empty() {
        Some(Uuid::parse_str(&proto.parent_id)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid parent ID: {}", e)))?)
    } else {
        None
    };
    
    // Parse the operation type
    let operation = match proto.operation_type {
        x if x == ProtoOperationType::Create as i32 => InternalStateOperation::Create,
        x if x == ProtoOperationType::Update as i32 => InternalStateOperation::Update,
        x if x == ProtoOperationType::Delete as i32 => InternalStateOperation::Delete,
        _ => InternalStateOperation::Sync,
    };
    
    // Parse the data
    let data = if !proto.data.is_empty() {
        serde_json::from_slice(&proto.data)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid data format: {}", e)))?
    } else {
        serde_json::Value::Null
    };
    
    // Parse the metadata if present
    let metadata = if !proto.metadata.is_empty() {
        Some(serde_json::from_slice(&proto.metadata)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid metadata format: {}", e)))?)
    } else {
        None
    };
    
    // Use the timestamps from the proto
    let timestamp = if proto.created_at_unix_secs > 0 {
        match Utc.timestamp_opt(proto.created_at_unix_secs, 0) {
            chrono::LocalResult::Single(ts) => ts,
            _ => Utc::now()
        }
    } else {
        Utc::now()
    };
    
    // Build the state change
    Ok(StateChange {
        id: Uuid::new_v4(),
        context_id,
        operation,
        data,
        timestamp,
        version: 0, // Version will be assigned by the state manager
        name: if proto.name.is_empty() { None } else { Some(proto.name) },
        metadata,
        parent_id,
    })
}

/// Connects to the gRPC sync server
///
/// # Arguments
/// * `server_url` - URL of the sync server (e.g., "[::1]:50051")
/// * `timeout_duration` - Maximum time to wait for connection
///
/// # Returns
/// A Result containing a gRPC SyncServiceClient, or an error
async fn connect_to_server(server_url: &str, timeout_duration: Duration) -> Result<SyncServiceClient<Channel>> {
    // Create channel options with timeout
    let channel = Channel::from_shared(server_url.to_string())
        .map_err(|e| SquirrelError::mcp(format!("Invalid server URL: {}", e)))?
        .timeout(timeout_duration)
        .connect()
        .await
        .map_err(|e| SquirrelError::mcp(format!("Failed to connect to server: {}", e)))?;
    
    // Create client
    let client = SyncServiceClient::new(channel);
    Ok(client)
}

// Re-export StateOperation for use within the crate
pub use crate::sync::state::StateOperation; 