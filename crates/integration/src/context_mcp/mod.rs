// A module that integrates the Squirrel context system with the MCP context manager
// providing bidirectional synchronization and AI-enhanced context capabilities
//
// AI-enhanced context capabilities include:
// - Apply AI insights to contexts
// - Summarize contexts
// - Identify patterns and make recommendations
// - Batch processing of contexts

use async_trait::async_trait;
use serde_json;

mod adapter;
mod ai_enhancement;
mod batch;
mod circuit_breaker;
pub mod config;
pub mod errors;
pub mod sync;
pub mod types;

// Re-exports from this module
pub use adapter::ContextMcpAdapter;
pub use batch::find_contexts_by_tags;
pub use config::ContextMcpAdapterConfig;
pub use errors::ContextMcpError;

pub use circuit_breaker::CircuitBreaker;

// Re-export AI enhancement types
pub use config::{
    ContextAiEnhancementOptions, ContextEnhancementOptions, ContextEnhancementType,
};

// Re-export from ai_enhancement
pub use ai_enhancement::apply_ai_enhancement;

// Re-export sync types
pub use sync::{SyncOptions, SyncResult, SyncStatus};

// Sync direction enum for specifying sync operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Sync from Squirrel to MCP
    SquirrelToMcp,
    /// Sync from MCP to Squirrel
    McpToSquirrel,
}

// Status tracking for the adapter
#[derive(Debug, Clone, Default)]
pub struct AdapterStatus {
    /// Number of successful sync operations
    pub sync_count: u64,
    /// Number of errors encountered
    pub error_count: u64,
    /// Last error message
    pub last_error: Option<String>,
    /// Last sync timestamp
    pub last_sync: Option<u64>,
}

/// Trait for context management operations
#[async_trait]
pub trait ContextManager: Send + Sync + 'static {
    /// Create a new context with the given data
    async fn create_context(&self, data: serde_json::Value) -> std::result::Result<String, String>;
    
    /// Update an existing context with new data
    async fn update_context(&self, id: &str, data: serde_json::Value) -> std::result::Result<(), String>;
    
    /// Delete a context by ID
    async fn delete_context(&self, id: &str) -> std::result::Result<(), String>;
    
    /// Retrieve context data by ID
    async fn with_context(&self, id: &str) -> std::result::Result<serde_json::Value, String>;
    
    /// List all context IDs
    async fn list_contexts(&self) -> std::result::Result<Vec<String>, String>;
}

// Mock context manager for initialization
#[derive(Debug, Default, Clone)]
struct MockContextManager;

#[async_trait]
impl ContextManager for MockContextManager {
    async fn create_context(&self, _data: serde_json::Value) -> std::result::Result<String, String> {
        Ok("mock-id".to_string())
    }

    async fn update_context(&self, _id: &str, _data: serde_json::Value) -> std::result::Result<(), String> {
        Ok(())
    }

    async fn delete_context(&self, _id: &str) -> std::result::Result<(), String> {
        Ok(())
    }

    async fn with_context(&self, _id: &str) -> std::result::Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }

    async fn list_contexts(&self) -> std::result::Result<Vec<String>, String> {
        Ok(vec![])
    }
} 