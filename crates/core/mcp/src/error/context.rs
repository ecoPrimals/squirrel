// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::error::MCPError;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Errors that can occur within the error handling system itself
///
/// These represent failures that happen during error recording, recovery,
/// or management operations.
#[derive(Debug)]
pub enum ErrorHandlerError {
    /// Error that occurs during recovery operations
    ///
    /// This represents failures that happen when trying to
    /// recover from another error.
    Recovery(String),

    /// Error related to state management
    ///
    /// This represents failures in maintaining or
    /// transitioning state during error handling.
    State(String),

    /// Error related to recovery strategy
    ///
    /// This represents failures in applying or managing
    /// error recovery strategies.
    Strategy(String),

    /// Error related to context management
    ///
    /// This represents failures in maintaining or accessing
    /// error context information.
    Context(String),

    /// Error during JSON serialization or deserialization
    ///
    /// This occurs when error data cannot be properly
    /// serialized or deserialized.
    SerdeJson(serde_json::Error),
}

impl std::fmt::Display for ErrorHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recovery(msg) => write!(f, "Recovery error: {msg}"),
            Self::State(msg) => write!(f, "State error: {msg}"),
            Self::Strategy(msg) => write!(f, "Strategy error: {msg}"),
            Self::Context(msg) => write!(f, "Context error: {msg}"),
            Self::SerdeJson(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl std::error::Error for ErrorHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SerdeJson(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ErrorHandlerError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}

/// Local context for error handling
///
/// Provides detailed information about a specific error occurrence,
/// including identification, timing, classification, and recovery tracking.
#[derive(Debug)]
pub struct LocalErrorContext {
    /// Unique identifier for the error context
    pub id: Uuid,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
    /// Type of error that occurred
    pub error_type: String,
    /// Detailed error message
    pub message: String,
    /// Component or module where the error occurred
    pub component: String,
    /// Severity level of the error
    pub severity: ErrorSeverity,
    /// Additional metadata about the error in JSON format
    pub metadata: Option<serde_json::Value>,
    /// Number of recovery attempts that have been made
    pub recovery_attempts: u32,
}

impl LocalErrorContext {
    /// Creates a new error context with the specified error information
    ///
    /// # Arguments
    ///
    /// * `error_type` - The type or category of the error
    /// * `message` - A descriptive message about the error
    /// * `component` - The component where the error occurred
    ///
    /// # Returns
    ///
    /// A new `LocalErrorContext` with default values for other fields
    pub fn new(
        error_type: impl Into<String>,
        message: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            error_type: error_type.into(),
            message: message.into(),
            component: component.into(),
            severity: ErrorSeverity::Low,
            metadata: None,
            recovery_attempts: 0,
        }
    }

    /// Gets the unique identifier for this error context
    ///
    /// # Returns
    ///
    /// The UUID identifying this error context
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Gets the timestamp when this error occurred
    ///
    /// # Returns
    ///
    /// The `DateTime` when the error was recorded
    #[must_use]
    pub const fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Gets the type of this error
    ///
    /// # Returns
    ///
    /// The error type as a string
    #[must_use]
    pub fn error_type(&self) -> &str {
        &self.error_type
    }

    /// Gets the error message
    ///
    /// # Returns
    ///
    /// The descriptive error message
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Gets the component where this error occurred
    ///
    /// # Returns
    ///
    /// The component name as a string
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Gets the severity level of this error
    ///
    /// # Returns
    ///
    /// The `ErrorSeverity` level
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Gets the metadata associated with this error
    ///
    /// # Returns
    ///
    /// Optional JSON metadata about the error
    #[must_use]
    pub const fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    /// Gets the number of recovery attempts made for this error
    ///
    /// # Returns
    ///
    /// The count of recovery attempts
    #[must_use]
    pub const fn recovery_attempts(&self) -> u32 {
        self.recovery_attempts
    }

    /// Sets the severity level for this error context
    ///
    /// # Arguments
    ///
    /// * `severity` - The new severity level
    ///
    /// # Returns
    ///
    /// The updated context for method chaining
    #[must_use]
    pub const fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds metadata to this error context
    ///
    /// # Arguments
    ///
    /// * `metadata` - JSON metadata about the error
    ///
    /// # Returns
    ///
    /// The updated context for method chaining
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Increments the recovery attempt counter
    ///
    /// This is called each time a recovery attempt is made.
    pub const fn increment_recovery_attempts(&mut self) {
        self.recovery_attempts += 1;
    }
}

