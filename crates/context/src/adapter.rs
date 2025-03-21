//! Context adapter module
//!
//! This module provides the adapter functionality for connecting the context system
//! to external components and managing context activation.

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::{Mutex as AsyncMutex, RwLock};
use uuid::Uuid;

use crate::{
    ContextError, ContextState, ContextSnapshot, Result,
    manager::ContextManager,
    tracker::{ContextTracker, ContextTrackerFactory},
};

/// Context adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAdapterConfig {
    /// Default context ID to use if none is specified
    pub default_context_id: String,
    /// Maximum number of concurrently active contexts
    pub max_active_contexts: usize,
    /// Whether to automatically create contexts that don't exist
    pub auto_create_contexts: bool,
    /// Whether to automatically activate the default context on startup
    pub auto_activate_default: bool,
}

impl Default for ContextAdapterConfig {
    fn default() -> Self {
        Self {
            default_context_id: "default".to_string(),
            max_active_contexts: 5,
            auto_create_contexts: true,
            auto_activate_default: true,
        }
    }
}

/// Context adapter status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextStatus {
    /// Context is active
    Active,
    /// Context exists but is not active
    Inactive,
    /// Context does not exist
    NonExistent,
}

/// Context adapter structure
///
/// This structure provides adapter functionality for connecting the context system
/// to external components and managing context activation.
#[derive(Debug)]
pub struct ContextAdapter {
    /// Configuration
    config: ContextAdapterConfig,
    /// Manager reference
    manager: Arc<ContextManager>,
    /// Active contexts by ID
    active_contexts: RwLock<HashMap<String, Arc<ContextTracker>>>,
    /// Current active context ID
    current_context_id: RwLock<String>,
    /// Tracker factory for creating trackers
    tracker_factory: ContextTrackerFactory,
    /// Lock for async operations
    _async_lock: Arc<AsyncMutex<()>>,
}

impl ContextAdapter {
    /// Create a new context adapter
    pub fn new(manager: Arc<ContextManager>) -> Self {
        let config = ContextAdapterConfig::default();
        let config_clone = config.clone(); // Clone to avoid move
        let tracker_factory = ContextTrackerFactory::new(Some(manager.clone()));
        
        Self {
            config: config_clone,
            manager,
            active_contexts: RwLock::new(HashMap::new()),
            current_context_id: RwLock::new(config.default_context_id.clone()),
            tracker_factory,
            _async_lock: Arc::new(AsyncMutex::new(())),
        }
    }
    
    /// Create a new context adapter with configuration
    pub fn with_config(
        manager: Arc<ContextManager>,
        config: ContextAdapterConfig,
    ) -> Self {
        let config_clone = config.clone(); // Clone to avoid move
        let tracker_factory = ContextTrackerFactory::new(Some(manager.clone()));
        
        Self {
            config: config_clone,
            manager,
            active_contexts: RwLock::new(HashMap::new()),
            current_context_id: RwLock::new(config.default_context_id.clone()),
            tracker_factory,
            _async_lock: Arc::new(AsyncMutex::new(())),
        }
    }
    
    /// Initialize the adapter
    ///
    /// This function prepares the adapter for use and activates the default context
    /// if configured to do so.
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to activate default context
    pub async fn initialize(&self) -> Result<()> {
        // Activate the default context if configured
        if self.config.auto_activate_default {
            let default_id = self.config.default_context_id.clone();
            self.activate_context(&default_id).await?;
        }
        
        Ok(())
    }
    
    /// Get the current context ID
    pub async fn get_current_context_id(&self) -> Result<String> {
        let current_id = self.current_context_id.read().await;
        Ok(current_id.clone())
    }
    
    /// Get the current context tracker
    pub async fn get_current_tracker(&self) -> Result<Arc<ContextTracker>> {
        let current_id = self.get_current_context_id().await?;
        self.get_tracker(&current_id).await
    }
    
    /// Get a context tracker by ID
    pub async fn get_tracker(&self, id: &str) -> Result<Arc<ContextTracker>> {
        let active_contexts = self.active_contexts.read().await;
        
        if let Some(tracker) = active_contexts.get(id) {
            Ok(tracker.clone())
        } else {
            Err(ContextError::NotFound(format!("Active context not found: {}", id)))
        }
    }
    
    /// Check if a context is active
    pub async fn is_context_active(&self, id: &str) -> Result<bool> {
        let active_contexts = self.active_contexts.read().await;
        Ok(active_contexts.contains_key(id))
    }
    
    /// Get the status of a context
    pub async fn get_context_status(&self, id: &str) -> Result<ContextStatus> {
        // First check if it's active
        let is_active = self.is_context_active(id).await?;
        
        if is_active {
            return Ok(ContextStatus::Active);
        }
        
        // Check if it exists in the manager
        let context_exists = match self.manager.get_context_state(id).await {
            Ok(_) => true,
            Err(ContextError::NotFound(_)) => false,
            Err(err) => return Err(err),
        };
        
        if context_exists {
            Ok(ContextStatus::Inactive)
        } else {
            Ok(ContextStatus::NonExistent)
        }
    }
    
