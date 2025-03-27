use thiserror::Error;

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Error type for command operations
#[derive(Error, Debug)]
pub enum CommandError {
    /// Command not found
    #[error("Command not found: {0}")]
    NotFound(String),
    
    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    /// Execution error (added to match expected variant)
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// Registration error
    #[error("Registration error: {0}")]
    RegistrationError(String),
}

/// Convert a string error to a command error
pub fn to_command_error<E: std::fmt::Display>(err: E) -> CommandError {
    CommandError::InternalError(err.to_string())
}

// Implement From trait for conversion from commands::CommandError to our local CommandError
impl From<commands::CommandError> for CommandError {
    fn from(err: commands::CommandError) -> Self {
        match err {
            commands::CommandError::CommandNotFound(msg) => CommandError::NotFound(msg),
            commands::CommandError::RegistrationError(msg) => CommandError::RegistrationError(msg),
            commands::CommandError::ValidationError(msg) => CommandError::ValidationError(msg),
            commands::CommandError::ExecutionError(msg) => CommandError::ExecutionError(msg),
            commands::CommandError::JournalError(err) => CommandError::ExecutionFailed(err.to_string()),
            commands::CommandError::TransactionError(err) => CommandError::ExecutionFailed(err.to_string()),
            commands::CommandError::CommandAlreadyExists(msg) => CommandError::ConfigurationError(format!("Command already exists: {}", msg)),
            commands::CommandError::RegistryError(msg) => CommandError::RegistryError(msg),
            commands::CommandError::ResourceError(msg) => CommandError::InternalError(format!("Resource error: {}", msg)),
            commands::CommandError::AuthenticationError(msg) => CommandError::PermissionDenied(format!("Authentication error: {}", msg)),
            commands::CommandError::AuthorizationError(msg) => CommandError::PermissionDenied(format!("Authorization error: {}", msg)),
            commands::CommandError::ObservabilityError(msg) => CommandError::InternalError(format!("Observability error: {}", msg)),
        }
    }
}

// Implement From trait for conversion from our local CommandError to commands::CommandError
impl From<CommandError> for commands::CommandError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::NotFound(msg) => commands::CommandError::CommandNotFound(msg),
            CommandError::InvalidArguments(msg) => commands::CommandError::ValidationError(format!("Invalid arguments: {}", msg)),
            CommandError::ValidationError(msg) => commands::CommandError::ValidationError(msg),
            CommandError::ExecutionError(msg) => commands::CommandError::ExecutionError(msg),
            CommandError::ExecutionFailed(msg) => commands::CommandError::ExecutionError(msg),
            CommandError::PermissionDenied(msg) => commands::CommandError::AuthorizationError(format!("Permission denied: {}", msg)),
            CommandError::ConfigurationError(msg) => commands::CommandError::ValidationError(format!("Configuration error: {}", msg)),
            CommandError::InternalError(msg) => commands::CommandError::ExecutionError(format!("Internal error: {}", msg)),
            CommandError::PluginError(msg) => commands::CommandError::ExecutionError(format!("Plugin error: {}", msg)),
            CommandError::RegistryError(msg) => commands::CommandError::RegistryError(msg),
            CommandError::RegistrationError(msg) => commands::CommandError::RegistrationError(msg),
        }
    }
} 