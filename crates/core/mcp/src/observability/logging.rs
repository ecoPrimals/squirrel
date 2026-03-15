// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Structured Logging
//! 
//! This module provides structured logging capabilities for the MCP,
//! enabling detailed, searchable, and contextual log messages.
//!
//! ## Key Components
//!
//! - **LogLevel**: Log severity levels
//! - **LogRecord**: A structured log entry
//! - **LogContext**: Contextual information to include in logs
//! - **Logger**: Interface for logging messages

use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;
use std::time::Instant;
use tracing::Level;
use crate::observability::{ObservabilityError, ObservabilityResult};

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error messages
    Error,
    /// Warning messages
    Warning,
    /// Info messages
    Info,
    /// Debug messages
    Debug,
    /// Trace messages
    Trace,
    /// Critical error messages
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warning => Level::WARN,
            LogLevel::Error => Level::ERROR,
            LogLevel::Critical => Level::ERROR,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

/// A structured log record
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Log message
    message: String,
    /// Log level
    level: LogLevel,
    /// Component that generated the log
    component: String,
    /// When the log was created
    timestamp: Instant,
    /// Additional fields for the log record
    fields: HashMap<String, String>,
    /// Trace ID if part of a trace
    trace_id: Option<String>,
    /// Span ID if part of a span
    span_id: Option<String>,
}

impl LogRecord {
    /// Create a new log record
    pub fn new(
        message: impl Into<String>,
        level: LogLevel,
        component: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            level,
            component: component.into(),
            timestamp: Instant::now(),
            fields: HashMap::new(),
            trace_id: None,
            span_id: None,
        }
    }

    /// Add a field to the log record
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add a trace ID to the log record
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Add a span ID to the log record
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Get the log message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the log level
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Get the component
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Get all fields
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Get the span ID
    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }
}

/// Configuration for the logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Default log level
    pub default_level: LogLevel,
    /// Component-specific log levels
    pub component_levels: HashMap<String, LogLevel>,
    /// Whether to include trace context
    pub include_trace_context: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            default_level: LogLevel::Info,
            component_levels: HashMap::new(),
            include_trace_context: true,
        }
    }
}

/// Thread-local logging context
#[derive(Debug, Clone, Default)]
pub struct LogContext {
    /// Fields to include in all logs
    fields: HashMap<String, String>,
    /// Trace ID for the current context
    trace_id: Option<String>,
    /// Span ID for the current context
    span_id: Option<String>,
}

impl LogContext {
    /// Create a new logging context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a field to the context
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add a trace ID to the context
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Add a span ID to the context
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Get all fields
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Get the span ID
    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }
}

/// A logger for structured logging
#[derive(Debug)]
pub struct Logger {
    /// Logger configuration
    config: RwLock<LoggerConfig>,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Self {
        Self {
            config: RwLock::new(LoggerConfig::default()),
        }
    }

