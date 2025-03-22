//! Context System Adapter for MCP
//!
//! This module provides an adapter layer between the general context system
//! and the MCP-specific context requirements. It establishes a clear boundary
//! between these systems and enables proper separation of concerns.
//!
//! ## Concurrency and Locking
//!
//! The context adapter uses tokio's asynchronous locks (`RwLock`) to ensure 
//! thread safety while maintaining good performance in an async environment. 
//! Key locking practices implemented in this module:
//!
//! - Using scope-based locking to minimize lock duration
//! - Avoiding holding locks across `.await` points
//! - Using read locks for operations that don't modify data
//! - Using write locks for operations that modify data
//! - Dropping locks explicitly before async operations
//!
//! When working with the context adapter in asynchronous code, it's important to
//! follow these same patterns to avoid potential deadlocks or performance issues.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use chrono::{DateTime, Utc};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

use squirrel_context::{ContextState, ContextError as GenericContextError};
use squirrel_core::error::{Result, SquirrelError};

/// Errors specific to context adapter operations
#[derive(Debug, Error)]
pub enum ContextAdapterError {
    /// Context is not initialized
    #[error("Context not initialized")]
    NotInitialized,
    
    /// Context is already initialized
    #[error("Context already initialized")]
    AlreadyInitialized,
    
    /// The operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    /// Original context system error
    #[error("Context error: {0}")]
    ContextError(#[from] GenericContextError),
}

/// Configuration for the context adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAdapterConfig {
    /// Maximum number of contexts to store
    pub max_contexts: usize,
    /// Time to live for contexts in seconds
    pub ttl_seconds: u64,
    /// Whether to enable automatic cleanup of expired contexts
    pub enable_auto_cleanup: bool,
}

impl Default for ContextAdapterConfig {
    fn default() -> Self {
        Self {
            max_contexts: 1000,
            ttl_seconds: 3600,
            enable_auto_cleanup: true,
        }
    }
}

/// Data stored in a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterContextData {
    /// Unique identifier for the context
    pub id: String,
    /// Data stored in the context
    pub data: Value,
    /// When the context was created
    pub created_at: DateTime<Utc>,
    /// When the context was last updated
    pub updated_at: DateTime<Utc>,
}

/// Context adapter for connecting the general context system to MCP
#[derive(Debug)]
pub struct ContextAdapter {
    /// Configuration for the context adapter
    config: Arc<RwLock<ContextAdapterConfig>>,
    /// Map of context ID to context data
    contexts: Arc<RwLock<HashMap<String, AdapterContextData>>>,
    // Additional fields for integration with the general context system would go here
}

impl ContextAdapter {
    /// Creates a new context adapter with the given configuration
    #[must_use]
    pub fn new(config: ContextAdapterConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for ContextAdapter {
    fn default() -> Self {
        Self::new(ContextAdapterConfig::default())
    }
}

impl ContextAdapter {
    /// Creates a new context
    ///
    /// This method creates a new context with the specified ID and data.
    /// It follows best practices for async lock management by:
    /// 1. Using a read lock for initial validation
    /// 2. Dropping the read lock before acquiring a write lock 
    /// 3. Using separate lock scopes to minimize lock duration
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum number of contexts has been reached.
    pub async fn create_context(&self, id: String, data: Value) -> Result<()> {
        // First check if we can create the context
        let max_contexts = {
            let config = self.config.read().await;
            config.max_contexts
        }; // Config lock is dropped here
        
        {
            let contexts = self.contexts.read().await;
            if contexts.len() >= max_contexts {
                return Err(SquirrelError::Other(
                    ContextAdapterError::OperationFailed("Maximum number of contexts reached".to_string()).to_string()
                ));
            }
        } // Read lock is dropped here

        // Create the context with write lock
        let now = Utc::now();
        let context_data = AdapterContextData {
            id: id.clone(),
            data,
            created_at: now,
            updated_at: now,
        };

        let mut contexts = self.contexts.write().await;
        contexts.insert(id, context_data);
        Ok(())
    }

    /// Gets a context by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the context with the specified ID doesn't exist.
    pub async fn get_context(&self, id: &str) -> Result<AdapterContextData> {
        let contexts = self.contexts.read().await;
        contexts.get(id)
            .cloned()
            .ok_or_else(|| SquirrelError::Other(
                ContextAdapterError::OperationFailed("Context not found".to_string()).to_string()
            ))
    }

    /// Updates a context
    ///
    /// # Errors
    ///
    /// Returns an error if the context with the specified ID doesn't exist.
    pub async fn update_context(&self, id: &str, data: Value) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        
        if let Some(context) = contexts.get_mut(id) {
            context.data = data;
            context.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SquirrelError::Other(
                ContextAdapterError::OperationFailed("Context not found".to_string()).to_string()
            ))
        }
    }

