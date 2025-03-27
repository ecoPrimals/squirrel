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
//! use mcp::error::{MCPError, Result};
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

use thiserror::Error;
pub use types::MCPError;

/// Context-specific errors that can occur during MCP operations.
///
/// These errors relate to the management and handling of context
/// in the MCP system, such as initialization failures or invalid states.
#[derive(Error, Debug)]
pub enum ContextError {
    /// Error when context initialization fails
    #[error("Context initialization failed: {0}")]
    InitializationFailed(String),

    /// Error when attempting to create a context that already exists
    #[error("Context already exists: {0}")]
    AlreadyExists(String),

    /// Error when a required context is not found
    #[error("Context not found: {0}")]
    NotFound(String),

    /// Error when a context operation times out
    #[error("Context operation timed out")]
    Timeout,

    /// Error when a context is in an invalid state for an operation
    #[error("Invalid context state: {0}")]
    InvalidState(String),
}

/// Result type for MCP operations that can return an error.
///
/// This type alias provides a consistent return type for functions
/// that can fail with an `MCPError`.
///
/// # Examples
///
/// ```
/// use mcp::error::{MCPError, Result};
///
/// fn read_data() -> Result<Vec<u8>> {
///     // Code that might fail
///     Ok(vec![1, 2, 3])
/// }
/// ```
pub type Result<T> = std::result::Result<T, MCPError>;

// Re-export specific types from types module
pub use types::{ConnectionError, ErrorContext, PortErrorKind, ProtocolError, SecurityError};

// Re-export specific types from context module
pub use context::{ErrorHandler, ErrorHandlerError, ErrorRecord, ErrorSeverity, RecoveryStrategy};
