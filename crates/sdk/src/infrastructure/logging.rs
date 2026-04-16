// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Logging functionality for plugins

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;

thread_local! {
    static GLOBAL_LOGGER: RefCell<Logger> = RefCell::new(Logger::new("global".to_string()));
}

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
    /// Run `f` with the process-global logger (thread-local on native).
    pub fn with_global<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Logger) -> R,
    {
        GLOBAL_LOGGER.with(|cell| f(&mut cell.borrow_mut()))
    }

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
            file: None, // NOTE: Could extract from caller info if available
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
    fn send_to_host(&self, entry: &LogEntry) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::CustomEvent;

            let Some(window) = web_sys::window() else {
                return;
            };
            let detail = match serde_wasm_bindgen::to_value(entry) {
                Ok(v) => v,
                Err(_) => return,
            };
            let init = web_sys::CustomEventInit::new();
            init.set_detail(&detail);
            let Ok(event) =
                CustomEvent::new_with_custom_event_init_dict("squirrel-plugin-log", &init)
            else {
                return;
            };
            let _ = window.dispatch_event(event.as_ref().unchecked_ref());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let payload = serde_json::to_string(entry).unwrap_or_else(|_| entry.message.clone());
            match entry.level {
                LogLevel::Trace => {
                    tracing::trace!(target: "squirrel_sdk_plugin", "{}", payload);
                }
                LogLevel::Debug => {
                    tracing::debug!(target: "squirrel_sdk_plugin", "{}", payload);
                }
                LogLevel::Info => {
                    tracing::info!(target: "squirrel_sdk_plugin", "{}", payload);
                }
                LogLevel::Warn => {
                    tracing::warn!(target: "squirrel_sdk_plugin", "{}", payload);
                }
                LogLevel::Error => {
                    tracing::error!(target: "squirrel_sdk_plugin", "{}", payload);
                }
            }
        }
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
    pub fn with_context(&self, context: HashMap<String, serde_json::Value>) -> ScopedLogger {
        ScopedLogger {
            plugin_id: self.plugin_id.clone(),
            context,
        }
    }
}

fn format_context_prefix(context: &HashMap<String, serde_json::Value>) -> String {
    if context.is_empty() {
        return String::new();
    }
    let mut keys: Vec<_> = context.keys().collect();
    keys.sort_unstable();
    let parts: Vec<String> = keys
        .into_iter()
        .filter_map(|k| context.get(k).map(|v| format!("{k}={v}")))
        .collect();
    format!("[{}]", parts.join(", "))
}

/// Scoped logger for contextual logging
#[derive(Default)]
pub struct ScopedLogger {
    plugin_id: String,
    context: HashMap<String, serde_json::Value>,
}

impl ScopedLogger {
    /// Create a new scoped logger
    pub fn new() -> Self {
        Self::default()
    }

    /// Log a message with the current context
    pub fn log(&self, level: LogLevel, message: &str) {
        let prefix = format_context_prefix(&self.context);
        let full = if prefix.is_empty() {
            message.to_string()
        } else {
            format!("{prefix} {message}")
        };
        Logger::with_global(|logger| {
            let prev = logger.plugin_id.clone();
            if !self.plugin_id.is_empty() {
                logger.plugin_id.clone_from(&self.plugin_id);
            }
            logger.log_with_metadata(level, &full, Some(self.context.clone()));
            logger.plugin_id = prev;
        });
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
        $crate::logging::Logger::with_global(|logger| {
            logger.trace(&format!($($arg)*))
        })
    };
}

