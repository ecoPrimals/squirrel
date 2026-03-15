// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security error types and implementations
//!
//! This module defines all error types used throughout the security system.

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),
    /// Authorization failed
    #[error("Authorization failed: {0}")]
    Authorization(String),
    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),
    /// Token error
    #[error("Token error: {0}")]
    Token(String),
    /// Certificate error
    #[error("Certificate error: {0}")]
    Certificate(String),
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Invalid credentials
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    /// Expired credentials
    #[error("Expired credentials: {0}")]
    ExpiredCredentials(String),
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl SecurityError {
    /// Create an authentication error
    pub fn authentication<T: ToString>(msg: T) -> Self {
        Self::Authentication(msg.to_string())
    }

    /// Create an authorization error
    pub fn authorization<T: ToString>(msg: T) -> Self {
        Self::Authorization(msg.to_string())
    }

    /// Create an encryption error
    pub fn encryption<T: ToString>(msg: T) -> Self {
        Self::Encryption(msg.to_string())
    }

    /// Create a network error
    pub fn network<T: ToString>(msg: T) -> Self {
        Self::Network(msg.to_string())
    }

    /// Create a configuration error
    pub fn configuration<T: ToString>(msg: T) -> Self {
        Self::Configuration(msg.to_string())
    }

    /// Check if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    /// Check if this is a configuration-related error
    pub fn is_configuration_error(&self) -> bool {
        matches!(self, Self::Configuration(_))
    }

    /// Check if this is a recoverable error (might succeed on retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Token(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let auth_error = SecurityError::authentication("test auth error");
        assert!(matches!(auth_error, SecurityError::Authentication(_)));

        let network_error = SecurityError::network("test network error");
        assert!(network_error.is_network_error());
        assert!(network_error.is_recoverable());
    }

    #[test]
    fn test_error_classification() {
        let config_error = SecurityError::configuration("test config error");
        assert!(config_error.is_configuration_error());
        assert!(!config_error.is_recoverable());

        let token_error = SecurityError::Token("expired".to_string());
        assert!(token_error.is_recoverable());
        assert!(!token_error.is_network_error());
    }

    #[test]
    fn test_display_all_variants() {
        let cases = vec![
            (
                SecurityError::Authentication("bad".into()),
                "Authentication failed: bad",
            ),
            (
                SecurityError::Authorization("denied".into()),
                "Authorization failed: denied",
            ),
            (
                SecurityError::Encryption("aes fail".into()),
                "Encryption error: aes fail",
            ),
            (
                SecurityError::Token("expired".into()),
                "Token error: expired",
            ),
            (
                SecurityError::Certificate("invalid".into()),
                "Certificate error: invalid",
            ),
            (
                SecurityError::Network("timeout".into()),
                "Network error: timeout",
            ),
            (
                SecurityError::Configuration("missing".into()),
                "Configuration error: missing",
            ),
            (
                SecurityError::InvalidCredentials("wrong".into()),
                "Invalid credentials: wrong",
            ),
            (
                SecurityError::ExpiredCredentials("old".into()),
                "Expired credentials: old",
            ),
            (
                SecurityError::PermissionDenied("no access".into()),
                "Permission denied: no access",
            ),
            (SecurityError::Other("misc".into()), "Other error: misc"),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_all_constructors() {
        let auth = SecurityError::authentication("a");
        assert!(matches!(auth, SecurityError::Authentication(_)));

        let authz = SecurityError::authorization("b");
        assert!(matches!(authz, SecurityError::Authorization(_)));

        let enc = SecurityError::encryption("c");
        assert!(matches!(enc, SecurityError::Encryption(_)));

        let net = SecurityError::network("d");
        assert!(matches!(net, SecurityError::Network(_)));

        let cfg = SecurityError::configuration("e");
        assert!(matches!(cfg, SecurityError::Configuration(_)));
    }

    #[test]
    fn test_is_recoverable_comprehensive() {
        // Network errors are recoverable
        assert!(SecurityError::Network("x".into()).is_recoverable());
        // Token errors are recoverable
        assert!(SecurityError::Token("x".into()).is_recoverable());
        // Other errors are NOT recoverable
        assert!(!SecurityError::Authentication("x".into()).is_recoverable());
        assert!(!SecurityError::Authorization("x".into()).is_recoverable());
        assert!(!SecurityError::Encryption("x".into()).is_recoverable());
        assert!(!SecurityError::Certificate("x".into()).is_recoverable());
        assert!(!SecurityError::Configuration("x".into()).is_recoverable());
        assert!(!SecurityError::InvalidCredentials("x".into()).is_recoverable());
        assert!(!SecurityError::ExpiredCredentials("x".into()).is_recoverable());
        assert!(!SecurityError::PermissionDenied("x".into()).is_recoverable());
        assert!(!SecurityError::Other("x".into()).is_recoverable());
    }

    #[test]
    fn test_is_network_error_comprehensive() {
        assert!(SecurityError::Network("x".into()).is_network_error());
        assert!(!SecurityError::Authentication("x".into()).is_network_error());
        assert!(!SecurityError::Token("x".into()).is_network_error());
    }

    #[test]
    fn test_is_configuration_error_comprehensive() {
        assert!(SecurityError::Configuration("x".into()).is_configuration_error());
        assert!(!SecurityError::Network("x".into()).is_configuration_error());
        assert!(!SecurityError::Authentication("x".into()).is_configuration_error());
    }

    #[test]
    fn test_error_is_std_error() {
        let error: Box<dyn std::error::Error> = Box::new(SecurityError::Other("test".into()));
        assert!(error.to_string().contains("Other error"));
    }
}