    /// Initialize the logger
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // In a real implementation, we would set up tracing/logging here
        // This is a simplified version that uses the existing tracing setup
        Ok(())
    }

    /// Set the logger configuration
    pub fn set_config(&self, config: LoggerConfig) -> ObservabilityResult<()> {
        let mut current_config = self.config.write().map_err(|e| 
            ObservabilityError::LoggingError(format!("Failed to acquire config write lock: {}", e)))?;
        *current_config = config;
        Ok(())
    }

    /// Log a message with context
    pub fn log(&self, record: LogRecord, context: Option<&LogContext>) -> ObservabilityResult<()> {
        let config = self.config.read().map_err(|e| 
            ObservabilityError::LoggingError(format!("Failed to acquire config read lock: {}", e)))?;
            
        // Check if the log level is enabled for this component
        let component_level = config.component_levels.get(&record.component).copied()
            .unwrap_or(config.default_level);
            
        if record.level < component_level {
            // This log level is filtered out
            return Ok(());
        }
        
        // Build fields for the log
        let mut fields = record.fields.clone();
        fields.insert("component".to_string(), record.component.clone());
        
        // Add trace context if available and enabled
        if config.include_trace_context {
            if let Some(trace_id) = record.trace_id() {
                fields.insert("trace_id".to_string(), trace_id.to_string());
            } else if let Some(ctx) = context {
                if let Some(trace_id) = ctx.trace_id() {
                    fields.insert("trace_id".to_string(), trace_id.to_string());
                }
            }
            
            if let Some(span_id) = record.span_id() {
                fields.insert("span_id".to_string(), span_id.to_string());
            } else if let Some(ctx) = context {
                if let Some(span_id) = ctx.span_id() {
                    fields.insert("span_id".to_string(), span_id.to_string());
                }
            }
        }
        
        // Add context fields
        if let Some(ctx) = context {
            for (key, value) in ctx.fields() {
                if !fields.contains_key(key) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Log the message with appropriate level
        match record.level {
            LogLevel::Debug => {
                tracing::debug!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
            LogLevel::Info => {
                tracing::info!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
            LogLevel::Warning => {
                tracing::warn!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
            LogLevel::Error => {
                tracing::error!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
            LogLevel::Critical => {
                tracing::error!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
            LogLevel::Trace => {
                tracing::trace!(
                    component = %record.component,
                    message = %record.message, 
                    ?fields
                );
            },
        }
        
        Ok(())
    }

    /// Log a debug message
    pub fn debug(
        &self,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<&LogContext>,
    ) -> ObservabilityResult<()> {
        let record = LogRecord::new(message, LogLevel::Debug, component);
        self.log(record, context)
    }

    /// Log an info message
    pub fn info(
        &self,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<&LogContext>,
    ) -> ObservabilityResult<()> {
        let record = LogRecord::new(message, LogLevel::Info, component);
        self.log(record, context)
    }

    /// Log a warning message
    pub fn warning(
        &self,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<&LogContext>,
    ) -> ObservabilityResult<()> {
        let record = LogRecord::new(message, LogLevel::Warning, component);
        self.log(record, context)
    }

    /// Log an error message
    pub fn error(
        &self,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<&LogContext>,
    ) -> ObservabilityResult<()> {
        let record = LogRecord::new(message, LogLevel::Error, component);
        self.log(record, context)
    }

    /// Log a critical message
    pub fn critical(
        &self,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<&LogContext>,
    ) -> ObservabilityResult<()> {
        let record = LogRecord::new(message, LogLevel::Critical, component);
        self.log(record, context)
    }

    /// Set the log level for a component
    pub fn set_component_level(
        &self,
        component: impl Into<String>,
        level: LogLevel,
    ) -> ObservabilityResult<()> {
        let mut config = self.config.write().map_err(|e| 
            ObservabilityError::LoggingError(format!("Failed to acquire config write lock: {}", e)))?;
        
        config.component_levels.insert(component.into(), level);
        Ok(())
    }

    /// Set the default log level
    pub fn set_default_level(&self, level: LogLevel) -> ObservabilityResult<()> {
        let mut config = self.config.write().map_err(|e| 
            ObservabilityError::LoggingError(format!("Failed to acquire config write lock: {}", e)))?;
        
        config.default_level = level;
        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local logging context manager
pub struct ContextManager;

impl ContextManager {
    thread_local! {
        static CONTEXT: RwLock<LogContext> = RwLock::new(LogContext::new());
    }

    /// Set the current logging context
    pub fn set_context(context: LogContext) -> ObservabilityResult<()> {
        Self::CONTEXT.with(|ctx| {
            let mut current = ctx.write().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context write lock: {}", e)))?;
            *current = context;
            Ok(())
        })
    }

    /// Get the current logging context
    pub fn current_context() -> ObservabilityResult<LogContext> {
        Self::CONTEXT.with(|ctx| {
            let current = ctx.read().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context read lock: {}", e)))?;
            Ok(current.clone())
        })
    }

    /// Add a field to the current context
    pub fn add_field(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> ObservabilityResult<()> {
        Self::CONTEXT.with(|ctx| {
            let mut current = ctx.write().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context write lock: {}", e)))?;
            current.fields.insert(key.into(), value.into());
            Ok(())
        })
    }

    /// Set the trace ID in the current context
    pub fn set_trace_id(trace_id: impl Into<String>) -> ObservabilityResult<()> {
        Self::CONTEXT.with(|ctx| {
            let mut current = ctx.write().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context write lock: {}", e)))?;
            current.trace_id = Some(trace_id.into());
            Ok(())
        })
    }

    /// Set the span ID in the current context
    pub fn set_span_id(span_id: impl Into<String>) -> ObservabilityResult<()> {
        Self::CONTEXT.with(|ctx| {
            let mut current = ctx.write().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context write lock: {}", e)))?;
            current.span_id = Some(span_id.into());
            Ok(())
        })
    }

    /// Clear the current context
    pub fn clear_context() -> ObservabilityResult<()> {
        Self::CONTEXT.with(|ctx| {
            let mut current = ctx.write().map_err(|e| 
                ObservabilityError::LoggingError(format!("Failed to acquire context write lock: {}", e)))?;
            *current = LogContext::new();
            Ok(())
        })
    }
} 