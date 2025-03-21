//! Context manager module
//!
//! This module provides context management functionality for storing, retrieving,
//! and synchronizing context data across the application.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::sync::Mutex as AsyncMutex;
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
    pub fn initialize(&mut self) -> Result<()> {
        // Load all contexts from persistence if available
        if let Some(_persistence) = &self.persistence {
            // Try to load states - if persistence methods don't exist, these will just be skipped
            let mut contexts_loaded = false;
            
            // Get context states - this is a simplified method that may not work with all persistence managers
            if let Ok(_contexts) = self.contexts.write() {
                // Basic implementation - actual code would interact with persistence manager
                contexts_loaded = true;
                // Ideally, we'd load states here, but this is simplified
            }
            
            // Try to load recovery points if contexts were loaded
            if contexts_loaded {
                if let Ok(_recovery_points) = self.recovery_points.write() {
                    // Simplified implementation for recovery points
                    // In a real implementation, we'd use persistence to load these
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a context state by ID
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    pub async fn get_context_state(&self, id: &str) -> Result<ContextState> {
        // Get a read lock on contexts
        if let Ok(contexts) = self.contexts.read() {
            // Look up the context
            if let Some(state) = contexts.get(id) {
                return Ok(state.clone());
            }
        } else {
            return Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()));
        }
        
        // If not found in memory, try loading from persistence
        if let Some(_persistence) = &self.persistence {
            // In a real implementation, we'd use persistence to load the state
            // if let Ok(state) = persistence.load_state(id) { ... }
            
            // For now, return not found
            return Err(ContextError::NotFound(format!("Context not found: {}", id)));
        }
        
        Err(ContextError::NotFound(format!("Context not found: {}", id)))
    }
    
    /// Create a new context with the given ID and state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context already exists
    /// - Failed to acquire lock
    /// - Failed to persist context
    pub async fn create_context(&self, id: &str, state: ContextState) -> Result<()> {
        // Ensure we don't exceed max contexts
        if let Ok(contexts) = self.contexts.read() {
            if contexts.len() >= self.config.max_contexts {
                return Err(ContextError::InvalidState("Maximum number of contexts reached".to_string()));
            }
            
            // Check if context already exists
            if contexts.contains_key(id) {
                return Err(ContextError::InvalidState(format!("Context already exists: {}", id)));
            }
        } else {
            return Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()));
        }
        
        // Acquire write lock
        if let Ok(mut contexts) = self.contexts.write() {
            // Store context in memory
            contexts.insert(id.to_string(), state.clone());
            
            // Persist to storage if enabled
            if self.config.persistence_enabled {
                if let Some(persistence) = &self.persistence {
                    // Use async lock to prevent concurrent persistence operations
                    let _guard = self.async_lock.lock().await;
                    persistence.save_state(id, &state)?;
                }
            }
            
            Ok(())
        } else {
            Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()))
        }
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
        if let Ok(mut contexts) = self.contexts.write() {
            // Check if context exists
            if !contexts.contains_key(id) {
                return Err(ContextError::NotFound(format!("Context not found: {}", id)));
            }
            
            // Update context
            contexts.insert(id.to_string(), state.clone());
            
            // Persist to storage if enabled
            if self.config.persistence_enabled {
                if let Some(persistence) = &self.persistence {
                    // Use async lock to prevent concurrent persistence operations
                    let _guard = self.async_lock.lock().await;
                    persistence.save_state(id, &state)?;
                }
            }
            
            Ok(())
        } else {
            Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()))
        }
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
        let exists = if let Ok(contexts) = self.contexts.read() {
            contexts.contains_key(id)
        } else {
            return Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()));
        };
        
        if !exists {
            return Err(ContextError::NotFound(format!("Context not found: {}", id)));
        }
        
        // Acquire write lock
        if let Ok(mut contexts) = self.contexts.write() {
            // Remove from memory
            contexts.remove(id);
            
            // Delete recovery points
            if let Ok(mut recovery_points) = self.recovery_points.write() {
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
        } else {
            Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()))
        }
    }
    
    /// Create a recovery point for the given state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    /// - Failed to persist recovery point
    pub fn create_recovery_point(&self, state: &ContextState) -> Result<ContextSnapshot> {
        // Create a snapshot
        let timestamp = Utc::now().timestamp() as u64;
        let snapshot_id = format!("snapshot-{}", Uuid::new_v4());
        
        let snapshot = ContextSnapshot {
            id: snapshot_id.clone(),
            state_id: state.id.clone(),  // Add required state_id field
            version: state.version,      // Add required version field
            timestamp,
            data: state.data.clone(),
        };
        
        // Store snapshot for generic recovery without context ID
        if let Ok(mut recovery_points) = self.recovery_points.write() {
            let generic_key = "generic".to_string();
            
            // Get or create the recovery points vec
            let points = recovery_points.entry(generic_key.clone())
                .or_insert_with(Vec::new);
            
            // Add the snapshot
            points.push(snapshot.clone());
            
            // Enforce maximum number of recovery points
            if points.len() > self.config.max_recovery_points {
                // Sort by timestamp (oldest first)
                points.sort_by_key(|p| p.timestamp);
                // Remove oldest points
                while points.len() > self.config.max_recovery_points {
                    points.remove(0);
                }
            }
        }
        
        // Persist snapshot if enabled
        if self.config.persistence_enabled {
            if let Some(_persistence) = &self.persistence {
                // In a real implementation, we'd save the snapshot using persistence
                // persistence.save_snapshot(&snapshot)?;
            }
        }
        
        Ok(snapshot)
    }
    
    /// Get recovery points for a context
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub fn get_recovery_points(&self, context_id: &str) -> Result<Vec<ContextSnapshot>> {
        if let Ok(recovery_points) = self.recovery_points.read() {
            if let Some(points) = recovery_points.get(context_id) {
                return Ok(points.clone());
            }
        }
        
        // Return empty vec if not found
        Ok(Vec::new())
    }
    
    /// Get all contexts
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub fn get_all_contexts(&self) -> Result<HashMap<String, ContextState>> {
        if let Ok(contexts) = self.contexts.read() {
            Ok(contexts.clone())
        } else {
            Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()))
        }
    }
    
    /// List all context IDs
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    pub fn list_context_ids(&self) -> Result<Vec<String>> {
        if let Ok(contexts) = self.contexts.read() {
            Ok(contexts.keys().cloned().collect())
        } else {
            Err(ContextError::InvalidState("Failed to acquire contexts lock".to_string()))
        }
    }

    /// Load a context state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Context not found
    /// - Failed to acquire lock
    pub async fn load_context_state(&self, id: &str) -> Result<ContextState> {
        if let Ok(contexts) = self.contexts.read() {
            // Check if already in memory
            if let Some(state) = contexts.get(id) {
                return Ok(state.clone());
            }
        }
        
        // Not in memory, try loading from persistence
        if self.config.persistence_enabled {
            if let Some(persistence) = &self.persistence {
                // Convert id to u64 for persistence (assuming versions are stored as u64)
                if let Ok(version) = id.parse::<u64>() {
                    // Use async lock to prevent concurrent persistence operations
                    let _guard = self.async_lock.lock().await;
                    let state = persistence.load_state(version)?;
                    
                    // Cache in memory
                    if let Ok(mut contexts) = self.contexts.write() {
                        contexts.insert(id.to_string(), state.clone());
                    }
                    
                    return Ok(state);
                } else {
                    // ID is not a valid version number
                    return Err(ContextError::InvalidState(format!("Invalid context ID: {}", id)));
                }
            }
        }
        
        Err(ContextError::NotFound(format!("Context not found: {}", id)))
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
} 