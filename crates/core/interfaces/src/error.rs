// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error handling types and utilities for Squirrel

use thiserror::Error;

/// A type alias for a Result with a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>;

/// Persistence-related error types
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Error related to storage operations
    #[error("Storage error: {0}")]
    Storage(String),

    /// Error related to data serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error related to invalid data
    #[error("Invalid data error: {0}")]
    InvalidData(String),

    /// Error related to data not found
    #[error("Data not found: {0}")]
    NotFound(String),
}

/// The main error type for Squirrel
#[derive(Error, Debug)]
pub enum SquirrelError {
    /// A generic error
    #[error("Generic error: {0}")]
    Generic(String),

    /// An error related to MCP
    #[error("MCP error: {0}")]
    MCP(String),

    /// An error related to context
    #[error("Context error: {0}")]
    Context(String),

    /// An error related to plugins
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// An error related to I/O
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// An error related to serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// An error related to networking
    #[error("Network error: {0}")]
    Network(String),

    /// An error related to authentication
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// An error related to authorization
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// An error related to validation
    #[error("Validation error: {0}")]
    Validation(String),

    /// An error related to configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// An error from an external service
    #[error("External service error: {0}")]
    ExternalService(String),

    /// An error related to timing out
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// An error related to persistence
    #[error("Persistence error: {0}")]
    Persistence(PersistenceError),

    /// An error related to session management
    #[error("Session error: {0}")]
    Session(String),

    /// An error with additional context
    #[error("{context}: {source}")]
    WithContext {
        /// The source error
        source: Box<Self>,
        /// Additional context for the error
        context: String,
    },
}

impl SquirrelError {
    /// Create a new generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }

    /// Create a new MCP error
    pub fn mcp<S: Into<String>>(msg: S) -> Self {
        Self::MCP(msg.into())
    }

    /// Create a new context error
    pub fn context<S: Into<String>>(msg: S) -> Self {
        Self::Context(msg.into())
    }

