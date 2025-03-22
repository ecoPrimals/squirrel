use std::fmt;
use std::error::Error;

/// Errors that can occur in the plugin system
#[derive(Debug)]
pub enum PluginError {
    /// Plugin not found
    NotFound(String),
    /// Plugin already exists
    AlreadyExists(String),
    /// IO error
    IoError(std::io::Error),
    /// Plugin loading error
    LoadError(String),
    /// Plugin initialization error
    InitError(String),
    /// Plugin validation error
    ValidationError(String),
    /// Command registration error
    RegisterError(String),
    /// Unknown plugin error
    Unknown(String),
}

impl PluginError {
    /// Create a new NotFound error
    pub fn plugin_not_found(name: &str) -> Self {
        PluginError::NotFound(name.to_string())
    }

    /// Create a new AlreadyExists error
    pub fn plugin_already_exists(name: &str) -> Self {
        PluginError::AlreadyExists(name.to_string())
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::NotFound(name) => write!(f, "Plugin not found: {}", name),
            PluginError::AlreadyExists(name) => write!(f, "Plugin already exists: {}", name),
            PluginError::IoError(err) => write!(f, "IO error: {}", err),
            PluginError::LoadError(msg) => write!(f, "Plugin loading error: {}", msg),
            PluginError::InitError(msg) => write!(f, "Plugin initialization error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Plugin validation error: {}", msg),
            PluginError::RegisterError(msg) => write!(f, "Command registration error: {}", msg),
            PluginError::Unknown(msg) => write!(f, "Unknown plugin error: {}", msg),
        }
    }
}

impl Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::IoError(err)
    }
}

impl From<String> for PluginError {
    fn from(err: String) -> Self {
        PluginError::Unknown(err)
    }
}

impl From<&str> for PluginError {
    fn from(err: &str) -> Self {
        PluginError::Unknown(err.to_string())
    }
} 