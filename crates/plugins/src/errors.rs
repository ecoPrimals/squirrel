//! Plugin system error types
//!
//! This module defines the error types used in the plugin system.

use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

/// Plugin system error types
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(Uuid),
    
    /// Plugin not found by name
    #[error("Plugin not found with name: {0}")]
    PluginNotFound(String),
    
    /// Plugin already registered
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(Uuid),
    
    /// Plugin dependency not found (by string ID)
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),
    
    /// Plugin dependency not found (by UUID)
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFoundUuid(Uuid),
    
    /// Plugin dependency cycle detected
    #[error("Plugin dependency cycle detected: {0}")]
    DependencyCycle(Uuid),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Plugin shutdown failed
    #[error("Plugin shutdown failed: {0}")]
    ShutdownFailed(String),
    
    /// Plugin state error
    #[error("Plugin state error: {0}")]
    StateError(String),
    
    /// Plugin loading error
    #[error("Plugin loading error: {0}")]
    LoadingError(String),
    
    /// Plugin validation error
    #[error("Plugin validation error: {0}")]
    ValidationError(String),
    
    /// Security constraint
    #[error("Security constraint: {0}")]
    SecurityConstraint(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for plugin operations
pub type Result<T> = anyhow::Result<T>; 