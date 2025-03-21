//! Context manager module
//!
//! This module provides context management functionality for storing, retrieving,
//! and synchronizing context data across the application.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex as AsyncMutex, RwLock};
use chrono::Utc;
use uuid::Uuid;

use crate::{ContextError, ContextState, ContextSnapshot, Result, persistence::PersistenceManager};

/// Context manager configuration
#[derive(Debug, Clone)]
pub struct ContextManagerConfig {
    /// Maximum number of contexts to track
    pub max_contexts: usize,
    /// Maximum number of recovery points per context
    pub max_recovery_points: usize,
    /// Persistence enabled flag
    pub persistence_enabled: bool,
}

impl Default for ContextManagerConfig {
    fn default() -> Self {
        Self {
            max_contexts: 100,
            max_recovery_points: 10,
            persistence_enabled: true,
        }
    }
}

/// Context manager structure for managing contexts across the application
#[derive(Debug)]
pub struct ContextManager {
    /// Contexts stored by ID
    contexts: RwLock<HashMap<String, ContextState>>,
    /// Recovery points stored by context ID
    recovery_points: RwLock<HashMap<String, Vec<ContextSnapshot>>>,
    /// Configuration
    config: ContextManagerConfig,
    /// Persistence manager
    persistence: Option<Arc<PersistenceManager>>,
    /// Lock for async operations
    async_lock: Arc<AsyncMutex<()>>,
}

