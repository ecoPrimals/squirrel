//! Logging functionality for plugins

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum LogLevel {
    /// Trace level - very verbose debugging
    Trace = 0,
    /// Debug level - debugging information
    Debug = 1,
    /// Info level - informational messages
    Info = 2,
    /// Warn level - warning messages
    Warn = 3,
    /// Error level - error messages
    Error = 4,
}

impl LogLevel {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: String,
    /// Plugin ID
    pub plugin_id: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Source file (if available)
    pub file: Option<String>,
    /// Source line (if available)
    pub line: Option<u32>,
    /// Module path (if available)
    pub module: Option<String>,
}

/// Logger configuration
#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(serde::Serialize, serde::Deserialize))]
pub struct LoggerConfig {
    /// Minimum log level to output
    pub min_level: LogLevel,
    /// Whether to include file/line information
    pub include_location: bool,
    /// Maximum log entries to keep in memory
    pub max_entries: usize,
    /// Whether to send logs to the host system
    pub send_to_host: bool,
    /// Whether to log to console (disabled in tests)
    #[cfg(test)]
    pub log_to_console: bool,
}

/// Serializable logger configuration for WASM compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableLoggerConfig {
    /// Minimum log level to capture
    pub min_level: LogLevel,
    /// Whether to include file location information
    pub include_location: bool,
    /// Maximum number of log entries to keep in memory
    pub max_entries: usize,
    /// Whether to send logs to the host system
    pub send_to_host: bool,
}

#[cfg(test)]
impl From<SerializableLoggerConfig> for LoggerConfig {
    fn from(config: SerializableLoggerConfig) -> Self {
        Self {
            min_level: config.min_level,
            include_location: config.include_location,
            max_entries: config.max_entries,
            send_to_host: config.send_to_host,
            log_to_console: false, // Default to false in tests
        }
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        let max_entries = std::env::var("LOG_MAX_ENTRIES")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000);

        let min_level = match std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase()
            .as_str()
        {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        };

        Self {
            min_level,
            include_location: std::env::var("LOG_INCLUDE_LOCATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            max_entries,
            send_to_host: std::env::var("LOG_SEND_TO_HOST")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            #[cfg(test)]
            log_to_console: false, // Disable console logging in tests
        }
    }
}

/// Plugin logger
#[wasm_bindgen]
pub struct Logger {
    plugin_id: String,
    config: LoggerConfig,
    entries: VecDeque<LogEntry>,
}

#[wasm_bindgen]
impl Logger {
    /// Create a new logger instance
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            config: LoggerConfig::default(),
            entries: VecDeque::new(),
        }
    }

    /// Get a thread-local logger instance
    pub fn current() -> Logger {
        Logger::new("current_plugin".to_string())
    }

    /// Configure the logger
    #[wasm_bindgen]
    pub fn configure(&mut self, config: JsValue) -> Result<(), JsValue> {
        #[cfg(not(test))]
        {
            let config: LoggerConfig = serde_wasm_bindgen::from_value(config)?;
            self.config = config;
        }

        #[cfg(test)]
        {
            let serializable_config: SerializableLoggerConfig =
                serde_wasm_bindgen::from_value(config)?;
            self.config = serializable_config.into();
        }

        Ok(())
    }

    /// Log a trace message
    #[wasm_bindgen]
    pub fn trace(&mut self, message: &str) {
        self.log_with_metadata(LogLevel::Trace, message, None);
    }

    /// Log a debug message
    #[wasm_bindgen]
    pub fn debug(&mut self, message: &str) {
        self.log_with_metadata(LogLevel::Debug, message, None);
    }

    /// Log an info message
    #[wasm_bindgen]
    pub fn info(&mut self, message: &str) {
        self.log_with_metadata(LogLevel::Info, message, None);
    }

    /// Log a warning message
    #[wasm_bindgen]
    pub fn warn(&mut self, message: &str) {
        self.log_with_metadata(LogLevel::Warn, message, None);
    }

    /// Log an error message
    #[wasm_bindgen]
    pub fn error(&mut self, message: &str) {
        self.log_with_metadata(LogLevel::Error, message, None);
    }

    /// Get recent log entries
    #[wasm_bindgen]
    pub fn get_recent_entries(&self, limit: usize) -> JsValue {
        let entries: Vec<&LogEntry> = self.entries.iter().rev().take(limit).collect();
        serde_wasm_bindgen::to_value(&entries).unwrap_or(JsValue::NULL)
    }

    /// Get entries for a specific log level
    #[wasm_bindgen]
    pub fn get_entries_by_level(&self, level: LogLevel) -> JsValue {
        let entries: Vec<&LogEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect();
        serde_wasm_bindgen::to_value(&entries).unwrap_or(JsValue::NULL)
    }

    /// Get log entries count
    #[wasm_bindgen]
    pub fn get_entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Clear all log entries
    #[wasm_bindgen]
    pub fn clear_entries(&mut self) {
        self.entries.clear();
    }

    /// Set minimum log level
    #[wasm_bindgen]
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.config.min_level = level;
    }

    /// Get current minimum log level
    #[wasm_bindgen]
    pub fn get_min_level(&self) -> LogLevel {
        self.config.min_level
    }
}

