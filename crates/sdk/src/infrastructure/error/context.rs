//! Error context and enhanced error handling for the Squirrel Plugin SDK

use super::core::PluginError;
use super::severity::{ErrorCategory, ErrorSeverity, PluginErrorClassification};
use serde::{Deserialize, Serialize};

/// Error context for chaining and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// Module where error occurred
    pub module: Option<String>,
    /// Function where error occurred
    pub function: Option<String>,
    /// Line number where error occurred
    pub line: Option<u32>,
    /// Additional context data
    pub context_data: std::collections::HashMap<String, serde_json::Value>,
    /// Timestamp when error occurred
    pub timestamp: String,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            module: None,
            function: None,
            line: None,
            context_data: std::collections::HashMap::new(),
            timestamp: crate::infrastructure::utils::current_timestamp_iso(),
        }
    }

    /// Add module information
    pub fn with_module(mut self, module: &str) -> Self {
        self.module = Some(module.to_string());
        self
    }

    /// Add function information
    pub fn with_function(mut self, function: &str) -> Self {
        self.function = Some(function.to_string());
        self
    }

    /// Add line number information
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context_data.insert(key.to_string(), value);
        self
    }
}

/// Enhanced error with context and chaining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedError {
    /// The core error
    pub error: PluginError,
    /// Error context
    pub context: ErrorContext,
    /// Chained source error
    pub source: Option<Box<EnhancedError>>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Whether this error is recoverable
    pub recoverable: bool,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<String>,
    /// Error category
    pub category: ErrorCategory,
}

impl EnhancedError {
    /// Create a new enhanced error
    pub fn new(error: PluginError, context: ErrorContext) -> Self {
        let severity = error.severity();
        let category = error.category();
        let recoverable = error.is_recoverable();
        let recovery_suggestions = error.recovery_suggestions();

        Self {
            error,
            context,
            source: None,
            severity,
            recoverable,
            recovery_suggestions,
            category,
        }
    }

    /// Add a source error to this error
    pub fn with_source(mut self, source: EnhancedError) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the error type as a string
    pub fn error_type(&self) -> &'static str {
        self.error.error_type()
    }

    /// Get the full error chain as a string
    pub fn full_chain(&self) -> String {
        let mut chain = vec![self.error.to_string()];
        let mut current = &self.source;

        while let Some(source) = current {
            chain.push(source.error.to_string());
            current = &source.source;
        }

        chain.join(" -> ")
    }
}

impl std::fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} ({})",
            self.severity.as_str(),
            self.category.as_str(),
            self.error,
            self.context.operation
        )
    }
}

impl std::error::Error for EnhancedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e as &dyn std::error::Error)
    }
}

/// Enhanced result type with full error context
pub type EnhancedResult<T> = Result<T, EnhancedError>;

/// Extension trait for converting PluginError to EnhancedError
pub trait PluginErrorExt {
    /// Convert to an enhanced error with context
    fn with_context(self, context: ErrorContext) -> EnhancedError;

    /// Convert to an enhanced error with simple context
    fn with_operation(self, operation: &str) -> EnhancedError;
}

impl PluginErrorExt for PluginError {
    fn with_context(self, context: ErrorContext) -> EnhancedError {
        EnhancedError::new(self, context)
    }

    fn with_operation(self, operation: &str) -> EnhancedError {
        let context = ErrorContext::new(operation);
        EnhancedError::new(self, context)
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error result
    fn with_context(self, context: ErrorContext) -> EnhancedResult<T>;

    /// Add simple operation context to an error result
    fn with_operation(self, operation: &str) -> EnhancedResult<T>;
}

impl<T> ResultExt<T> for Result<T, PluginError> {
    fn with_context(self, context: ErrorContext) -> EnhancedResult<T> {
        self.map_err(|e| e.with_context(context))
    }

    fn with_operation(self, operation: &str) -> EnhancedResult<T> {
        self.map_err(|e| e.with_operation(operation))
    }
}