/// Log a debug-level message with cross-platform support
///
/// This macro provides cross-platform debug logging that works in both
/// WASM (browser console) and native (tracing) environments.
/// Debug messages are used for detailed diagnostic information.
///
/// # Examples
/// ```ignore
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
/// ```ignore
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
/// ```ignore
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
/// ```ignore
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

    #[test]
    fn log_level_as_str_and_display() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn serializable_logger_config_json_roundtrip() {
        let c = SerializableLoggerConfig {
            min_level: LogLevel::Debug,
            include_location: true,
            max_entries: 42,
            send_to_host: false,
        };
        let s = serde_json::to_string(&c).expect("serde");
        let back: SerializableLoggerConfig = serde_json::from_str(&s).expect("de");
        assert_eq!(back.max_entries, 42);
        assert_eq!(back.min_level, LogLevel::Debug);
    }

    #[test]
    fn logger_max_entries_trims_oldest() {
        let mut logger = Logger::new("trim".to_string());
        logger.config.max_entries = 2;
        logger.info("one");
        logger.info("two");
        logger.info("three");
        assert_eq!(logger.get_entry_count(), 2);
        assert_eq!(logger.entries.front().expect("front").message, "two");
    }

    #[test]
    fn logger_clear_and_min_level_accessors() {
        let mut logger = Logger::new("acc".to_string());
        logger.info("x");
        assert_eq!(logger.get_entry_count(), 1);
        logger.clear_entries();
        assert_eq!(logger.get_entry_count(), 0);
        logger.set_min_level(LogLevel::Error);
        assert_eq!(logger.get_min_level(), LogLevel::Error);
    }

    #[test]
    fn logger_trace_warn_error_and_metadata_path() {
        let mut logger = Logger::new("lvl".to_string());
        logger.config.min_level = LogLevel::Trace;
        logger.trace("t");
        logger.warn("w");
        logger.error("e");
        let mut meta = HashMap::new();
        meta.insert("k".to_string(), serde_json::json!(1));
        logger.log_with_metadata(LogLevel::Debug, "d", Some(meta));
        assert_eq!(logger.get_entry_count(), 4);
        assert!(logger.entries[3].metadata.contains_key("k"));
    }

    #[test]
    fn scoped_logger_adds_sorted_context_prefix() {
        Logger::with_global(|g| {
            g.clear_entries();
            g.config.min_level = LogLevel::Info;
        });
        let mut ctx = HashMap::new();
        ctx.insert("zebra".to_string(), serde_json::json!(1));
        ctx.insert("alpha".to_string(), serde_json::json!(2));
        let base = Logger::new("scoped-base".to_string());
        let scoped = base.with_context(ctx);
        scoped.log(LogLevel::Info, "hello");
        Logger::with_global(|g| {
            assert_eq!(g.get_entry_count(), 1);
            assert!(g.entries[0].message.contains("alpha="));
            assert!(g.entries[0].message.contains("zebra="));
            assert!(g.entries[0].message.contains("hello"));
        });
    }

    #[test]
    fn scoped_logger_new_uses_empty_plugin_id_branch() {
        Logger::with_global(|g| {
            g.clear_entries();
            g.config.min_level = LogLevel::Info;
            g.plugin_id = "global-plugin".to_string();
        });
        let scoped = ScopedLogger::new();
        scoped.log(LogLevel::Warn, "plain");
        Logger::with_global(|g| {
            assert_eq!(g.get_entry_count(), 1);
            assert_eq!(g.entries[0].message, "plain");
            assert_eq!(g.entries[0].plugin_id, "global-plugin");
        });
    }

    #[test]
    fn logger_current_constructor() {
        let _ = Logger::current();
    }

    #[test]
    fn log_entry_serde_roundtrip() {
        let mut meta = HashMap::new();
        meta.insert("m".to_string(), serde_json::json!(true));
        let e = LogEntry {
            level: LogLevel::Info,
            message: "msg".to_string(),
            plugin_id: "p".to_string(),
            timestamp: "t".to_string(),
            metadata: meta,
            file: Some("f.rs".to_string()),
            line: Some(10),
            module: Some("mod".to_string()),
        };
        let json = serde_json::to_string(&e).expect("serde");
        let back: LogEntry = serde_json::from_str(&json).expect("de");
        assert_eq!(back.message, "msg");
        assert_eq!(back.line, Some(10));
    }

    #[test]
    fn log_debug_macro_native_tracing_path() {
        crate::log_debug!("sdk test {}", 42);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn logger_configure_from_js_value() {
        let mut logger = Logger::new("cfg".to_string());
        let c = SerializableLoggerConfig {
            min_level: LogLevel::Warn,
            include_location: false,
            max_entries: 5,
            send_to_host: false,
        };
        let js = serde_wasm_bindgen::to_value(&c).expect("js");
        logger.configure(js).expect("configure");
        assert_eq!(logger.get_min_level(), LogLevel::Warn);
        assert_eq!(logger.config.max_entries, 5);
    }
}
