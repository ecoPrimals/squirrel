// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration-related error types for the MCP system

use thiserror::Error;

/// Errors that can occur during integration operations
#[derive(Error, Debug, Clone)]
pub enum IntegrationError {
    /// Adapter not found
    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    /// Adapter initialization failed
    #[error("Adapter initialization failed: {0}")]
    AdapterInitializationFailed(String),

    /// Integration protocol mismatch
    #[error("Integration protocol mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch {
        /// Expected protocol version or format
        expected: String,
        /// Actual protocol version or format received
        actual: String,
    },

    /// External service unavailable
    #[error("External service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Configuration error
    #[error("Integration configuration error: {0}")]
    ConfigurationError(String),

    /// Compatibility error
    #[error("Integration compatibility error: {0}")]
    CompatibilityError(String),
}

impl IntegrationError {
    /// Create a new adapter not found error
    pub fn adapter_not_found(name: impl Into<String>) -> Self {
        Self::AdapterNotFound(name.into())
    }

    /// Create a new service unavailable error
    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::ServiceUnavailable(service.into())
    }

    /// Create a new protocol mismatch error
    pub fn protocol_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::ProtocolMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}
