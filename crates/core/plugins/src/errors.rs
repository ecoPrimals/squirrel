//! Error types for the plugin system

use thiserror::Error;
use uuid;

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

    /// Anyhow error
    #[error("General error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    /// Plugin validation error
    #[error("Plugin validation error: {0}")]
    ValidationError(String),

    /// Plugin configuration error
    #[error("Plugin configuration error: {0}")]
    ConfigurationError(String),

    /// Plugin state error
    #[error("Plugin state error: {0}")]
    StateError(String),

    /// Unknown error
    #[error("Unknown plugin error: {0}")]
    Unknown(String),

    /// Plugin not found error (with UUID)
    #[error("Plugin not found with ID: {0}")]
    NotFound(uuid::Uuid),

    /// Plugin already registered
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(uuid::Uuid),

    /// Plugin dependency not found
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),

    /// Plugin dependency cycle detected
    #[error("Plugin dependency cycle detected: {0}")]
    DependencyCycle(uuid::Uuid),

    /// Command not found
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Invalid version format
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Version conflict detected
    #[error("Version conflict: {0}")]
    VersionConflict(String),

    /// Platform incompatibility
    #[error("Platform incompatible: {0}")]
    PlatformIncompatible(String),

    /// Dependency resolution failed
    #[error("Dependency resolution failed: {0}")]
    ResolutionFailed(String),
}

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;