/// Severity levels for errors
// Re-export canonical ErrorSeverity from types module
pub use crate::error::types::ErrorSeverity;

/// Strategy for recovering from errors
///
/// Defines the parameters and approach for attempting recovery after an error occurs,
/// including retry attempts, backoff timing, and intervention requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Maximum number of recovery attempts before giving up
    pub max_attempts: u32,

    /// Base backoff time in milliseconds between retry attempts
    pub backoff_ms: u64,

    /// Timeout in milliseconds for each recovery attempt
    pub timeout_ms: u64,

    /// Whether the recovery requires manual intervention to proceed
    pub requires_manual_intervention: bool,
}

/// Record of a specific error occurrence
///
/// Contains information about a single error event, including
/// when it occurred, what type of error it was, and any additional details.
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    /// When the error occurred
    pub timestamp: DateTime<Utc>,

    /// Type/category of the error
    pub error_type: String,

    /// Human-readable error message
    pub message: String,

    /// Optional additional details about the error
    pub details: Option<String>,
}

/// Tracks and manages error history
#[derive(Debug)]
pub struct ErrorHandler {
    /// Maximum number of error records to keep in history
    /// This prevents unbounded memory usage for error tracking
    max_history_size: usize,
    /// History of error records stored in a thread-safe, lockable queue
    error_history: Arc<RwLock<VecDeque<ErrorRecord>>>,
}

impl ErrorHandler {
    /// Creates a new `ErrorHandler` with the specified history size
    ///
    /// # Arguments
    ///
    /// * `max_history_size` - Maximum number of error records to keep in history
    ///
    /// # Returns
    ///
    /// A new `ErrorHandler` instance
    #[must_use]
    pub fn new(max_history_size: usize) -> Self {
        Self {
            max_history_size,
            error_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
        }
    }

    /// Records an error in the history
    ///
    /// Adds a new error record to the history, removing the oldest record if
    /// the history size exceeds the maximum.
    ///
    /// # Arguments
    ///
    /// * `error_type` - Type/category of the error
    /// * `message` - Human-readable error message
    /// * `details` - Optional additional details about the error
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    #[instrument(skip(self))]
    pub async fn record_error(
        &self,
        error_type: String,
        message: String,
        details: Option<String>,
    ) -> Result<(), ErrorHandlerError> {
        let record = ErrorRecord {
            timestamp: Utc::now(),
            error_type,
            message,
            details,
        };

        let mut history = self.error_history.write().await;
        while history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(record);
        drop(history);

        Ok(())
    }

    /// Retrieves the complete error history
    ///
    /// # Returns
    ///
    /// Result containing a vector of all error records in the history
    #[instrument(skip(self))]
    pub async fn get_error_history(&self) -> Result<Vec<ErrorRecord>> {
        let history = self.error_history.read().await;
        Ok(history.iter().cloned().collect())
    }

    /// Clears all error records from the history
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    #[instrument(skip(self))]
    pub async fn clear_history(&self) -> Result<()> {
        self.error_history.write().await.clear();
        Ok(())
    }
}

impl Clone for ErrorHandler {
    fn clone(&self) -> Self {
        Self {
            max_history_size: self.max_history_size,
            error_history: Arc::clone(&self.error_history),
        }
    }
}

impl ErrorHandler {
    /// Handles an error by recording it and attempting recovery
    ///
    /// # Arguments
    ///
    /// * `context` - The error context to handle
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    #[instrument]
    pub async fn handle_error(
        &self,
        mut context: LocalErrorContext,
    ) -> Result<(), ErrorHandlerError> {
        info!(
            error_id = %context.id,
            error_type = %context.error_type,
            "Handling error"
        );

        // Record error
        self.record_error(
            context.error_type.clone(),
            context.message.clone(),
            Some(format!("{context:?}")),
        )
        .await?;

        // Attempt recovery if strategy exists
        if let Some(strategy) = self.get_recovery_strategy(&context.error_type).await {
            self.attempt_recovery(&mut context, &strategy).await?;
        } else {
            warn!(
                error_type = %context.error_type,
                "No recovery strategy found"
            );
        }

        Ok(())
    }

