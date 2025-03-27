// Plugin Error Module
//
// This module defines the error types used in the plugin system.

use std::fmt;
use std::error::Error;
use std::result;

/// A specialized Result type for plugin operations
pub type Result<T> = result::Result<T, PluginError>;

/// Errors that can occur in the plugin system
#[derive(Debug)]
pub enum PluginError {
    /// Error loading a plugin
    LoadError(String),
    
    /// Error initializing a plugin
    InitError(String),
    
    /// Error executing a plugin method or command
    ExecutionError(String),
    
    /// A plugin was not found
    NotFoundError(String),
    
    /// A plugin dependency is missing
    DependencyError(String),
    
    /// A plugin version is incompatible
    VersionError(String),
    
    /// A plugin is already loaded
    AlreadyLoadedError(String),
    
    /// A security violation occurred
    SecurityError(String),
    
    /// An I/O error occurred
    IoError(std::io::Error),
    
    /// A serialization or deserialization error
    SerializationError(String),
    
    /// A network error occurred
    NetworkError(String),
    
    /// A validation error occurred
    ValidationError(String),
    
    /// A plugin resource is not available
    ResourceError(String),
    
    /// An unexpected error occurred
    UnexpectedError(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::LoadError(msg) => write!(f, "Plugin load error: {}", msg),
            PluginError::InitError(msg) => write!(f, "Plugin initialization error: {}", msg),
            PluginError::ExecutionError(msg) => write!(f, "Plugin execution error: {}", msg),
            PluginError::NotFoundError(msg) => write!(f, "Plugin not found: {}", msg),
            PluginError::DependencyError(msg) => write!(f, "Plugin dependency error: {}", msg),
            PluginError::VersionError(msg) => write!(f, "Plugin version error: {}", msg),
            PluginError::AlreadyLoadedError(msg) => write!(f, "Plugin already loaded: {}", msg),
            PluginError::SecurityError(msg) => write!(f, "Plugin security error: {}", msg),
            PluginError::IoError(err) => write!(f, "Plugin I/O error: {}", err),
            PluginError::SerializationError(msg) => write!(f, "Plugin serialization error: {}", msg),
            PluginError::NetworkError(msg) => write!(f, "Plugin network error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Plugin validation error: {}", msg),
            PluginError::ResourceError(msg) => write!(f, "Plugin resource error: {}", msg),
            PluginError::UnexpectedError(msg) => write!(f, "Unexpected plugin error: {}", msg),
        }
    }
}

impl Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::IoError(err)
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    
    #[test]
    fn test_plugin_error_display() {
        let errors = [
            (PluginError::LoadError("Failed to load plugin".to_string()), "Plugin load error: Failed to load plugin"),
            (PluginError::InitError("Failed to initialize plugin".to_string()), "Plugin initialization error: Failed to initialize plugin"),
            (PluginError::ExecutionError("Command failed".to_string()), "Plugin execution error: Command failed"),
            (PluginError::NotFoundError("Plugin not in registry".to_string()), "Plugin not found: Plugin not in registry"),
            (PluginError::DependencyError("Missing dependency".to_string()), "Plugin dependency error: Missing dependency"),
            (PluginError::VersionError("Incompatible version".to_string()), "Plugin version error: Incompatible version"),
            (PluginError::AlreadyLoadedError("Already loaded".to_string()), "Plugin already loaded: Already loaded"),
            (PluginError::SecurityError("Unauthorized action".to_string()), "Plugin security error: Unauthorized action"),
            (PluginError::SerializationError("Invalid JSON".to_string()), "Plugin serialization error: Invalid JSON"),
            (PluginError::NetworkError("Connection failed".to_string()), "Plugin network error: Connection failed"),
            (PluginError::ValidationError("Invalid input".to_string()), "Plugin validation error: Invalid input"),
            (PluginError::ResourceError("Resource not available".to_string()), "Plugin resource error: Resource not available"),
            (PluginError::UnexpectedError("Something went wrong".to_string()), "Unexpected plugin error: Something went wrong"),
        ];
        
        for (error, expected) in errors.iter() {
            assert_eq!(error.to_string(), *expected);
        }
        
        // Test IoError display
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let plugin_error = PluginError::IoError(io_error);
        assert!(plugin_error.to_string().contains("Plugin I/O error: File not found"));
    }
    
    #[test]
    fn test_plugin_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let plugin_error: PluginError = io_error.into();
        
        match plugin_error {
            PluginError::IoError(err) => {
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(err.to_string(), "File not found");
            }
            _ => panic!("Expected IoError variant"),
        }
    }
    
    #[test]
    fn test_plugin_error_from_serde_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let plugin_error: PluginError = json_error.into();
        
        match plugin_error {
            PluginError::SerializationError(msg) => {
                assert!(msg.contains("expected value"));
            }
            _ => panic!("Expected SerializationError variant"),
        }
    }
} 