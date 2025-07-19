//! Core error types for the Squirrel Plugin SDK

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for plugin operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Command not found
    #[error("Unknown command: {command}")]
    UnknownCommand { command: String },

    /// Missing required parameter
    #[error("Missing required parameter: {parameter}")]
    MissingParameter { parameter: String },

    /// Invalid parameter value
    #[error("Invalid parameter '{name}': {reason}")]
    InvalidParameter { name: String, reason: String },

    /// Permission denied for operation
    #[error("Permission denied: {operation} - {reason}")]
    PermissionDenied { operation: String, reason: String },

    /// Network operation failed
    #[error("Network error in {operation}: {message}")]
    NetworkError { operation: String, message: String },

    /// File system operation failed
    #[error("File system error in {operation}: {message}")]
    FileSystemError { operation: String, message: String },

    /// MCP protocol error
    #[error("MCP protocol error: {message}")]
    McpError { message: String },

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {reason}")]
    InitializationError { reason: String },

    /// Plugin configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Timeout error
    #[error("Operation '{operation}' timed out after {seconds} seconds")]
    TimeoutError { operation: String, seconds: u64 },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded { resource: String, limit: String },

    /// Quota exceeded (for sandbox resource management)
    #[error("Quota exceeded: {resource} - {message}")]
    QuotaExceeded { resource: String, message: String },

    /// Plugin not found
    #[error("Plugin not found: {plugin_id}")]
    PluginNotFound { plugin_id: String },

    /// Plugin already exists
    #[error("Plugin already exists: {plugin_id}")]
    PluginAlreadyExists { plugin_id: String },

    /// Dependency error
    #[error("Dependency error: {dependency} - {message}")]
    DependencyError { dependency: String, message: String },

    /// Version compatibility error
    #[error("Version incompatible: required {required}, found {found}")]
    VersionIncompatible { required: String, found: String },

    /// Invalid version format
    #[error("Invalid version format: {version} - {reason}")]
    InvalidVersion { version: String, reason: String },

    /// Security violation
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Execution error
    #[error("Execution error in {context}: {message}")]
    ExecutionError { context: String, message: String },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    /// JavaScript error
    #[error("JavaScript error: {message}")]
    JsError { message: String },

    /// Unknown error
    #[error("Unknown error: {message}")]
    Unknown { message: String },

    /// HTTP-specific errors
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    /// JSON parsing error
    #[error("JSON parsing error: {message}")]
    JsonError { message: String },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Connection error
    #[error("Connection error: {endpoint} - {message}")]
    ConnectionError { endpoint: String, message: String },

    /// Authentication error
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Authorization error
    #[error("Authorization error: {resource} - {message}")]
    AuthorizationError { resource: String, message: String },

    /// Rate limiting error
    #[error("Rate limit exceeded: {resource} - retry after {retry_after} seconds")]
    RateLimitError { resource: String, retry_after: u64 },

    /// Plugin lifecycle error
    #[error("Plugin lifecycle error: {state} -> {target_state} - {message}")]
    LifecycleError {
        state: String,
        target_state: String,
        message: String,
    },

    /// Command execution error
    #[error("Command execution error: {command} - {message}")]
    CommandExecutionError { command: String, message: String },

    /// Event handling error
    #[error("Event handling error: {event_type} - {message}")]
    EventHandlingError { event_type: String, message: String },

    /// Context error
    #[error("Context error: {context} - {message}")]
    ContextError { context: String, message: String },

    /// Storage error
    #[error("Storage error: {operation} - {message}")]
    StorageError { operation: String, message: String },

    /// Cache error
    #[error("Cache error: {operation} - {message}")]
    CacheError { operation: String, message: String },

    /// Lock error
    #[error("Lock error: {resource} - {message}")]
    LockError { resource: String, message: String },

    /// Communication error
    #[error("Communication error: {target} - {message}")]
    CommunicationError { target: String, message: String },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },

    /// Resource already exists
    #[error("Resource already exists: {resource}")]
    ResourceAlreadyExists { resource: String },

    /// Temporary failure
    #[error("Temporary failure: {operation} - {message}")]
    TemporaryFailure { operation: String, message: String },

    /// Permanent failure
    #[error("Permanent failure: {operation} - {message}")]
    PermanentFailure { operation: String, message: String },

    /// External service error
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    /// Not implemented
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },

    /// Not supported
    #[error("Not supported: {feature}")]
    NotSupported { feature: String },

    /// Deprecated feature
    #[error("Deprecated feature: {feature}. Please use {alternative}")]
    Deprecated {
        feature: String,
        alternative: String,
    },
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