impl Logger {
    /// Log a message with optional metadata
    pub fn log_with_metadata(
        &mut self,
        level: LogLevel,
        message: &str,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        // Check if this level should be logged
        if level < self.config.min_level {
            return;
        }

        let entry = LogEntry {
            level,
            message: message.to_string(),
            plugin_id: self.plugin_id.clone(),
            timestamp: crate::utils::current_timestamp_iso(),
            metadata: metadata.unwrap_or_default(),
            file: None, // TODO: Extract from caller info if available
            line: None,
            module: None,
        };

        // Add to entries
        self.entries.push_back(entry.clone());

        // Trim entries if we exceed max
        if self.entries.len() > self.config.max_entries {
            self.entries.pop_front();
        }

        // Send to host if configured
        if self.config.send_to_host {
            self.send_to_host(&entry);
        }

        // Also log to console for development
        self.log_to_console(&entry);
    }

    /// Send log entry to host system
    fn send_to_host(&self, _entry: &LogEntry) {
        // TODO: Implement host communication
    }

    /// Log to browser console
    fn log_to_console(&self, entry: &LogEntry) {
        // Skip console logging in tests to avoid web_sys panics
        #[cfg(test)]
        if !self.config.log_to_console {
            return;
        }

        let message = format!(
            "[{}] [{}] {}: {}",
            entry.timestamp, entry.level, entry.plugin_id, entry.message
        );

        #[cfg(not(test))]
        {
            match entry.level {
                LogLevel::Trace | LogLevel::Debug => web_sys::console::log_1(&message.into()),
                LogLevel::Info => web_sys::console::info_1(&message.into()),
                LogLevel::Warn => web_sys::console::warn_1(&message.into()),
                LogLevel::Error => web_sys::console::error_1(&message.into()),
            }
        }

        // In test mode, we can optionally print to stdout instead
        #[cfg(test)]
        if self.config.log_to_console {
            println!("{}", message);
        }
    }

    /// Create a scoped logger with additional context
    pub fn with_context(&self, _context: HashMap<String, serde_json::Value>) -> ScopedLogger {
        ScopedLogger {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Scoped logger for contextual logging
pub struct ScopedLogger<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScopedLogger<'a> {
    /// Create a new scoped logger
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Log a message with the current context
    pub fn log(&self, _level: LogLevel, _message: &str) {
        // TODO: Implement scoped logging
    }
}

/// Convenience macros for logging

/// Log a trace-level message using the global logger
///
/// This macro logs trace-level messages through the global logger instance.
/// Trace messages are the most detailed logging level and are typically used
/// for fine-grained debugging information.
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logging::Logger::global().trace(&format!($($arg)*))
    };
}

/// Log a debug-level message with cross-platform support
///
/// This macro provides cross-platform debug logging that works in both
/// WASM (browser console) and native (tracing) environments.
/// Debug messages are used for detailed diagnostic information.
///
/// # Examples
/// ```
/// log_debug!("Processing item: {}", item_id);
/// log_debug!("Cache hit ratio: {:.2}%", ratio * 100.0);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::debug_1(&format!($($arg)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::debug!($($arg)*);
        }
    };
}

/// Log an info-level message with cross-platform support
///
/// This macro provides cross-platform informational logging that works in both
/// WASM (browser console) and native (tracing) environments.
/// Info messages are used for general information about program execution.
///
/// # Examples
/// ```
/// log_info!("Service started on port {}", port);
/// log_info!("Processing {} items", item_count);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::info_1(&format!($($arg)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::info!($($arg)*);
        }
    };
}

/// Log a warning-level message with cross-platform support
///
/// This macro provides cross-platform warning logging that works in both
/// WASM (browser console) and native (tracing) environments.
/// Warning messages are used for potentially problematic situations that
/// don't prevent the program from continuing.
///
/// # Examples
/// ```
/// log_warn!("Connection timeout, retrying...");
/// log_warn!("Memory usage is high: {}MB", memory_mb);
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::warn_1(&format!($($arg)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::warn!($($arg)*);
        }
    };
}

/// Log an error-level message with cross-platform support
///
/// This macro provides cross-platform error logging that works in both
/// WASM (browser console) and native (tracing) environments.
/// Error messages are used for error conditions that require attention
/// but allow the program to continue running.
///
/// # Examples
/// ```
/// log_error!("Failed to save file: {}", error);
/// log_error!("Database connection failed after {} attempts", attempts);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::error_1(&format!($($arg)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::error!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new("test-plugin".to_string());
        assert_eq!(logger.plugin_id, "test-plugin");
        assert_eq!(logger.config.min_level, LogLevel::Info);
    }

    #[test]
    fn test_logging() {
        let mut logger = Logger::new("test-plugin".to_string());
        logger.info("Test message");
        assert_eq!(logger.entries.len(), 1);
        assert_eq!(logger.entries[0].message, "Test message");
        assert_eq!(logger.entries[0].level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_filtering() {
        let mut logger = Logger::new("test-plugin".to_string());
        logger.config.min_level = LogLevel::Warn;

        logger.debug("Debug message");
        logger.info("Info message");
        logger.warn("Warning message");

        assert_eq!(logger.entries.len(), 1);
        assert_eq!(logger.entries[0].message, "Warning message");
    }
}
