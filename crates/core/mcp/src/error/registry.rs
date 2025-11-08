//! Registry-related error types for the MCP system

use thiserror::Error;

/// Errors that can occur during registry operations
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Service not found in registry
    #[error("Service not found in registry: {0}")]
    ServiceNotFound(String),

    /// Service already registered
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),

    /// Registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Registry corruption detected
    #[error("Registry corruption detected: {0}")]
    CorruptionDetected(String),

    /// Registry access denied
    #[error("Registry access denied: {0}")]
    AccessDenied(String),
}

impl RegistryError {
    /// Create a new service not found error
    pub fn service_not_found(name: impl Into<String>) -> Self {
        Self::ServiceNotFound(name.into())
    }

    /// Create a new service already registered error
    pub fn service_already_registered(name: impl Into<String>) -> Self {
        Self::ServiceAlreadyRegistered(name.into())
    }

    /// Create a new registration failed error
    pub fn registration_failed(msg: impl Into<String>) -> Self {
        Self::RegistrationFailed(msg.into())
    }
}
