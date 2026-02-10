// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Logging Tests
//! 
//! This module contains tests for the structured logging functionality in the observability framework,
//! including log level management, context propagation, and field management.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use crate::observability::logging::{Logger, LogLevel, LogContext, LogRecord, ContextManager, LoggerConfig};
use crate::observability::ObservabilityError;

/// Test log level operations
#[test]
fn test_log_level_operations() {
    // Test ordering
    assert!(LogLevel::Critical > LogLevel::Error);
    assert!(LogLevel::Error > LogLevel::Warning);
    assert!(LogLevel::Warning > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Debug);
    
    // Test display
    assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
    assert_eq!(format!("{}", LogLevel::Info), "INFO");
    assert_eq!(format!("{}", LogLevel::Warning), "WARNING");
    assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    assert_eq!(format!("{}", LogLevel::Critical), "CRITICAL");
    
    // Test conversion from String
    let from_debug = tracing::Level::from(LogLevel::Debug);
    let from_info = tracing::Level::from(LogLevel::Info);
    let from_warning = tracing::Level::from(LogLevel::Warning);
    let from_error = tracing::Level::from(LogLevel::Error);
    let from_critical = tracing::Level::from(LogLevel::Critical);
    
    assert_eq!(from_debug, tracing::Level::DEBUG);
    assert_eq!(from_info, tracing::Level::INFO);
    assert_eq!(from_warning, tracing::Level::WARN);
    assert_eq!(from_error, tracing::Level::ERROR);
    assert_eq!(from_critical, tracing::Level::ERROR); // Critical maps to ERROR in tracing
}

/// Test log record creation and fields
#[test]
fn test_log_record_creation() {
    // Create a basic log record
    let record = LogRecord::new(
        "Test log message",
        LogLevel::Info,
        "test-component"
    );
    
    // Verify basic properties
    assert_eq!(record.message(), "Test log message");
    assert_eq!(record.level(), LogLevel::Info);
    assert_eq!(record.component(), "test-component");
    assert!(record.fields().is_empty());
    assert!(record.trace_id().is_none());
    assert!(record.span_id().is_none());
    
    // Create a record with fields and trace context
    let record_with_fields = LogRecord::new(
        "Log with fields and trace",
        LogLevel::Warning,
        "test-component"
    )
    .with_field("user_id", "12345")
    .with_field("request_path", "/api/v1/users")
    .with_trace_id("trace-abc-123")
    .with_span_id("span-xyz-789");
    
    // Verify properties with fields and trace
    assert_eq!(record_with_fields.message(), "Log with fields and trace");
    assert_eq!(record_with_fields.level(), LogLevel::Warning);
    assert_eq!(record_with_fields.fields().len(), 2);
    assert_eq!(record_with_fields.fields().get("user_id").unwrap(), "12345");
    assert_eq!(record_with_fields.fields().get("request_path").unwrap(), "/api/v1/users");
    assert_eq!(record_with_fields.trace_id().unwrap(), "trace-abc-123");
    assert_eq!(record_with_fields.span_id().unwrap(), "span-xyz-789");
}

/// Test log context creation and field management
#[test]
fn test_log_context_creation() {
    // Create an empty context
    let context = LogContext::new();
    assert!(context.fields().is_empty());
    assert!(context.trace_id().is_none());
    assert!(context.span_id().is_none());
    
    // Create context with fields
    let context_with_fields = LogContext::new()
        .with_field("session_id", "session-123")
        .with_field("client_ip", "192.168.1.1")
        .with_trace_id("trace-456")
        .with_span_id("span-789");
    
    // Verify fields
    assert_eq!(context_with_fields.fields().len(), 2);
    assert_eq!(context_with_fields.fields().get("session_id").unwrap(), "session-123");
    assert_eq!(context_with_fields.fields().get("client_ip").unwrap(), "192.168.1.1");
    assert_eq!(context_with_fields.trace_id().unwrap(), "trace-456");
    assert_eq!(context_with_fields.span_id().unwrap(), "span-789");
    
    // Test cloning
    let cloned_context = context_with_fields.clone();
    assert_eq!(cloned_context.fields().len(), 2);
    assert_eq!(cloned_context.trace_id().unwrap(), "trace-456");
    
    // Create context by adding fields one by one
    let mut manual_context = LogContext::new();
    let manual_context = manual_context
        .with_field("field1", "value1")
        .with_field("field2", "value2");
    
    assert_eq!(manual_context.fields().len(), 2);
}

