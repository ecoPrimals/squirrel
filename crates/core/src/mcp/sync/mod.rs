use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use crate::error::{MCPError, Result};
use crate::mcp::types::ProtocolVersion;
use crate::mcp::context_manager::Context;
use crate::mcp::persistence::{MCPPersistence, PersistenceConfig, PersistentState};
use crate::mcp::monitoring::MCPMonitor;
use std::time::Instant;

mod state;
pub use state::{StateChange, StateOperation, StateSyncManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub sync_interval: u64,
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub cleanup_older_than_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub is_syncing: bool,
    pub last_sync: DateTime<Utc>,
    pub sync_count: u64,
    pub error_count: u64,
    pub last_version: u64,
}

#[derive(Debug)]
pub struct MCPSync {
    config: Arc<RwLock<SyncConfig>>,
    state: Arc<RwLock<SyncState>>,
    state_manager: Arc<StateSyncManager>,
    persistence: Arc<MCPPersistence>,
    monitor: Arc<MCPMonitor>,
    lock: Arc<Mutex<()>>,
    initialized: bool,
}

impl MCPSync {
    /// Creates a new MCPSync instance with the given configuration and dependencies
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

    /// Creates a new MCPSync instance with default dependencies
    pub async fn create(config: SyncConfig) -> Result<Self> {
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::new().await?);
        let state_manager = Arc::new(StateSyncManager::new());
        
        Ok(Self::new(config, persistence, monitor, state_manager))
    }

    /// Initializes the MCPSync instance
    pub async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.monitor.record_message("initializing_sync").await;
        
        // Initialize persistence
        self.persistence.init().await?;

        // Try to load persisted state
        if let Some(persisted) = self.persistence.load_state().await? {
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

    /// Helper function to check initialization status and return error if not initialized
    async fn ensure_initialized(&self) -> Result<()> {
        if !self.initialized {
            self.monitor.record_error("not_initialized").await;
            return Err(MCPError::NotInitialized("MCPSync not initialized".into()));
        }
        Ok(())
    }

    async fn load_persisted_changes(&self) -> Result<()> {
        let start = Instant::now();
        let changes = self.persistence.load_changes().await?;
        
        for change in changes {
            if let Err(e) = self.state_manager.apply_change(change).await {
                tracing::error!("Failed to apply persisted change: {}", e);
                self.monitor.record_error("apply_persisted_change_failed").await;
            }
        }

        self.monitor.record_message("persisted_changes_loaded").await;
        self.monitor.record_sync_operation(start.elapsed().as_millis() as f64, true).await;
        Ok(())
    }

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
        for change in changes.iter() {
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
        };
        if let Err(e) = self.persistence.save_state(&persistent_state).await {
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

        // Record sync metrics
        let duration_ms = start.elapsed().as_millis() as f64;
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
            
            if let Err(e) = self.persistence.save_change(&change).await {
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

    pub async fn subscribe_changes(&self) -> Result<tokio::sync::broadcast::Receiver<StateChange>> {
        self.ensure_initialized().await?;
        Ok(self.state_manager.subscribe_changes().await)
    }

    pub async fn update_config(&self, config: SyncConfig) -> Result<()> {
        self.ensure_initialized().await?;
        let mut current_config = self.config.write().await;
        *current_config = config;
        self.monitor.record_message("config_updated").await;
        Ok(())
    }

    pub async fn get_config(&self) -> Result<SyncConfig> {
        self.ensure_initialized().await?;
        let config = self.config.read().await;
        Ok(config.clone())
    }

    pub async fn get_state(&self) -> Result<SyncState> {
        self.ensure_initialized().await?;
        let state = self.state.read().await;
        Ok(state.clone())
    }

    pub async fn record_error(&self) -> Result<()> {
        self.ensure_initialized().await?;
        let mut state = self.state.write().await;
        state.error_count += 1;
        self.monitor.record_error("sync_error").await;
        Ok(())
    }

    pub async fn get_current_version(&self) -> Result<u64> {
        self.ensure_initialized().await?;
        self.state_manager.get_current_version().await
    }

    pub async fn get_monitor(&self) -> Result<Arc<MCPMonitor>> {
        self.ensure_initialized().await?;
        Ok(self.monitor.clone())
    }

    /// Returns a clone of this instance
    pub fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            state_manager: self.state_manager.clone(),
            persistence: self.persistence.clone(),
            monitor: self.monitor.clone(),
            lock: self.lock.clone(),
            initialized: self.initialized,
        }
    }
}

