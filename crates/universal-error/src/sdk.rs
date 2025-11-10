//! SDK Error Types
//!
//! This module provides error types for the Squirrel Plugin SDK following
//! the excellent MCP error architecture pattern. It encompasses infrastructure,
//! communication, and client errors.
//!
//! # Architecture
//!
//! ```text
//! SDKError
//!     ├── Infrastructure (logging, validation, utilities)
//!     ├── Communication (events, commands, MCP communication)
//!     ├── Client (HTTP, connections)
//!     └── General (catch-all for SDK operations)
//! ```
//!
//! # Examples
//!
//! ```
//! use universal_error::sdk::{SDKError, InfrastructureError};
//!
//! fn validate_input(input: &str) -> Result<(), SDKError> {
//!     if input.is_empty() {
//!         return Err(InfrastructureError::Validation(
//!             "Input cannot be empty".to_string()
//!         ).into());
//!     }
//!     Ok(())
//! }
//! ```

use thiserror::Error;
use super::{ErrorContextTrait, ErrorSeverity};

/// Top-level SDK error type
///
/// This encompasses all SDK-related errors with automatic conversions
/// from sub-domain errors via `#[from]` attribute.
#[derive(Error, Debug, Clone)]
pub enum SDKError {
    /// Error originating from infrastructure components
    #[error(transparent)]
    Infrastructure(#[from] InfrastructureError),
    
    /// Error originating from communication layer
    #[error(transparent)]
    Communication(#[from] CommunicationError),
    
    /// Error originating from client operations
    #[error(transparent)]
    Client(#[from] ClientError),
    
    /// General SDK error
    #[error("SDK error: {0}")]
    General(String),
}

/// Infrastructure-related errors
///
/// Covers logging, validation, utility functions, and other infrastructure concerns.
#[derive(Error, Debug, Clone)]
pub enum InfrastructureError {
    /// Logging system error
    #[error("Logging error: {0}")]
    Logging(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Utility function error
    #[error("Utility error: {0}")]
    Utility(String),
    
    /// Error type conversion error
    #[error("Type conversion error: {0}")]
    Conversion(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Communication-related errors
///
/// Covers events, commands, and MCP communication.
#[derive(Error, Debug, Clone)]
pub enum CommunicationError {
    /// Event handling error
    #[error("Event error: {0}")]
    Event(String),
    
    /// Command execution error
    #[error("Command error: {0}")]
    Command(String),
    
    /// MCP communication error
    #[error("MCP communication error: {0}")]
    MCP(String),
    
    /// Message serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Message deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Client-related errors
///
/// Covers HTTP clients, connections, and client-side operations.
#[derive(Error, Debug, Clone)]
pub enum ClientError {
    /// HTTP client error
    #[error("HTTP client error: {0}")]
    Http(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Request error
    #[error("Request error: {0}")]
    Request(String),
    
    /// Response error
    #[error("Response error: {0}")]
    Response(String),
    
    /// Timeout error
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

// Implement ErrorContextTrait for SDK errors following MCP pattern
impl ErrorContextTrait for SDKError {
    fn severity(&self) -> ErrorSeverity {
        match self {
            SDKError::Infrastructure(_) => ErrorSeverity::Medium,
            SDKError::Communication(_) => ErrorSeverity::High,
            SDKError::Client(_) => ErrorSeverity::Medium,
            SDKError::General(_) => ErrorSeverity::Low,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("SDK")
    }
    
    fn is_recoverable(&self) -> bool {
        match self {
            SDKError::Infrastructure(InfrastructureError::Validation(_)) => true,
            SDKError::Client(ClientError::Timeout(_)) => true,
            SDKError::Client(ClientError::Connection(_)) => true,
            _ => false,
        }
    }
}

impl ErrorContextTrait for InfrastructureError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            InfrastructureError::Validation(_) => ErrorSeverity::Low,
            InfrastructureError::Configuration(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("SDK.Infrastructure")
    }
}

impl ErrorContextTrait for CommunicationError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            CommunicationError::MCP(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("SDK.Communication")
    }
}

impl ErrorContextTrait for ClientError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            ClientError::Timeout(_) => ErrorSeverity::Medium,
            ClientError::Connection(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("SDK.Client")
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(self, ClientError::Timeout(_) | ClientError::Connection(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_infrastructure_error() {
        let err = InfrastructureError::Validation("test".to_string());
        assert_eq!(err.severity(), ErrorSeverity::Low);
    }
    
    #[test]
    fn test_communication_error() {
        let err = CommunicationError::MCP("test".to_string());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }
    
    #[test]
    fn test_client_error() {
        let err = ClientError::Timeout(30);
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_sdk_error_conversion() {
        let infra_err = InfrastructureError::Logging("test".to_string());
        let sdk_err: SDKError = infra_err.into();
        assert!(matches!(sdk_err, SDKError::Infrastructure(_)));
        assert_eq!(sdk_err.component(), Some("SDK"));
    }
}