    /// Activate a context by ID
    pub async fn activate_context(&self, id: &str) -> Result<Arc<ContextTracker>> {
        // Check if already active
        if let Ok(true) = self.is_context_active(id).await {
            // If already active, just return the tracker
            return self.get_tracker(id).await;
        }
        
        // Check if we've reached the maximum
        {
            let active_contexts = self.active_contexts.read().await;
            if active_contexts.len() >= self.config.max_active_contexts {
                return Err(ContextError::InvalidState(
                    "Maximum number of active contexts reached".to_string()
                ));
            }
        }
        
        // Create and add the tracker
        let tracker = {
            // First try to load state from manager
            let state = match self.manager.get_context_state(id).await {
                Ok(state) => state,
                Err(ContextError::NotFound(_)) => {
                    if self.config.auto_create_contexts {
                        // Create a new context state
                        let new_state = ContextState {
                            id: Uuid::new_v4().to_string(),
                            version: 1,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            data: HashMap::new(),
                            metadata: HashMap::new(),
                            synchronized: false,
                        };
                        
                        // Create the context in the manager
                        self.manager.create_context(id, new_state.clone()).await?;
                        
                        new_state
                    } else {
                        return Err(ContextError::NotFound(
                            format!("Context not found and auto-create disabled: {}", id)
                        ));
                    }
                }
                Err(err) => return Err(err),
            };
            
            // Create a new tracker
            let tracker = self.tracker_factory.create_tracker(state);
            
            // Activate the context in the tracker
            tracker.activate_context(id).await?;
            
            tracker
        };
        
        // Store the tracker in active contexts
        {
            let mut active_contexts = self.active_contexts.write().await;
            active_contexts.insert(id.to_string(), tracker.clone());
        }
        
        // Update current context ID
        {
            let mut current_id = self.current_context_id.write().await;
            *current_id = id.to_string();
        }
        
        Ok(tracker)
    }
    
    /// Deactivate a context by ID
    pub async fn deactivate_context(&self, id: &str) -> Result<()> {
        // Check if context is active
        let is_active = self.is_context_active(id).await?;
        
        if !is_active {
            return Err(ContextError::NotFound(format!("Context not active: {}", id)));
        }
        
        // Get the tracker and deactivate
        let tracker = {
            let active_contexts = self.active_contexts.read().await;
            active_contexts.get(id).cloned()
        };
        
        if let Some(tracker) = tracker {
            // Deactivate the context in the tracker
            tracker.deactivate_context().await?;
        }
        
        // Remove from active contexts
        {
            let mut active_contexts = self.active_contexts.write().await;
            active_contexts.remove(id);
        }
        
        // If this was the current context, activate the default context
        {
            let current_id = self.current_context_id.read().await;
            if *current_id == id {
                // Check if default is already active
                if id != self.config.default_context_id {
                    // Drop the read lock before making async calls
                    drop(current_id);
                    
                    // Activate default context
                    self.activate_context(&self.config.default_context_id).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Switch to a different context
    ///
    /// This function activates a new context and sets it as the current context.
    /// If the context does not exist, it will be created if auto_create_contexts is true.
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context does not exist and auto_create_contexts is false
    /// - Maximum number of active contexts reached
    /// - Failed to acquire lock
    pub async fn switch_context(&self, id: &str) -> Result<Arc<ContextTracker>> {
        // Activate the context (this will also set it as current)
        self.activate_context(id).await
    }
    
    /// Create and activate a new context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context already exists
    /// - Maximum number of active contexts reached
    /// - Failed to acquire lock
    pub async fn create_and_activate_context(&self, id: &str) -> Result<Arc<ContextTracker>> {
        // Check if the context exists
        let status = self.get_context_status(id).await?;
        
        if status != ContextStatus::NonExistent {
            return Err(ContextError::InvalidState(
                format!("Context already exists: {}", id)
            ));
        }
        
        // Create a new context state
        let new_state = ContextState {
            id: Uuid::new_v4().to_string(),
            version: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: HashMap::new(),
            metadata: HashMap::new(),
            synchronized: false,
        };
        
        // Create the context in the manager
        self.manager.create_context(id, new_state).await?;
        
        // Activate the context
        self.activate_context(id).await
    }
    
    /// List all context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub async fn list_context_ids(&self) -> Result<Vec<String>> {
        self.manager.list_context_ids().await
    }
    
    /// List all active context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub async fn list_active_context_ids(&self) -> Result<Vec<String>> {
        let active_contexts = self.active_contexts.read().await;
        let ids = active_contexts.keys().cloned().collect();
        Ok(ids)
    }
    
    /// Create a recovery point for the current context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - No active context
    /// - Failed to create recovery point
    pub async fn create_recovery_point(&self) -> Result<ContextSnapshot> {
        let tracker = self.get_current_tracker().await?;
        let state = tracker.get_state().await?;
        self.manager.create_recovery_point(&state).await
    }
}

impl Default for ContextAdapter {
    fn default() -> Self {
        let manager = ContextManager::default();
        Self::new(Arc::new(manager))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_adapter() {
        let manager = Arc::new(ContextManager::new());
        let adapter = ContextAdapter::new(manager);
        
        assert_eq!(adapter.config.default_context_id, "default");
        assert_eq!(adapter.config.max_active_contexts, 5);
        assert!(adapter.config.auto_create_contexts);
        assert!(adapter.config.auto_activate_default);
    }
    
    #[test]
    fn test_adapter_with_config() {
        let manager = Arc::new(ContextManager::new());
        let config = ContextAdapterConfig {
            default_context_id: "custom".to_string(),
            max_active_contexts: 10,
            auto_create_contexts: false,
            auto_activate_default: false,
        };
        
        let adapter = ContextAdapter::with_config(manager, config.clone());
        
        assert_eq!(adapter.config.default_context_id, "custom");
        assert_eq!(adapter.config.max_active_contexts, 10);
        assert!(!adapter.config.auto_create_contexts);
        assert!(!adapter.config.auto_activate_default);
    }
} 