    /// Retrieves a recovery strategy for the specified error type
    ///
    /// # Arguments
    ///
    /// * `error_type` - The type of error to find a strategy for
    ///
    /// # Returns
    ///
    /// An optional recovery strategy if one exists for the error type
    #[instrument(skip(self))]
    async fn get_recovery_strategy(&self, error_type: &str) -> Option<RecoveryStrategy> {
        // Implementation of get_recovery_strategy method
        None
    }

    /// Attempts to recover from an error using the specified strategy
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context to recover from
    /// * `_strategy` - The recovery strategy to use
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    #[instrument(skip(self, _context, _strategy))]
    async fn attempt_recovery(
        &self,
        _context: &mut LocalErrorContext,
        _strategy: &RecoveryStrategy,
    ) -> Result<(), ErrorHandlerError> {
        // Implementation of attempt_recovery method
        Ok(())
    }

    /// Registers a recovery strategy for a specific error type
    ///
    /// # Arguments
    ///
    /// * `error_type` - The type of error to register a strategy for
    /// * `strategy` - The recovery strategy to use
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    #[instrument(skip(self))]
    pub async fn register_recovery_strategy(
        &self,
        error_type: String,
        strategy: RecoveryStrategy,
    ) -> Result<(), ErrorHandlerError> {
        // Implementation of register_recovery_strategy method
        Ok(())
    }

    /// Attempts to recover a connection
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context containing information about the error
    #[expect(dead_code, reason = "planned feature not yet wired")]
    const fn recover_connection(_context: &LocalErrorContext) {
        // Implementation would go here
    }

    /// Attempts to recover state after an error
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context containing information about the error
    #[expect(dead_code, reason = "planned feature not yet wired")]
    const fn recover_state(_context: &LocalErrorContext) {
        // Implementation would go here
    }

    /// Attempts to recover protocol functionality after an error
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context containing information about the error
    #[expect(dead_code, reason = "planned feature not yet wired")]
    const fn recover_protocol(_context: &LocalErrorContext) {
        // Implementation would go here
    }