/// Helper function for creating and initializing an MCPSync instance
pub async fn create_mcp_sync(config: SyncConfig) -> Result<MCPSync> {
    let mut sync = MCPSync::create(config).await?;
    sync.init().await?;
    Ok(sync)
}

/// Helper function for creating a customized MCPSync instance with provided dependencies
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sync_flow() {
        let temp_dir = tempdir().unwrap();
        let config = SyncConfig::default();

        let mut sync = MCPSync::create(config).await.unwrap();
        sync.init().await.unwrap();
        
        let context = Context {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        // Record a change
        assert!(sync.record_context_change(&context, StateOperation::Create).await.is_ok());

        // Perform sync
        assert!(sync.sync().await.is_ok());

        // Verify state
        let state = sync.get_state().await.unwrap();
        assert_eq!(state.sync_count, 1);
        assert_eq!(state.error_count, 0);
        assert_eq!(state.last_version, 1);

        // Verify metrics
        let monitor = sync.get_monitor().await.unwrap();
        let metrics = monitor.get_metrics().await.unwrap();
        assert_eq!(metrics.context_operations, 1);
        assert!(metrics.sync_operations >= 1); // At least one sync
        assert_eq!(metrics.total_errors, 0);
    }

    #[tokio::test]
    async fn test_change_subscription() {
        let config = SyncConfig::default();
        let mut sync = MCPSync::create(config).await.unwrap();
        sync.init().await.unwrap();
        
        let mut rx = sync.subscribe_changes().await.unwrap();

        let context = Context {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        // Record change in separate task
        tokio::spawn({
            let sync = sync.clone();
            let context = context.clone();
            async move {
                sync.record_context_change(&context, StateOperation::Create).await.unwrap();
            }
        });

        // Verify change notification
        let change = rx.recv().await.unwrap();
        assert_eq!(change.context_id, context.id);
        assert_eq!(change.version, 1);

        // Verify metrics
        let monitor = sync.get_monitor().await.unwrap();
        let metrics = monitor.get_metrics().await.unwrap();
        assert_eq!(metrics.context_operations, 1);
        assert!(metrics.total_messages >= 2); // At least initialization + context_change_recorded
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = tempdir().unwrap();
        let config = SyncConfig::default();

        // Create first sync instance
        let mut sync1 = MCPSync::create(config.clone()).await.unwrap();
        sync1.init().await.unwrap();
        
        let context = Context {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        // Record change and sync
        assert!(sync1.record_context_change(&context, StateOperation::Create).await.is_ok());
        assert!(sync1.sync().await.is_ok());

        // Create second sync instance
        let mut sync2 = MCPSync::create(config).await.unwrap();
        sync2.init().await.unwrap();
        
        // Verify state was loaded
        let state = sync2.get_state().await.unwrap();
        assert_eq!(state.last_version, 1);

        // Verify metrics
        let monitor = sync2.get_monitor().await.unwrap();
        let metrics = monitor.get_metrics().await.unwrap();
        assert!(metrics.total_messages > 0);
        assert_eq!(metrics.total_errors, 0);
    }
    
    #[tokio::test]
    async fn test_helper_functions() {
        let config = SyncConfig::default();
        
        // Test create_mcp_sync helper
        let sync1 = create_mcp_sync(config.clone()).await.unwrap();
        assert!(sync1.get_state().await.is_ok()); // Should be initialized
        
        // Test create_mcp_sync_with_deps helper
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::new().await.unwrap());
        let state_manager = Arc::new(StateSyncManager::new());
        
        let sync2 = create_mcp_sync_with_deps(
            config,
            persistence,
            monitor,
            state_manager
        ).await.unwrap();
        
        assert!(sync2.get_state().await.is_ok()); // Should be initialized
    }
    
    #[tokio::test]
    async fn test_uninitialized_error() {
        let config = SyncConfig::default();
        let sync = MCPSync::create(config).await.unwrap();
        // Don't call init()
        
        // All operations should fail with NotInitialized error
        assert!(matches!(
            sync.sync().await,
            Err(MCPError::NotInitialized(_))
        ));
        
        let context = Context {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        
        assert!(matches!(
            sync.record_context_change(&context, StateOperation::Create).await,
            Err(MCPError::NotInitialized(_))
        ));
    }
} 