use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use crate::error::Result;
use crate::mcp::MCPError;
use crate::mcp::sync::state::StateSyncManager;
use crate::mcp::context_manager::Context;
use crate::mcp::persistence::{MCPPersistence, PersistenceConfig, PersistentState};
use crate::mcp::monitoring::MCPMonitor;
use std::time::Instant;
use uuid;

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

/// Current state of the synchronization system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Whether a sync operation is currently in progress
    pub is_syncing: bool,
    /// Timestamp of the last successful sync operation
    pub last_sync: DateTime<Utc>,
    /// Total number of successful sync operations
    pub sync_count: u64,
    /// Number of failed sync operations
    pub error_count: u64,
    /// Latest version number of synchronized data
    pub last_version: u64,
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
    initialized: bool,
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
            state: Arc::new(RwLock::new(SyncState {
                is_syncing: false,
                last_sync: Utc::now(),
                sync_count: 0,
                error_count: 0,
                last_version: 0,
            })),
            state_manager,
            persistence,
            monitor,
            lock: Arc::new(Mutex::new(())),
            initialized: false,
        }
    }

    /// Creates a new `MCPSync` instance with default dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if creating the monitor fails or if any of the dependencies
    /// cannot be initialized properly.
    pub async fn create(config: SyncConfig) -> Result<Self> {
        // Create persistence with default config
        let mut persistence = MCPPersistence::new(PersistenceConfig::default());
        
        // Initialize persistence directly
        if let Err(e) = persistence.init() {
            return Err(MCPError::StorageError(format!("Failed to initialize persistence: {e}")).into());
        }
        
        // Wrap in Arc after initialization
        let persistence = Arc::new(persistence);
        
        let monitor = Arc::new(MCPMonitor::new().await?);
        let state_manager = Arc::new(StateSyncManager::new());
        
        Ok(Self::new(config, persistence, monitor, state_manager))
    }

    /// Initializes the `MCPSync` instance
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence layer cannot be initialized,
    /// if loading the persisted state fails, or if applying persisted changes fails.
    pub async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.monitor.record_message("initializing_sync").await;
        
        // Persistence is already initialized in create()
        
        // Try to load persisted state
        if let Some(persisted) = self.persistence.load_state()? {
            self.monitor.record_message("state_loaded").await;
            let mut state = self.state.write().await;
            *state = SyncState {
                is_syncing: false,
                last_sync: persisted.last_sync,
                sync_count: 0,
                error_count: 0,
                last_version: persisted.last_version,
            };
        } else {
            self.monitor.record_message("state_initialized").await;
        }

        // Load and apply persisted changes
        if let Err(e) = self.load_persisted_changes().await {
            self.monitor.record_error("load_persisted_changes_failed").await;
            return Err(e);
        }

        self.initialized = true;
        self.monitor.record_message("sync_initialized").await;
        Ok(())
    }

    /// Ensures the `MCPSync` instance is initialized
    ///
    /// This method is called before performing any synchronization operations
    /// to ensure the instance is properly initialized.
    ///
    /// # Errors
    /// Returns an error if initialization fails
    async fn ensure_initialized(&self) -> Result<()> {
        if !self.initialized {
            self.monitor.record_error("not_initialized").await;
            return Err(MCPError::NotInitialized("MCPSync not initialized".into()).into());
        }
        Ok(())
    }

    /// Loads persisted changes from storage
    ///
    /// This method loads previously saved state changes from persistent storage
    /// and applies them to the current state.
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if the changes couldn't be loaded
    async fn load_persisted_changes(&self) -> Result<()> {
        let start = Instant::now();
        let changes = self.persistence.load_changes()?;
        
        for change in changes {
            if let Err(e) = self.state_manager.apply_change(change).await {
                tracing::error!("Failed to apply persisted change: {}", e);
                self.monitor.record_error("apply_persisted_change_failed").await;
            }
        }

        self.monitor.record_message("persisted_changes_loaded").await;
        let elapsed_millis = start.elapsed().as_millis();
        #[allow(clippy::cast_precision_loss)]
        self.monitor.record_sync_operation(elapsed_millis as f64, true).await;
        Ok(())
    }

    /// Perform a synchronization operation
    ///
    /// Synchronizes the local state with remote instances, ensuring
    /// consistent state across the distributed system.
    ///
    /// # Errors
    /// Returns an error if synchronization fails
    pub async fn sync(&self) -> Result<()> {
        self.ensure_initialized().await?;

        let _guard = self.lock.lock().await;
        let mut state = self.state.write().await;
        let start = Instant::now();
        
        if state.is_syncing {
            return Ok(());
        }

        state.is_syncing = true;
        self.monitor.record_message("sync_started").await;
        
        // Get current version before sync
        let current_version = self.state_manager.get_current_version().await?;
        
        // Get changes since last sync
        let changes = self.state_manager.get_changes_since(state.last_version).await?;
        
        let mut success = true;
        // Apply changes in order
        for change in &changes {
            if let Err(e) = self.state_manager.apply_change(change.clone()).await {
                tracing::error!("Failed to apply change: {}", e);
                self.monitor.record_error("apply_change_failed").await;
                state.error_count += 1;
                success = false;
            }
        }

        // Persist state
        let persistent_state = PersistentState {
            contexts: vec![], // TODO: Get contexts from context manager
            changes,
            last_version: current_version,
            last_sync: Utc::now(),
            id: uuid::Uuid::new_v4().to_string(),
        };
        if let Err(e) = self.persistence.save_state(&persistent_state) {
            tracing::error!("Failed to persist state: {}", e);
            self.monitor.record_error("persist_state_failed").await;
            state.error_count += 1;
            success = false;
        }

        // Update sync state
        state.sync_count += 1;
        state.last_sync = Utc::now();
        state.last_version = current_version;
        state.is_syncing = false;

        self.monitor.record_message("sync_complete").await;
        
        // Record sync metrics
        let elapsed_millis = start.elapsed().as_millis();
        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = f64::from(elapsed_millis as u32);
        self.monitor.record_sync_operation(duration_ms, success).await;

        // Cleanup old changes
        let config = self.config.read().await;
        let cleanup_before = Utc::now() - Duration::days(config.cleanup_older_than_days);
        if let Err(e) = self.state_manager.cleanup_old_changes(cleanup_before).await {
            tracing::error!("Failed to cleanup old changes: {}", e);
            self.monitor.record_error("cleanup_changes_failed").await;
        }

        Ok(())
    }

    /// Record a context change for synchronization
    ///
    /// When a context is created, updated, or deleted, this method
    /// records the change for synchronization across instances.
    ///
    /// # Arguments
    /// * `context` - The context being changed
    /// * `operation` - The type of operation performed
    ///
    /// # Errors
    /// Returns an error if the change cannot be recorded
    pub async fn record_context_change(&self, context: &Context, operation: StateOperation) -> Result<()> {
        self.ensure_initialized().await?;

        // Record change in state manager
        let result = self.state_manager.record_change(context, operation.clone()).await;

        // If successful, persist the change
        if let Ok(()) = result {
            let current_version = self.state_manager.get_current_version().await?;
            let change = StateChange {
                id: uuid::Uuid::new_v4(),
                context_id: context.id,
                operation: operation.clone(),
                data: serde_json::to_value(context)?,
                timestamp: Utc::now(),
                version: current_version,
            };
            
            if let Err(e) = self.persistence.save_change(&change) {
                tracing::error!("Failed to persist change: {}", e);
                self.monitor.record_error("persist_change_failed").await;
            }

            // Record context operation in monitoring
            self.monitor.record_context_operation(operation, context).await;
            self.monitor.record_message("context_change_recorded").await;
        } else if let Err(e) = &result {
            self.monitor.record_error("record_change_failed").await;
            tracing::error!("Failed to record change: {}", e);
        }

        result
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

    /// Update the synchronization configuration
    ///
    /// # Arguments
    /// * `config` - The new configuration to use
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be updated
    pub async fn update_config(&self, config: SyncConfig) -> Result<()> {
        self.ensure_initialized().await?;
        let mut current_config = self.config.write().await;
        *current_config = config;
        self.monitor.record_message("config_updated").await;
        Ok(())
    }

    /// Get the current synchronization configuration
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be retrieved
    pub async fn get_config(&self) -> Result<SyncConfig> {
        self.ensure_initialized().await?;
        let config = self.config.read().await;
        Ok(config.clone())
    }

    /// Get the current synchronization state
    ///
    /// # Errors
    /// Returns an error if the state cannot be retrieved
    pub async fn get_state(&self) -> Result<SyncState> {
        self.ensure_initialized().await?;
        let state = self.state.read().await;
        Ok(state.clone())
    }

    /// Record a synchronization error
    ///
    /// Increments the error counter and logs the error.
    ///
    /// # Errors
    /// Returns an error if the error counter cannot be updated
    pub async fn record_error(&self) -> Result<()> {
        self.ensure_initialized().await?;
        let mut state = self.state.write().await;
        state.error_count += 1;
        self.monitor.record_error("sync_error").await;
        Ok(())
    }

    /// Get the current version number
    ///
    /// # Errors
    /// Returns an error if the version cannot be retrieved
    pub async fn get_current_version(&self) -> Result<u64> {
        self.ensure_initialized().await?;
        self.state_manager.get_current_version().await
    }

    /// Get the monitor instance
    ///
    /// # Errors
    /// Returns an error if the monitor cannot be retrieved
    pub fn get_monitor(&self) -> Result<Arc<MCPMonitor>> {
        Ok(Arc::clone(&self.monitor))
    }
}

