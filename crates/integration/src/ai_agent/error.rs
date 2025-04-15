//! AI Agent adapter error types
//!
//! This module provides error types for the AI Agent adapter.

use thiserror::Error;

/// Errors that can occur in the AI Agent adapter
#[derive(Error, Debug)]
pub enum AIAgentError {
    /// Adapter not initialized
    #[error("AI Agent adapter not initialized")]
    NotInitialized,
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Service error
    #[error("AI service error: {0}")]
    ServiceError(String),
    
    /// Timeout error
    #[error("Operation timed out after {0}ms")]
    TimeoutError(u64),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Circuit breaker is open
    #[error("Circuit breaker is open: {0}")]
    CircuitBreakerOpen(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceExceeded(String),
    
    /// AI tools error
    #[error("AI tools error: {0}")]
    AIToolsError(#[from] squirrel_ai_tools::Error),
    
    /// MCP error
    #[error("MCP error: {0}")]
    MCPError(String),
    
    /// Integration error
    #[error("Integration error: {0}")]
    IntegrationError(#[from] crate::error::IntegrationError),
    
    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<squirrel_mcp::MCPError> for AIAgentError {
    fn from(err: squirrel_mcp::MCPError) -> Self {
        AIAgentError::MCPError(err.to_string())
    }
}

/// Result type for AI agent operations
pub type Result<T> = std::result::Result<T, AIAgentError>; 