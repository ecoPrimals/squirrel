// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Comprehensive Observability Framework
//!
//! This module provides standardized observability utilities for the entire ecosystem,
//! including structured logging, correlation IDs, performance metrics, and distributed tracing.
//!
//! ## Key Features
//!
//! - **Correlation ID Management**: Consistent request tracking across services
//! - **Performance Metrics**: Standardized timing and performance measurement
//! - **Structured Logging**: Consistent log formatting with metadata
//! - **Operation Tracking**: Start-to-finish operation monitoring
//! - **Error Context**: Rich error information with diagnostic data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// correlation, metrics, tracing_utils removed - HTTP-based observability utilities

/// Correlation ID for tracking requests across services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Generate a new correlation ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from existing string
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total operation duration
    pub total_duration: Duration,
    /// Individual phase durations
    pub phase_durations: HashMap<String, Duration>,
    /// Attempt count (for retries)
    pub attempts: u32,
    /// Success indicator
    pub success: bool,
    /// Error information if failed
    pub error_info: Option<String>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_duration: Duration::ZERO,
            phase_durations: HashMap::new(),
            attempts: 0,
            success: false,
            error_info: None,
        }
    }

    /// Record a phase duration
    pub fn record_phase(&mut self, phase: impl Into<String>, duration: Duration) {
        self.phase_durations.insert(phase.into(), duration);
    }

    /// Mark as successful
    pub const fn mark_success(&mut self, total_duration: Duration) {
        self.success = true;
        self.total_duration = total_duration;
    }

    /// Mark as failed
    pub fn mark_failure(&mut self, total_duration: Duration, error: impl Into<String>) {
        self.success = false;
        self.total_duration = total_duration;
        self.error_info = Some(error.into());
    }

    /// Increment attempt count
    pub const fn increment_attempts(&mut self) {
        self.attempts += 1;
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation context for tracking
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// Correlation ID for the operation
    pub correlation_id: CorrelationId,
    /// Operation name/type
    pub operation: String,
    /// Start time
    pub start_time: Instant,
    /// Metadata associated with the operation
    pub metadata: HashMap<String, String>,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

impl OperationContext {
    /// Create a new operation context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            correlation_id: CorrelationId::new(),
            operation: operation.into(),
            start_time: Instant::now(),
            metadata: HashMap::new(),
            metrics: PerformanceMetrics::new(),
        }
    }

    /// Create with existing correlation ID
    pub fn with_correlation_id(
        operation: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            correlation_id,
            operation: operation.into(),
            start_time: Instant::now(),
            metadata: HashMap::new(),
            metrics: PerformanceMetrics::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries
    #[must_use]
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Get elapsed time since operation start
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Log operation start
    pub fn log_start(&self) {
        info!(
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            metadata = ?self.metadata,
            "Operation started"
        );
    }

    /// Log operation success
    pub fn log_success(&self) {
        let duration = self.elapsed();
        info!(
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            metadata = ?self.metadata,
            "Operation completed successfully"
        );
    }

    /// Log operation attempt
    pub fn log_attempt(&self, attempt: u32, max_attempts: u32) {
        debug!(
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            attempt = attempt,
            max_attempts = max_attempts,
            duration_ms = self.elapsed().as_millis(),
            "Operation attempt"
        );
    }

    /// Log operation retry
    pub fn log_retry(&self, attempt: u32, delay: Duration, reason: &str) {
        warn!(
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            attempt = attempt,
            delay_ms = delay.as_millis(),
            reason = reason,
            "Operation retry scheduled"
        );
    }

    /// Log operation failure
    pub fn log_failure(&self, error: &str) {
        let duration = self.elapsed();
        error!(
            correlation_id = %self.correlation_id,
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            error = error,
            metadata = ?self.metadata,
            "Operation failed"
        );
    }

    /// Complete the operation with success
    #[must_use]
    pub fn complete_success(mut self) -> OperationResult {
        let duration = self.elapsed();
        self.metrics.mark_success(duration);
        self.log_success();

        OperationResult {
            correlation_id: self.correlation_id,
            operation: self.operation,
            metrics: self.metrics,
            metadata: self.metadata,
        }
    }

    /// Complete the operation with failure
    pub fn complete_failure(mut self, error: impl Into<String>) -> OperationResult {
        let duration = self.elapsed();
        let error_str = error.into();
        self.metrics.mark_failure(duration, error_str.clone());
        self.log_failure(&error_str);

        OperationResult {
            correlation_id: self.correlation_id,
            operation: self.operation,
            metrics: self.metrics,
            metadata: self.metadata,
        }
    }
}

/// Result of a completed operation
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Operation name
    pub operation: String,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl OperationResult {
    /// Check if operation was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.metrics.success
    }

    /// Get total duration
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.metrics.total_duration
    }

    /// Get error information if failed
    #[must_use]
    pub fn error_info(&self) -> Option<&str> {
        self.metrics.error_info.as_deref()
    }
}

