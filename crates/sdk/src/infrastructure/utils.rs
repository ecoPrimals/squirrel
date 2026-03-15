// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Utility functions for plugin development

use crate::error::{PluginError, PluginResult};
use std::sync::{Mutex, MutexGuard};
use wasm_bindgen::prelude::*;

/// Set up panic hook for better error messages in WASM
#[cfg(feature = "console")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Import the console.log function from web-sys
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Macro for logging to the browser console
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::utils::console_log(&format_args!($($t)*).to_string()))
}

/// Log a message to the browser console
pub fn console_log(message: &str) {
    log(message);
}

/// Convert a Result<T, E> to a Result<T, JsValue> for WASM compatibility
pub fn to_js_result<T, E>(result: Result<T, E>) -> Result<T, JsValue>
where
    E: std::fmt::Display,
{
    result.map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert a JsValue to a serde Value
pub fn js_to_serde(js_val: JsValue) -> Result<serde_json::Value, JsValue> {
    serde_wasm_bindgen::from_value(js_val).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert a serde Value to a JsValue
pub fn serde_to_js(val: &serde_json::Value) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Generate a UUID v4
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current timestamp in milliseconds
pub fn current_timestamp() -> u64 {
    #[cfg(not(test))]
    {
        js_sys::Date::now() as u64
    }

    #[cfg(test)]
    {
        // In tests, return a fixed timestamp to avoid WASM calls
        1640995200000 // January 1, 2022 00:00:00 UTC
    }
}

/// Get current timestamp as ISO 8601 string
pub fn current_timestamp_iso() -> String {
    #[cfg(not(test))]
    {
        chrono::Utc::now().to_rfc3339()
    }

    #[cfg(test)]
    {
        // In tests, return a fixed timestamp
        "2022-01-01T00:00:00Z".to_string()
    }
}

/// Safe mutex lock acquisition with error handling
///
/// This utility function provides a consistent way to acquire mutex locks
/// with proper error handling for poisoned mutexes.
///
/// # Arguments
///
/// * `mutex` - The mutex to acquire
/// * `context` - Context string for error messages
///
/// # Returns
///
/// Returns a `MutexGuard` on success, or a `PluginError` on failure
pub fn safe_lock<'a, T>(mutex: &'a Mutex<T>, context: &str) -> PluginResult<MutexGuard<'a, T>> {
    mutex.lock().map_err(|e| PluginError::LockError {
        resource: context.to_string(),
        message: format!("Failed to acquire lock: {}", e),
    })
}

/// Safe mutex lock acquisition with fallback value
///
/// This utility function attempts to acquire a mutex lock, but returns a
/// fallback value if the lock is poisoned, allowing for graceful degradation.
///
/// # Arguments
///
/// * `mutex` - The mutex to acquire
/// * `fallback` - Function to generate fallback value
/// * `context` - Context string for error logging
///
/// # Returns
///
/// Returns either the locked value or the fallback value
pub fn safe_lock_or_fallback<T, F>(mutex: &Mutex<T>, fallback: F, context: &str) -> T
where
    T: Clone,
    F: FnOnce() -> T,
{
    match mutex.lock() {
        Ok(guard) => guard.clone(),
        Err(e) => {
            eprintln!("Warning: Failed to acquire {} lock: {}", context, e);
            fallback()
        }
    }
}

/// Singleton pattern helper macro
///
/// This macro provides a consistent way to implement singleton patterns
/// across the SDK, reducing code duplication.
///
/// # Usage
///
/// ```ignore
/// use crate::utils::singleton;
///
/// struct MyService { /* ... */ }
///
/// impl MyService {
///     fn new() -> Self { /* ... */ }
///     
///     pub fn global() -> &'static MyService {
///         singleton!(MyService::new())
///     }
/// }
/// ```
#[macro_export]
macro_rules! singleton {
    ($init:expr) => {{
        static INSTANCE: std::sync::OnceLock<Box<$crate::events::EventBus>> =
            std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Box::new($init)).as_ref()
    }};
}

/// Generate a unique ID with a prefix
///
/// This utility function generates a unique ID with a specified prefix,
/// providing a consistent ID generation pattern across the SDK.
///
/// # Arguments
///
/// * `prefix` - The prefix to use for the ID
///
/// # Returns
///
/// Returns a unique ID string in the format "{prefix}_{uuid}"
pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, generate_uuid())
}