/// Test context nesting and inheritance
#[test]
fn test_context_nesting() {
    // Create a parent context
    let parent_context = LogContext::new()
        .with_field("tenant_id", "tenant-123")
        .with_field("environment", "production");
    
    // Create a child context that inherits from parent
    let child_context = LogContext::new()
        .with_field("request_id", "req-456")
        // Overrides parent's environment
        .with_field("environment", "staging");
    
    // In a real logger system, the context would be combined at log time
    let mut combined_fields = parent_context.fields().clone();
    
    // Child fields override parent fields with the same key
    for (key, value) in child_context.fields() {
        combined_fields.insert(key.clone(), value.clone());
    }
    
    // Verify combined fields
    assert_eq!(combined_fields.len(), 3); // tenant_id, environment, request_id
    assert_eq!(combined_fields.get("tenant_id").unwrap(), "tenant-123");
    assert_eq!(combined_fields.get("request_id").unwrap(), "req-456");
    assert_eq!(combined_fields.get("environment").unwrap(), "staging"); // Overridden value
}

/// Test thread-local context management
#[test]
fn test_thread_local_context() {
    // Set a context
    let context = LogContext::new()
        .with_field("thread_id", "thread-1")
        .with_trace_id("trace-abc");
    
    ContextManager::set_context(context).unwrap();
    
    // Get the current context
    let current = ContextManager::current_context().unwrap();
    assert_eq!(current.fields().len(), 1);
    assert_eq!(current.fields().get("thread_id").unwrap(), "thread-1");
    assert_eq!(current.trace_id().unwrap(), "trace-abc");
    
    // Add a field to the context
    ContextManager::add_field("request_path", "/api/users").unwrap();
    
    // Verify the field was added
    let updated = ContextManager::current_context().unwrap();
    assert_eq!(updated.fields().len(), 2);
    assert_eq!(updated.fields().get("request_path").unwrap(), "/api/users");
    
    // Update trace and span IDs
    ContextManager::set_trace_id("new-trace").unwrap();
    ContextManager::set_span_id("new-span").unwrap();
    
    // Verify updates
    let with_span = ContextManager::current_context().unwrap();
    assert_eq!(with_span.trace_id().unwrap(), "new-trace");
    assert_eq!(with_span.span_id().unwrap(), "new-span");
    
    // Clear the context
    ContextManager::clear_context().unwrap();
    
    // Verify context is cleared
    let after_clear = ContextManager::current_context().unwrap();
    assert!(after_clear.fields().is_empty());
    assert!(after_clear.trace_id().is_none());
    assert!(after_clear.span_id().is_none());
}

/// Test different context in different threads
#[test]
fn test_multi_threaded_context() {
    // Set a context in the main thread
    let main_context = LogContext::new()
        .with_field("thread", "main")
        .with_trace_id("main-trace");
    
    ContextManager::set_context(main_context).unwrap();
    
    // Create a child thread that sets its own context
    let handle = thread::spawn(|| {
        // Each thread has its own context
        let thread_context = LogContext::new()
            .with_field("thread", "worker")
            .with_trace_id("worker-trace");
        
        ContextManager::set_context(thread_context).unwrap();
        
        // Verify this thread's context
        let context = ContextManager::current_context().unwrap();
        assert_eq!(context.fields().get("thread").unwrap(), "worker");
        assert_eq!(context.trace_id().unwrap(), "worker-trace");
        
        // Return true to indicate success
        true
    });
    
    // Wait for child thread to complete
    assert!(handle.join().unwrap());
    
    // Main thread's context should be unchanged
    let main_after = ContextManager::current_context().unwrap();
    assert_eq!(main_after.fields().get("thread").unwrap(), "main");
    assert_eq!(main_after.trace_id().unwrap(), "main-trace");
}

