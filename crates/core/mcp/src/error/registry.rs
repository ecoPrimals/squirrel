// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Registry-related error types for the MCP system

use thiserror::Error;

/// Errors that can occur during registry operations
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Service not found in registry
    #[error("Service not found in registry: {0}")]
    ServiceNotFound(String),

    /// Service already registered
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),

    /// Registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Registry corruption detected
    #[error("Registry corruption detected: {0}")]
    CorruptionDetected(String),

    /// Registry access denied
    #[error("Registry access denied: {0}")]
    AccessDenied(String),
}

impl RegistryError {
    /// Create a new service not found error
    pub fn service_not_found(name: impl Into<String>) -> Self {
        Self::ServiceNotFound(name.into())
    }

    /// Create a new service already registered error
    pub fn service_already_registered(name: impl Into<String>) -> Self {
        Self::ServiceAlreadyRegistered(name.into())
    }

    /// Create a new registration failed error
    pub fn registration_failed(msg: impl Into<String>) -> Self {
        Self::RegistrationFailed(msg.into())
    }
}

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::RegistryError;
    use std::fmt::Write as _;

    #[test]
    fn registry_error_display_service_not_found() {
        let err = RegistryError::ServiceNotFound("svc".into());
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("svc"));
    }

    #[test]
    fn registry_error_display_service_already_registered() {
        let err = RegistryError::ServiceAlreadyRegistered("dup".into());
        assert!(err.to_string().contains("already registered"));
    }

    #[test]
    fn registry_error_display_registration_failed() {
        let err = RegistryError::RegistrationFailed("why".into());
        assert!(err.to_string().contains("Registration failed"));
    }

    #[test]
    fn registry_error_display_corruption_detected() {
        let err = RegistryError::CorruptionDetected("idx".into());
        assert!(err.to_string().contains("corruption"));
    }

    #[test]
    fn registry_error_display_access_denied() {
        let err = RegistryError::AccessDenied("nope".into());
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn registry_error_debug_all_variants() {
        let cases = [
            RegistryError::ServiceNotFound("a".into()),
            RegistryError::ServiceAlreadyRegistered("b".into()),
            RegistryError::RegistrationFailed("c".into()),
            RegistryError::CorruptionDetected("d".into()),
            RegistryError::AccessDenied("e".into()),
        ];
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn registry_error_helpers_match_constructors() {
        assert!(matches!(
            RegistryError::service_not_found("x"),
            RegistryError::ServiceNotFound(s) if s == "x"
        ));
        assert!(matches!(
            RegistryError::service_already_registered("y"),
            RegistryError::ServiceAlreadyRegistered(s) if s == "y"
        ));
        assert!(matches!(
            RegistryError::registration_failed("z"),
            RegistryError::RegistrationFailed(s) if s == "z"
        ));
    }
}
