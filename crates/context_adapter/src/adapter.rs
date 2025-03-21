//! Context System Adapter for MCP
//!
//! This module provides an adapter layer between the general context system
//! and the MCP-specific context requirements. It establishes a clear boundary
//! between these systems and enables proper separation of concerns.

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
    /// # Errors
    ///
    /// Returns an error if the maximum number of contexts has been reached.
    pub async fn create_context(&self, id: String, data: Value) -> Result<()> {
        let config = self.config.read().await;
        let mut contexts = self.contexts.write().await;

        if contexts.len() >= config.max_contexts {
            return Err(SquirrelError::Other(
                ContextAdapterError::OperationFailed("Maximum number of contexts reached".to_string()).to_string()
            ));
        }

        let now = Utc::now();
        let context_data = AdapterContextData {
            id: id.clone(),
            data,
            created_at: now,
            updated_at: now,
        };

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
    /// # Errors
    ///
    /// This function does not currently return errors, but maintains the Result
    /// return type for compatibility with other methods and potential future error cases.
    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        let config = {
            let config_guard = self.config.read().await;
            config_guard.clone()
        };
        
        if !config.enable_auto_cleanup {
            return Ok(());
        }

        let now = Utc::now();
        let mut contexts = self.contexts.write().await;
        
        let mut to_remove = Vec::new();
        
        for (id, context) in contexts.iter() {
            let age = now.signed_duration_since(context.updated_at);
            match i64::try_from(config.ttl_seconds) {
                Ok(ttl) => {
                    if age.num_seconds() >= ttl {
                        to_remove.push(id.clone());
                    }
                },
                Err(_) => {}, // If conversion fails, retain the context to be safe
            }
        }
        
        for id in to_remove {
            contexts.remove(&id);
        }

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
        AdapterContextData {
            id,
            data: serde_json::Value::from(state.data.clone()),
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