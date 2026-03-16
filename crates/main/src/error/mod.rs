// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error handling for the Squirrel primal

use thiserror::Error;

/// Main error type for Squirrel Primal operations
#[derive(Error, Debug)]
pub enum PrimalError {
    /// I/O operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing failed
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Network communication failed
    #[error("Network error: {0}")]
    Network(String),

    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Configuration is invalid
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Parsing failed
    #[error("Parsing error: {0}")]
    ParsingError(String),

    /// Invalid operation requested
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Service discovery failed
    #[error("Service discovery failed: {0}")]
    ServiceDiscoveryFailed(String),

    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscoveryError(String),

    /// Registry operation failed
    #[error("Registry error: {0}")]
    Registry(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFoundError(String),

    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// General error
    #[error("General error: {0}")]
    General(String),

    /// Validation failed
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Serialization failed
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Security violation
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Compute operation failed
    #[error("Compute error: {0}")]
    ComputeError(String),

    /// Storage operation failed
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Generic error
    #[error("Generic error: {0}")]
    Generic(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Feature not supported
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Invalid endpoint
    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),

    /// Invalid response received
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Remote service error
    #[error("Remote error: {0}")]
    RemoteError(String),
}

// Add support for Box<dyn Error> conversion for our Arc<str> modernization
impl From<Box<dyn std::error::Error + Send + Sync>> for PrimalError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Generic(format!("Boxed error: {err}"))
    }
}

// Add support for DiscoveryError conversion
impl From<crate::capabilities::discovery::DiscoveryError> for PrimalError {
    fn from(err: crate::capabilities::discovery::DiscoveryError) -> Self {
        Self::NetworkError(format!("Discovery error: {err}"))
    }
}

#[cfg(test)]
mod error_path_coverage_tests;
#[cfg(test)]
mod tests;
