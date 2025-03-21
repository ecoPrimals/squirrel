//! Error types for the core module
//!
//! This module defines the error types used in the core functionality.

use thiserror::Error;

/// Errors that can occur in core operations
#[derive(Debug, Error)]
pub enum CoreError {
    /// An IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// A database error occurred
    #[error("Database error: {0}")]
    Database(String),
    
    /// A configuration error occurred
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// A context error occurred
    #[error("Context error: {0}")]
    Context(String),
    
    /// A monitoring error occurred
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    
    /// A command error occurred
    #[error("Command error: {0}")]
    Command(String),
    
    /// A serialization/deserialization error occurred
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// A plugin error occurred
    #[error("Plugin error: {0}")]
    Plugin(String),
}

/// A Result type alias for core error handling
pub type Result<T> = std::result::Result<T, CoreError>; 

/// Re-export `SquirrelError` from `squirrel_core`
pub use squirrel_core::error::SquirrelError;

/// Import from context module
use crate::context::ContextError;
/// Import from event module
use crate::event::EventError;

impl From<ContextError> for CoreError {
    fn from(err: ContextError) -> Self {
        Self::Context(err.to_string())
    }
}

impl From<EventError> for CoreError {
    fn from(err: EventError) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<SquirrelError> for CoreError {
    fn from(err: SquirrelError) -> Self {
        match err {
            SquirrelError::Security(msg) => Self::Config(format!("Security: {msg}")),
            SquirrelError::Metric(msg) => Self::Monitoring(format!("Metric: {msg}")),
            SquirrelError::Health(msg) => Self::Monitoring(format!("Health: {msg}")),
            SquirrelError::Alert(msg) => Self::Monitoring(format!("Alert: {msg}")),
            _ => Self::Config(format!("Core: {err}")),
        }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(format!("JSON error: {err}"))
    }
}

impl From<toml::de::Error> for CoreError {
    fn from(err: toml::de::Error) -> Self {
        Self::Serialization(format!("TOML error: {err}"))
    }
} 