// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core error types for the Squirrel Plugin SDK
//!
//! **DEPRECATED**: This error module is being replaced by the unified error system.
//! Please migrate to `universal-error` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use crate::infrastructure::error::{PluginError, PluginResult};
//! // New:
//! use universal_error::{Result, sdk::SDKError};
//! ```
//!
//! For detailed migration instructions, see: `crates/universal-error/README.md`

#![deprecated(since = "0.2.0", note = "Use `universal-error` crate instead")]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for plugin operations
///
/// **DEPRECATED**: Use `universal_error::sdk::SDKError` instead.
#[deprecated(since = "0.2.0", note = "Use `universal_error::sdk::SDKError` instead")]
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Command not found
    #[error("Unknown command: {command}")]
    UnknownCommand {
        /// The command name that was not found
        command: String,
    },

    /// Missing required parameter
    #[error("Missing required parameter: {parameter}")]
    MissingParameter {
        /// The name of the missing parameter
        parameter: String,
    },

    /// Invalid parameter value
    #[error("Invalid parameter '{name}': {reason}")]
    InvalidParameter {
        /// The name of the invalid parameter
        name: String,
        /// The reason why the parameter is invalid
        reason: String,
    },

    /// Permission denied for operation
    #[error("Permission denied: {operation} - {reason}")]
    PermissionDenied {
        /// The operation that was denied
        operation: String,
        /// The reason for the denial
        reason: String,
    },

    /// Network operation failed
    #[error("Network error in {operation}: {message}")]
    NetworkError {
        /// The network operation that failed
        operation: String,
        /// The error message describing the failure
        message: String,
    },

    /// File system operation failed
    #[error("File system error in {operation}: {message}")]
    FileSystemError {
        /// The file system operation that failed
        operation: String,
        /// The error message describing the failure
        message: String,
    },

    /// MCP protocol error
    #[error("MCP protocol error: {message}")]
    McpError {
        /// The MCP protocol error message
        message: String,
    },

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {reason}")]
    InitializationError {
        /// The reason why initialization failed
        reason: String,
    },

    /// Plugin configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError {
        /// The configuration error message
        message: String,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// The serialization error message
        message: String,
    },

    /// Timeout error
    #[error("Operation '{operation}' timed out after {seconds} seconds")]
    TimeoutError {
        /// The operation that timed out
        operation: String,
        /// The timeout duration in seconds
        seconds: u64,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded {
        /// The resource that exceeded its limit
        resource: String,
        /// The limit that was exceeded
        limit: String,
    },

    /// Quota exceeded (for sandbox resource management)
    #[error("Quota exceeded: {resource} - {message}")]
    QuotaExceeded {
        /// The resource that exceeded quota
        resource: String,
        /// Additional error message
        message: String,
    },

    /// Plugin not found
    #[error("Plugin not found: {plugin_id}")]
    PluginNotFound {
        /// The ID of the plugin that was not found
        plugin_id: String,
    },

    /// Plugin already exists
    #[error("Plugin already exists: {plugin_id}")]
    PluginAlreadyExists {
        /// The ID of the plugin that already exists
        plugin_id: String,
    },

    /// Dependency error
    #[error("Dependency error: {dependency} - {message}")]
    DependencyError {
        /// The dependency that caused the error
        dependency: String,
        /// The error message describing the dependency issue
        message: String,
    },

    /// Version compatibility error
    #[error("Version incompatible: required {required}, found {found}")]
    VersionIncompatible {
        /// The required version string
        required: String,
        /// The version string that was found instead
        found: String,
    },

    /// Invalid version format
    #[error("Invalid version format: {version} - {reason}")]
    InvalidVersion {
        /// The invalid version string
        version: String,
        /// The reason why the version format is invalid
        reason: String,
    },

    /// Security violation
    #[error("Security violation: {violation}")]
    SecurityViolation {
        /// Description of the security violation
        violation: String,
    },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError {
        /// The internal error message
        message: String,
    },

    /// Execution error
    #[error("Execution error in {context}: {message}")]
    ExecutionError {
        /// The execution context where the error occurred
        context: String,
        /// The execution error message
        message: String,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration {
        /// The configuration error message
        message: String,
    },

    /// JavaScript error
    #[error("JavaScript error: {message}")]
    JsError {
        /// The JavaScript error message
        message: String,
    },

    /// Unknown error
    #[error("Unknown error: {message}")]
    Unknown {
        /// The unknown error message
        message: String,
    },

    /// HTTP-specific errors
    #[error("HTTP error: {status} - {message}")]
    HttpError {
        /// The HTTP status code
        status: u16,
        /// The HTTP error message
        message: String,
    },

    /// JSON parsing error
    #[error("JSON parsing error: {message}")]
    JsonError {
        /// The JSON parsing error message
        message: String,
    },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        /// The field that failed validation
        field: String,
        /// The validation error message
        message: String,
    },

    /// Connection error
    #[error("Connection error: {endpoint} - {message}")]
    ConnectionError {
        /// The endpoint that failed to connect
        endpoint: String,
        /// The connection error message
        message: String,
    },

    /// Authentication error
    #[error("Authentication error: {message}")]
    AuthenticationError {
        /// The authentication error message
        message: String,
    },

    /// Authorization error
    #[error("Authorization error: {resource} - {message}")]
    AuthorizationError {
        /// The resource that access was denied for
        resource: String,
        /// The authorization error message
        message: String,
    },

    /// Rate limiting error
    #[error("Rate limit exceeded: {resource} - retry after {retry_after} seconds")]
    RateLimitError {
        /// The resource that was rate limited
        resource: String,
        /// Number of seconds to wait before retrying
        retry_after: u64,
    },

    /// Plugin lifecycle error
    #[error("Plugin lifecycle error: {state} -> {target_state} - {message}")]
    LifecycleError {
        /// The current plugin state
        state: String,
        /// The target plugin state that failed to be reached
        target_state: String,
        /// The lifecycle error message
        message: String,
    },

    /// Command execution error
    #[error("Command execution error: {command} - {message}")]
    CommandExecutionError {
        /// The command that failed to execute
        command: String,
        /// The command execution error message
        message: String,
    },

    /// Event handling error
    #[error("Event handling error: {event_type} - {message}")]
    EventHandlingError {
        /// The type of event that failed to be handled
        event_type: String,
        /// The event handling error message
        message: String,
    },

    /// Context error
    #[error("Context error: {context} - {message}")]
    ContextError {
        /// The context where the error occurred
        context: String,
        /// The context error message
        message: String,
    },

    /// Storage error
    #[error("Storage error: {operation} - {message}")]
    StorageError {
        /// The storage operation that failed
        operation: String,
        /// The storage error message
        message: String,
    },

    /// Cache error
    #[error("Cache error: {operation} - {message}")]
    CacheError {
        /// The cache operation that failed
        operation: String,
        /// The cache error message
        message: String,
    },

    /// Lock error
    #[error("Lock error: {resource} - {message}")]
    LockError {
        /// The resource that could not be locked
        resource: String,
        /// The lock error message
        message: String,
    },

    /// Communication error
    #[error("Communication error: {target} - {message}")]
    CommunicationError {
        /// The communication target that failed
        target: String,
        /// The communication error message
        message: String,
    },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    ResourceNotFound {
        /// The resource that was not found
        resource: String,
    },

    /// Resource already exists
    #[error("Resource already exists: {resource}")]
    ResourceAlreadyExists {
        /// The resource that already exists
        resource: String,
    },

    /// Temporary failure
    #[error("Temporary failure: {operation} - {message}")]
    TemporaryFailure {
        /// The operation that experienced a temporary failure
        operation: String,
        /// The temporary failure error message
        message: String,
    },

    /// Permanent failure
    #[error("Permanent failure: {operation} - {message}")]
    PermanentFailure {
        /// The operation that experienced a permanent failure
        operation: String,
        /// The permanent failure error message
        message: String,
    },

    /// External service error
    #[error("External service error: {service} - {message}")]
    ExternalServiceError {
        /// The external service that failed
        service: String,
        /// The external service error message
        message: String,
    },

    /// Not implemented
    #[error("Not implemented: {feature}")]
    NotImplemented {
        /// The feature that is not yet implemented
        feature: String,
    },

    /// Not supported
    #[error("Not supported: {feature}")]
    NotSupported {
        /// The feature that is not supported
        feature: String,
    },

    /// Deprecated feature
    #[error("Deprecated feature: {feature}. Please use {alternative}")]
    Deprecated {
        /// The deprecated feature name
        feature: String,
        /// The recommended alternative to use instead
        alternative: String,
    },
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
