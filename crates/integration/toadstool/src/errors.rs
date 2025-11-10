//! Error types for Toadstool integration
//!
//! **DEPRECATED**: This error module is being replaced by the unified error system.
//! Please migrate to `universal-error` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use crate::errors::{ToadstoolError, ToadstoolResult};
//! // New:
//! use universal_error::{Result, integration::EcosystemError};
//! ```
//!
//! For detailed migration instructions, see: `crates/universal-error/README.md`

use thiserror::Error;

/// Result type for Toadstool operations
pub type ToadstoolResult<T> = std::result::Result<T, ToadstoolError>;

/// Errors that can occur during Toadstool integration
///
/// **DEPRECATED**: Use `universal_error::integration::EcosystemError` instead.
#[deprecated(since = "0.2.0", note = "Use `universal_error::integration::EcosystemError` instead")]
#[derive(Debug, Error)]
pub enum ToadstoolError {
    /// Connection error to Toadstool service
    #[error("Connection error: {0}")]
    Connection(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Timeout error
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Sandbox violation
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    /// Execution not found
    #[error("Execution not found: {0}")]
    ExecutionNotFound(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalServer(String),
}

impl ToadstoolError {
    /// Create a connection error
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create an authentication error
    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create an execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Execution(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        Self::ResourceExhausted(msg.into())
    }

    /// Create a sandbox violation error
    pub fn sandbox_violation(msg: impl Into<String>) -> Self {
        Self::SandboxViolation(msg.into())
    }

    /// Create a plugin not found error
    pub fn plugin_not_found(msg: impl Into<String>) -> Self {
        Self::PluginNotFound(msg.into())
    }

    /// Create an execution not found error
    pub fn execution_not_found(msg: impl Into<String>) -> Self {
        Self::ExecutionNotFound(msg.into())
    }

    /// Create an invalid request error
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    /// Create an internal server error
    pub fn internal_server(msg: impl Into<String>) -> Self {
        Self::InternalServer(msg.into())
    }
}
