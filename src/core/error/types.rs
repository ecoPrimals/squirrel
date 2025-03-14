use std::error::Error;
use std::fmt;
use thiserror::Error;

/// Core error type for the Squirrel project.
#[derive(Debug, Error)]
pub enum SquirrelError {
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Database errors
    #[error("Context error: {0}")]
    Context(String),

    /// Authentication errors
    #[error("Auth error: {0}")]
    Auth(String),

    /// Command execution errors
    #[error("Command error: {0}")]
    Command(String),

    /// Event handling errors
    #[error("Event error: {0}")]
    Event(String),

    /// Metrics errors
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// Encryption errors
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// MCP protocol errors
    #[error("MCP error: {0}")]
    Mcp(String),

    /// Generic errors
    #[error("Other error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Result type alias for Squirrel operations
pub type Result<T> = std::result::Result<T, SquirrelError>;

// Implement conversion from string errors
impl From<String> for SquirrelError {
    fn from(err: String) -> Self {
        Self::Other(Box::new(err))
    }
}

// Implement conversion from &str errors
impl From<&str> for SquirrelError {
    fn from(err: &str) -> Self {
        Self::Other(Box::new(err.to_string()))
    }
}

// Explicitly implement Send and Sync
unsafe impl Send for SquirrelError {}
unsafe impl Sync for SquirrelError {} 