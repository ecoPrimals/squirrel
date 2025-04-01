//! Error handling and management for MCP operations.
//!
//! This module provides error types, context management, and error handling
//! utilities for the Machine Context Protocol (MCP) system. It defines a
//! comprehensive hierarchy of error types specific to MCP operations and
//! error handling strategies for various scenarios.
//!
//! # Key Components
//!
//! - `MCPError`: The core error type used throughout the MCP system
//! - `ErrorContext`: Provides additional context for errors
//! - `ErrorHandler`: Implements retry and recovery strategies
//! - `Result<T>`: Type alias for MCP operation results
//!
//! # Examples
//!
//! Using the Result type:
//!
//! ```
//! use squirrel_mcp::error::{MCPError, Result};
//!
//! fn perform_operation() -> Result<String> {
//!     // Operation that could fail
//!     Ok("Operation succeeded".to_string())
//! }
//! ```

/// Error context management for MCP operations
pub mod context;
/// Error type definitions for MCP operations
pub mod types;
/// Transport error conversions
pub mod transport;
/// Client error types
pub mod client;
/// Connection error types
pub mod connection;
/// Protocol error types
pub mod protocol_err;
/// Security error types
pub mod security_err;
/// Session error types
pub mod session;
/// Context-specific error types
pub mod context_err;
/// Alert error types
pub mod alert;
/// RBAC error types
pub mod rbac;
/// Error handler struct
pub mod handler;
/// Port error kind types
pub mod port;
/// Tests for error types
#[cfg(test)]
mod types_tests;
/// Added tool error module
pub mod tool;
/// Added config error module
pub mod config;
/// Added plugin error module
pub mod plugin;

pub use types::MCPError;
pub use client::ClientError;
pub use connection::ConnectionError;
pub use protocol_err::ProtocolError;
pub use security_err::SecurityError;
pub use session::SessionError;
pub use context_err::ContextError;
pub use alert::AlertError;
pub use rbac::RBACError;
pub use handler::ErrorHandler;
pub use port::PortErrorKind;
pub use tool::ToolError;
pub use config::ConfigError;
pub use plugin::PluginError;

pub use types::{ErrorContext};
pub use transport::TransportError;

use std::fmt::{Debug};

/// Result type for MCP operations that can return an error.
///
/// This type alias provides a consistent return type for functions
/// that can fail with an `MCPError`.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::error::{MCPError, Result};
///
/// fn read_data() -> Result<Vec<u8>> {
///     // Code that might fail
///     Ok(vec![1, 2, 3])
/// }
/// ```
pub type Result<T> = std::result::Result<T, MCPError>;

// Add a type alias for MCPResult, which is used in some parts of the code
/// Alias for Result type to maintain backward compatibility
/// with code that uses `MCPResult` instead of Result.
pub type MCPResult<T> = Result<T>;

// Re-export specific types from context module
pub use context::{ErrorHandlerError, ErrorRecord, ErrorSeverity, RecoveryStrategy};

// Commenting out conflicting implementation - Use MCPError::from_message instead for now.
/*
impl TryFrom<Message> for MCPError {
    type Error = Self;

    fn try_from(message: Message) -> std::result::Result<Self, Self::Error> {
        if message.message_type != crate::message::MessageType::Error {
            return Err(Self::Protocol(ProtocolError::InvalidFormat(
                format!("Expected error message, got {:?}", message.message_type)
            )));
        }

        // Extract error details from message metadata
        let error_type = message.metadata.get("error_type")
            .map_or("unknown", std::string::String::as_str);

        // Create appropriate error based on type
        match error_type {
            // Using .into() to convert between the different TransportError types
            "transport" => Ok(Self::Transport(transport::TransportError::ProtocolError(message.content).into())),
            "protocol" => Ok(Self::Protocol(ProtocolError::InvalidFormat(message.content))),
            "security" => Ok(Self::Security(SecurityError::AuthenticationFailed(message.content))),
            "connection" => Ok(Self::Connection(ConnectionError::Closed(message.content))),
            "session" => Ok(Self::Session(SessionError::InvalidSession(message.content))),
            "context" => Ok(Self::Context(ContextError::NotFound(uuid::Uuid::new_v4()))),
            "client" => Ok(Self::Client(ClientError::RemoteError(message.content))),
            _ => Ok(Self::Remote(message.content)),
        }
    }
}
*/

/// Generic result type used throughout MCP
// pub type Result<T, E = MCPError> = std::result::Result<T, E>; // Remove this duplicate alias

/// Documentation structure for failure operation contexts
#[derive(Debug, Clone)]
pub struct FailedOperation {
    /// The name of the operation that failed
    pub operation: String,
    /// Additional details about the failure
    pub details: Option<String>,
}

impl std::fmt::Display for FailedOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation '{}' failed", self.operation)?;
        if let Some(details) = &self.details {
            write!(f, ": {}", details)?;
        }
        Ok(())
    }
}

// Add implementation for RecoveryError
impl From<crate::resilience::recovery::RecoveryError> for MCPError {
    fn from(err: crate::resilience::recovery::RecoveryError) -> Self {
        Self::General(format!("Recovery error: {}", err))
    }
}

impl MCPError {
    /// Create an error from a string message
    pub fn from_string(message: String) -> Self {
        MCPError::General(message)
    }
}
