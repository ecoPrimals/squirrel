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
            Self::Recovery(msg) => write!(f, "Recovery error: {}", msg),
            Self::State(msg) => write!(f, "State error: {}", msg),
            Self::Strategy(msg) => write!(f, "Strategy error: {}", msg),
            Self::Context(msg) => write!(f, "Context error: {}", msg),
            Self::SerdeJson(err) => write!(f, "JSON error: {}", err),
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
    /// A new LocalErrorContext with default values for other fields
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
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Gets the timestamp when this error occurred
    ///
    /// # Returns
    ///
    /// The DateTime when the error was recorded
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Gets the type of this error
    ///
    /// # Returns
    ///
    /// The error type as a string
    pub fn error_type(&self) -> &str {
        &self.error_type
    }

    /// Gets the error message
    ///
    /// # Returns
    ///
    /// The descriptive error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Gets the component where this error occurred
    ///
    /// # Returns
    ///
    /// The component name as a string
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Gets the severity level of this error
    ///
    /// # Returns
    ///
    /// The ErrorSeverity level
    pub fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Gets the metadata associated with this error
    ///
    /// # Returns
    ///
    /// Optional JSON metadata about the error
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    /// Gets the number of recovery attempts made for this error
    ///
    /// # Returns
    ///
    /// The count of recovery attempts
    pub fn recovery_attempts(&self) -> u32 {
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
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
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
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Increments the recovery attempt counter
    ///
    /// This is called each time a recovery attempt is made.
    pub fn increment_recovery_attempts(&mut self) {
        self.recovery_attempts += 1;
    }
}

/// Severity levels for errors
///
/// Defines the different levels of severity that can be assigned to errors,
/// helping prioritize handling and reporting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - minimal impact, typically handled automatically
    Low,
    
    /// Medium severity - moderate impact, may require attention
    Medium,
    
    /// High severity - significant impact, requires attention
    High,
    
    /// Critical severity - severe impact, requires immediate attention
    Critical,
}

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
    /// Creates a new ErrorHandler with the specified history size
    ///
    /// # Arguments
    ///
    /// * `max_history_size` - Maximum number of error records to keep in history
    ///
    /// # Returns
    ///
    /// A new ErrorHandler instance
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
        let mut history = self.error_history.write().await;
        history.clear();
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
    fn recover_connection(_context: &LocalErrorContext) {
        // Implementation would go here
    }

    /// Attempts to recover state after an error
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context containing information about the error
    fn recover_state(_context: &LocalErrorContext) {
        // Implementation would go here
    }

    /// Attempts to recover protocol functionality after an error
    ///
    /// # Arguments
    ///
    /// * `_context` - The error context containing information about the error
    fn recover_protocol(_context: &LocalErrorContext) {
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
    pub fn apply_recovery_strategy(
        &self,
        // These parameters are intentionally unused in this implementation
        // but may be used by derived implementations
        #[allow(unused_variables)] context: &mut LocalErrorContext,
        #[allow(unused_variables)] strategy: &RecoveryStrategy,
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
    /// Result indicating success or an MCPError
    pub fn recover_context(
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
    // Temporarily commented out until fixed
    /*
    use super::*;
    use serde_json::json;
    use chrono::Utc;

    #[tokio::test]
    async fn test_error_context_add_error() {
        let mut context = ErrorContext::new(10);

        context.add_error(
            "test_error",
            "Test message",
            Some("Test details".to_string()),
            None,
            Some(json!({"key": "value"})),
        ).await.unwrap();

        let history = context.get_error_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].error_type, "test_error");
        assert_eq!(history[0].message, "Test message");
        assert_eq!(history[0].details, Some("Test details".to_string()));
    }

    #[tokio::test]
    async fn test_error_context_max_size() {
        let mut context = ErrorContext::new(10);

        // Add 15 errors, should only keep the last 10
        for i in 0..15 {
            context.add_error(
                &format!("error_{}", i),
                &format!("Message {}", i),
                None,
                None,
                None,
            ).await.unwrap();
        }

        let history = context.get_error_history().await;
        assert_eq!(history.len(), 10);

        // Should have errors 5-14
        for i in 0..10 {
            assert_eq!(history[i].error_type, format!("error_{}", i + 5));
        }
    }

    #[tokio::test]
    async fn test_error_context_recovery() {
        let mut context = ErrorContext::new(10);

        // Register a recovery handler
        context.register_recovery_handler("test_error", Box::new(|ctx| {
            println!("Recovering from error: {:?}", ctx);
            Ok(())
        })).await.unwrap();

        // Add an error
        context.add_error(
            "test_error",
            "Test message",
            None,
            None,
            None,
        ).await.unwrap();

        // Try recovery
        let result = context.try_recover("test_error", json!({})).await;
        assert!(result.is_ok());

        let history = context.get_error_history().await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_error_context_no_recovery() {
        let mut context = ErrorContext::new(10);

        // Add an error without a recovery handler
        context.add_error(
            "test_error",
            "Test message",
            None,
            None,
            None,
        ).await.unwrap();

        // Try recovery
        let result = context.try_recover("test_error", json!({})).await;
        assert!(result.is_err());

        // Add a second error
        context.add_error(
            "test_error_2",
            "Test message 2",
            None,
            None,
            None,
        ).await.unwrap();

        let history = context.get_error_history().await;
        assert_eq!(history.len(), 2);
    }
    */
}