impl Clone for MCPSync {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            state_manager: Arc::clone(&self.state_manager),
            persistence: Arc::clone(&self.persistence),
            monitor: Arc::clone(&self.monitor),
            lock: Arc::clone(&self.lock),
            initialized: self.initialized,
        }
    }
}

/// Creates a new `MCPSync` instance with provided configuration
///
/// # Errors
///
/// Returns an error if the instance cannot be created or if any of its
/// dependencies fail to initialize.
pub async fn create_mcp_sync(config: SyncConfig) -> Result<MCPSync> {
    let mut sync = MCPSync::create(config).await?;
    sync.init().await?;
    Ok(sync)
}

/// Creates a new `MCPSync` instance with explicitly provided dependencies
///
/// # Parameters
///
/// * `config` - Synchronization configuration
/// * `persistence` - Persistence layer for storing sync data
/// * `monitor` - Monitoring component for sync operations
/// * `state_manager` - State synchronization manager
///
/// # Errors
///
/// Returns an error if the instance cannot be created with the provided dependencies.
pub async fn create_mcp_sync_with_deps(
    config: SyncConfig,
    persistence: Arc<MCPPersistence>,
    monitor: Arc<MCPMonitor>,
    state_manager: Arc<StateSyncManager>,
) -> Result<MCPSync> {
    let mut sync = MCPSync::new(config, persistence, monitor, state_manager);
    sync.init().await?;
    Ok(sync)
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval: 60, // 1 minute
            max_retries: 3,
            timeout_ms: 5000, // 5 seconds
            cleanup_older_than_days: 7, // Keep changes for 7 days
        }
    }
}

impl Default for MCPSync {
    fn default() -> Self {
        // Create an uninitialized instance with default dependencies
        let config = SyncConfig::default();
        let persistence = Arc::new(MCPPersistence::default());
        let monitor = Arc::new(MCPMonitor::default_sync());
        let state_manager = Arc::new(StateSyncManager::new());
        
        MCPSync::new(config, persistence, monitor, state_manager)
    }
} 