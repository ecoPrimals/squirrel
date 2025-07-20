//! Error types for the context module

use thiserror::Error;

/// Error types for context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Plugins are disabled
    #[error("Plugins are disabled")]
    PluginsDisabled,

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    /// Transformation not found
    #[error("Transformation not found: {0}")]
    TransformationNotFound(String),

    /// Transformation failed
    #[error("Transformation failed for {0}: {1}")]
    TransformationFailed(String, String),

    /// Adapter not found
    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    /// Manager not initialized
    #[error("Context manager not initialized")]
    NotInitialized,

    /// Initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Visualization error
    #[error("Visualization error: {0}")]
    VisualizationError(String),

    /// Invalid format error
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for context operations
pub type Result<T> = std::result::Result<T, ContextError>;

impl From<Box<dyn std::error::Error + Send + Sync>> for ContextError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ContextError::Other(err.to_string())
    }
}
