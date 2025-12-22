//! Universal Error System for Squirrel
//!
//! This crate provides a unified error handling system that extends the excellent
//! MCP error architecture to the entire Squirrel codebase. It re-exports the
//! world-class MCP error system (validated Phase 3E) and adds domain error modules
//! for SDK, Tools, and Integration components following the same patterns.
//!
//! # Architecture
//!
//! The error system follows a hierarchical architecture with automatic type conversions:
//!
//! ```text
//! UniversalError (Top Level)
//!     ├── MCPError (from mcp/error) - Re-exported ✅ World-class
//!     ├── SDKError (new) - Infrastructure, Communication, Client
//!     ├── ToolsError (new) - AI Tools, CLI, Rule System
//!     └── IntegrationError (new) - Web, API Clients, Adapters
//! ```
//!
//! All domain errors automatically convert to `UniversalError` via `#[from]` attribute,
//! providing zero-cost error propagation while maintaining type safety.
//!
//! # Design Principles
//!
//! 1. **Extend Excellence**: Preserve the MCP error system (already world-class)
//! 2. **Pattern Consistency**: All new errors follow MCP architecture
//! 3. **Zero-Cost Conversions**: Automatic `#[from]` conversions at compile-time
//! 4. **Domain Separation**: Clear boundaries between error domains
//! 5. **Type Safety**: Full compile-time error checking
//!
//! # Usage
//!
//! ## Basic Usage
//!
//! ```
//! use universal_error::{UniversalError, Result};
//!
//! fn operation() -> Result<String> {
//!     // Any domain error automatically converts to UniversalError
//!     Ok("success".to_string())
//! }
//! ```
//!
//! ## Pattern Matching
//!
//! ```
//! use universal_error::{UniversalError, MCPError};
//!
//! fn handle_error(err: UniversalError) {
//!     match err {
//!         UniversalError::MCP(mcp_err) => {
//!             // Handle MCP-specific errors
//!             println!("MCP error: {}", mcp_err);
//!         }
//!         UniversalError::SDK(sdk_err) => {
//!             // Handle SDK-specific errors
//!             println!("SDK error: {}", sdk_err);
//!         }
//!         _ => {
//!             // Handle other errors
//!             println!("Other error: {}", err);
//!         }
//!     }
//! }
//! ```
//!
//! ## Automatic Conversions
//!
//! ```
//! use universal_error::{UniversalError, Result};
//! use squirrel_mcp::error::MCPError;
//!
//! fn mcp_operation() -> std::result::Result<(), MCPError> {
//!     // ... MCP operation
//!     Ok(())
//! }
//!
//! fn unified_operation() -> Result<()> {
//!     // MCPError automatically converts to UniversalError
//!     mcp_operation()?;
//!     Ok(())
//! }
//! ```
//!
//! # Migration Guide
//!
//! ## From MCP Errors
//!
//! No changes needed! MCP errors are re-exported:
//!
//! ```ignore
//! // Old
//! use squirrel_mcp::error::{MCPError, ErrorContext};
//!
//! // New (same thing, just different path)
//! use universal_error::{MCPError, ErrorContext};
//! ```
//!
//! ## From Scattered Errors
//!
//! Old code with multiple error types:
//!
//! ```ignore
//! // Old
//! use sdk::infrastructure::error::SdkError;
//! use ai_tools::error::AIError;
//! use squirrel_mcp::error::MCPError;
//!
//! fn operation() -> Result<(), SomeCustomError> {
//!     // Manual error wrapping needed
//!     mcp_operation().map_err(|e| SomeCustomError::MCP(e))?;
//!     sdk_operation().map_err(|e| SomeCustomError::SDK(e))?;
//!     Ok(())
//! }
//! ```
//!
//! New code with unified errors:
//!
//! ```ignore
//! // New
//! use universal_error::{UniversalError, Result};
//!
//! fn operation() -> Result<()> {
//!     // Automatic conversions!
//!     mcp_operation()?;
//!     sdk_operation()?;
//!     Ok(())
//! }
//! ```

// Re-export the excellent MCP error system (world-class, validated Phase 3E)
pub use squirrel_mcp::error::{
    // Domain-specific MCP errors
    AlertError,
    ClientError as MCPClientError, // Disambiguate from SDK ClientError
    ConfigError as MCPConfigError, // Disambiguate from SDK ConfigError
    ConnectionError,
    ContextError,
    ErrorContext,
    ErrorContextTrait,
    ErrorSeverity,

    HandlerError,
    IntegrationError as MCPIntegrationError, // Disambiguate
    // Main error types
    MCPError,
    PluginError as MCPPluginError, // Disambiguate
    PortErrorKind,
    ProtocolError,
    RBACError,
    RegistryError,
    // Utilities
    SecurityLevel,
    SessionError,
    TaskError,
    ToolError,
    TransportError,

    WireFormatError,
};

// New domain error modules following MCP pattern
pub mod integration;
pub mod sdk;
pub mod tools;

