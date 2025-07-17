//! Error handling for the Squirrel primal

use std::fmt;

/// Primary error type for the Squirrel primal
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrimalError {
    /// Internal error
    Internal(String),
    /// Configuration error
    Configuration(String),
    /// Network error
    NetworkError(String),
    /// Network error (alternative name)
    Network(String),
    /// Registration error
    RegistrationError(String),
    /// Invalid context error
    InvalidContext(String),
    /// Unsupported operation error
    UnsupportedOperation(String),
    /// Session error
    Session(String),
    /// Ecosystem error
    Ecosystem(String),
    /// Monitoring error
    Monitoring(String),
    /// Benchmarking error
    Benchmarking(String),
    /// Shutdown error
    Shutdown(String),
    /// Self-healing error
    SelfHealing(String),
    /// Protocol error
    Protocol(String),
    /// Service mesh error
    ServiceMesh(String),
    /// IO error
    Io(String),
    /// Timeout error
    Timeout(String),
    /// Config error
    ConfigError(String),
    /// General error
    General(String),
    /// Operation failed error
    OperationFailed(String),
    /// Not found error
    NotFoundError(String),
    /// Serialization error
    SerializationError(String),
    /// Resource error
    ResourceError(String),
    /// Security error
    SecurityError(String),
}

impl fmt::Display for PrimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimalError::Internal(msg) => write!(f, "Internal error: {}", msg),
            PrimalError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            PrimalError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PrimalError::Network(msg) => write!(f, "Network error: {}", msg),
            PrimalError::RegistrationError(msg) => write!(f, "Registration error: {}", msg),
            PrimalError::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            PrimalError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            PrimalError::Session(msg) => write!(f, "Session error: {}", msg),
            PrimalError::Ecosystem(msg) => write!(f, "Ecosystem error: {}", msg),
            PrimalError::Monitoring(msg) => write!(f, "Monitoring error: {}", msg),
            PrimalError::Benchmarking(msg) => write!(f, "Benchmarking error: {}", msg),
            PrimalError::Shutdown(msg) => write!(f, "Shutdown error: {}", msg),
            PrimalError::SelfHealing(msg) => write!(f, "Self-healing error: {}", msg),
            PrimalError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            PrimalError::ServiceMesh(msg) => write!(f, "Service mesh error: {}", msg),
            PrimalError::Io(msg) => write!(f, "IO error: {}", msg),
            PrimalError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            PrimalError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            PrimalError::General(msg) => write!(f, "General error: {}", msg),
            PrimalError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            PrimalError::NotFoundError(msg) => write!(f, "Not found: {}", msg),
            PrimalError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            PrimalError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            PrimalError::SecurityError(msg) => write!(f, "Security error: {}", msg),
        }
    }
}

impl std::error::Error for PrimalError {}

impl From<std::io::Error> for PrimalError {
    fn from(error: std::io::Error) -> Self {
        PrimalError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for PrimalError {
    fn from(error: serde_json::Error) -> Self {
        PrimalError::SerializationError(error.to_string())
    }
}

impl From<reqwest::Error> for PrimalError {
    fn from(error: reqwest::Error) -> Self {
        PrimalError::Network(error.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for PrimalError {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        PrimalError::Timeout(error.to_string())
    }
} 