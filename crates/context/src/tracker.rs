//! Context tracking functionality
//!
//! This module provides functionality for tracking context changes.

use std::sync::{Arc, Mutex, RwLock};
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
    pub fn get_state(&self) -> Result<ContextState> {
        let state = self.state.lock()
            .map_err(|_| ContextError::InvalidState("Failed to acquire state lock".to_string()))?;
        Ok(state.clone())
    }

    /// Update the current state
    pub fn update_state(&self, state: ContextState) -> Result<()> {
        let mut current_state = self.state.lock()
            .map_err(|_| ContextError::InvalidState("Failed to acquire state lock".to_string()))?;
        
        // Only update if the new state has a higher version
        if state.version > current_state.version {
            *current_state = state;
            
            // Update the last sync time
            if let Ok(mut last_sync) = self.last_sync.write() {
                *last_sync = Instant::now();
            }
            
            // Trigger automatic recovery point if enabled
            if self.config.auto_recovery {
                self.create_recovery_point()?;
            }
        }
        
        Ok(())
    }
    
    /// Activate a context by ID
    pub async fn activate_context(&self, id: &str) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get context state from manager
            let state = manager.get_context_state(id).await?;
            
            // Update our local state
            self.update_state(state)?;
            
            // Set the active context ID
            if let Ok(mut active_id) = self.active_context_id.write() {
                *active_id = Some(id.to_string());
            }
            
            Ok(())
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Deactivate the current context
    pub async fn deactivate_context(&self) -> Result<()> {
        if let Ok(mut active_id) = self.active_context_id.write() {
            // Clear the active context ID
            *active_id = None;
            Ok(())
        } else {
            Err(ContextError::InvalidState("Failed to acquire context ID lock".to_string()))
        }
    }
    
    /// Get the active context ID
    pub fn get_active_context_id(&self) -> Result<Option<String>> {
        if let Ok(active_id) = self.active_context_id.read() {
            Ok(active_id.clone())
        } else {
            Err(ContextError::InvalidState("Failed to acquire context ID lock".to_string()))
        }
    }
    
    /// Synchronize state with persistence
    pub async fn sync_state(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state()?;
            
            // If we have an active context, sync to that ID
            if let Ok(active_id) = self.active_context_id.read() {
                if let Some(id) = &*active_id {
                    // Update the context state in the manager
                    manager.update_context_state(id, state).await?;
                    
                    // Update the last sync time
                    if let Ok(mut last_sync) = self.last_sync.write() {
                        *last_sync = Instant::now();
                    }
                    
                    return Ok(());
                }
            }
            
            // If no active context, return an error
            Err(ContextError::NotInitialized("No active context".to_string()))
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Synchronize method that can be called on an Arc<ContextTracker>
    pub async fn synchronize(self: &Arc<Self>) -> Result<()> {
        self.sync_state().await
    }
    
    /// Create a recovery point for the current state
    pub fn create_recovery_point(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state()?;
            
            // Create recovery point using manager
            let _ = manager.create_recovery_point(&state);
            
            Ok(())
        } else {
            Err(ContextError::NotInitialized("Context manager not set".to_string()))
        }
    }
    
    /// Check if sync is needed based on configured interval
    pub fn is_sync_needed(&self) -> bool {
        if self.config.sync_interval_seconds == 0 {
            return false; // Auto-sync disabled
        }
        
        if let Ok(last_sync) = self.last_sync.read() {
            let elapsed = last_sync.elapsed();
            let interval = Duration::from_secs(self.config.sync_interval_seconds);
            elapsed >= interval
        } else {
            false
        }
    }
}

/// Factory for creating ContextTracker instances
#[derive(Debug, Clone)]
pub struct ContextTrackerFactory {
    /// Optional manager reference
    manager: Option<Arc<ContextManager>>,
    /// Optional configuration
    config: Option<ContextTrackerConfig>,
    /// Default state for new trackers
    default_state: Option<ContextState>,
}

impl ContextTrackerFactory {
    /// Create a new context tracker factory
    #[must_use]
    pub fn new(manager: Option<Arc<ContextManager>>) -> Self {
        Self {
            manager,
            config: None,
            default_state: None,
        }
    }
    
    /// Create a new context tracker factory with configuration
    #[must_use]
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
        // Use default state or create empty one
        let state = if let Some(default_state) = &self.default_state {
            default_state.clone()
        } else {
            ContextState {
                id: Uuid::new_v4().to_string(),
                version: 1,
                timestamp: Utc::now().timestamp() as u64,
                data: HashMap::new(),
                metadata: HashMap::new(),
                synchronized: false,
            }
        };
        
        // Use configuration if provided
        if let Some(config) = &self.config {
            Ok(ContextTracker::with_config_and_manager(
                state,
                config.clone(),
                self.manager.clone(),
            ))
        } else {
            let tracker = ContextTracker::new(state);
            Ok(tracker)
        }
    }
    
    /// Alias for create() to be used with adapter
    pub fn create_tracker(&self, state: ContextState) -> Arc<ContextTracker> {
        let config = self.config.clone().unwrap_or_default();
        Arc::new(ContextTracker::with_config_and_manager(
            state,
            config,
            self.manager.clone(),
        ))
    }
    
    /// Create a new context tracker with specific configuration
    pub fn create_with_config(&self, config: ContextTrackerConfig) -> Result<ContextTracker> {
        // Use default state or create empty one
        let state = if let Some(default_state) = &self.default_state {
            default_state.clone()
        } else {
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