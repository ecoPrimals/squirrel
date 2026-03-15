// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Logging module for MCP
//!
//! This module provides structured logging functionality for MCP components.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::json;

use tracing::{info, warn, error, debug};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LogLevel {
    /// Trace level for detailed tracing information
    Trace,
    /// Debug level for debugging information
    Debug,
    /// Info level for general information
    Info,
    /// Warning level for potential issues
    Warn,
    /// Error level for errors
    Error,
    /// Critical level for critical issues
    Critical,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log timestamp
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Log component (e.g., "transport", "protocol")
    pub component: String,
    /// Additional context data
    pub context: serde_json::Value,
}

impl LogEntry {
    /// Create a new log entry
    #[must_use]
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        component: impl Into<String>,
        context: Option<serde_json::Value>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            component: component.into(),
            context: context.unwrap_or_else(|| json!({})),
        }
    }
}

/// Logger for MCP components
#[derive(Debug, Clone)]
pub struct Logger {
    /// The component name
    component: String,
    /// Whether to include timestamps in logs
    include_timestamps: bool,
}

impl Logger {
    /// Create a new logger with the specified component name
    #[must_use]
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            include_timestamps: true,
        }
    }

    /// Create a test logger with simplified output
    #[must_use]
    pub fn new_test() -> Self {
        Self {
            component: "test".to_string(),
            include_timestamps: false,
        }
    }

    /// Log a message at the specified level
    pub fn log(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        context: Option<serde_json::Value>,
    ) {
        let message = message.into();
        let entry = LogEntry::new(level, message, self.component.clone(), context);
        
        // Use tracing for actual logging
        match level {
            LogLevel::Trace => debug!("{}", self.format_entry(&entry)),
            LogLevel::Debug => debug!("{}", self.format_entry(&entry)),
            LogLevel::Info => info!("{}", self.format_entry(&entry)),
            LogLevel::Warn => warn!("{}", self.format_entry(&entry)),
            LogLevel::Error => error!("{}", self.format_entry(&entry)),
            LogLevel::Critical => error!("CRITICAL: {}", self.format_entry(&entry)),
        }
    }

    /// Format a log entry for display
    fn format_entry(&self, entry: &LogEntry) -> String {
        let timestamp = if self.include_timestamps {
            format!("[{}] ", entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            String::new()
        };

        let context = if entry.context == json!({}) {
            String::new()
        } else {
            format!(" {}", entry.context)
        };

        format!(
            "{}[{}][{}] {}{}",
            timestamp,
            entry.level,
            entry.component,
            entry.message,
            context
        )
    }

    /// Log a debug message
    pub fn debug(&self, message: impl Into<String>, context: Option<serde_json::Value>) {
        self.log(LogLevel::Debug, message, context);
    }

    /// Log an info message
    pub fn info(&self, message: impl Into<String>, context: Option<serde_json::Value>) {
        self.log(LogLevel::Info, message, context);
    }

    /// Log a warning message
    pub fn warn(&self, message: impl Into<String>, context: Option<serde_json::Value>) {
        self.log(LogLevel::Warn, message, context);
    }

    /// Log an error message
    pub fn error(&self, message: impl Into<String>, context: Option<serde_json::Value>) {
        self.log(LogLevel::Error, message, context);
    }

    /// Log a critical message
    pub fn critical(&self, message: impl Into<String>, context: Option<serde_json::Value>) {
        self.log(LogLevel::Critical, message, context);
    }

    /// Create a child logger with a sub-component name
    #[must_use]
    pub fn with_subcomponent(&self, subcomponent: impl Into<String>) -> Self {
        let component = format!("{}:{}", self.component, subcomponent.into());
        Self {
            component,
            include_timestamps: self.include_timestamps,
        }
    }
}

/// Initialize the logging system
///
/// # Errors
/// 
/// This function will return an error if the logging system cannot be initialized
pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // This would normally set up the tracing subscriber, but for now just return Ok
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_formatting() {
        let logger = Logger::new("test-component");
        let entry = LogEntry::new(
            LogLevel::Info,
            "Test message",
            "test-component",
            Some(json!({"key": "value"})),
        );
        
        let formatted = logger.format_entry(&entry);
        assert!(formatted.contains("[INFO]"));
        assert!(formatted.contains("[test-component]"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("\"key\":\"value\""));
    }

    #[test]
    fn test_logger_methods() {
        let logger = Logger::new_test();
        
        // These calls should not panic
        logger.debug("Debug message", None);
        logger.info("Info message", Some(json!({"test": true})));
        logger.warn("Warning message", None);
        logger.error("Error message", None);
        logger.critical("Critical message", None);
    }

    #[test]
    fn test_subcomponent_logger() {
        let logger = Logger::new("parent");
        let child_logger = logger.with_subcomponent("child");
        
        assert_eq!(child_logger.component, "parent:child");
    }
} 