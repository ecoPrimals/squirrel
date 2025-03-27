use thiserror::Error;
use std::sync::PoisonError;
use crate::commands::error::CommandError;

/// Result type for adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Error type for adapter operations
#[derive(Error, Debug)]
pub enum AdapterError {
    /// Command not found
    #[error("Command not found: {0}")]
    NotFound(String),
    
    /// Invalid command
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    /// Command execution failed
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Registry error
    #[error("Registry error: {0}")]
    Registry(String),
    
    /// Adapter error
    #[error("Adapter error: {0}")]
    Adapter(String),
    
    /// Registration error
    #[error("Registration error: {0}")]
    Registration(String),
    
    /// Lock error
    #[error("Lock error: {0}")]
    LockError(String),
}

/// Convert mutex poison errors to adapter errors
impl<T> From<PoisonError<T>> for AdapterError {
    fn from(err: PoisonError<T>) -> Self {
        AdapterError::Internal(format!("Mutex poisoned: {}", err))
    }
}

/// Convert command errors to adapter errors
impl From<CommandError> for AdapterError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::NotFound(cmd) => AdapterError::NotFound(cmd),
            CommandError::InvalidArguments(msg) => AdapterError::InvalidCommand(msg),
            CommandError::ExecutionFailed(msg) => AdapterError::ExecutionFailed(msg),
            CommandError::ExecutionError(msg) => AdapterError::ExecutionFailed(msg), 
            CommandError::PermissionDenied(msg) => AdapterError::AuthorizationFailed(msg),
            CommandError::ConfigurationError(msg) => AdapterError::ValidationFailed(msg),
            CommandError::InternalError(msg) => AdapterError::Internal(msg),
            CommandError::ValidationError(msg) => AdapterError::ValidationFailed(msg),
            CommandError::PluginError(msg) => AdapterError::PluginError(msg),
            CommandError::RegistryError(msg) => AdapterError::Registry(msg),
            CommandError::RegistrationError(msg) => AdapterError::Registration(msg),
        }
    }
}

/// Convert commands lib errors to adapter errors
impl From<commands::CommandError> for AdapterError {
    fn from(err: commands::CommandError) -> Self {
        // Convert commands::CommandError to our local CommandError first
        let local_err: CommandError = err.into();
        // Then convert local CommandError to AdapterError
        local_err.into()
    }
}

/// Convert a string error to an adapter error
pub fn to_adapter_error<E: std::fmt::Display>(err: E) -> AdapterError {
    AdapterError::Internal(err.to_string())
} 