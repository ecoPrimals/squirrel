//! Error handling for the Squirrel primal

use thiserror::Error;

/// Main error type for Squirrel Primal operations
#[derive(Error, Debug)]
pub enum PrimalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Service discovery failed: {0}")]
    ServiceDiscoveryFailed(String),

    #[error("Service discovery error: {0}")]
    ServiceDiscoveryError(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Resource error: {0}")]
    ResourceError(String),

    #[error("General error: {0}")]
    General(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Compute error: {0}")]
    ComputeError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Add support for Box<dyn Error> conversion for our Arc<str> modernization
impl From<Box<dyn std::error::Error + Send + Sync>> for PrimalError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        PrimalError::Generic(format!("Boxed error: {err}"))
    }
}