    /// Create a new plugin error
    pub fn plugin<S: Into<String>>(msg: S) -> Self {
        Self::Plugin(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create a new authorization error
    pub fn authorization<S: Into<String>>(msg: S) -> Self {
        Self::Authorization(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a new external service error
    pub fn external_service<S: Into<String>>(msg: S) -> Self {
        Self::ExternalService(msg.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a new persistence error
    #[must_use]
    pub const fn persistence(err: PersistenceError) -> Self {
        Self::Persistence(err)
    }

    /// Create a new persistence storage error
    pub fn persistence_storage<S: Into<String>>(msg: S) -> Self {
        Self::Persistence(PersistenceError::Storage(msg.into()))
    }

    /// Create a new session error
    pub fn session<S: Into<String>>(msg: S) -> Self {
        Self::Session(msg.into())
    }

    /// Add context to an error
    #[must_use]
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        Self::WithContext {
            source: Box::new(self),
            context: context.into(),
        }
    }

    /// Check if the error is recoverable
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) | Self::ExternalService(_) => true,
            Self::WithContext { source, .. } => source.is_recoverable(),
            _ => false,
        }
    }
}

impl From<anyhow::Error> for SquirrelError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for SquirrelError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<&str> for SquirrelError {
    fn from(s: &str) -> Self {
        Self::Generic(s.to_string())
    }
}

impl From<String> for SquirrelError {
    fn from(s: String) -> Self {
        Self::Generic(s)
    }
}

/// A trait for errors that can be converted to a `SquirrelError`
pub trait IntoSquirrelError {
    /// Convert the error to a `SquirrelError`
    fn into_squirrel_error(self) -> SquirrelError;

    /// Convert the error to a `SquirrelError` with context
    fn into_squirrel_error_with_context<C: Into<String>>(self, context: C) -> SquirrelError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoSquirrelError for E {
    fn into_squirrel_error(self) -> SquirrelError {
        SquirrelError::Generic(self.to_string())
    }

    fn into_squirrel_error_with_context<C: Into<String>>(self, context: C) -> SquirrelError {
        SquirrelError::WithContext {
            source: Box::new(SquirrelError::Generic(self.to_string())),
            context: context.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // PERSISTENCE ERROR
    // ====================================================================

    #[test]
    fn test_persistence_error_storage_display() {
        let err = PersistenceError::Storage("disk full".to_string());
        assert_eq!(err.to_string(), "Storage error: disk full");
    }

    #[test]
    fn test_persistence_error_serialization_display() {
        let err = PersistenceError::Serialization("invalid JSON".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid JSON");
    }

    #[test]
    fn test_persistence_error_invalid_data_display() {
        let err = PersistenceError::InvalidData("missing field".to_string());
        assert_eq!(err.to_string(), "Invalid data error: missing field");
    }

    #[test]
    fn test_persistence_error_not_found_display() {
        let err = PersistenceError::NotFound("record-123".to_string());
        assert_eq!(err.to_string(), "Data not found: record-123");
    }

    // ====================================================================
    // SQUIRREL ERROR - FACTORY METHODS
    // ====================================================================

    #[test]
    fn test_squirrel_error_generic() {
        let err = SquirrelError::generic("something went wrong");
        assert_eq!(err.to_string(), "Generic error: something went wrong");
    }

    #[test]
    fn test_squirrel_error_mcp() {
        let err = SquirrelError::mcp("protocol error");
        assert_eq!(err.to_string(), "MCP error: protocol error");
    }

    #[test]
    fn test_squirrel_error_context() {
        let err = SquirrelError::context("context not found");
        assert_eq!(err.to_string(), "Context error: context not found");
    }

    #[test]
    fn test_squirrel_error_plugin() {
        let err = SquirrelError::plugin("plugin crashed");
        assert_eq!(err.to_string(), "Plugin error: plugin crashed");
    }

    #[test]
    fn test_squirrel_error_serialization() {
        let err = SquirrelError::serialization("bad format");
        assert_eq!(err.to_string(), "Serialization error: bad format");
    }

    #[test]
    fn test_squirrel_error_network() {
        let err = SquirrelError::network("connection refused");
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_squirrel_error_authentication() {
        let err = SquirrelError::authentication("invalid token");
        assert_eq!(err.to_string(), "Authentication error: invalid token");
    }

    #[test]
    fn test_squirrel_error_authorization() {
        let err = SquirrelError::authorization("access denied");
        assert_eq!(err.to_string(), "Authorization error: access denied");
    }

    #[test]
    fn test_squirrel_error_validation() {
        let err = SquirrelError::validation("missing required field");
        assert_eq!(err.to_string(), "Validation error: missing required field");
    }

    #[test]
    fn test_squirrel_error_configuration() {
        let err = SquirrelError::configuration("bad config file");
        assert_eq!(err.to_string(), "Configuration error: bad config file");
    }

    #[test]
    fn test_squirrel_error_external_service() {
        let err = SquirrelError::external_service("service unavailable");
        assert_eq!(
            err.to_string(),
            "External service error: service unavailable"
        );
    }

    #[test]
    fn test_squirrel_error_timeout() {
        let err = SquirrelError::timeout("exceeded 30s");
        assert_eq!(err.to_string(), "Timeout error: exceeded 30s");
    }

    #[test]
    fn test_squirrel_error_persistence() {
        let err = SquirrelError::persistence(PersistenceError::Storage("disk error".to_string()));
        assert_eq!(
            err.to_string(),
            "Persistence error: Storage error: disk error"
        );
    }

    #[test]
    fn test_squirrel_error_persistence_storage() {
        let err = SquirrelError::persistence_storage("write failed");
        assert_eq!(
            err.to_string(),
            "Persistence error: Storage error: write failed"
        );
    }

    #[test]
    fn test_squirrel_error_session() {
        let err = SquirrelError::session("session expired");
        assert_eq!(err.to_string(), "Session error: session expired");
    }

    // ====================================================================
    // WITH CONTEXT
    // ====================================================================

    #[test]
    fn test_squirrel_error_with_context() {
        let err = SquirrelError::network("timeout").with_context("while calling discovery service");
        assert_eq!(
            err.to_string(),
            "while calling discovery service: Network error: timeout"
        );
    }

    // ====================================================================
    // IS RECOVERABLE
    // ====================================================================

    #[test]
    fn test_is_recoverable_network() {
        assert!(SquirrelError::network("timeout").is_recoverable());
    }

    #[test]
    fn test_is_recoverable_timeout() {
        assert!(SquirrelError::timeout("too slow").is_recoverable());
    }

    #[test]
    fn test_is_recoverable_external_service() {
        assert!(SquirrelError::external_service("503").is_recoverable());
    }

    #[test]
    fn test_is_recoverable_with_context_network() {
        let err = SquirrelError::network("conn reset").with_context("during health check");
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_not_recoverable_generic() {
        assert!(!SquirrelError::generic("fatal").is_recoverable());
    }

    #[test]
    fn test_not_recoverable_validation() {
        assert!(!SquirrelError::validation("bad input").is_recoverable());
    }

    #[test]
    fn test_not_recoverable_authentication() {
        assert!(!SquirrelError::authentication("bad creds").is_recoverable());
    }

    #[test]
    fn test_not_recoverable_configuration() {
        assert!(!SquirrelError::configuration("missing file").is_recoverable());
    }

    // ====================================================================
    // FROM CONVERSIONS
    // ====================================================================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: SquirrelError = io_err.into();
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_from_str() {
        let err: SquirrelError = "simple error".into();
        assert_eq!(err.to_string(), "Generic error: simple error");
    }

    #[test]
    fn test_from_string() {
        let err: SquirrelError = String::from("string error").into();
        assert_eq!(err.to_string(), "Generic error: string error");
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: SquirrelError = json_err.into();
        assert!(err.to_string().starts_with("Serialization error:"));
    }

    // ====================================================================
    // INTO SQUIRREL ERROR TRAIT
    // ====================================================================

    #[test]
    fn test_into_squirrel_error() {
        let io_err = std::io::Error::other("io problem");
        let err = io_err.into_squirrel_error();
        assert_eq!(err.to_string(), "Generic error: io problem");
    }

    #[test]
    fn test_into_squirrel_error_with_context() {
        let io_err = std::io::Error::other("io problem");
        let err = io_err.into_squirrel_error_with_context("reading config");
        assert_eq!(err.to_string(), "reading config: Generic error: io problem");
    }

    // ====================================================================
    // RESULT TYPE ALIAS
    // ====================================================================

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(SquirrelError::generic("fail"));
        assert!(result.is_err());
    }
}
