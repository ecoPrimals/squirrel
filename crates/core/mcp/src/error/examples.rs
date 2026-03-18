// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error Context Trait Examples
//!
//! This module demonstrates how to implement and use the ErrorContextTrait
//! for consistent error handling across the codebase.

use std::collections::HashMap;
use tracing::{debug, error, warn};

use super::context_trait::{ErrorContextTrait, WithContext};
use super::types::{ErrorContext, ErrorSeverity};

/// Example: Implementing `ErrorContextTrait` for a custom error type
///
/// This shows how to provide full context information for an error.
#[derive(Debug)]
pub struct ServiceError {
    message: String,
    context: ErrorContext,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServiceError {}

impl ErrorContextTrait for ServiceError {
    fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        Some(self.context.timestamp)
    }

    fn operation(&self) -> Option<&str> {
        Some(&self.context.operation)
    }

    fn component(&self) -> Option<&str> {
        Some(&self.context.component)
    }

    fn severity(&self) -> ErrorSeverity {
        self.context.severity
    }

    fn is_recoverable(&self) -> bool {
        self.context.is_recoverable
    }

    fn get_context(&self) -> Option<&ErrorContext> {
        Some(&self.context)
    }
}

/// Example: Using error context in logging
///
/// This demonstrates how to extract context information for structured logging.
pub fn log_error_with_context(error: &dyn ErrorContextTrait) {
    let log_entry = error.to_log_entry();

    eprintln!("Error occurred:");
    for (key, value) in log_entry {
        eprintln!("  {key}: {value}");
    }

    if error.should_alert() {
        eprintln!("  ⚠️  Alert triggered!");
    }
}

/// Example: Error handling with context propagation
///
/// This shows how to add context as errors propagate up the call stack.
pub mod context_propagation {
    use super::{ErrorContextTrait, ErrorSeverity, WithContext};

    /// Example database error type for demonstrating error context propagation.
    #[derive(Debug, Clone)]
    pub struct DatabaseError {
        message: String,
        severity: ErrorSeverity,
    }

    impl std::fmt::Display for DatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Database error: {}", self.message)
        }
    }

    impl std::error::Error for DatabaseError {}

    impl ErrorContextTrait for DatabaseError {
        fn severity(&self) -> ErrorSeverity {
            self.severity
        }
    }

    impl WithContext for DatabaseError {
        fn with_context(self, operation: &str, component: &str) -> Self {
            // In a real implementation, you'd add this context to the error
            eprintln!("Adding context: {operation} in {component}");
            self
        }

        fn with_severity(mut self, severity: ErrorSeverity) -> Self {
            self.severity = severity;
            self
        }
    }

    /// Attempts to save data to the database. Returns error for demonstration.
    pub fn save_to_database(data: &str) -> Result<(), DatabaseError> {
        // Simulate database error
        Err(DatabaseError {
            message: format!("Failed to save: {data}"),
            severity: ErrorSeverity::High,
        })
    }

    /// Processes a request by saving to database, propagating context on error.
    pub fn process_request(data: &str) -> Result<(), DatabaseError> {
        // Add context as error propagates
        let result = save_to_database(data);
        match result {
            Ok(val) => Ok(val),
            Err(e) => Err(e.with_context("saving user data", "request_processor")),
        }
    }
}

/// Example: Pattern matching with error context
///
/// This demonstrates how to handle different error severities.
pub fn handle_error_by_severity(error: &dyn ErrorContextTrait) {
    match error.severity() {
        ErrorSeverity::Low => {
            debug!("Low severity error: {:?}", error.to_log_entry());
        }
        ErrorSeverity::Medium => {
            warn!("Medium severity error: {:?}", error.to_log_entry());
        }
        ErrorSeverity::High => {
            error!("High severity error: {:?}", error.to_log_entry());
            // Trigger monitoring alert
        }
        ErrorSeverity::Critical => {
            error!("CRITICAL error: {:?}", error.to_log_entry());
            // Trigger immediate alert and possibly initiate recovery
        }
    }
}

/// Example: Collecting error statistics
///
/// This shows how to use error context for monitoring and analytics.
#[derive(Default)]
pub struct ErrorStats {
    /// Total number of errors recorded
    pub total_errors: usize,
    /// Count of errors grouped by severity level
    pub errors_by_severity: HashMap<String, usize>,
    /// Count of errors grouped by component
    pub errors_by_component: HashMap<String, usize>,
    /// Number of errors that were recoverable
    pub recoverable_count: usize,
}

impl ErrorStats {
    /// Create new empty error stats
    pub fn new() -> Self {
        Self::default()
    }
    /// Record an error for analytics
    pub fn record_error(&mut self, error: &dyn ErrorContextTrait) {
        self.total_errors += 1;

        // Count by severity
        let severity_key = format!("{:?}", error.severity());
        *self.errors_by_severity.entry(severity_key).or_insert(0) += 1;

        // Count by component
        if let Some(component) = error.component() {
            *self
                .errors_by_component
                .entry(component.to_string())
                .or_insert(0) += 1;
        }

        // Count recoverable errors
        if error.is_recoverable() {
            self.recoverable_count += 1;
        }
    }

    /// Returns the percentage of errors that were critical severity.
    #[expect(clippy::cast_precision_loss, reason = "Example percentage calculation")]
    pub fn get_critical_rate(&self) -> f64 {
        if self.total_errors == 0 {
            return 0.0;
        }

        let critical = self
            .errors_by_severity
            .get(&format!("{:?}", ErrorSeverity::Critical))
            .copied()
            .unwrap_or(0);

        (critical as f64 / self.total_errors as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_error_with_full_context() {
        let error = ServiceError {
            message: "Test error".to_string(),
            context: ErrorContext {
                timestamp: Utc::now(),
                operation: "test_operation".to_string(),
                component: "test_component".to_string(),
                severity: ErrorSeverity::High,
                error_code: "TEST_ERR".to_string(),
                is_recoverable: false,
                retry_count: 0,
                source_location: None,
                details: serde_json::Map::default(),
                message_type: None,
                security_level: None,
            },
        };

        assert_eq!(error.operation(), Some("test_operation"));
        assert_eq!(error.component(), Some("test_component"));
        assert_eq!(error.severity(), ErrorSeverity::High);
        assert!(!error.is_recoverable());
        assert!(error.should_alert());
    }

    #[test]
    fn test_error_stats_collection() {
        let mut stats = ErrorStats::new();

        let error1 = ServiceError {
            message: "Error 1".to_string(),
            context: ErrorContext {
                timestamp: Utc::now(),
                operation: "op1".to_string(),
                component: "comp1".to_string(),
                severity: ErrorSeverity::Critical,
                error_code: "ERR1".to_string(),
                is_recoverable: false,
                retry_count: 0,
                source_location: None,
                details: serde_json::Map::default(),
                message_type: None,
                security_level: None,
            },
        };

        let error2 = ServiceError {
            message: "Error 2".to_string(),
            context: ErrorContext {
                timestamp: Utc::now(),
                operation: "op2".to_string(),
                component: "comp2".to_string(),
                severity: ErrorSeverity::Low,
                error_code: "ERR2".to_string(),
                is_recoverable: true,
                retry_count: 0,
                source_location: None,
                details: serde_json::Map::default(),
                message_type: None,
                security_level: None,
            },
        };

        stats.record_error(&error1);
        stats.record_error(&error2);

        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.recoverable_count, 1);
        assert!(stats.get_critical_rate() > 0.0);
    }
}
