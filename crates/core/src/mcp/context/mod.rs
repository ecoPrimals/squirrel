pub mod adapter;
pub use adapter::{MCPContextAdapter, create_context_adapter, create_context_adapter_with_context};

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use crate::error::Result;
use crate::mcp::types::{ProtocolVersion, ProtocolState};

/// Configuration for the MCP context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum number of contexts to store
    pub max_contexts: usize,
    /// Time to live for contexts in seconds
    pub ttl_seconds: u64,
    /// Whether to enable automatic cleanup of expired contexts
    pub enable_auto_cleanup: bool,
}

impl Default for ContextConfig {
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
pub struct ContextData {
    /// Unique identifier for the context
    pub id: String,
    /// Data stored in the context
    pub data: Value,
    /// When the context was created
    pub created_at: DateTime<Utc>,
    /// When the context was last updated
    pub updated_at: DateTime<Utc>,
}

/// MCP context manager
#[derive(Debug)]
pub struct MCPContext {
    config: Arc<RwLock<ContextConfig>>,
    contexts: Arc<RwLock<HashMap<String, ContextData>>>,
}

impl MCPContext {
    /// Creates a new context manager with the given configuration
    #[must_use]
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new context manager with dependencies
    #[must_use]
    pub fn with_dependencies(
        config: Arc<RwLock<ContextConfig>>,
        contexts: Arc<RwLock<HashMap<String, ContextData>>>,
    ) -> Self {
        Self {
            config,
            contexts,
        }
    }

    /// Creates a new context
    pub async fn create_context(&self, id: String, data: Value) -> Result<()> {
        let config = self.config.read().await;
        let mut contexts = self.contexts.write().await;

        if contexts.len() >= config.max_contexts {
            return Err("Maximum number of contexts reached".into());
        }

        let now = Utc::now();
        let context_data = ContextData {
            id: id.clone(),
            data,
            created_at: now,
            updated_at: now,
        };

        contexts.insert(id, context_data);
        Ok(())
    }

    /// Gets a context by ID
    pub async fn get_context(&self, id: &str) -> Result<ContextData> {
        let contexts = self.contexts.read().await;
        contexts.get(id)
            .cloned()
            .ok_or_else(|| "Context not found".into())
    }

    /// Updates a context
    pub async fn update_context(&self, id: &str, data: Value) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        
        if let Some(context) = contexts.get_mut(id) {
            context.data = data;
            context.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Context not found".into())
        }
    }

    /// Deletes a context
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.remove(id)
            .map(|_| ())
            .ok_or_else(|| "Context not found".into())
    }

    /// Lists all contexts
    pub async fn list_contexts(&self) -> Result<Vec<ContextData>> {
        let contexts = self.contexts.read().await;
        Ok(contexts.values().cloned().collect())
    }

    /// Updates the context configuration
    pub async fn update_config(&self, config: ContextConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Gets the current configuration
    pub async fn get_config(&self) -> Result<ContextConfig> {
        let config = self.config.read().await;
        Ok((*config).clone())
    }

    /// Cleans up expired contexts
    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_auto_cleanup {
            return Ok(());
        }

        let now = Utc::now();
        let mut contexts = self.contexts.write().await;
        
        contexts.retain(|_, context| {
            let age = now.signed_duration_since(context.updated_at);
            age.num_seconds() < config.ttl_seconds as i64
        });

        Ok(())
    }
}

/// Factory for creating MCP contexts
#[derive(Debug, Default)]
pub struct MCPContextFactory;

impl MCPContextFactory {
    /// Creates a new context with the given configuration
    #[must_use]
    pub fn create_context(config: ContextConfig) -> Arc<MCPContext> {
        Arc::new(MCPContext::new(config))
    }

    /// Creates a new context with dependencies
    #[must_use]
    pub fn create_context_with_dependencies(
        config: Arc<RwLock<ContextConfig>>,
        contexts: Arc<RwLock<HashMap<String, ContextData>>>,
    ) -> Arc<MCPContext> {
        Arc::new(MCPContext::with_dependencies(config, contexts))
    }

    /// Creates a new context adapter
    #[must_use]
    pub fn create_adapter() -> Arc<MCPContextAdapter> {
        create_context_adapter()
    }

    /// Creates a new context adapter with an existing context
    #[must_use]
    pub fn create_adapter_with_context(context: Arc<MCPContext>) -> Arc<MCPContextAdapter> {
        create_context_adapter_with_context(context)
    }
} 