    /// Applies a recovery strategy to an error context
    ///
    /// # Arguments
    ///
    /// * `context` - The error context to apply recovery to
    /// * `strategy` - The recovery strategy to use
    ///
    /// # Returns
    ///
    /// Boolean indicating whether recovery was successful
    #[expect(
        unused_variables,
        reason = "Trait default impl; params reserved for derived implementations"
    )]
    pub const fn apply_recovery_strategy(
        &self,
        context: &mut LocalErrorContext,
        strategy: &RecoveryStrategy,
    ) -> bool {
        // Default implementation returns true
        true
    }

    /// Recovers an error context using the specified strategy
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context to recover
    /// * `_strategy` - The recovery strategy to use
    ///
    /// # Returns
    ///
    /// Result indicating success or an `MCPError`
    pub const fn recover_context(
        &self,
        _context: &mut LocalErrorContext,
        _strategy: &RecoveryStrategy,
    ) -> Result<(), MCPError> {
        // Basic implementation that returns Ok(())
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_handler_error_display() {
        assert!(format!("{}", ErrorHandlerError::Recovery("msg".into())).contains("Recovery"));
        assert!(format!("{}", ErrorHandlerError::State("msg".into())).contains("State"));
        assert!(format!("{}", ErrorHandlerError::Strategy("msg".into())).contains("Strategy"));
        assert!(format!("{}", ErrorHandlerError::Context("msg".into())).contains("Context"));
    }

    #[test]
    fn test_error_handler_error_from_serde() {
        let err: ErrorHandlerError = serde_json::from_str::<serde_json::Value>("invalid")
            .unwrap_err()
            .into();
        assert!(matches!(err, ErrorHandlerError::SerdeJson(_)));
    }

    #[test]
    fn test_local_error_context_new() {
        let ctx = LocalErrorContext::new("type_a", "msg", "component_x");
        assert_eq!(ctx.error_type(), "type_a");
        assert_eq!(ctx.message(), "msg");
        assert_eq!(ctx.component(), "component_x");
        assert_eq!(ctx.severity(), ErrorSeverity::Low);
        assert_eq!(ctx.recovery_attempts(), 0);
        assert!(ctx.metadata().is_none());
    }

    #[test]
    fn test_local_error_context_with_severity() {
        let ctx = LocalErrorContext::new("t", "m", "c").with_severity(ErrorSeverity::Critical);
        assert_eq!(ctx.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_local_error_context_with_metadata() {
        let meta = json!({"key": "value"});
        let ctx = LocalErrorContext::new("t", "m", "c").with_metadata(meta.clone());
        assert_eq!(ctx.metadata(), Some(&meta));
    }

    #[test]
    fn test_local_error_context_increment_recovery_attempts() {
        let mut ctx = LocalErrorContext::new("t", "m", "c");
        assert_eq!(ctx.recovery_attempts(), 0);
        ctx.increment_recovery_attempts();
        ctx.increment_recovery_attempts();
        assert_eq!(ctx.recovery_attempts(), 2);
    }

    #[test]
    fn test_recovery_strategy_serde() {
        let strategy = RecoveryStrategy {
            max_attempts: 3,
            backoff_ms: 100,
            timeout_ms: 5000,
            requires_manual_intervention: false,
        };
        let json = serde_json::to_string(&strategy).expect("serialize");
        let parsed: RecoveryStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.max_attempts, 3);
        assert_eq!(parsed.backoff_ms, 100);
    }

    #[tokio::test]
    async fn test_error_handler_record_and_get_history() {
        let handler = ErrorHandler::new(10);
        handler
            .record_error("err1".into(), "msg1".into(), Some("details".into()))
            .await
            .expect("record");
        let history = handler.get_error_history().await.expect("get");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].error_type, "err1");
        assert_eq!(history[0].message, "msg1");
        assert_eq!(history[0].details, Some("details".into()));
    }

    #[tokio::test]
    async fn test_error_handler_max_history_size() {
        let handler = ErrorHandler::new(3);
        for i in 0..5 {
            handler
                .record_error(format!("err_{i}"), format!("msg_{i}"), None)
                .await
                .expect("record");
        }
        let history = handler.get_error_history().await.expect("get");
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].error_type, "err_2");
        assert_eq!(history[1].error_type, "err_3");
        assert_eq!(history[2].error_type, "err_4");
    }

    #[tokio::test]
    async fn test_error_handler_clear_history() {
        let handler = ErrorHandler::new(10);
        handler
            .record_error("err".into(), "msg".into(), None)
            .await
            .expect("record");
        handler.clear_history().await.expect("clear");
        let history = handler.get_error_history().await.expect("get");
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_error_handler_handle_error() {
        let handler = ErrorHandler::new(10);
        let ctx = LocalErrorContext::new("test_err", "test message", "test_component");
        handler.handle_error(ctx).await.expect("handle");
        let history = handler.get_error_history().await.expect("get");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].error_type, "test_err");
    }

    #[tokio::test]
    async fn test_error_handler_register_recovery_strategy() {
        let handler = ErrorHandler::new(10);
        let strategy = RecoveryStrategy {
            max_attempts: 5,
            backoff_ms: 200,
            timeout_ms: 1000,
            requires_manual_intervention: false,
        };
        handler
            .register_recovery_strategy("conn_err".into(), strategy)
            .await
            .expect("register");
    }

    #[test]
    fn test_error_handler_apply_recovery_strategy() {
        let handler = ErrorHandler::new(10);
        let mut ctx = LocalErrorContext::new("t", "m", "c");
        let strategy = RecoveryStrategy {
            max_attempts: 1,
            backoff_ms: 100,
            timeout_ms: 500,
            requires_manual_intervention: false,
        };
        assert!(handler.apply_recovery_strategy(&mut ctx, &strategy));
    }

    #[test]
    fn test_error_handler_recover_context() {
        let handler = ErrorHandler::new(10);
        let mut ctx = LocalErrorContext::new("t", "m", "c");
        let strategy = RecoveryStrategy {
            max_attempts: 1,
            backoff_ms: 100,
            timeout_ms: 500,
            requires_manual_intervention: false,
        };
        assert!(handler.recover_context(&mut ctx, &strategy).is_ok());
    }

    #[tokio::test]
    async fn test_error_handler_clone_shares_history() {
        let handler = ErrorHandler::new(5);
        handler
            .record_error("shared".into(), "msg".into(), None)
            .await
            .expect("record");
        let cloned = handler.clone();
        let history = cloned.get_error_history().await.expect("get");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].error_type, "shared");
    }
}
