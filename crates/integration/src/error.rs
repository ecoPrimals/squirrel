//! Error types for the integration crate.
//!
//! This module provides error types for various integration components.

use thiserror::Error;

/// Error type for integration operations
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// Error in MCP operations
    #[error("MCP error: {0}")]
    Mcp(String),
    
    /// Error in Context operations
    #[error("Context error: {0}")]
    Context(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Timeout error
    #[error("Timeout error")]
    Timeout,
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for integration operations
pub type Result<T> = std::result::Result<T, IntegrationError>; 