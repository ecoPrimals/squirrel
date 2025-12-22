//! Error Context Helpers
//!
//! This module provides utilities for adding rich context to errors,
//! making debugging and diagnostics much easier.
//!
//! ## Philosophy
//!
//! Good error handling isn't just about propagating errors - it's about
//! providing enough context to understand:
//! - **What** went wrong
//! - **Where** it went wrong
//! - **Why** it went wrong
//! - **What** can be done about it
//!
//! ## Quick Start
//!
//! ```rust
//! use squirrel::error_handling::context::ResultExt;
//!
//! fn process_file(path: &str) -> Result<String, Error> {
//!     let content = std::fs::read_to_string(path)
//!         .context("Failed to read configuration file")?;
//!     
//!     let parsed = parse_config(&content)
//!         .with_context(|| format!("Failed to parse config from {}", path))?;
//!     
//!     Ok(parsed)
//! }
//! ```
//!
//! ## Benefits
//!
//! ### Without Context
//! ```text
//! Error: No such file or directory (os error 2)
//! ```
//!
//! ### With Context
//! ```text
//! Error: Failed to read configuration file
//! Caused by: Failed to open file at /etc/squirrel/config.toml
//! Caused by: No such file or directory (os error 2)
//! ```

use crate::error::PrimalError;
use std::fmt;

/// Extension trait for adding context to Result types
///
/// This trait provides ergonomic methods for wrapping errors with additional
/// context information, similar to `anyhow::Context` but integrated with
/// our error system.
pub trait ResultExt<T> {
    /// Add context to an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use squirrel::error_handling::context::ResultExt;
    ///
    /// fn load_config() -> Result<Config, PrimalError> {
    ///     let content = std::fs::read_to_string("config.toml")
    ///         .context("Failed to read config file")?;
    ///     // ...
    /// }
    /// ```
    fn context(self, context: &str) -> Result<T, PrimalError>;

    /// Add context to an error using a closure
    ///
    /// This is useful when the context message is expensive to compute
    /// and should only be evaluated if there's an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use squirrel::error_handling::context::ResultExt;
    ///
    /// fn process_items(items: &[Item]) -> Result<(), PrimalError> {
    ///     for (i, item) in items.iter().enumerate() {
    ///         process_item(item)
    ///             .with_context(|| format!("Failed to process item {} of {}", i + 1, items.len()))?;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn with_context<F>(self, f: F) -> Result<T, PrimalError>
    where
        F: FnOnce() -> String;
}

// Implementation for any Result that can be converted to PrimalError
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<PrimalError>,
{
    fn context(self, context: &str) -> Result<T, PrimalError> {
        self.map_err(|e| {
            let base_error = e.into();
            PrimalError::ConfigError(format!("{}: {}", context, base_error))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T, PrimalError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            PrimalError::ConfigError(format!("{}: {}", f(), base_error))
        })
    }
}

/// Helper for creating operation-specific error contexts
///
/// This struct helps build consistent error contexts for common operations
/// like file I/O, network calls, database queries, etc.
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// The operation being performed (e.g., "read_file", "api_call")
    pub operation: String,
    /// The resource being operated on (e.g., file path, URL)
    pub resource: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl OperationContext {
    /// Create a new operation context
    ///
    /// # Example
    ///
    /// ```rust
    /// use squirrel::error_handling::context::OperationContext;
    ///
    /// let ctx = OperationContext::new("database_query")
    ///     .with_resource("users table")
    ///     .with_metadata("query_type", "SELECT");
    /// ```
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            resource: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a resource identifier
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Format the context as an error message
    pub fn format_error(&self, base_message: &str) -> String {
        let mut parts = vec![base_message.to_string()];

        parts.push(format!("Operation: {}", self.operation));

        if let Some(resource) = &self.resource {
            parts.push(format!("Resource: {}", resource));
        }

        if !self.metadata.is_empty() {
            let metadata_str: Vec<String> = self
                .metadata
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            parts.push(format!("Details: {}", metadata_str.join(", ")));
        }

        parts.join("\n")
    }

    /// Wrap a result with this operation context
    pub fn wrap_result<T, E>(self, result: Result<T, E>) -> Result<T, PrimalError>
    where
        E: Into<PrimalError>,
    {
        result.map_err(|e| {
            let base_error = e.into();
            let message = self.format_error(&format!("Operation '{}' failed", self.operation));

            PrimalError::ConfigError(format!("{}: {}", message, base_error))
        })
    }
}

impl fmt::Display for OperationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error("Operation context"))
    }
}

/// Common error context builders for frequent operations
pub mod common {
    use super::OperationContext;

    /// Create context for file operations
    pub fn file_operation(operation: &str, path: &str) -> OperationContext {
        OperationContext::new(operation).with_resource(path)
    }

    /// Create context for network operations
    pub fn network_operation(operation: &str, url: &str) -> OperationContext {
        OperationContext::new(operation)
            .with_resource(url)
            .with_metadata("operation_type", "network")
    }

    /// Create context for database operations
    pub fn database_operation(operation: &str, table: &str, query_type: &str) -> OperationContext {
        OperationContext::new(operation)
            .with_resource(table)
            .with_metadata("query_type", query_type)
    }

    /// Create context for service discovery operations
    pub fn service_discovery(capability: &str) -> OperationContext {
        OperationContext::new("service_discovery")
            .with_resource(capability)
            .with_metadata("discovery_type", "capability_based")
    }

    /// Create context for authentication operations
    pub fn authentication(auth_type: &str) -> OperationContext {
        OperationContext::new("authentication").with_metadata("auth_type", auth_type)
    }
}

/// Macro for quickly adding context with file and line information
///
/// # Example
///
/// ```rust
/// use squirrel::error_context;
///
/// fn risky_operation() -> Result<(), Error> {
///     some_function()
///         .error_context!("Failed to perform risky operation")?;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! error_context {
    ($result:expr, $msg:expr) => {
        $result.map_err(|e| $crate::error::PrimalError::ContextError {
            message: format!("{} (at {}:{})", $msg, file!(), line!()),
            source: Box::new(e.into()),
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_ext_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let with_context = result.context("Failed to read file");

        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        assert!(err.to_string().contains("Failed to read file"));
    }

    #[test]
    fn test_operation_context() {
        let ctx = OperationContext::new("test_operation")
            .with_resource("/path/to/file")
            .with_metadata("key", "value");

        let formatted = ctx.format_error("Something went wrong");

        assert!(formatted.contains("test_operation"));
        assert!(formatted.contains("/path/to/file"));
        assert!(formatted.contains("key: value"));
    }

    #[test]
    fn test_common_file_operation() {
        let ctx = common::file_operation("read", "/etc/config.toml");

        assert_eq!(ctx.operation, "read");
        assert_eq!(ctx.resource, Some("/etc/config.toml".to_string()));
    }

    #[test]
    fn test_common_service_discovery() {
        let ctx = common::service_discovery("authentication");

        assert_eq!(ctx.operation, "service_discovery");
        assert_eq!(ctx.resource, Some("authentication".to_string()));
        assert_eq!(
            ctx.metadata.get("discovery_type"),
            Some(&"capability_based".to_string())
        );
    }
}