    /// Deletes a context
    ///
    /// # Errors
    ///
    /// Returns an error if the context with the specified ID doesn't exist.
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if contexts.remove(id).is_some() {
            Ok(())
        } else {
            Err(SquirrelError::Other(
                ContextAdapterError::OperationFailed("Context not found".to_string()).to_string()
            ))
        }
    }

    /// Lists all contexts
    ///
    /// # Errors
    ///
    /// This function does not currently return errors, but maintains the Result
    /// return type for compatibility with other methods and potential future error cases.
    pub async fn list_contexts(&self) -> Result<Vec<AdapterContextData>> {
        let contexts = self.contexts.read().await;
        
        let mut result = Vec::new();
        for context in contexts.values() {
            result.push(context.clone());
        }
        
        Ok(result)
    }

    /// Updates the context configuration
    ///
    /// # Errors
    ///
    /// This function does not currently return errors, but maintains the Result
    /// return type for compatibility with other methods and potential future error cases.
    pub async fn update_config(&self, config: ContextAdapterConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Gets the current configuration
    ///
    /// # Errors
    ///
    /// This function does not currently return errors, but maintains the Result
    /// return type for compatibility with other methods and potential future error cases.
    pub async fn get_config(&self) -> Result<ContextAdapterConfig> {
        let config = self.config.read().await;
        Ok((*config).clone())
    }

    /// Cleans up expired contexts
    ///
    /// This method removes contexts that have exceeded their time-to-live.
    /// It follows best practices for async lock management by:
    /// 1. Reading configuration without holding the contexts lock
    /// 2. Creating a list of expired IDs before acquiring a write lock
    /// 3. Using separate lock scopes to minimize lock duration
    ///
    /// # Errors
    ///
    /// This function does not currently return errors, but maintains the Result
    /// return type for compatibility with other methods and potential future error cases.
    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        // Get the TTL from config
        let ttl_seconds = {
            let config = self.config.read().await;
            config.ttl_seconds
        }; // Config lock is dropped here
        
        // Get current time
        let now = Utc::now();
        
        // Collect expired context IDs
        let expired_ids = {
            let contexts = self.contexts.read().await;
            contexts
                .iter()
                .filter_map(|(id, data)| {
                    let age = now.signed_duration_since(data.updated_at);
                    
                    // Safe conversion from u64 to i64, capping at i64::MAX if needed
                    #[allow(clippy::cast_possible_wrap)]
                    let ttl_seconds_i64 = if ttl_seconds > i64::MAX as u64 {
                        i64::MAX
                    } else {
                        ttl_seconds as i64
                    };
                    
                    if age.num_seconds() > ttl_seconds_i64 {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
        }; // Read lock is dropped here
        
        // Remove expired contexts
        if !expired_ids.is_empty() {
            let mut contexts = self.contexts.write().await;
            for id in expired_ids {
                contexts.remove(&id);
            }
        } // Write lock is dropped here
        
        Ok(())
    }

    /// Converts a `ContextState` to `AdapterContextData`
    ///
    /// # Arguments
    /// * `state` - The context state to convert
    ///
    /// # Returns
    /// The converted adapter context data with appropriate ID and timestamps
    #[allow(dead_code)]
    fn convert_context_state(state: &ContextState) -> AdapterContextData {
        let now = Utc::now();
        // Generate a new UUID for the ID since ContextState doesn't have an id field
        let id = Uuid::new_v4().to_string();
        
        // Convert HashMap<String, String> to a serde_json::Value
        let json_data = {
            let mut map = serde_json::Map::new();
            for (k, v) in &state.data {
                map.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            serde_json::Value::Object(map)
        };
        
        AdapterContextData {
            id,
            data: json_data,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Factory for creating context adapters
#[derive(Debug, Default)]
pub struct ContextAdapterFactory;

impl ContextAdapterFactory {
    /// Creates a new context adapter with default configuration
    #[must_use]
    pub fn create_adapter() -> Arc<ContextAdapter> {
        Arc::new(ContextAdapter::default())
    }
    
    /// Creates a new context adapter with the given configuration
    #[must_use]
    pub fn create_adapter_with_config(config: ContextAdapterConfig) -> Arc<ContextAdapter> {
        Arc::new(ContextAdapter::new(config))
    }
}

/// Creates a new context adapter with default configuration
#[must_use]
pub fn create_context_adapter() -> Arc<ContextAdapter> {
    ContextAdapterFactory::create_adapter()
}

/// Creates a new context adapter with the given configuration
#[must_use]
pub fn create_context_adapter_with_config(config: ContextAdapterConfig) -> Arc<ContextAdapter> {
    ContextAdapterFactory::create_adapter_with_config(config)
}