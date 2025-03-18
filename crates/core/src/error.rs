//! Error types for the Squirrel project
//!
//! This module defines the main error types and results used throughout the project.

use thiserror::Error;

/// Main error type for the Squirrel project
#[derive(Debug, Error)]
pub enum SquirrelError {
    /// Errors originating from the app module
    #[error("App error: {0}")]
    App(String),

    /// Errors originating from the MCP module
    #[error("MCP error: {0}")]
    MCP(String),
    
    /// Errors originating from the monitoring module
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    
    /// Errors related to security operations
    #[error("Security error: {0}")]
    Security(String),
    
    /// Other miscellaneous errors that don't fit into specific categories
    #[error("Other error: {0}")]
    Other(String),

    /// Error from a lock operation
    #[error("Lock error: {0}")]
    Lock(String),
    
    /// Error from a command operation
    #[error("Command error: {0}")]
    Command(Box<dyn std::error::Error + Send + Sync>),
}

impl SquirrelError {
    /// Determines if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            SquirrelError::App(_) => false,
            SquirrelError::MCP(_) => false,
            SquirrelError::Monitoring(_) => true,
            SquirrelError::Security(_) => false,
            SquirrelError::Other(_) => false,
            SquirrelError::Lock(_) => true,
            SquirrelError::Command(_) => false,
        }
    }
}

// Implement From for &str for SquirrelError
impl From<&str> for SquirrelError {
    fn from(s: &str) -> Self {
        SquirrelError::Other(s.to_string())
    }
}

// Implement From for String for SquirrelError
impl From<String> for SquirrelError {
    fn from(s: String) -> Self {
        SquirrelError::Other(s)
    }
}

// MCP-specific errors are now defined in crate::mcp::error

/// A Result type alias for operations that may return a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>; 