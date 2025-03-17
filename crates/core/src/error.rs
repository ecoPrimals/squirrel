//! Error types for the Squirrel project
//!
//! This module defines the main error types and results used throughout the project.

use thiserror::Error;

/// Main error type for the Squirrel project
#[derive(Debug, Error)]
pub enum SquirrelError {
    /// Errors originating from the app module
    #[error("App error: {0}")]
    App(String),

    /// Errors originating from the MCP module
    #[error("MCP error: {0}")]
    MCP(#[from] MCPError),
    
    /// Errors originating from the monitoring module
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    
    /// Other miscellaneous errors that don't fit into specific categories
    #[error("Other error: {0}")]
    Other(String),
}

/// Error type for MCP-related operations
#[derive(Debug, Error)]
pub enum MCPError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Protocol-related errors
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization errors
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Compression errors
    #[error("Compression error: {0}")]
    Compression(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Security errors
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    /// State errors
    #[error("State error: {0}")]
    State(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Error type for security-related operations
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization errors
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Token-related errors
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Token expiration errors
    #[error("Token expired")]
    TokenExpired,

    /// Security level errors
    #[error("Invalid security level: required {required:?}, provided {provided:?}")]
    InvalidSecurityLevel {
        /// Required security level for the operation
        required: crate::mcp::types::SecurityLevel,
        /// Actual security level provided
        provided: crate::mcp::types::SecurityLevel,
    },

    /// Encryption errors
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption errors
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Role-related errors
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    /// Permission-related errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Role assignment errors
    #[error("Role assignment failed: {0}")]
    RoleAssignmentFailed(String),
    
    /// Permission validation errors
    #[error("Invalid permission: {0}")]
    InvalidPermission(String),

    /// Other security-related errors
    #[error("Other security error: {0}")]
    Other(String),
}

/// A Result type alias for operations that may return a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>; 