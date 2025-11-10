use crate::error::types::ErrorContext;
use crate::error::types::MCPError;
use serde::{Deserialize, Serialize};

/// Error type for handler operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum HandlerError {
    /// Handler not found
    #[error("Handler not found: {0}")]
    NotFound(String),
    
    /// Handler execution failed
    #[error("Handler execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Invalid handler configuration
    #[error("Invalid handler config: {0}")]
    InvalidConfig(String),
    
    /// Handler timeout
    #[error("Handler timeout: {0}")]
    Timeout(String),
}

impl Default for HandlerError {
    fn default() -> Self {
        Self::NotFound("Unknown handler".to_string())
    }
}

/// Error handler with retry capabilities
///
/// Provides mechanisms for handling errors, including automatic retry with
/// configurable backoff, error context tracking, and recovery strategies.
#[derive(Debug)]
pub struct ErrorHandler {
    /// Maximum number of retry attempts
    /// This defines how many times the handler will retry an operation before giving up
    max_retries: u32,
    /// Delay between retry attempts
    /// Specifies how long to wait between retry attempts
    retry_delay: std::time::Duration,
    /// Context information for errors
    /// Contains metadata and context about the errors being handled
    error_context: ErrorContext,
}

impl ErrorHandler {
    /// Creates a new `ErrorHandler` with the specified retry parameters
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of times to retry failed operations
    /// * `retry_delay` - How long to wait between retry attempts
    /// * `operation` - Name or description of the operation being handled
    /// * `component` - Name of the component where the operation is performed
    ///
    /// # Returns
    ///
    /// A new `ErrorHandler` configured with the specified parameters
    pub fn new(
        max_retries: u32,
        retry_delay: std::time::Duration,
        operation: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        Self {
            max_retries,
            retry_delay,
            error_context: ErrorContext::new(operation, component),
        }
    }

    /// Handles operation errors with automatic retries
    ///
    /// # Arguments
    /// * `operation` - A closure that returns a `Result<T, MCPError>`
    ///
    /// # Returns
    /// * `Result<T, MCPError>` - The result of the operation or the last error encountered
    ///
    /// # Errors
    /// Returns an error if the operation failed after all retry attempts or
    /// if the error is not recoverable
    pub async fn handle_error<F, T>(&mut self, operation: F) -> Result<T, MCPError>
    where
        F: Fn() -> Result<T, MCPError> + Send + Sync,
    {
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    self.error_context.increment_retry_count();

                    if !error.is_recoverable() || self.error_context.retry_count >= self.max_retries
                    {
                        return Err(error);
                    }

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    /// Gets the current error context
    ///
    /// # Returns
    ///
    /// A reference to the current error context
    #[must_use]
    pub const fn error_context(&self) -> &ErrorContext {
        &self.error_context
    }
}
