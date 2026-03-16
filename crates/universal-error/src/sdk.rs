// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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

use super::{ErrorContextTrait, ErrorSeverity};
use thiserror::Error;

/// Top-level SDK error type
///
/// This encompasses all SDK-related errors with automatic conversions
/// from sub-domain errors via `#[from]` attribute.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
        matches!(
            self,
            SDKError::Infrastructure(InfrastructureError::Validation(_))
                | SDKError::Client(ClientError::Timeout(_))
                | SDKError::Client(ClientError::Connection(_))
        )
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

    // Additional comprehensive tests
    #[test]
    fn test_infrastructure_error_display() {
        let err = InfrastructureError::Logging("log failed".to_string());
        assert_eq!(err.to_string(), "Logging error: log failed");

        let err = InfrastructureError::Validation("invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid input");

        let err = InfrastructureError::Conversion("type mismatch".to_string());
        assert_eq!(err.to_string(), "Type conversion error: type mismatch");
    }

    #[test]
    fn test_infrastructure_error_severity() {
        assert_eq!(
            InfrastructureError::Validation("test".to_string()).severity(),
            ErrorSeverity::Low
        );
        assert_eq!(
            InfrastructureError::Configuration("test".to_string()).severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            InfrastructureError::Logging("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            InfrastructureError::Utility("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
    }

    #[test]
    fn test_communication_error_display() {
        let err = CommunicationError::Event("event failed".to_string());
        assert_eq!(err.to_string(), "Event error: event failed");

        let err = CommunicationError::Serialization("ser error".to_string());
        assert_eq!(err.to_string(), "Serialization error: ser error");
    }

    #[test]
    fn test_communication_error_component() {
        let err = CommunicationError::MCP("test".to_string());
        assert_eq!(err.component(), Some("SDK.Communication"));
    }

    #[test]
    fn test_client_error_display() {
        let err = ClientError::Http("http error".to_string());
        assert_eq!(err.to_string(), "HTTP client error: http error");

        let err = ClientError::Timeout(60);
        assert_eq!(err.to_string(), "Timeout after 60 seconds");

        let err = ClientError::Connection("refused".to_string());
        assert_eq!(err.to_string(), "Connection error: refused");
    }

    #[test]
    fn test_client_error_recoverability() {
        assert!(ClientError::Timeout(30).is_recoverable());
        assert!(ClientError::Connection("failed".to_string()).is_recoverable());
        assert!(!ClientError::Http("error".to_string()).is_recoverable());
        assert!(!ClientError::Request("bad".to_string()).is_recoverable());
    }

    #[test]
    fn test_client_error_severity() {
        assert_eq!(ClientError::Timeout(30).severity(), ErrorSeverity::Medium);
        assert_eq!(
            ClientError::Connection("test".to_string()).severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            ClientError::Http("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
    }

    #[test]
    fn test_sdk_error_general() {
        let err = SDKError::General("general error".to_string());
        assert_eq!(err.to_string(), "SDK error: general error");
        assert_eq!(err.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_communication_error_conversion() {
        let comm_err = CommunicationError::Command("cmd failed".to_string());
        let sdk_err: SDKError = comm_err.into();
        assert!(matches!(sdk_err, SDKError::Communication(_)));
    }

    #[test]
    fn test_client_error_conversion() {
        let client_err = ClientError::Response("bad response".to_string());
        let sdk_err: SDKError = client_err.into();
        assert!(matches!(sdk_err, SDKError::Client(_)));
    }

    #[test]
    fn test_sdk_error_severity_propagation() {
        let infra_err =
            SDKError::Infrastructure(InfrastructureError::Configuration("test".to_string()));
        assert_eq!(infra_err.severity(), ErrorSeverity::Medium); // SDKError has its own mapping

        let comm_err = SDKError::Communication(CommunicationError::MCP("test".to_string()));
        assert_eq!(comm_err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_sdk_error_recoverability() {
        let val_err = SDKError::Infrastructure(InfrastructureError::Validation("test".to_string()));
        assert!(val_err.is_recoverable());

        let timeout_err = SDKError::Client(ClientError::Timeout(30));
        assert!(timeout_err.is_recoverable());

        let general_err = SDKError::General("test".to_string());
        assert!(!general_err.is_recoverable());
    }

    #[test]
    fn test_all_infrastructure_error_variants() {
        let variants = vec![
            InfrastructureError::Logging("test".to_string()),
            InfrastructureError::Validation("test".to_string()),
            InfrastructureError::Utility("test".to_string()),
            InfrastructureError::Conversion("test".to_string()),
            InfrastructureError::Configuration("test".to_string()),
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity(); // Ensure no panic
        }
    }

    #[test]
    fn test_all_communication_error_variants() {
        let variants = vec![
            CommunicationError::Event("test".to_string()),
            CommunicationError::Command("test".to_string()),
            CommunicationError::MCP("test".to_string()),
            CommunicationError::Serialization("test".to_string()),
            CommunicationError::Deserialization("test".to_string()),
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity(); // Ensure no panic
        }
    }

    #[test]
    fn test_all_client_error_variants() {
        let variants = vec![
            ClientError::Http("test".to_string()),
            ClientError::Connection("test".to_string()),
            ClientError::Request("test".to_string()),
            ClientError::Response("test".to_string()),
            ClientError::Timeout(30),
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity(); // Ensure no panic
            let _ = variant.is_recoverable(); // Ensure no panic
        }
    }
}