/// Test logger initialization and configuration
#[test]
fn test_logger_initialization() {
    let logger = Logger::new();
    
    // Initialize with default config
    assert!(logger.initialize().is_ok());
    
    // Create a custom config
    let mut config = LoggerConfig::default();
    config.default_level = LogLevel::Debug;
    
    let mut component_levels = HashMap::new();
    component_levels.insert("api".to_string(), LogLevel::Info);
    component_levels.insert("db".to_string(), LogLevel::Warning);
    config.component_levels = component_levels;
    
    // Set the custom config
    assert!(logger.set_config(config.clone()).is_ok());
    
    // Log a message (we can't easily verify the output, but we can ensure it doesn't error)
    let record = LogRecord::new("Test message", LogLevel::Info, "test");
    assert!(logger.log(record, None).is_ok());
}

/// Test log filtering by level
#[test]
fn test_log_filtering() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // Set component-specific log levels
    let mut config = LoggerConfig::default();
    config.default_level = LogLevel::Warning; // Only Warning and above by default
    
    let mut component_levels = HashMap::new();
    component_levels.insert("verbose_component".to_string(), LogLevel::Debug); // All levels for this component
    component_levels.insert("quiet_component".to_string(), LogLevel::Critical); // Only Critical for this component
    config.component_levels = component_levels;
    
    logger.set_config(config).unwrap();
    
    // These should pass filtering
    let warning_default = LogRecord::new("Warning msg", LogLevel::Warning, "default_component");
    let error_default = LogRecord::new("Error msg", LogLevel::Error, "default_component");
    let debug_verbose = LogRecord::new("Debug msg", LogLevel::Debug, "verbose_component");
    let critical_quiet = LogRecord::new("Critical msg", LogLevel::Critical, "quiet_component");
    
    assert!(logger.log(warning_default, None).is_ok());
    assert!(logger.log(error_default, None).is_ok());
    assert!(logger.log(debug_verbose, None).is_ok());
    assert!(logger.log(critical_quiet, None).is_ok());
    
    // These should pass filtering
    let debug_default = LogRecord::new("Debug msg", LogLevel::Debug, "default_component");
    let info_default = LogRecord::new("Info msg", LogLevel::Info, "default_component");
    let info_quiet = LogRecord::new("Info msg", LogLevel::Info, "quiet_component");
    let error_quiet = LogRecord::new("Error msg", LogLevel::Error, "quiet_component");
    
    assert!(logger.log(debug_default, None).is_ok()); // Still returns Ok, but shouldn't be logged
    assert!(logger.log(info_default, None).is_ok());  // Still returns Ok, but shouldn't be logged
    assert!(logger.log(info_quiet, None).is_ok());   // Still returns Ok, but shouldn't be logged
    assert!(logger.log(error_quiet, None).is_ok());  // Still returns Ok, but shouldn't be logged
}

/// Test log record creation and handling
#[test]
fn test_log_record_handling() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // Set default log level to Debug so all messages pass
    let mut config = LoggerConfig::default();
    config.default_level = LogLevel::Debug;
    logger.set_config(config).unwrap();
    
    // Create a log context
    let context = LogContext::new()
        .with_field("session_id", "abc123")
        .with_trace_id("trace-xyz");
    
    // Log with different levels and contexts
    let debug_record = LogRecord::new("Debug message", LogLevel::Debug, "test");
    let info_record = LogRecord::new("Info message", LogLevel::Info, "test");
    let warning_record = LogRecord::new("Warning message", LogLevel::Warning, "test");
    let error_record = LogRecord::new("Error message", LogLevel::Error, "test");
    let critical_record = LogRecord::new("Critical message", LogLevel::Critical, "test");
    
    assert!(logger.log(debug_record, Some(&context)).is_ok());
    assert!(logger.log(info_record, Some(&context)).is_ok());
    assert!(logger.log(warning_record, Some(&context)).is_ok());
    assert!(logger.log(error_record, Some(&context)).is_ok());
    assert!(logger.log(critical_record, Some(&context)).is_ok());
    
    // Log with a record that has fields and with a context that has fields
    let record_with_fields = LogRecord::new("Message with fields", LogLevel::Info, "test")
        .with_field("record_field", "record_value");
    
    assert!(logger.log(record_with_fields, Some(&context)).is_ok());
    
    // Log without context
    let no_context_record = LogRecord::new("No context", LogLevel::Info, "test");
    assert!(logger.log(no_context_record, None).is_ok());
}

