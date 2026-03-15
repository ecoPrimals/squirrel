// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Command System Observability
//!
//! This module provides comprehensive observability features for the Command System, including distributed tracing and metrics collection.
//! These capabilities enable detailed monitoring of command execution performance, error tracking, and system behavior analysis.
//!
//! ## Key Features
//!
//! ### 1. Distributed Tracing
//!
//! - **Trace Context Management**: Track command execution across the entire system
//! - **Span Creation**: Create hierarchical spans for command operations
//! - **Attribute Recording**: Capture detailed contextual information
//! - **Error Tracing**: Correlate errors with command execution
//! - **Trace Propagation**: Maintain trace context across async boundaries
//!
//! ### 2. Performance Metrics
//!
//! - **Execution Timing**: Measure command execution durations
//! - **Success/Failure Rates**: Track command reliability
//! - **Resource Usage**: Monitor system resources during execution
//! - **Hook Performance**: Measure performance of command hooks
//! - **Statistical Analysis**: Calculate min/max/avg performance metrics
//!
//! ### 3. Observability Hook
//!
//! - **Lifecycle Integration**: Automatic tracing at key execution points
//! - **Non-intrusive Design**: Add observability without changing existing code
//! - **Structured Logging**: Consistent log format with correlation IDs
//! - **Performance Impact**: Minimal overhead for production systems
//!
//! ## Usage Example
//!
//! ```
//! // Use tracing for structured logging
//! use tracing::{info, debug, error};
//!
//! // Log command execution
//! info!(command = "example", "Executing command");
//!
//! // Log performance metrics
//! debug!(execution_time_ms = 42, "Command execution completed");
//!
//! // Log errors
//! error!(error = "Connection failed", "Command execution failed");
//! ```

use thiserror::Error;
use tracing::{debug, error, info};

use crate::registry::CommandResult;

/// Error types specific to observability operations
#[derive(Debug, Error, Clone)]
pub enum ObservabilityError {
    /// Error during metrics collection
    #[error("Metrics error: {0}")]
    MetricsError(String),

    /// Error during tracing
    #[error("Tracing error: {0}")]
    TracingError(String),

    /// Error related to the observability configuration
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Type alias for observability result
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

/// Logs command execution with structured data
pub fn log_command_execution(
    command_name: &str,
    args: &[String],
    result: &CommandResult<String>,
    execution_time_ms: u64,
) {
    match result {
        Ok(_) => {
            info!(
                command = %command_name,
                args = ?args,
                execution_time_ms = %execution_time_ms,
                "Command executed successfully"
            );
        }
        Err(e) => {
            error!(
                command = %command_name,
                args = ?args,
                execution_time_ms = %execution_time_ms,
                error = %e,
                "Command execution failed"
            );
        }
    }
}

/// Records resource usage metrics
pub fn record_resource_usage(command_name: &str, memory_kb: u64, cpu_percent: f64) {
    debug!(
        command = %command_name,
        memory_kb = %memory_kb,
        cpu_percent = %cpu_percent,
        "Resource usage recorded"
    );
}

/// A simplified placeholder for the more complete observability system
/// This minimal implementation allows the demo to run without compatibility issues
pub struct ObservabilitySystem;

impl ObservabilitySystem {
    /// Creates a new observability system
    pub fn new() -> Self {
        info!("Observability system initialized");
        Self
    }

    /// Logs command execution
    pub fn log_command(&self, command_name: &str, args: &[String], result: &CommandResult<String>) {
        match result {
            Ok(_) => {
                info!(
                    command = %command_name,
                    args = ?args,
                    "Command executed successfully"
                );
            }
            Err(e) => {
                error!(
                    command = %command_name,
                    args = ?args,
                    error = %e,
                    "Command execution failed"
                );
            }
        }
    }
}

impl Default for ObservabilitySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_error_display() {
        let err = ObservabilityError::MetricsError("bad metric".to_string());
        assert_eq!(err.to_string(), "Metrics error: bad metric");

        let err = ObservabilityError::TracingError("trace fail".to_string());
        assert_eq!(err.to_string(), "Tracing error: trace fail");

        let err = ObservabilityError::ConfigurationError("bad config".to_string());
        assert_eq!(err.to_string(), "Configuration error: bad config");
    }

    #[test]
    fn test_observability_system_new() {
        let _sys = ObservabilitySystem::new();
    }

    #[test]
    fn test_observability_system_default() {
        let _sys = ObservabilitySystem::default();
    }

    #[test]
    fn test_log_command_execution_success() {
        let result: CommandResult<String> = Ok("done".to_string());
        // Should not panic
        log_command_execution("test_cmd", &["arg1".to_string()], &result, 42);
    }

    #[test]
    fn test_log_command_execution_failure() {
        let result: CommandResult<String> = Err(crate::error::CommandError::ExecutionError(
            "fail".to_string(),
        ));
        // Should not panic
        log_command_execution("test_cmd", &[], &result, 100);
    }

    #[test]
    fn test_record_resource_usage() {
        // Should not panic
        record_resource_usage("test_cmd", 1024, 50.0);
    }

    #[test]
    fn test_observability_system_log_command() {
        let sys = ObservabilitySystem::new();
        let result: CommandResult<String> = Ok("ok".to_string());
        sys.log_command("test", &[], &result);

        let result: CommandResult<String> = Err(crate::error::CommandError::InputError(
            "missing".to_string(),
        ));
        sys.log_command("test", &[], &result);
    }
}
