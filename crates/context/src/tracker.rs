//! Context tracking functionality
//!
//! This module provides functionality for tracking context changes.
//!
//! ## Concurrency and Locking
//!
//! The context tracker uses tokio's asynchronous locks (`Mutex`, `RwLock`) to ensure 
//! thread safety while maintaining good performance in an async environment. 
//! Key locking practices implemented in this module:
//!
//! - Using scope-based locking to minimize lock duration
//! - Avoiding holding locks across `.await` points
//! - Using read locks for operations that don't modify data
//! - Using write locks for operations that modify data
//! - Dropping locks explicitly before async operations
//!
//! When working with the context tracker in asynchronous code, it's important to
//! follow these same patterns to avoid potential deadlocks or performance issues.

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

use crate::{ContextError, ContextState, Result, manager::ContextManager};

/// Configuration for context tracker
#[derive(Debug, Clone)]
pub struct ContextTrackerConfig {
    /// Automatic state synchronization interval in seconds (0 = disabled)
    pub sync_interval_seconds: u64,
    /// Whether to create recovery points automatically
    pub auto_recovery: bool,
    /// Maximum number of recovery points to maintain
    pub max_recovery_points: usize,
}

impl Default for ContextTrackerConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 60,
            auto_recovery: true,
            max_recovery_points: 10,
        }
    }
}

/// Context tracker for managing state changes
#[derive(Debug)]
pub struct ContextTracker {
    /// Current state of the context
    state: Arc<Mutex<ContextState>>,
    /// Configuration
    config: ContextTrackerConfig,
    /// Manager reference if available
    manager: Option<Arc<ContextManager>>,
    /// Active context ID
    active_context_id: Arc<RwLock<Option<String>>>,
    /// Last sync time
    last_sync: Arc<RwLock<Instant>>,
}

