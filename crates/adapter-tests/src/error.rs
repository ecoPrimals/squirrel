use std::fmt;

/// Error type for command execution and adapter operations
#[derive(Debug)]
pub enum AdapterError {
    /// Indicates a command was not found in the registry
    NotFound(String),
    
    /// Indicates an error occurred during command execution
    Execution(String),
    
    /// Indicates an authorization failure
    AuthorizationFailed {
        /// The username that failed authorization
        username: String,
        /// The command that was attempted
        command: String,
    },
    
    /// Indicates authentication failure
    AuthenticationFailed {
        /// The reason for authentication failure
        reason: String,
    },
    
    /// Indicates an adapter has not been properly initialized
    NotInitialized,
    
    /// Indicates an adapter is already initialized during an initialization attempt
    AlreadyInitialized,
    
    /// Indicates a mutex lock acquisition failure
    LockError(String),
    
    /// Represents any other error type
    Other(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::NotFound(cmd) => write!(f, "Command not found: {}", cmd),
            AdapterError::Execution(reason) => write!(f, "Execution failed: {}", reason),
            AdapterError::AuthorizationFailed { username, command } => 
                write!(f, "User '{}' is not authorized to execute command '{}'", username, command),
            AdapterError::AuthenticationFailed { reason } => 
                write!(f, "Authentication failed: {}", reason),
            AdapterError::NotInitialized => 
                write!(f, "Adapter is not initialized"),
            AdapterError::AlreadyInitialized => 
                write!(f, "Adapter is already initialized"),
            AdapterError::LockError(msg) => 
                write!(f, "Failed to acquire lock: {}", msg),
            AdapterError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AdapterError {}

/// Result type alias for adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Converts a string error to an AdapterError::Other
/// 
/// This is useful for converting errors from external crates or standard library
/// functions that return string-based errors.
pub fn to_adapter_error<E: ToString>(err: E) -> AdapterError {
    AdapterError::Other(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let not_found = AdapterError::NotFound("test".to_string());
        assert_eq!(not_found.to_string(), "Command not found: test");
        
        let auth_failed = AdapterError::AuthorizationFailed { 
            username: "user".to_string(), 
            command: "admin-cmd".to_string() 
        };
        assert_eq!(
            auth_failed.to_string(), 
            "User 'user' is not authorized to execute command 'admin-cmd'"
        );
    }
    
    #[test]
    fn test_to_adapter_error() {
        let string_err = "An error occurred";
        let adapter_err = to_adapter_error(string_err);
        
        assert!(matches!(adapter_err, AdapterError::Other(msg) if msg == string_err));
    }
} 