use std::sync::Arc;
use serde_json::Value;
use crate::error::Result;
use super::{MCPContext, ContextConfig, ContextData};

/// Adapter for the MCP context to support dependency injection
#[derive(Debug)]
pub struct MCPContextAdapter {
    inner: Option<Arc<MCPContext>>,
}

impl MCPContextAdapter {
    /// Creates a new context adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing context
    #[must_use]
    pub fn with_context(context: Arc<MCPContext>) -> Self {
        Self {
            inner: Some(context),
        }
    }

    /// Creates a new context
    pub async fn create_context(&self, id: String, data: Value) -> Result<()> {
        if let Some(context) = &self.inner {
            context.create_context(id, data).await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(ContextConfig::default());
            Arc::new(context).create_context(id, data).await
        }
    }

    /// Gets a context by ID
    pub async fn get_context(&self, id: &str) -> Result<ContextData> {
        if let Some(context) = &self.inner {
            context.get_context(id).await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(ContextConfig::default());
            Arc::new(context).get_context(id).await
        }
    }

    /// Updates a context
    pub async fn update_context(&self, id: &str, data: Value) -> Result<()> {
        if let Some(context) = &self.inner {
            context.update_context(id, data).await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(ContextConfig::default());
            Arc::new(context).update_context(id, data).await
        }
    }

    /// Deletes a context
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        if let Some(context) = &self.inner {
            context.delete_context(id).await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(ContextConfig::default());
            Arc::new(context).delete_context(id).await
        }
    }

    /// Lists all contexts
    pub async fn list_contexts(&self) -> Result<Vec<ContextData>> {
        if let Some(context) = &self.inner {
            context.list_contexts().await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(ContextConfig::default());
            Arc::new(context).list_contexts().await
        }
    }

    /// Updates the context configuration
    pub async fn update_config(&self, config: ContextConfig) -> Result<()> {
        if let Some(context) = &self.inner {
            context.update_config(config).await
        } else {
            // Initialize on-demand with default configuration
            let context = MCPContext::new(config);
            self.inner = Some(Arc::new(context));
            Ok(())
        }
    }

    /// Gets the current configuration
    pub async fn get_config(&self) -> Result<ContextConfig> {
        if let Some(context) = &self.inner {
            context.get_config().await
        } else {
            Ok(ContextConfig::default())
        }
    }

    /// Cleans up expired contexts
    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        if let Some(context) = &self.inner {
            context.cleanup_expired_contexts().await
        } else {
            Ok(())
        }
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

/// Creates a new context adapter
#[must_use]
pub fn create_context_adapter() -> Arc<MCPContextAdapter> {
    Arc::new(MCPContextAdapter::new())
}

/// Creates a new context adapter with an existing context
#[must_use]
pub fn create_context_adapter_with_context(context: Arc<MCPContext>) -> Arc<MCPContextAdapter> {
    Arc::new(MCPContextAdapter::with_context(context))
} 