impl ContextTracker {
    /// Create a new context tracker with the given state
    #[must_use]
    pub fn new(state: ContextState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            config: ContextTrackerConfig::default(),
            manager: None,
            active_context_id: Arc::new(RwLock::new(None)),
            last_sync: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a new context tracker with configuration and manager
    #[must_use]
    pub fn with_config_and_manager(
        state: ContextState,
        config: ContextTrackerConfig,
        manager: Option<Arc<ContextManager>>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            config,
            manager,
            active_context_id: Arc::new(RwLock::new(None)),
            last_sync: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Get the current state
    pub async fn get_state(&self) -> Result<ContextState> {
        let state = self.state.lock().await;
        Ok(state.clone())
    }

    /// Update the current state
    ///
    /// This method updates the current context state.
    /// It follows best practices for async lock management by:
    /// 1. Acquiring a lock only to read the current state
    /// 2. Dropping the lock before conditional logic
    /// 3. Acquiring a separate lock for the update operation
    /// 4. Using separate lock scopes to minimize lock duration
    /// 5. Not holding any locks during manager operations
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    /// - Failed to create recovery point (if auto_recovery is enabled)
    pub async fn update_state(&self, state: ContextState) -> Result<()> {
        // First check if we need to update
        let should_update = {
            let current_state = self.state.lock().await;
            state.version > current_state.version
        }; // Lock is dropped here
        
        if should_update {
            // Update the state
            {
                let mut current_state = self.state.lock().await;
                *current_state = state.clone();
            } // Lock is dropped here
            
            // Update the last sync time
            {
                let mut last_sync = self.last_sync.write().await;
                *last_sync = Instant::now();
            } // Lock is dropped here
            
            // Trigger automatic recovery point if enabled
            if self.config.auto_recovery {
                if let Some(manager) = &self.manager {
                    // Get state for recovery point
                    let state = self.get_state().await?;
                    
                    // Create recovery point
                    manager.create_recovery_point(&state).await?;
                }
            }
            
            Ok(())
        } else {
            // No update needed, same version or older
            Ok(())
        }
    }

    /// Activate a context by ID
    pub async fn activate_context(&self, id: &str) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Load the context from the manager
            let state = manager.get_context_state(id).await?;
            
            // Set the active context ID
            {
                let mut active_id = self.active_context_id.write().await;
                *active_id = Some(id.to_string());
            }
            
            // Update the state
            self.update_state(state).await?;
            
            Ok(())
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Deactivate the current context
    pub async fn deactivate_context(&self) -> Result<()> {
        let mut active_id = self.active_context_id.write().await;
        // Clear the active context ID
        *active_id = None;
        Ok(())
    }
    
    /// Get the active context ID
    pub async fn get_active_context_id(&self) -> Result<Option<String>> {
        let active_id = self.active_context_id.read().await;
        Ok(active_id.clone())
    }
    
    /// Synchronize state with persistence
    ///
    /// This method synchronizes the current state with the manager.
    /// It follows best practices for async lock management by:
    /// 1. Getting references outside of locks
    /// 2. Getting state without holding other locks
    /// 3. Reading active context ID without holding state lock
    /// 4. Using separate lock scopes to minimize lock duration
    /// 5. Not holding any locks during manager operations
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - No active context
    /// - Context manager not set
    /// - Failed to acquire lock
    /// - Failed to update context state
    pub async fn sync_state(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state().await?;
            
            // Get active context ID without holding lock across await
            let active_id_option = self.active_context_id.read().await.clone();
            
            if let Some(id) = active_id_option {
                // Update the context state in the manager
                manager.update_context_state(&id, state).await?;
                
                // Update the last sync time
                {
                    let mut last_sync = self.last_sync.write().await;
                    *last_sync = Instant::now();
                } // Lock is dropped here
                
                return Ok(());
            }
            
            // If no active context, return an error
            Err(ContextError::NotInitialized("No active context".to_string()))
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Start automatic synchronization for this tracker
    pub async fn synchronize(self: &Arc<Self>) -> Result<()> {
        // Implementation left for actual code
        Ok(())
    }
    
    /// Create a recovery point for the current state
    pub async fn create_recovery_point(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state().await?;
            
            // Create a recovery point
            manager.create_recovery_point(&state).await?;
            
            Ok(())
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Check if synchronization is needed
    pub async fn is_sync_needed(&self) -> bool {
        if self.config.sync_interval_seconds == 0 {
            // Auto-sync disabled
            return false;
        }
        
        // Check time since last sync
        let interval = Duration::from_secs(self.config.sync_interval_seconds);
        let last_sync_time = *self.last_sync.read().await;
        let elapsed = last_sync_time.elapsed();
        
        elapsed > interval
    }
}

/// Factory for creating context trackers
#[derive(Debug)]
pub struct ContextTrackerFactory {
    /// Optional manager reference
    manager: Option<Arc<ContextManager>>,
    /// Optional configuration
    config: Option<ContextTrackerConfig>,
    /// Default state for new trackers
    default_state: Option<ContextState>,
}

impl ContextTrackerFactory {
    /// Create a new factory with the given manager
    pub fn new(manager: Option<Arc<ContextManager>>) -> Self {
        Self {
            manager,
            config: None,
            default_state: None,
        }
    }
    
    /// Create a new factory with the given manager and config
    pub fn with_config(manager: Option<Arc<ContextManager>>, config: ContextTrackerConfig) -> Self {
        Self {
            manager,
            config: Some(config),
            default_state: None,
        }
    }
    
    /// Set the default state for new trackers
    pub fn set_default_state(&mut self, state: ContextState) {
        self.default_state = Some(state);
    }
    
    /// Create a new context tracker
    pub fn create(&self) -> Result<ContextTracker> {
        // Create a default state if none provided
        let state = if let Some(state) = &self.default_state {
            state.clone()
        } else {
            // Create a new empty state
            ContextState {
                id: Uuid::new_v4().to_string(),
                version: 1,
                timestamp: Utc::now().timestamp() as u64,
                data: HashMap::new(),
                metadata: HashMap::new(),
                synchronized: false,
            }
        };
        
        // Create with config if provided
        if let Some(config) = &self.config {
            Ok(ContextTracker::with_config_and_manager(
                state,
                config.clone(),
                self.manager.clone(),
            ))
        } else {
            // Create with defaults
            let tracker = ContextTracker::new(state);
            
            // Set manager if provided
            if self.manager.is_some() {
                // Create a new tracker with our configuration and manager
                let config = tracker.config.clone();
                let manager_clone = self.manager.clone();
                
                // Create a new state
                let empty_state = ContextState {
                    id: Uuid::new_v4().to_string(),
                    version: 1,
                    timestamp: Utc::now().timestamp() as u64,
                    data: HashMap::new(),
                    metadata: HashMap::new(),
                    synchronized: false,
                };
                
                Ok(ContextTracker::with_config_and_manager(
                    empty_state,
                    config,
                    manager_clone,
                ))
            } else {
                Ok(tracker)
            }
        }
    }
    
    /// Create a new tracker as an Arc
    pub fn create_tracker(&self, state: ContextState) -> Arc<ContextTracker> {
        let config = self.config.clone().unwrap_or_default();
        
        Arc::new(ContextTracker::with_config_and_manager(
            state,
            config,
            self.manager.clone(),
        ))
    }
    
    /// Create a new tracker with the given config
    pub fn create_with_config(&self, config: ContextTrackerConfig) -> Result<ContextTracker> {
        // Create a default state if none provided
        let state = if let Some(state) = &self.default_state {
            state.clone()
        } else {
            // Create a new empty state
            ContextState {
                id: Uuid::new_v4().to_string(),
                version: 1,
                timestamp: Utc::now().timestamp() as u64,
                data: HashMap::new(),
                metadata: HashMap::new(),
                synchronized: false,
            }
        };
        
        Ok(ContextTracker::with_config_and_manager(
            state,
            config,
            self.manager.clone(),
        ))
    }
} 