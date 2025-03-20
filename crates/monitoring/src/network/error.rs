use std::fmt;
use std::error::Error;
use squirrel_core::error::SquirrelError;

/// Network monitoring errors
#[derive(Debug)]
pub enum NetworkError {
    /// System operation errors
    System(String),
    /// Configuration errors
    Configuration(String),
    /// Monitoring errors
    Monitoring(String),
    /// Interface errors
    Interface(String),
    /// Stat collection errors
    Stats(String),
    /// Adapter not initialized
    AdapterNotInitialized,
    /// Adapter already initialized
    AdapterAlreadyInitialized,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::System(msg) => write!(f, "Network system error: {msg}"),
            NetworkError::Configuration(msg) => write!(f, "Network configuration error: {msg}"),
            NetworkError::Monitoring(msg) => write!(f, "Network monitoring error: {msg}"),
            NetworkError::Interface(msg) => write!(f, "Network interface error: {msg}"),
            NetworkError::Stats(msg) => write!(f, "Network stats error: {msg}"),
            NetworkError::AdapterNotInitialized => write!(f, "Network adapter not initialized"),
            NetworkError::AdapterAlreadyInitialized => write!(f, "Network adapter already initialized"),
        }
    }
}

impl Error for NetworkError {}

// Implementation of conversion from NetworkError to SquirrelError
impl From<NetworkError> for SquirrelError {
    fn from(err: NetworkError) -> Self {
        SquirrelError::Network(err.to_string())
    }
}

/// Helper method to create a system error
pub fn system_error(msg: impl Into<String>) -> NetworkError {
    NetworkError::System(msg.into())
}

/// Helper method to create a configuration error
pub fn config_error(msg: impl Into<String>) -> NetworkError {
    NetworkError::Configuration(msg.into())
}

/// Helper method to create a monitoring error
pub fn monitoring_error(msg: impl Into<String>) -> NetworkError {
    NetworkError::Monitoring(msg.into())
}

/// Helper method to create an interface error
pub fn interface_error(msg: impl Into<String>) -> NetworkError {
    NetworkError::Interface(msg.into())
}

/// Helper method to create a stats error
pub fn stats_error(msg: impl Into<String>) -> NetworkError {
    NetworkError::Stats(msg.into())
} 