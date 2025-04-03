//! Common types for the Context-MCP adapter
//!
//! This module contains the type definitions and trait implementations
//! used throughout the Context-MCP integration.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use squirrel_mcp::resilience::circuit_breaker::BreakerConfig;
use squirrel_context::ContextManagerConfig as SquirrelContextConfig;

/// A structure representing a Squirrel Context
#[derive(Debug, Clone)]
pub struct SquirrelContext {
    /// Context ID
    pub id: String,
    
    /// Context name
    pub name: String,
    
    /// Context data
    pub data: serde_json::Value,
    
    /// Context metadata
    pub metadata: serde_json::Value,
}

/// Define the context manager trait
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Create a new context
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Get a context by ID
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext>;
    
    /// Update a context
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Delete a context
    async fn delete_context(&self, id: &str) -> anyhow::Result<()>;
    
    /// List all contexts
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>>;
}

/// Configuration for the Context-MCP adapter
#[derive(Debug, Clone)]
pub struct ContextMcpAdapterConfig {
    /// MCP context configuration
    pub mcp_config: Option<serde_json::Value>,
    
    /// Squirrel context configuration
    pub context_config: Option<SquirrelContextConfig>,
    
    /// Synchronization interval in seconds
    pub sync_interval_secs: u64,
    
    /// Circuit breaker configuration
    pub circuit_breaker_config: Option<BreakerConfig>,
    
    /// Max retry attempts for operations
    pub max_retries: u32,
    
    /// Timeout for operations in milliseconds
    pub timeout_ms: u64,
}

impl Default for ContextMcpAdapterConfig {
    fn default() -> Self {
        Self {
            mcp_config: None,
            context_config: None,
            sync_interval_secs: 60,
            circuit_breaker_config: None,
            max_retries: 3,
            timeout_ms: 5000,
        }
    }
}

/// Direction of synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Sync from Squirrel context to MCP
    SquirrelToMcp,
    
    /// Sync from MCP to Squirrel context
    McpToSquirrel,
    
    /// Bidirectional synchronization
    Bidirectional,
}

/// Status of the Context-MCP adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStatus {
    /// Is the adapter connected to MCP
    pub connected_to_mcp: bool,
    
    /// Is the adapter connected to Squirrel context
    pub connected_to_context: bool,
    
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    
    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Errors since startup
    pub error_count: u64,
    
    /// Successful syncs since startup
    pub successful_syncs: u64,
}

impl Default for AdapterStatus {
    fn default() -> Self {
        Self {
            connected_to_mcp: false,
            connected_to_context: false,
            circuit_breaker_state: "CLOSED".to_string(),
            last_sync: None,
            error_count: 0,
            successful_syncs: 0,
        }
    }
}

/// Options for AI context enhancement (legacy version)
#[derive(Debug, Clone)]
pub struct AiEnhancementOptions {
    /// AI provider to use (openai, anthropic, gemini)
    pub provider: String,
    
    /// API key for the AI provider
    pub api_key: String,
    
    /// Model to use (optional, defaults to an appropriate model for the provider)
    pub model: Option<String>,
    
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl AiEnhancementOptions {
    /// Create new options with the given provider and API key
    pub fn new(provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            api_key: api_key.into(),
            model: None,
            timeout_ms: None,
        }
    }
    
    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Set the timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Context Manager V2 trait with improved thread safety
///
/// This version provides explicit Send + Sync bounds and uses callbacks
/// instead of direct adapter references to avoid potential Send/Sync issues.
#[async_trait::async_trait]
pub trait ContextManagerV2: Send + Sync + std::fmt::Debug {
    /// Create a new context
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Get a context by ID
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext>;
    
    /// Update a context
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Delete a context
    async fn delete_context(&self, id: &str) -> anyhow::Result<()>;
    
    /// List all contexts
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>>;
    
    /// Register callbacks for adapter interaction
    fn register_callbacks(&mut self, callbacks: ContextManagerCallbacks) {
        // Default empty implementation
        let _ = callbacks; // Suppress unused variable warning
    }
}

/// Callbacks for ContextManagerV2
#[derive(Clone)]
pub struct ContextManagerCallbacks {
    /// Callback for accessing MCP service
    pub mcp_service: Option<Box<dyn Fn(&str) -> anyhow::Result<String> + Send + Sync>>,
    
    /// Callback for logging
    pub log_event: Option<Box<dyn Fn(&str, &str) -> anyhow::Result<()> + Send + Sync>>,
}

// Helper struct to adapt ContextManagerV2 to ContextManager for backward compatibility
#[derive(Debug)]
pub struct ContextManagerWrapper<T: ContextManagerV2> {
    inner: T,
}

impl<T: ContextManagerV2> ContextManagerWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl<T: ContextManagerV2 + 'static> ContextManager for ContextManagerWrapper<T> {
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        self.inner.create_context(id, name, data, metadata).await
    }
    
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext> {
        self.inner.with_context(id).await
    }
    
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        self.inner.update_context(id, data, metadata).await
    }
    
    async fn delete_context(&self, id: &str) -> anyhow::Result<()> {
        self.inner.delete_context(id).await
    }
    
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>> {
        self.inner.list_contexts().await
    }
} 