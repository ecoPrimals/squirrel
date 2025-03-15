//! MCP (Machine Context Protocol) module
//!
//! This module implements the core functionality for the Machine Context Protocol.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub mod types;
pub mod security;

use types::SecurityLevel;

/// Error type for MCP operations
#[derive(Debug, Error)]
pub enum MCPError {
    /// Lock acquisition failed
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

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
        required: SecurityLevel,
        /// Actual security level provided
        provided: SecurityLevel,
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

/// A Result type alias for MCP operations
pub type Result<T> = std::result::Result<T, MCPError>;

/// Configuration for the MCP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// The protocol version
    pub version: String,
    /// Maximum message size in bytes
    pub max_message_size: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// The main MCP system controller
pub struct MCP {
    /// The MCP configuration, wrapped in a thread-safe read-write lock
    config: Arc<RwLock<MCPConfig>>,
}

impl MCP {
    /// Creates a new MCP instance with the specified configuration
    #[must_use]
    pub fn new(config: MCPConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Gets the current configuration
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::LockError` if the configuration lock cannot be acquired
    #[must_use = "This returns the current MCP configuration which may be needed for further operations"]
    pub async fn get_config(&self) -> Result<MCPConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024 * 10, // 10MB
            timeout_ms: 30000, // 30 seconds
        }
    }
}

impl Default for MCP {
    fn default() -> Self {
        Self::new(MCPConfig::default())
    }
}

pub use security::{SecurityConfig, SecurityManager, Credentials};
pub use types::EncryptionFormat; 