/// Unified result type using UniversalError
///
/// This is the recommended result type for all operations that may fail.
///
/// # Example
///
/// ```
/// use universal_error::Result;
///
/// fn operation() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, UniversalError>;

/// Top-level universal error that encompasses all domain errors
///
/// This enum provides a unified error type that all domain-specific errors
/// automatically convert into via the `#[from]` attribute. This enables
/// zero-cost error propagation while maintaining type safety and domain context.
///
/// # Architecture
///
/// ```text
/// UniversalError
///     ├── MCP (MCPError) - 123 error types, world-class architecture
///     ├── SDK (SDKError) - Infrastructure, Communication, Client errors
///     ├── Tools (ToolsError) - AI Tools, CLI, Rule System errors
///     └── Integration (IntegrationError) - Web, API, Adapter errors
/// ```
///
/// # Examples
///
/// ## Automatic Conversion
///
/// ```
/// use universal_error::{UniversalError, Result};
///
/// fn operation_that_fails() -> Result<()> {
///     // Any domain error automatically converts
///     Err(UniversalError::General("something went wrong".to_string()))
/// }
/// ```
///
/// ## Pattern Matching
///
/// ```
/// use universal_error::{UniversalError, MCPError};
///
/// fn handle_error(err: UniversalError) {
///     match err {
///         UniversalError::MCP(mcp_err) => {
///             println!("MCP error: {}", mcp_err);
///         }
///         UniversalError::General(msg) => {
///             println!("General error: {}", msg);
///         }
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum UniversalError {
    /// Error originating from the MCP system
    ///
    /// This encompasses all MCP-related errors including transport, protocol,
    /// connection, session, plugin, tool, and other MCP domain errors.
    /// The MCP error system is world-class and validated (Phase 3E).
    #[error(transparent)]
    MCP(#[from] MCPError),

    /// Error originating from the SDK infrastructure
    ///
    /// This encompasses SDK-related errors including infrastructure errors,
    /// communication errors, and client errors.
    #[error(transparent)]
    SDK(#[from] sdk::SDKError),

    /// Error originating from tools (AI, CLI, Rule System)
    ///
    /// This encompasses all tool-related errors including AI tools, CLI commands,
    /// and rule system operations.
    #[error(transparent)]
    Tools(#[from] tools::ToolsError),

    /// Error originating from integration components
    ///
    /// This encompasses integration-related errors including web integrations,
    /// API clients, context adapters, and ecosystem connections.
    #[error(transparent)]
    Integration(#[from] integration::IntegrationError),

    /// General error that doesn't fit into specific domains
    ///
    /// Use this sparingly - prefer domain-specific errors when possible.
    #[error("General error: {0}")]
    General(String),

    /// Internal error (should never happen in production)
    ///
    /// This indicates a logic error or invariant violation.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl UniversalError {
    /// Create a general error from a string
    pub fn general<S: Into<String>>(msg: S) -> Self {
        Self::General(msg.into())
    }

    /// Create an internal error from a string
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Check if this error is from the MCP domain
    pub fn is_mcp(&self) -> bool {
        matches!(self, Self::MCP(_))
    }

    /// Check if this error is from the SDK domain
    pub fn is_sdk(&self) -> bool {
        matches!(self, Self::SDK(_))
    }

    /// Check if this error is from the Tools domain
    pub fn is_tools(&self) -> bool {
        matches!(self, Self::Tools(_))
    }

    /// Check if this error is from the Integration domain
    pub fn is_integration(&self) -> bool {
        matches!(self, Self::Integration(_))
    }
}

// Implement conversion from common standard library errors
impl From<std::io::Error> for UniversalError {
    fn from(err: std::io::Error) -> Self {
        Self::general(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for UniversalError {
    fn from(err: serde_json::Error) -> Self {
        Self::general(format!("JSON error: {}", err))
    }
}

impl From<&str> for UniversalError {
    fn from(s: &str) -> Self {
        Self::general(s)
    }
}

impl From<String> for UniversalError {
    fn from(s: String) -> Self {
        Self::general(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_error() {
        let err = UniversalError::general("test error");
        assert!(matches!(err, UniversalError::General(_)));
        assert_eq!(err.to_string(), "General error: test error");
    }

    #[test]
    fn test_internal_error() {
        let err = UniversalError::internal("internal issue");
        assert!(matches!(err, UniversalError::Internal(_)));
        assert_eq!(err.to_string(), "Internal error: internal issue");
    }

    #[test]
    fn test_is_mcp() {
        let mcp_err = MCPError::General("test".to_string());
        let universal_err: UniversalError = mcp_err.into();
        assert!(universal_err.is_mcp());
        assert!(!universal_err.is_sdk());
    }

    #[test]
    fn test_string_conversion() {
        let err: UniversalError = "test error".into();
        assert!(matches!(err, UniversalError::General(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let universal_err: UniversalError = io_err.into();
        assert!(matches!(universal_err, UniversalError::General(_)));
    }
}