/// Macro for creating and managing operation contexts
#[macro_export]
macro_rules! observe_operation {
    ($operation:expr) => {
        $crate::observability::OperationContext::new($operation)
    };

    ($operation:expr, $correlation_id:expr) => {
        $crate::observability::OperationContext::with_correlation_id($operation, $correlation_id)
    };
}

/// Macro for structured logging with correlation
#[macro_export]
macro_rules! log_with_correlation {
    (info, $ctx:expr, $msg:expr $(, $($arg:tt)*)?) => {
        tracing::info!(
            correlation_id = %$ctx.correlation_id,
            operation = %$ctx.operation,
            duration_ms = $ctx.elapsed().as_millis(),
            $msg $(, $($arg)*)?
        );
    };

    (warn, $ctx:expr, $msg:expr $(, $($arg:tt)*)?) => {
        tracing::warn!(
            correlation_id = %$ctx.correlation_id,
            operation = %$ctx.operation,
            duration_ms = $ctx.elapsed().as_millis(),
            $msg $(, $($arg)*)?
        );
    };

    (error, $ctx:expr, $msg:expr $(, $($arg:tt)*)?) => {
        tracing::error!(
            correlation_id = %$ctx.correlation_id,
            operation = %$ctx.operation,
            duration_ms = $ctx.elapsed().as_millis(),
            $msg $(, $($arg)*)?
        );
    };

    (debug, $ctx:expr, $msg:expr $(, $($arg:tt)*)?) => {
        tracing::debug!(
            correlation_id = %$ctx.correlation_id,
            operation = %$ctx.operation,
            duration_ms = $ctx.elapsed().as_millis(),
            $msg $(, $($arg)*)?
        );
    };
}

#[cfg(test)]
mod error_path_tests;

/// Utility functions for observability
pub mod utils {
    use super::HashMap;

    /// Create a standardized metadata map for service calls
    #[must_use]
    pub fn service_call_metadata(
        service_name: &str,
        endpoint: &str,
        method: &str,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("service_name".to_string(), service_name.to_string());
        metadata.insert("endpoint".to_string(), endpoint.to_string());
        metadata.insert("method".to_string(), method.to_string());
        metadata
    }

    /// Create metadata for API operations
    #[must_use]
    pub fn api_operation_metadata(
        operation: &str,
        resource: &str,
        version: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("api_operation".to_string(), operation.to_string());
        metadata.insert("resource".to_string(), resource.to_string());
        if let Some(v) = version {
            metadata.insert("api_version".to_string(), v.to_string());
        }
        metadata
    }

    /// Create metadata for database operations
    #[must_use]
    pub fn database_operation_metadata(
        operation: &str,
        table: &str,
        query_type: &str,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("db_operation".to_string(), operation.to_string());
        metadata.insert("table".to_string(), table.to_string());
        metadata.insert("query_type".to_string(), query_type.to_string());
        metadata
    }
}

#[cfg(test)]
mod observability_more_tests {
    use super::utils;
    use super::{CorrelationId, OperationContext, OperationResult, PerformanceMetrics};
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn utils_metadata_helpers_cover_paths() {
        let m1 = utils::service_call_metadata("svc", "/rpc", "POST");
        assert_eq!(m1.get("method"), Some(&"POST".to_string()));
        let m2 = utils::api_operation_metadata("get", "ctx", None);
        assert!(!m2.contains_key("api_version"));
        let m3 = utils::api_operation_metadata("get", "ctx", Some("2"));
        assert_eq!(m3.get("api_version"), Some(&"2".to_string()));
        let m4 = utils::database_operation_metadata("select", "users", "read");
        assert_eq!(m4.get("query_type"), Some(&"read".to_string()));
    }

    #[test]
    fn operation_context_with_metadata_map_merges() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), "b".to_string());
        let ctx = OperationContext::new("op").with_metadata_map(m);
        assert_eq!(ctx.metadata.get("a"), Some(&"b".to_string()));
    }

    #[test]
    fn operation_result_accessors() {
        let mut pm = PerformanceMetrics::new();
        pm.mark_success(Duration::from_nanos(100));
        let res = OperationResult {
            correlation_id: CorrelationId::from_string("c"),
            operation: "x".to_string(),
            metrics: pm,
            metadata: HashMap::new(),
        };
        assert!(res.is_success());
        assert_eq!(res.duration(), Duration::from_nanos(100));
        assert!(res.error_info().is_none());
    }

    #[test]
    fn operation_context_logging_smoke() {
        let ctx = OperationContext::new("logged").with_metadata("k", "v");
        ctx.log_start();
        ctx.log_attempt(1, 3);
        ctx.log_retry(2, Duration::from_millis(1), "backoff");
        ctx.log_success();
        ctx.log_failure("failed");
    }
}
