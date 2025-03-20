/// Error type definitions for MCP operations
pub mod types;
/// Error context management for MCP operations
pub mod context;

use thiserror::Error;
pub use types::MCPError;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Context initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Context already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Context not found: {0}")]
    NotFound(String),
    
    #[error("Context operation timed out")]
    Timeout,
    
    #[error("Invalid context state: {0}")]
    InvalidState(String),
}

/// Result type for MCP operations that can return an error
pub type Result<T> = std::result::Result<T, MCPError>;

// Re-export specific types from types module
pub use types::{
    ProtocolError,
    ConnectionError,
    SecurityError,
    PortErrorKind,
    ErrorContext,
};

// Re-export specific types from context module
pub use context::{
    ErrorHandlerError,
    ErrorSeverity,
    RecoveryStrategy,
    ErrorRecord,
    ErrorHandler,
}; 