//! Error types for the Context-MCP adapter
//!
//! This module provides error types for various Context-MCP operations.

use thiserror::Error;

/// Error type for Context-MCP operations
#[derive(Debug, Error)]
pub enum ContextMcpError {
    /// Error in MCP operations
    #[error("MCP error: {0}")]
    McpError(String),
    
    /// Error in Context operations
    #[error("Context error: {0}")]
    ContextError(String),
    
    /// Error in AI operations
    #[error("AI error: {0}")]
    AiError(String),
    
    /// Synchronization error
    #[error("Sync error: {0}")]
    SyncError(String),
    
    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),
    
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// Circuit breaker is open
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

/// Result type for Context-MCP operations
pub type Result<T> = std::result::Result<T, ContextMcpError>; 

/// Convert a string error to a ContextMcpError
pub fn convert_error(error: impl std::fmt::Display) -> String {
    error.to_string()
} 