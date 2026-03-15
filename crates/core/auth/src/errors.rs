// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Modern error handling for Squirrel Authentication System
//!
//! Clean error types using thiserror patterns from beardog architecture.
//! Eliminates the anyhow/AuthError conflicts from the legacy system.

use thiserror::Error;

/// Result type alias for auth operations (leveraging beardog patterns)
pub type AuthResult<T> = Result<T, AuthError>;

/// Clean, comprehensive error types for authentication operations
#[derive(Error, Debug)]
pub enum AuthError {
    /// Authentication failed with provided credentials
    #[error("Authentication failed: {message}")]
    AuthenticationFailed {
        /// Specific reason for authentication failure
        message: String,
    },

    /// Token-related errors (JWT, session tokens, etc.)
    #[error("Token error in {operation}: {message}")]
    Token {
        /// The token operation that failed (validate, refresh, etc.)
        operation: String,
        /// Error message describing the token issue
        message: String,
    },

    /// Permission and authorization errors
    #[error("Authorization error: {message}")]
    Authorization {
        /// Error message describing the authorization issue
        message: String,
    },

    /// Session management errors
    #[error("Session error: {message}")]
    Session {
        /// Error message describing the session issue
        message: String,
    },

    /// Configuration-related auth errors
    #[error("Auth configuration error: {message}")]
    Configuration {
        /// Error message describing the configuration issue
        message: String,
    },

    /// Network/HTTP errors when communicating with auth services
    #[error("Network error during {operation}: {message}")]
    Network {
        /// The network operation that failed
        operation: String,
        /// Error message describing the network issue
        message: String,
    },

    /// Beardog integration errors
    #[error("Beardog integration error: {message}")]
    BeardogIntegration {
        /// Error message describing the beardog integration issue
        message: String,
    },

    /// Internal system errors
    #[error("Internal auth system error: {message}")]
    Internal {
        /// Error message describing the internal issue
        message: String,
    },

    // Simple error variants for JWT and capability-based integration (TRUE ecoBin)
    /// Token has expired
    #[error("Token has expired")]
    TokenExpired,

    /// Invalid token format or signature
    #[error("Invalid token")]
    InvalidToken,

    /// Invalid response from service
    #[error("Invalid response from auth service")]
    InvalidResponse,

    /// Capability provider unavailable
    #[error("JWT capability provider unavailable: {0}")]
    CapabilityProviderUnavailable(String),

    /// Capability provider returned an error
    #[error("JWT capability provider error: {0}")]
    CapabilityProviderError(String),

    // Legacy compatibility (for migration period)
    /// BearDog service unavailable (deprecated: use CapabilityProviderUnavailable)
    #[error("BearDog unavailable: {0}")]
    #[deprecated(note = "Use CapabilityProviderUnavailable instead (capability-based)")]
    BeardogUnavailable(String),

    /// BearDog returned an error (deprecated: use CapabilityProviderError)
    #[error("BearDog error: {0}")]
    #[deprecated(note = "Use CapabilityProviderError instead (capability-based)")]
    BeardogError(String),
}

impl AuthError {
    /// Create an authentication failed error
    pub fn authentication_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    /// Create a token error
    pub fn token_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Token {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::Authorization {
            message: message.into(),
        }
    }

    /// Create a session error
    pub fn session_error(message: impl Into<String>) -> Self {
        Self::Session {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Network {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a beardog integration error
    pub fn beardog_error(message: impl Into<String>) -> Self {
        Self::BeardogIntegration {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

// Convert from common error types
#[cfg(feature = "http-auth")]
impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        Self::network_error("http_request", err.to_string())
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        Self::internal_error(format!("JSON serialization error: {}", err))
    }
}

impl From<uuid::Error> for AuthError {
    fn from(err: uuid::Error) -> Self {
        Self::internal_error(format!("UUID error: {}", err))
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal_error(format!("Internal error: {}", err))
    }
}
