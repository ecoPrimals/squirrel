//! Error types for the core module
//!
//! This module defines the error types used in the core functionality.

use thiserror::Error;
use tokio::task::JoinError;

/// Errors that can occur in core operations
#[derive(Debug, Error, Clone)]
pub enum CoreError {
    /// An IO error occurred
    #[error("IO error: {0}")]
    Io(String),
    
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

    /// A security error occurred
    #[error("Security error: {0}")]
    Security(String),

    /// A synchronization error occurred
    #[error("Sync error: {0}")]
    Sync(String),
}

/// A Result type alias for core error handling
pub type Result<T> = std::result::Result<T, CoreError>; 

/// Re-export `SquirrelError` from `squirrel_core`
pub use squirrel_core::error::SquirrelError;

/// Import from context module
use crate::context::ContextError;
/// Import from event module
use crate::event::EventError;
/// Import from plugin module
use crate::plugin::{PluginError, SecurityError};
/// Import sandbox error
use crate::plugin::sandbox::SandboxError;

/// Error type for synchronization operations
pub struct SyncError(pub String);

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sync error: {}", self.0)
    }
}

impl std::fmt::Debug for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SyncError({})", self.0)
    }
}

impl std::error::Error for SyncError {}

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

impl From<PluginError> for CoreError {
    fn from(err: PluginError) -> Self {
        Self::Plugin(err.to_string())
    }
}

impl From<SecurityError> for CoreError {
    fn from(err: SecurityError) -> Self {
        Self::Security(err.to_string())
    }
}

impl From<SandboxError> for CoreError {
    fn from(err: SandboxError) -> Self {
        match err {
            SandboxError::PluginNotFound(id) => Self::Plugin(format!("Plugin not found in sandbox: {id}")),
            SandboxError::Creation(msg) => Self::Plugin(format!("Error creating sandbox: {msg}")),
            SandboxError::Destruction(msg) => Self::Plugin(format!("Error destroying sandbox: {msg}")),
            SandboxError::Permission(msg) => Self::Security(format!("Permission error: {msg}")),
            SandboxError::ResourceLimit(msg) => Self::Security(format!("Resource limit exceeded: {msg}")),
            SandboxError::PathAccess(msg) => Self::Security(format!("Path access denied: {msg}")),
            SandboxError::Capability(msg) => Self::Security(format!("Capability not allowed: {msg}")),
            SandboxError::Platform(msg) => Self::Plugin(format!("Platform error: {msg}")),
            SandboxError::Unsupported(msg) => Self::Plugin(format!("Feature not supported: {msg}")),
            SandboxError::Internal(msg) => Self::Plugin(format!("Internal error: {msg}")),
        }
    }
}

impl From<SyncError> for CoreError {
    fn from(err: SyncError) -> Self {
        Self::Sync(err.to_string())
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

impl From<anyhow::Error> for CoreError {
    fn from(err: anyhow::Error) -> Self {
        Self::Config(format!("Anyhow error: {err}"))
    }
}

impl From<JoinError> for CoreError {
    fn from(err: JoinError) -> Self {
        Self::Plugin(format!("Task join error: {err}"))
    }
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
} 
