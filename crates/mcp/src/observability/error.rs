//! Error handling for observability components.

use std::fmt;
use std::error::Error as StdError;

/// Result type for observability operations
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

/// Error type for observability operations
#[derive(Debug)]
pub enum ObservabilityError {
    /// Error during initialization
    InitializationError(String),
    
    /// Error during metric registration
    MetricRegistrationError(String),
    
    /// Error during logging
    LoggingError(String),
    
    /// Error during tracing
    TracingError(String),
    
    /// Error during health check
    HealthCheckError(String),
    
    /// Error during alerting
    AlertingError(String),
    
    /// Error from external system interactions
    ExternalSystemError(String),
    
    /// Error with I/O operations
    IoError(std::io::Error),
    
    /// Invalid operation
    InvalidOperation(String),
    
    /// Error from serialization
    SerializationError(String),
    
    /// Error from HTTP operations
    HttpError(String),
    
    /// Unknown error
    UnknownError(String),
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            Self::MetricRegistrationError(msg) => write!(f, "Metric registration error: {}", msg),
            Self::LoggingError(msg) => write!(f, "Logging error: {}", msg),
            Self::TracingError(msg) => write!(f, "Tracing error: {}", msg),
            Self::HealthCheckError(msg) => write!(f, "Health check error: {}", msg),
            Self::AlertingError(msg) => write!(f, "Alerting error: {}", msg),
            Self::ExternalSystemError(msg) => write!(f, "External system error: {}", msg),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            Self::UnknownError(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl StdError for ObservabilityError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ObservabilityError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<String> for ObservabilityError {
    fn from(msg: String) -> Self {
        Self::UnknownError(msg)
    }
}

impl From<&str> for ObservabilityError {
    fn from(msg: &str) -> Self {
        Self::UnknownError(msg.to_string())
    }
} 