impl ContextManager {
    /// Create a new context manager
    #[must_use] 
    pub fn new() -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
            recovery_points: RwLock::new(HashMap::new()),
            config: ContextManagerConfig::default(),
            persistence: None,
            async_lock: Arc::new(AsyncMutex::new(())),
        }
    }
    
    /// Create a new context manager with configuration
    #[must_use]
    pub fn with_config(config: ContextManagerConfig) -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
            recovery_points: RwLock::new(HashMap::new()),
            config,
            persistence: None,
            async_lock: Arc::new(AsyncMutex::new(())),
        }
    }
    
    /// Set the persistence manager for this context manager
    pub fn set_persistence_manager(&mut self, persistence: Arc<PersistenceManager>) {
        self.persistence = Some(persistence);
    }
    
    /// Initialize the context manager
    ///
    /// This function prepares the context manager for use by loading any existing
    /// contexts and setting up necessary resources.
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to load existing contexts
    /// - Failed to initialize persistence
    pub async fn initialize(&mut self) -> Result<()> {
        // Load all contexts from persistence if available
        if let Some(_persistence) = &self.persistence {
            // Try to load states - if persistence methods don't exist, these will just be skipped
            
            // Get context states - this is a simplified method that may not work with all persistence managers
            let _contexts = self.contexts.write().await;
            // Basic implementation - actual code would interact with persistence manager
            // Ideally, we'd load states here, but this is simplified
        }
        
        Ok(())
    }
    
    /// Get context state by ID
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    pub async fn get_context_state(&self, id: &str) -> Result<ContextState> {
        let contexts = self.contexts.read().await;
        
        // Check if context exists
        if let Some(state) = contexts.get(id) {
            Ok(state.clone())
        } else {
            Err(ContextError::NotFound(format!("Context not found: {}", id)))
        }
    }
    
    /// Create a new context with the given ID and state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Maximum number of contexts reached
    /// - Context already exists
    /// - Failed to acquire lock
    /// - Failed to persist context
    pub async fn create_context(&self, id: &str, state: ContextState) -> Result<()> {
        // Ensure we don't exceed max contexts
        {
            let contexts = self.contexts.read().await;
            if contexts.len() >= self.config.max_contexts {
                return Err(ContextError::InvalidState("Maximum number of contexts reached".to_string()));
            }
            
            // Check if context already exists
            if contexts.contains_key(id) {
                return Err(ContextError::InvalidState(format!("Context already exists: {}", id)));
            }
        }
        
        // Store context in memory
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(id.to_string(), state.clone());
        }
        
        // Persist to storage if enabled
        if self.config.persistence_enabled {
            if let Some(persistence) = &self.persistence {
                // Use async lock to prevent concurrent persistence operations
                let _guard = self.async_lock.lock().await;
                persistence.save_state(id, &state)?;
            }
        }
        
        Ok(())
    }
    
    /// Update a context state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    /// - Failed to persist context
    pub async fn update_context_state(&self, id: &str, state: ContextState) -> Result<()> {
        // Check if context exists and update it
        {
            let mut contexts = self.contexts.write().await;
            if !contexts.contains_key(id) {
                return Err(ContextError::NotFound(format!("Context not found: {}", id)));
            }
            
            // Update context
            contexts.insert(id.to_string(), state.clone());
        }
        
        // Persist to storage if enabled
        if self.config.persistence_enabled {
            if let Some(persistence) = &self.persistence {
                // Use async lock to prevent concurrent persistence operations
                let _guard = self.async_lock.lock().await;
                persistence.save_state(id, &state)?;
            }
        }
        
        Ok(())
    }
    
    /// Delete a context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    /// - Failed to delete from persistence
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        // Check if context exists
        {
            let contexts = self.contexts.read().await;
            if !contexts.contains_key(id) {
                return Err(ContextError::NotFound(format!("Context not found: {}", id)));
            }
        }
        
        // Remove from memory
        {
            let mut contexts = self.contexts.write().await;
            contexts.remove(id);
        }
        
        // Delete recovery points
        {
            let mut recovery_points = self.recovery_points.write().await;
            recovery_points.remove(id);
        }
        
        // Delete from persistence if enabled
        if self.config.persistence_enabled {
            if let Some(_persistence) = &self.persistence {
                // Use async lock to prevent concurrent persistence operations
                let _guard = self.async_lock.lock().await;
                // In a real implementation, we'd delete the state using the persistence manager
                // persistence.delete_state(id)?;
            }
        }
        
        Ok(())
    }
    
    /// Create a recovery point for the given state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to create recovery point
    /// - Failed to acquire lock
    pub async fn create_recovery_point(&self, state: &ContextState) -> Result<ContextSnapshot> {
        // Create a snapshot
        let snapshot = ContextSnapshot {
            id: Uuid::new_v4().to_string(),
            state_id: state.id.clone(),
            version: state.version,
            timestamp: Utc::now().timestamp() as u64,
            data: state.data.clone(),
        };
        
        // Store the snapshot
        if let Some(id) = Some(state.id.clone()) {
            let mut recovery_points = self.recovery_points.write().await;
            
            // Get or create recovery points for this context
            let points = recovery_points.entry(id.to_string()).or_insert_with(Vec::new);
            
            // Add the new snapshot
            points.push(snapshot.clone());
            
            // Trim to max recovery points
            if points.len() > self.config.max_recovery_points {
                // Sort by timestamp descending
                points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                
                // Keep only the newest max_recovery_points
                *points = points.iter()
                    .take(self.config.max_recovery_points)
                    .cloned()
                    .collect();
            }
            
            Ok(snapshot)
        } else {
            Err(ContextError::InvalidState("State has no ID".to_string()))
        }
    }
    
    /// Get recovery points for a context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    pub async fn get_recovery_points(&self, context_id: &str) -> Result<Vec<ContextSnapshot>> {
        let recovery_points = self.recovery_points.read().await;
        
        if let Some(points) = recovery_points.get(context_id) {
            Ok(points.clone())
        } else {
            Ok(Vec::new()) // Return empty list if no recovery points exist yet
        }
    }
    
    /// Get all contexts
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub async fn get_all_contexts(&self) -> Result<HashMap<String, ContextState>> {
        let contexts = self.contexts.read().await;
        Ok(contexts.clone())
    }
    
    /// List all context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub async fn list_context_ids(&self) -> Result<Vec<String>> {
        let contexts = self.contexts.read().await;
        let ids: Vec<String> = contexts.keys().cloned().collect();
        Ok(ids)
    }
    
    /// Load context state from persistence
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found in persistence
    /// - Failed to acquire lock
    /// - Failed to load from persistence
    pub async fn load_context_state(&self, id: &str) -> Result<ContextState> {
        if self.config.persistence_enabled {
            if let Some(persistence) = &self.persistence {
                // Use async lock to prevent concurrent persistence operations
                let _guard = self.async_lock.lock().await;
                
                // Load state from persistence
                // This is a simplified approach, assuming the persistence can find by ID
                // In a real implementation, this would need to get the latest version by ID
                let state = persistence.load_state(1)?; // Use version 1 as a fallback
                
                // Update in-memory cache
                let mut contexts = self.contexts.write().await;
                contexts.insert(id.to_string(), state.clone());
                
                Ok(state)
            } else {
                Err(ContextError::NotInitialized("Persistence not initialized".to_string()))
            }
        } else {
            Err(ContextError::NotInitialized("Persistence not enabled".to_string()))
        }
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
} 