//! Context adapter module
//!
//! This module provides the adapter functionality for connecting the context system
//! to external components and managing context activation.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex as AsyncMutex;

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
    async_lock: Arc<AsyncMutex<()>>,
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
            async_lock: Arc::new(AsyncMutex::new(())),
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
            async_lock: Arc::new(AsyncMutex::new(())),
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
            self.activate_context(&self.config.default_context_id).await?;
        }
        
        Ok(())
    }
    
    /// Get the current active context ID
    pub fn get_current_context_id(&self) -> Result<String> {
        if let Ok(id) = self.current_context_id.read() {
            Ok(id.clone())
        } else {
            Err(ContextError::InvalidState("Failed to acquire current context ID lock".to_string()))
        }
    }
    
    /// Get a reference to the current active context tracker
    pub fn get_current_tracker(&self) -> Result<Arc<ContextTracker>> {
        let id = self.get_current_context_id()?;
        self.get_tracker(&id)
    }
    
    /// Get a reference to a context tracker by ID
    pub fn get_tracker(&self, id: &str) -> Result<Arc<ContextTracker>> {
        if let Ok(contexts) = self.active_contexts.read() {
            if let Some(tracker) = contexts.get(id) {
                return Ok(tracker.clone());
            }
        } else {
            return Err(ContextError::InvalidState("Failed to acquire active contexts lock".to_string()));
        }
        
        Err(ContextError::NotFound(format!("Context not active: {}", id)))
    }
    
    /// Check if a context is active
    pub fn is_context_active(&self, id: &str) -> Result<bool> {
        if let Ok(contexts) = self.active_contexts.read() {
            Ok(contexts.contains_key(id))
        } else {
            Err(ContextError::InvalidState("Failed to acquire active contexts lock".to_string()))
        }
    }
    
    /// Get the status of a context
    pub async fn get_context_status(&self, id: &str) -> Result<ContextStatus> {
        // Check if context is active
        if self.is_context_active(id)? {
            return Ok(ContextStatus::Active);
        }
        
        // Check if context exists but is not active
        let ids = self.manager.list_context_ids()?;
        if ids.contains(&id.to_string()) {
            return Ok(ContextStatus::Inactive);
        }
        
        Ok(ContextStatus::NonExistent)
    }
    
    /// Activate a context
    ///
    /// This function makes a context active and sets it as the current context.
    /// If the context does not exist, it will be created if auto_create_contexts is true.
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context does not exist and auto_create_contexts is false
    /// - Maximum number of active contexts reached
    /// - Failed to acquire lock
    pub async fn activate_context(&self, id: &str) -> Result<Arc<ContextTracker>> {
        // Check if the context is already active
        if let Ok(active) = self.is_context_active(id) {
            if active {
                // If it's already active, just return the tracker
                return self.get_tracker(id);
            }
        }
        
        // Use async lock to prevent concurrent activation
        let _guard = self.async_lock.lock().await;
        
        // Check if the context exists
        let status = self.get_context_status(id).await?;
        
        // If it doesn't exist, create it if auto_create_contexts is true
        if status == ContextStatus::NonExistent {
            if !self.config.auto_create_contexts {
                return Err(ContextError::NotFound(format!("Context not found and auto-creation disabled: {}", id)));
            }
            
            // Create a new empty state
            let state = ContextState::new();
            
            // Create the context
            self.manager.create_context(id, state).await?;
        }
        
        // Check if we've reached the maximum number of active contexts
        if let Ok(contexts) = self.active_contexts.read() {
            if contexts.len() >= self.config.max_active_contexts {
                return Err(ContextError::InvalidState(format!(
                    "Maximum number of active contexts reached: {}",
                    self.config.max_active_contexts
                )));
            }
        }
        
        // Get the context state
        let state = self.manager.get_context_state(id).await?;
        
        // Create a tracker for the context
        let tracker = self.tracker_factory.create_tracker(state);
        
        // Add the tracker to active contexts
        if let Ok(mut contexts) = self.active_contexts.write() {
            contexts.insert(id.to_string(), tracker.clone());
        } else {
            return Err(ContextError::InvalidState("Failed to acquire active contexts lock".to_string()));
        }
        
        // Set as current context
        if let Ok(mut current) = self.current_context_id.write() {
            *current = id.to_string();
        } else {
            return Err(ContextError::InvalidState("Failed to acquire current context ID lock".to_string()));
        }
        
        Ok(tracker)
    }
    
    /// Deactivate a context
    ///
    /// This function makes a context inactive. If the context is the current context,
    /// the default context will be activated.
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context is not active
    /// - Failed to acquire lock
    /// - Failed to activate default context
    pub async fn deactivate_context(&self, id: &str) -> Result<()> {
        // Check if the context is active
        if !self.is_context_active(id)? {
            return Err(ContextError::NotFound(format!("Context not active: {}", id)));
        }
        
        // Use async lock to prevent concurrent deactivation
        let _guard = self.async_lock.lock().await;
        
        // Get the tracker to ensure state is synchronized before deactivating
        if let Ok(tracker) = self.get_tracker(id) {
            tracker.sync_state().await?;
        }
        
        // Remove from active contexts
        if let Ok(mut contexts) = self.active_contexts.write() {
            contexts.remove(id);
        } else {
            return Err(ContextError::InvalidState("Failed to acquire active contexts lock".to_string()));
        }
        
        // If this was the current context, activate the default context
        if let Ok(current) = self.current_context_id.read() {
            if *current == id {
                // Check if default is already active
                if id != self.config.default_context_id {
                    // Activate default context
                    drop(current); // Drop the read lock before acquiring write lock
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
            return Err(ContextError::InvalidState(format!("Context already exists: {}", id)));
        }
        
        // Create a new empty state
        let state = ContextState::new();
        
        // Create the context
        self.manager.create_context(id, state).await?;
        
        // Activate the context
        self.activate_context(id).await
    }
    
    /// List all context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub fn list_context_ids(&self) -> Result<Vec<String>> {
        self.manager.list_context_ids()
    }
    
    /// List active context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub fn list_active_context_ids(&self) -> Result<Vec<String>> {
        if let Ok(contexts) = self.active_contexts.read() {
            Ok(contexts.keys().cloned().collect())
        } else {
            Err(ContextError::InvalidState("Failed to acquire active contexts lock".to_string()))
        }
    }
    
    /// Create a recovery point for the current context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - No active context
    /// - Failed to acquire lock
    /// - Failed to create recovery point
    pub async fn create_recovery_point(&self) -> Result<ContextSnapshot> {
        // Get the current context tracker
        let tracker = self.get_current_tracker()?;
        
        // Get the current state
        let state = tracker.get_state()?;
        
        // Create a recovery point
        self.manager.create_recovery_point(&state)
    }
}

impl Default for ContextAdapter {
    fn default() -> Self {
        let config = ContextAdapterConfig::default();
        let config_clone = config.clone(); // Clone to avoid move
        let manager = Arc::new(ContextManager::new());
        
        Self {
            manager: manager.clone(),
            tracker_factory: ContextTrackerFactory::new(Some(manager)),
            config: config_clone,
            current_context_id: RwLock::new(config.default_context_id.clone()),
            active_contexts: RwLock::new(HashMap::new()),
            async_lock: Arc::new(AsyncMutex::new(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test creating a new adapter
    #[test]
    fn test_new_adapter() {
        let manager = Arc::new(ContextManager::new());
        let adapter = ContextAdapter::new(manager);
        
        assert_eq!(adapter.config.default_context_id, "default");
        assert_eq!(adapter.config.max_active_contexts, 5);
        assert_eq!(adapter.config.auto_create_contexts, true);
        assert_eq!(adapter.config.auto_activate_default, true);
    }
    
    // Test adapter with custom config
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
        assert_eq!(adapter.config.auto_create_contexts, false);
        assert_eq!(adapter.config.auto_activate_default, false);
    }
} 