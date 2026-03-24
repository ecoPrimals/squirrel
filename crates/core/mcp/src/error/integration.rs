// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::IntegrationError;
    use std::fmt::Write as _;

    #[test]
    fn integration_error_display_adapter_not_found() {
        let err = IntegrationError::AdapterNotFound("a".into());
        assert!(err.to_string().contains("Adapter not found"));
    }

    #[test]
    fn integration_error_display_adapter_initialization_failed() {
        let err = IntegrationError::AdapterInitializationFailed("init".into());
        assert!(err.to_string().contains("initialization"));
    }

    #[test]
    fn integration_error_display_protocol_mismatch() {
        let err = IntegrationError::ProtocolMismatch {
            expected: "1".into(),
            actual: "2".into(),
        };
        let s = err.to_string();
        assert!(s.contains("mismatch") || s.contains("expected"));
        assert!(s.contains('1') && s.contains('2'));
    }

    #[test]
    fn integration_error_display_service_unavailable() {
        let err = IntegrationError::ServiceUnavailable("api".into());
        assert!(err.to_string().contains("unavailable"));
    }

    #[test]
    fn integration_error_display_configuration_error() {
        let err = IntegrationError::ConfigurationError("cfg".into());
        assert!(err.to_string().contains("configuration"));
    }

    #[test]
    fn integration_error_display_compatibility_error() {
        let err = IntegrationError::CompatibilityError("cmp".into());
        assert!(err.to_string().contains("compatibility"));
    }

    #[test]
    fn integration_error_debug_all_variants() {
        let cases = [
            IntegrationError::AdapterNotFound("a".into()),
            IntegrationError::AdapterInitializationFailed("b".into()),
            IntegrationError::ProtocolMismatch {
                expected: "e".into(),
                actual: "a".into(),
            },
            IntegrationError::ServiceUnavailable("s".into()),
            IntegrationError::ConfigurationError("c".into()),
            IntegrationError::CompatibilityError("x".into()),
        ];
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn integration_error_helpers() {
        assert!(matches!(
            IntegrationError::adapter_not_found("n"),
            IntegrationError::AdapterNotFound(s) if s == "n"
        ));
        assert!(matches!(
            IntegrationError::service_unavailable("svc"),
            IntegrationError::ServiceUnavailable(s) if s == "svc"
        ));
        assert!(matches!(
            IntegrationError::protocol_mismatch("exp", "act"),
            IntegrationError::ProtocolMismatch { expected, actual }
                if expected == "exp" && actual == "act"
        ));
    }
}
