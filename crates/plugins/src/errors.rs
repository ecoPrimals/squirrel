//! Error types for the plugin system

use thiserror::Error;

/// Plugin system errors
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    
    /// Plugin already exists
    #[error("Plugin already exists: {0}")]
    PluginAlreadyExists(String),
    
    /// Plugin initialization error
    #[error("Plugin initialization error: {0}")]
    InitializationError(String),
    
    /// Plugin dependency error
    #[error("Plugin dependency error: {0}")]
    DependencyError(String),
    
    /// Plugin security error
    #[error("Plugin security error: {0}")]
    SecurityError(String),
    
    /// Plugin I/O error
    #[error("Plugin I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Plugin validation error
    #[error("Plugin validation error: {0}")]
    ValidationError(String),
    
    /// Plugin state error
    #[error("Plugin state error: {0}")]
    StateError(String),
    
    /// Unknown error
    #[error("Unknown plugin error: {0}")]
    Unknown(String),
}

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>; 