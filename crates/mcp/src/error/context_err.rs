use thiserror::Error;
use uuid;

/// Errors related to MCP context operations
///
/// This enum represents errors that can occur when working with MCP contexts,
/// including context lookup failures, validation errors, and synchronization issues.
#[derive(Debug, Clone, Error)]
pub enum ContextError {
    /// Error that occurs when a context with the specified UUID cannot be found
    ///
    /// This typically happens when trying to access a context that doesn't exist
    /// or has been removed.
    #[error("Context not found: {0}")]
    NotFound(uuid::Uuid),
    
    /// Error that occurs when context validation fails
    ///
    /// This can happen when a context contains invalid data or doesn't meet
    /// the required constraints.
    #[error("Context validation error: {0}")]
    ValidationError(String),
    
    /// Error that occurs during context synchronization
    ///
    /// This can happen when there are issues synchronizing context data
    /// between components or systems.
    #[error("Context sync error: {0}")]
    SyncError(String),

    /// General context error with a message
    ///
    /// This is used for errors that don't fit into the other categories.
    #[error("Context error: {0}")]
    General(String),
}

impl From<String> for ContextError {
    fn from(message: String) -> Self {
        ContextError::General(message)
    }
}

impl From<&str> for ContextError {
    fn from(message: &str) -> Self {
        ContextError::General(message.to_string())
    }
} 