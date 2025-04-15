use std::io;
use thiserror::Error;

/// MCP error types
#[derive(Debug, Error)]
pub enum McpError {
    /// Connection error
    #[error("MCP connection error: {0}")]
    ConnectionError(String),
    
    /// Command error
    #[error("MCP command error: {0}")]
    CommandError(String),
    
    /// Invalid response
    #[error("MCP invalid response: {0}")]
    InvalidResponse(String),
    
    /// Command not found
    #[error("MCP command not found: {0}")]
    CommandNotFound(String),
    
    /// Timeout
    #[error("MCP timeout: {0}")]
    Timeout(String),
    
    /// Internal error
    #[error("MCP internal error: {0}")]
    Internal(String),
    
    /// Authentication error
    #[error("MCP authentication error: {0}")]
    AuthenticationError(String),
    
    /// Context error
    #[error("MCP context error: {0}")]
    ContextError(String),
    
    /// Message error
    #[error("MCP message error: {0}")]
    MessageError(String),
    
    /// Serialization error
    #[error("MCP serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::SerializationError(err.to_string())
    }
}

impl From<io::Error> for McpError {
    fn from(err: io::Error) -> Self {
        McpError::Internal(format!("IO error: {}", err))
    }
}

// Define our own simplified version of MCPError that matches what we expect
#[derive(Debug)]
pub enum SimpleMCPError {
    ConnectionError(String),
    AuthError(String),
    CommandError(String),
    TimeoutError(String),
    SerializationError(String),
    ProtocolError(String),
    IoError(std::io::Error),
    Other(String),
}

impl From<SimpleMCPError> for McpError {
    fn from(err: SimpleMCPError) -> Self {
        match err {
            SimpleMCPError::ConnectionError(err) => McpError::ConnectionError(err),
            SimpleMCPError::AuthError(err) => McpError::AuthenticationError(err),
            SimpleMCPError::CommandError(err) => McpError::CommandError(err),
            SimpleMCPError::TimeoutError(err) => McpError::Timeout(err),
            SimpleMCPError::SerializationError(err) => McpError::SerializationError(err),
            SimpleMCPError::ProtocolError(err) => McpError::MessageError(err),
            SimpleMCPError::IoError(err) => McpError::Internal(err.to_string()),
            SimpleMCPError::Other(err) => McpError::Internal(err),
        }
    }
} 