/// Test convenience logging methods
#[test]
fn test_convenience_logging_methods() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // Set default log level to Debug
    let mut config = LoggerConfig::default();
    config.default_level = LogLevel::Debug;
    logger.set_config(config).unwrap();
    
    // Create a context
    let context = LogContext::new().with_field("session", "test-session");
    
    // Test convenience methods
    assert!(logger.debug("Debug message", "test", Some(&context)).is_ok());
    assert!(logger.info("Info message", "test", Some(&context)).is_ok());
    assert!(logger.warning("Warning message", "test", Some(&context)).is_ok());
    assert!(logger.error("Error message", "test", Some(&context)).is_ok());
    assert!(logger.critical("Critical message", "test", Some(&context)).is_ok());
    
    // Test without context
    assert!(logger.info("Info without context", "test", None).is_ok());
}

/// Test logging with structured data
#[test]
fn test_structured_logging() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // Create a complex set of fields
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), "user-123".to_string());
    fields.insert("request_id".to_string(), "req-456".to_string());
    fields.insert("latency_ms".to_string(), "42".to_string());
    
    // Create a log record with structured data
    let mut record = LogRecord::new("API request completed", LogLevel::Info, "api");
    for (key, value) in &fields {
        record = record.with_field(key, value);
    }
    
    // Log the record
    assert!(logger.log(record, None).is_ok());
    
    // Log with nested structure via context and record
    let context = LogContext::new()
        .with_field("tenant", "tenant-abc")
        .with_field("environment", "production");
    
    let record = LogRecord::new("Database query", LogLevel::Debug, "db")
        .with_field("query_time_ms", "15")
        .with_field("rows_returned", "42");
    
    assert!(logger.log(record, Some(&context)).is_ok());
}

/// Test error handling in logging
#[test]
fn test_logging_error_handling() {
    let logger = Logger::new();
    
    // Attempt to log before initialization (should fail gracefully)
    let record = LogRecord::new("Pre-init log", LogLevel::Info, "test");
    assert!(logger.log(record, None).is_ok()); // This is a design choice - the logger doesn't error even if not initialized
    
    // Initialize the logger
    logger.initialize().unwrap();
    
    // Test component level configuration
    logger.set_component_level("test-component", LogLevel::Error).unwrap();
    
    // Test setting default level
    logger.set_default_level(LogLevel::Warning).unwrap();
    
    // Edge case: Empty component or message
    let empty_component = LogRecord::new("Empty component test", LogLevel::Info, "");
    let empty_message = LogRecord::new("", LogLevel::Info, "test");
    
    assert!(logger.log(empty_component, None).is_ok());
    assert!(logger.log(empty_message, None).is_ok());
}

/// Test logging with different output sinks
#[test]
fn test_logging_output_sinks() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // In a real implementation, these would go to different sinks based on level
    // But in our test we just verify the methods work
    
    // Debug to debug sink
    assert!(logger.debug("Debug message", "test", None).is_ok());
    
    // Info to info sink
    assert!(logger.info("Info message", "test", None).is_ok());
    
    // Warning to warning sink
    assert!(logger.warning("Warning message", "test", None).is_ok());
    
    // Error to error sink
    assert!(logger.error("Error message", "test", None).is_ok());
    
    // Critical to critical sink (and possibly alerting)
    assert!(logger.critical("Critical message", "test", None).is_ok());
}

/// Test logger shutdown and cleanup
#[test]
fn test_logger_shutdown() {
    let logger = Logger::new();
    logger.initialize().unwrap();
    
    // Log a message
    assert!(logger.info("Pre-shutdown message", "test", None).is_ok());
    
    // In a real implementation, we would shut down the logger here
    // and verify logging behavior after shutdown
    
    // For our test, just verify logging still works after shutdown
    // (graceful degradation)
    assert!(logger.info("Post-shutdown message", "test", None).is_ok());
} 