/*!
 * Error handling for the Galaxy adapter.
 * 
 * This module defines the error types used throughout the Galaxy adapter crate.
 */

use std::fmt;
use thiserror::Error;

/// Main error type for the Galaxy adapter
#[derive(Error, Debug)]
pub enum Error {
    /// The adapter is not properly initialized
    #[error("Galaxy adapter not initialized")]
    NotInitialized,

    /// The adapter is already initialized
    #[error("Galaxy adapter already initialized")]
    AlreadyInitialized,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// API error from Galaxy
    #[error("Galaxy API error: {0}")]
    GalaxyApi(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Network response decoding error
    #[error("Network response decode error: {0}")]
    NetworkResponseDecode(String),

    /// Invalid tool definition
    #[error("Invalid tool definition: {0}")]
    InvalidTool(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),

    /// Workflow error
    #[error("Workflow error: {0}")]
    Workflow(String),

    /// Data handling error
    #[error("Data error: {0}")]
    Data(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// MCP integration error
    #[error("MCP integration error: {0}")]
    McpIntegration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for Galaxy operations
pub type Result<T> = std::result::Result<T, Error>;

/// Severity level for Galaxy errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Fatal error that cannot be recovered from
    Fatal,
    /// Error that can be recovered from, but requires intervention
    Error,
    /// Warning that doesn't prevent operation but should be addressed
    Warning,
    /// Informational message about a minor issue
    Info,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Fatal => write!(f, "FATAL"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Extended error details
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// Severity level
    pub severity: ErrorSeverity,
    /// Origin of the error
    pub origin: String,
    /// Time when the error occurred
    pub timestamp: time::OffsetDateTime,
    /// Additional context information
    pub context: Option<serde_json::Value>,
    /// Error code if available
    pub code: Option<String>,
    /// Suggested action to resolve the error
    pub suggested_action: Option<String>,
}

impl ErrorDetails {
    /// Create a new error details object
    pub fn new(severity: ErrorSeverity, origin: impl Into<String>) -> Self {
        Self {
            severity,
            origin: origin.into(),
            timestamp: time::OffsetDateTime::now_utc(),
            context: None,
            code: None,
            suggested_action: None,
        }
    }

    /// Add context information
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Add error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add suggested action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }
} 