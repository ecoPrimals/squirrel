// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::{ErrorHandler, HandlerError};
    use crate::error::MCPError;
    use crate::error::connection::ConnectionError;
    use std::fmt::Write as _;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[test]
    fn handler_error_display_not_found() {
        let err = HandlerError::NotFound("h".into());
        assert!(err.to_string().contains("Handler not found"));
    }

    #[test]
    fn handler_error_display_execution_failed() {
        let err = HandlerError::ExecutionFailed("run".into());
        assert!(err.to_string().contains("execution"));
    }

    #[test]
    fn handler_error_display_invalid_config() {
        let err = HandlerError::InvalidConfig("bad".into());
        assert!(err.to_string().contains("config"));
    }

    #[test]
    fn handler_error_display_timeout() {
        let err = HandlerError::Timeout("t".into());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn handler_error_debug_all_variants() {
        let cases = [
            HandlerError::NotFound("a".into()),
            HandlerError::ExecutionFailed("b".into()),
            HandlerError::InvalidConfig("c".into()),
            HandlerError::Timeout("d".into()),
        ];
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn handler_error_default_is_not_found() {
        let err = HandlerError::default();
        assert!(err.to_string().contains("Unknown handler"));
    }

    #[test]
    fn error_handler_new_exposes_context() {
        let h = ErrorHandler::new(2, Duration::from_millis(0), "operation", "component");
        assert_eq!(h.error_context().operation, "operation");
        assert_eq!(h.error_context().component, "component");
    }

    #[tokio::test]
    async fn error_handler_handle_error_returns_non_recoverable_immediately() {
        let mut h = ErrorHandler::new(5, Duration::from_millis(0), "op", "comp");
        let err = h
            .handle_error(|| -> Result<(), MCPError> { Err(MCPError::from_message("fail")) })
            .await
            .expect_err("non-recoverable");
        assert!(err.to_string().contains("fail"));
    }

    #[tokio::test]
    async fn error_handler_handle_error_retries_recoverable_then_ok() {
        let mut h = ErrorHandler::new(5, Duration::from_millis(0), "op", "comp");
        let attempts = AtomicU32::new(0);
        let out = h
            .handle_error(|| {
                let n = attempts.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(MCPError::Connection(ConnectionError::Timeout(1)))
                } else {
                    Ok(99u32)
                }
            })
            .await
            .expect("ok");
        assert_eq!(out, 99);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn error_handler_handle_error_stops_after_max_retries() {
        let mut h = ErrorHandler::new(2, Duration::from_millis(0), "op", "comp");
        let attempts = AtomicU32::new(0);
        let err = h
            .handle_error(|| -> Result<(), MCPError> {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err(MCPError::Connection(ConnectionError::Timeout(1)))
            })
            .await
            .expect_err("exhausted");
        assert!(matches!(
            err,
            MCPError::Connection(ConnectionError::Timeout(_))
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}