/// Generate a listener ID
///
/// Convenience function for generating event listener IDs.
pub fn generate_listener_id() -> String {
    generate_id("listener")
}

/// Generate a plugin ID
///
/// Convenience function for generating plugin IDs.
pub fn generate_plugin_id() -> String {
    generate_id("plugin")
}

/// Generate a command ID
///
/// Convenience function for generating command IDs.
pub fn generate_command_id() -> String {
    generate_id("command")
}

/// Sleep for the specified number of milliseconds
///
/// This function provides a platform-agnostic way to sleep in async code.
pub async fn sleep_ms(ms: u64) {
    #[cfg(not(test))]
    {
        use wasm_bindgen_futures::JsFuture;
        // JavaScript setTimeout accepts i32, so clamp to i32::MAX to avoid truncation
        let ms_clamped = ms.min(i32::MAX as u64) as i32;
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            if let Some(window) = web_sys::window() {
                if let Err(e) = window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms_clamped)
                {
                    tracing::warn!("Failed to set timeout in WASM environment: {:?}", e);
                }
            } else {
                tracing::warn!("Window object not available in WASM environment");
            }
        });
        let _ = JsFuture::from(promise).await;
    }

    #[cfg(test)]
    {
        // In tests, use tokio::time::sleep
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

/// Validate and normalize a plugin ID
///
/// This function ensures plugin IDs follow a consistent format and
/// contain only valid characters.
///
/// # Arguments
///
/// * `id` - The plugin ID to validate
///
/// # Returns
///
/// Returns the normalized ID on success, or an error if validation fails
pub fn validate_plugin_id(id: &str) -> PluginResult<String> {
    if id.is_empty() {
        return Err(PluginError::ValidationError {
            field: "plugin_id".to_string(),
            message: "Plugin ID cannot be empty".to_string(),
        });
    }

    let max_length = std::env::var("PERF_MAX_PLUGIN_ID_LENGTH")
        .unwrap_or_else(|_| "64".to_string())
        .parse()
        .unwrap_or(64);

    if id.len() > max_length {
        return Err(PluginError::ValidationError {
            field: "plugin_id".to_string(),
            message: format!("Plugin ID cannot be longer than {} characters", max_length),
        });
    }

    // Check for valid characters (alphanumeric, hyphens, underscores)
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(PluginError::ValidationError {
            field: "plugin_id".to_string(),
            message: "Plugin ID can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        });
    }

    Ok(id.to_string())
}

/// Common error conversion utilities
pub mod error_conversion {
    use super::*;

    /// Convert a serialization error to PluginError
    pub fn serde_error(e: serde_json::Error) -> PluginError {
        PluginError::SerializationError {
            message: e.to_string(),
        }
    }

    /// Convert a network error to PluginError
    pub fn network_error(operation: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::NetworkError {
            operation: operation.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert a filesystem error to PluginError
    pub fn fs_error(operation: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::FileSystemError {
            operation: operation.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert a configuration error to PluginError
    pub fn config_error(e: impl std::fmt::Display) -> PluginError {
        PluginError::ConfigurationError {
            message: e.to_string(),
        }
    }

    /// Convert a timeout error to PluginError
    pub fn timeout_error(operation: &str, seconds: u64) -> PluginError {
        PluginError::TimeoutError {
            operation: operation.to_string(),
            seconds,
        }
    }

    /// Convert a storage error to PluginError
    pub fn storage_error(operation: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::StorageError {
            operation: operation.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert a connection error to PluginError
    pub fn connection_error(endpoint: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::ConnectionError {
            endpoint: endpoint.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert a validation error to PluginError
    pub fn validation_error(field: &str, message: &str) -> PluginError {
        PluginError::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Convert a command execution error to PluginError
    pub fn command_error(command: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::CommandExecutionError {
            command: command.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert an event handling error to PluginError
    pub fn event_error(event_type: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::EventHandlingError {
            event_type: event_type.to_string(),
            message: e.to_string(),
        }
    }

    /// Convert a lock error to PluginError
    pub fn lock_error(resource: &str, e: impl std::fmt::Display) -> PluginError {
        PluginError::LockError {
            resource: resource.to_string(),
            message: e.to_string(),
        }
    }
}

/// Performance monitoring utilities
pub mod performance {
    use super::*;

    /// Simple performance timer
    pub struct Timer {
        start_time: u64,
        name: String,
    }

    impl Timer {
        /// Start a new timer
        pub fn new(name: &str) -> Self {
            Self {
                start_time: current_timestamp(),
                name: name.to_string(),
            }
        }

        /// Get elapsed time in milliseconds
        pub fn elapsed(&self) -> u64 {
            current_timestamp() - self.start_time
        }

        /// Log elapsed time
        pub fn log_elapsed(&self) {
            console_log(&format!("[{}] took {}ms", self.name, self.elapsed()));
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            self.log_elapsed();
        }
    }

    /// String pool for reducing allocations
    #[derive(Default)]
    pub struct StringPool {
        pool: std::collections::HashMap<String, String>,
    }

    impl StringPool {
        /// Create a new string pool
        pub fn new() -> Self {
            Self::default()
        }

        /// Get or create a string from the pool
        pub fn get_or_create(&mut self, key: &str) -> &str {
            if !self.pool.contains_key(key) {
                self.pool.insert(key.to_string(), key.to_string());
            }
            self.pool
                .get(key)
                .expect("StringPool: key should exist after insert")
        }

        /// Clear the pool
        pub fn clear(&mut self) {
            self.pool.clear();
        }

        /// Get pool size
        pub fn size(&self) -> usize {
            self.pool.len()
        }
    }

    /// Memory-efficient string builder
    pub struct StringBuilder {
        buffer: String,
    }

    impl StringBuilder {
        /// Create a new string builder with initial capacity
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                buffer: String::with_capacity(capacity),
            }
        }

        /// Append a string
        pub fn append(&mut self, s: &str) {
            self.buffer.push_str(s);
        }

        /// Append a character
        pub fn append_char(&mut self, c: char) {
            self.buffer.push(c);
        }

        /// Get the length
        pub fn len(&self) -> usize {
            self.buffer.len()
        }

        /// Check if empty
        pub fn is_empty(&self) -> bool {
            self.buffer.is_empty()
        }

        /// Build the final string
        pub fn build(self) -> String {
            self.buffer
        }

        /// Clear the builder for reuse
        pub fn clear(&mut self) {
            self.buffer.clear();
        }
    }

    /// Batch operation helper
    pub struct BatchProcessor<T> {
        items: Vec<T>,
        batch_size: usize,
    }

    impl<T> BatchProcessor<T> {
        /// Create a new batch processor
        pub fn new(batch_size: usize) -> Self {
            Self {
                items: Vec::with_capacity(batch_size),
                batch_size,
            }
        }

        /// Add an item to the batch
        pub fn add(&mut self, item: T) -> Option<Vec<T>> {
            self.items.push(item);
            if self.items.len() >= self.batch_size {
                let batch = std::mem::replace(&mut self.items, Vec::with_capacity(self.batch_size));
                Some(batch)
            } else {
                None
            }
        }

        /// Get remaining items
        pub fn finish(self) -> Vec<T> {
            self.items
        }

        /// Get current batch size
        pub fn current_size(&self) -> usize {
            self.items.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_generate_uuid() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();
        assert_ne!(uuid1, uuid2);
        assert!(uuid1.len() > 0);
    }

    #[test]
    fn test_generate_id() {
        let id = generate_id("test");
        assert!(id.starts_with("test_"));
        assert!(id.len() > 5);
    }

    #[test]
    fn test_validate_plugin_id() {
        assert!(validate_plugin_id("valid-plugin_123").is_ok());
        assert!(validate_plugin_id("").is_err());
        assert!(validate_plugin_id("invalid@plugin").is_err());
        assert!(validate_plugin_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_safe_lock() {
        let mutex = Mutex::new(42);
        let guard = safe_lock(&mutex, "test").unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_lock_or_fallback() {
        let mutex = Mutex::new(42);
        let result = safe_lock_or_fallback(&mutex, || 0, "test");
        assert_eq!(result, 42);
    }
}
