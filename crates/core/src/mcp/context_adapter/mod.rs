//! MCP Context Adapter
//!
//! This module provides an adapter for connecting the MCP system with the general
//! context system through the context_adapter module. It establishes a clean
//! separation between MCP and the context system.

use std::sync::Arc;
use serde_json::Value;
use crate::error::{Result, SquirrelError};
use crate::context_adapter::{ContextAdapter, AdapterContextData, ContextAdapterConfig};
use thiserror::Error;

/// Errors specific to MCP context adapter operations
#[derive(Debug, Error)]
pub enum MCPContextAdapterError {
    /// Context adapter is not initialized
    #[error("Context adapter not initialized")]
    NotInitialized,
    
    /// Context adapter is already initialized
    #[error("Context adapter already initialized")]
    AlreadyInitialized,
    
    /// The operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Adapter for working with the Context Manager
#[derive(Debug, Clone)]
pub struct ContextManagerAdapter {
    /// The wrapped context adapter implementation
    inner: Option<Arc<ContextAdapter>>,
}

/// Adapter for connecting MCP to the general context system
#[derive(Debug)]
pub struct MCPContextAdapter {
    /// The wrapped context adapter implementation
    inner: Option<Arc<ContextAdapter>>,
}

impl MCPContextAdapter {
    /// Creates a new MCP context adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new MCP context adapter with an existing context adapter
    #[must_use]
    pub fn with_adapter(adapter: Arc<ContextAdapter>) -> Self {
        Self {
            inner: Some(adapter),
        }
    }
    
    /// Initializes the adapter with default configuration
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::AlreadyInitialized` if the adapter
    /// is already initialized.
    pub fn initialize(&mut self) -> Result<()> {
        if self.inner.is_some() {
            return Err(SquirrelError::Other(MCPContextAdapterError::AlreadyInitialized.to_string()));
        }
        
        let adapter = crate::context_adapter::create_context_adapter();
        self.inner = Some(adapter);
        Ok(())
    }
    
    /// Initializes the adapter with a specific configuration
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::AlreadyInitialized` if the adapter
    /// is already initialized.
    pub fn initialize_with_config(&mut self, config: ContextAdapterConfig) -> Result<()> {
        if self.inner.is_some() {
            return Err(SquirrelError::Other(MCPContextAdapterError::AlreadyInitialized.to_string()));
        }
        
        let adapter = crate::context_adapter::create_context_adapter_with_config(config);
        self.inner = Some(adapter);
        Ok(())
    }

    /// Creates a new context
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn create_context(&self, id: String, data: Value) -> Result<()> {
        if let Some(adapter) = &self.inner {
            adapter.create_context(id, data).await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Gets a context by ID
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn get_context(&self, id: &str) -> Result<AdapterContextData> {
        if let Some(adapter) = &self.inner {
            adapter.get_context(id).await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Updates a context
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn update_context(&self, id: &str, data: Value) -> Result<()> {
        if let Some(adapter) = &self.inner {
            adapter.update_context(id, data).await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Deletes a context
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        if let Some(adapter) = &self.inner {
            adapter.delete_context(id).await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Lists all contexts
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn list_contexts(&self) -> Result<Vec<AdapterContextData>> {
        if let Some(adapter) = &self.inner {
            adapter.list_contexts().await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Updates the context configuration
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn update_config(&self, config: ContextAdapterConfig) -> Result<()> {
        if let Some(adapter) = &self.inner {
            adapter.update_config(config).await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Gets the current configuration
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn get_config(&self) -> Result<ContextAdapterConfig> {
        if let Some(adapter) = &self.inner {
            adapter.get_config().await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }

    /// Cleans up expired contexts
    /// 
    /// # Errors
    /// 
    /// Returns `MCPContextAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        if let Some(adapter) = &self.inner {
            adapter.cleanup_expired_contexts().await
        } else {
            Err(SquirrelError::Other(MCPContextAdapterError::NotInitialized.to_string()))
        }
    }
    
    /// Checks if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}

impl Clone for MCPContextAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for MCPContextAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new MCP context adapter
#[must_use]
pub fn create_mcp_context_adapter() -> Arc<MCPContextAdapter> {
    Arc::new(MCPContextAdapter::new())
}

/// Creates a new MCP context adapter with an existing context adapter
#[must_use]
pub fn create_mcp_context_adapter_with_adapter(adapter: Arc<ContextAdapter>) -> Arc<MCPContextAdapter> {
    Arc::new(MCPContextAdapter::with_adapter(adapter))
}

/// Creates a new MCP context adapter and initializes it with default configuration
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub fn create_initialized_mcp_context_adapter() -> Result<Arc<MCPContextAdapter>> {
    let mut adapter = MCPContextAdapter::new();
    adapter.initialize()?;
    Ok(Arc::new(adapter))
}

/// Creates a new MCP context adapter and initializes it with custom configuration
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub fn create_mcp_context_adapter_with_config(config: ContextAdapterConfig) -> Result<Arc<MCPContextAdapter>> {
    let mut adapter = MCPContextAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(Arc::new(adapter))
} 