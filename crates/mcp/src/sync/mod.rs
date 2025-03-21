use tokio::sync::RwLock;
use std::sync::Arc;
use crate::MCPError;
use crate::sync::state::StateSyncManager;
use crate::context_manager::Context;
use crate::persistence::{MCPPersistence, PersistenceConfig, PersistentState};
use crate::monitoring::MCPMonitor;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Instant;
use std::fmt::Debug;
use uuid;
use std::sync::Mutex;
use squirrel_core::error::{Result, SquirrelError};

/// Convert an MCPError to a SquirrelError
fn to_core_error<T>(result: std::result::Result<T, MCPError>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(SquirrelError::generic(format!("MCP error: {}", err))),
    }
}

/// State synchronization for MCP
pub mod state;
pub use state::{StateOperation, StateChange};

#[cfg(test)]
mod tests;

/// Configuration for MCP state synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
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
}

impl SyncState {
    /// Creates a new SyncState with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_syncing: false,
            last_sync: None,
            last_error: None,
            sync_count: 0,
            error_count: 0,
            last_version: None,
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
    /// Current version after sync
    pub version: u64,
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
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SyncState::new())),
            state_manager,
            persistence,
            monitor,
            lock: Arc::new(Mutex::new(())),
            initialized: Arc::new(RwLock::new(false)),
            changes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new MCPSync instance asynchronously
    ///
    /// # Arguments
    /// * `config` - Configuration for synchronization
    ///
    /// # Returns
    /// A Result containing the new MCPSync instance, or an error
    pub async fn create(config: SyncConfig) -> Result<Self> {
        let _start = Instant::now();
        
        // Create components
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::new().await?);
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
        
        let duration = _start.elapsed();
        instance.monitor.record_message(&format!("sync_creation_time_ms_{}", duration.as_millis())).await;
        
        Ok(instance)
    }
    
    /// Creates a new MCPSync instance synchronously
    ///
    /// This is used in cases where we need a synchronous constructor,
    /// such as in Default implementations. This method uses the synchronous
    /// monitor constructor.
    ///
    /// # Arguments
    /// * `config` - Configuration for synchronization
    ///
    /// # Returns
    /// A new MCPSync instance
    pub fn create_sync(config: SyncConfig) -> Self {
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

    /// Returns a default instance of MCPSync
    ///
    /// This is primarily used in testing or when a fully-featured instance is not required.
    /// This uses synchronous initialization and is suitable for default implementations.
    ///
    /// # Returns
    /// A new MCPSync instance
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
        let _start = Instant::now();
        self.monitor.record_message("sync_initializing").await;
        
        // Result is converted from MCPError to SquirrelError
        to_core_error(self.init_internal().await)
    }
    
    /// Internal implementation of init that returns MCPError
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
                    return Err(MCPError::Storage(format!("Failed to save default state: {persist_err}")));
                }
                
                default_state
            },
            Err(err) => {
                // Handle error
                self.monitor.record_error("load_state_error").await;
                tracing::warn!("Failed to load persisted state: {}", err);
                
                // If we can't load the state, create a default state
                let default_state = PersistentState {
                    contexts: Vec::new(),
                    changes: Vec::new(),
                    last_version: 0,
                    last_sync: Utc::now(),
                    id: Uuid::new_v4().to_string(),
                };
                
                // Return the default state directly (not wrapped in Some)
                default_state
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
    pub async fn ensure_initialized(&self) -> Result<()> {
        to_core_error(self.ensure_initialized_internal().await)
    }
    
    /// Internal implementation of ensure_initialized that returns MCPError
    async fn ensure_initialized_internal(&self) -> std::result::Result<(), MCPError> {
        let initialized = *self.initialized.read().await;
        if !initialized {
            return Err(MCPError::NotInitialized("Sync engine not initialized".to_string()));
        }
        Ok(())
    }

    /// Loads any changes that have been persisted to storage
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn load_persisted_changes(&self) -> Result<()> {
        to_core_error(self.load_persisted_changes_internal().await)
    }
    
    /// Internal implementation of load_persisted_changes that returns MCPError
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
                let _ = self.record_context_change(&context, StateOperation::Create).await;
            }
            
            tracing::info!("Loaded {} contexts from persisted state", state.contexts.len());
        }
        
        Ok(())
    }

    /// Synchronizes the local state with changes from the network
    ///
    /// # Returns
    /// A Result containing synchronization information
    pub async fn sync(&self) -> Result<SyncResult> {
        to_core_error(self.sync_internal().await)
    }
    
    /// Internal implementation of sync that returns MCPError
    async fn sync_internal(&self) -> std::result::Result<SyncResult, MCPError> {
        let _start = Instant::now();
        self.ensure_initialized_internal().await?;
        
        // Check if sync is already in progress
        {
            let state = self.state.read().await;
            if state.is_syncing {
                return Err(MCPError::AlreadyInProgress("Sync already in progress".to_string()));
            }
        }
        
        // Mark as syncing
        {
            let mut state = self.state.write().await;
            state.is_syncing = true;
        }
        
        self.monitor.record_message("sync_started").await;
        
        // Get latest state
        let current_version: u64;
        let changes;
        
        {
            let state = self.state.read().await;
            let version = state.last_version.unwrap_or(0);
            changes = self.state_manager.get_changes_since(version).await?;
            current_version = version + 1; // Increment version
        }
        
        let mut success = true;
        
        // Process changes
        for change in &changes {
            // Apply change to state
            if let Err(e) = self.state_manager.apply_change(change.clone()).await {
                tracing::error!("Failed to apply change: {}", e);
                self.monitor.record_error("apply_change_failed").await;
                success = false;
                continue;
            }
        }
        
        // Persist state
        if let Err(e) = self.persistence.save_state(&PersistentState {
            contexts: vec![],
            changes: changes.clone(),
            last_version: current_version,
            last_sync: Utc::now(),
            id: uuid::Uuid::new_v4().to_string(),
        }) {
            tracing::error!("Failed to persist state: {}", e);
            self.monitor.record_error("persist_state_failed").await;
            success = false;
        }
        
        // Update sync state
        let mut state = self.state.write().await;
        state.sync_count += 1;
        state.last_sync = Some(Utc::now());
        state.last_version = Some(current_version);
        state.is_syncing = false;
        
        self.monitor.record_message("sync_complete").await;
        
        // Record sync metrics
        let elapsed_millis = _start.elapsed().as_millis();
        self.monitor.record_message(&format!("sync_duration_ms_{}", elapsed_millis)).await;
        
        Ok(SyncResult {
            success,
            duration_ms: elapsed_millis as u64,
            changes_processed: changes.len(),
            version: current_version,
        })
    }

    /// Records a context change operation
    ///
    /// This records both in the state manager and the monitor when a context is modified.
    ///
    /// # Arguments
    /// * `context` - The context being modified
    /// * `operation` - The type of operation being performed
    ///
    /// # Errors
    /// Returns an error if the operation fails
    pub async fn record_context_change(&self, context: &Context, operation: StateOperation) -> Result<()> {
        // Check if initialized before proceeding
        self.ensure_initialized().await?;
        
        // Record the change in the state manager
        let result = self.state_manager.record_change(context, operation.clone()).await;
        
        // Also log the operation in the monitor
        self.monitor.record_context_operation(operation, context).await;
        
        // Convert and return the result
        to_core_error(result)
    }

    /// Get the monitor instance
    ///
    /// # Errors
    /// Returns an error if the monitor cannot be retrieved
    pub fn get_monitor(&self) -> Result<Arc<MCPMonitor>> {
        Ok(Arc::clone(&self.monitor))
    }

    /// Subscribe to state change notifications
    ///
    /// Returns a receiver that will be notified of all state changes.
    ///
    /// # Errors
    /// Returns an error if unable to create the subscription
    pub async fn subscribe_changes(&self) -> Result<tokio::sync::broadcast::Receiver<StateChange>> {
        self.ensure_initialized().await?;
        Ok(self.state_manager.subscribe_changes())
    }

    /// Alias for sync() method
    /// 
    /// This is provided for backward compatibility with code that expects
    /// a synchronize() method.
    /// 
    /// # Errors
    /// Returns an error if synchronization fails
    pub async fn synchronize(&self) -> Result<()> {
        match self.sync().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}