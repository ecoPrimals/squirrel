// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Fossil `PluginError` enum — kept for serde backward compatibility only.
//!
//! All new code should use `universal_error::sdk::SDKError`.
//! This enum exists solely so that old wire-format messages containing
//! `PluginError` variants can still be deserialized.

#![expect(deprecated, reason = "fossil module: defines + derives deprecated PluginError enum")]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[deprecated(since = "0.2.0", note = "Use `universal_error::sdk::SDKError` instead")]
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub(crate) enum PluginError {
    #[error("Unknown command: {command}")]
    UnknownCommand { command: String },
    #[error("Missing required parameter: {parameter}")]
    MissingParameter { parameter: String },
    #[error("Invalid parameter '{name}': {reason}")]
    InvalidParameter { name: String, reason: String },
    #[error("Permission denied: {operation} - {reason}")]
    PermissionDenied { operation: String, reason: String },
    #[error("Network error in {operation}: {message}")]
    NetworkError { operation: String, message: String },
    #[error("File system error in {operation}: {message}")]
    FileSystemError { operation: String, message: String },
    #[error("MCP protocol error: {message}")]
    McpError { message: String },
    #[error("Plugin initialization failed: {reason}")]
    InitializationError { reason: String },
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    #[error("Operation '{operation}' timed out after {seconds} seconds")]
    TimeoutError { operation: String, seconds: u64 },
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded { resource: String, limit: String },
    #[error("Quota exceeded: {resource} - {message}")]
    QuotaExceeded { resource: String, message: String },
    #[error("Plugin not found: {plugin_id}")]
    PluginNotFound { plugin_id: String },
    #[error("Plugin already exists: {plugin_id}")]
    PluginAlreadyExists { plugin_id: String },
    #[error("Dependency error: {dependency} - {message}")]
    DependencyError { dependency: String, message: String },
    #[error("Version incompatible: required {required}, found {found}")]
    VersionIncompatible { required: String, found: String },
    #[error("Invalid version format: {version} - {reason}")]
    InvalidVersion { version: String, reason: String },
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },
    #[error("Internal error: {message}")]
    InternalError { message: String },
    #[error("Execution error in {context}: {message}")]
    ExecutionError { context: String, message: String },
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
    #[error("JavaScript error: {message}")]
    JsError { message: String },
    #[error("Unknown error: {message}")]
    Unknown { message: String },
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },
    #[error("JSON parsing error: {message}")]
    JsonError { message: String },
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
    #[error("Connection error: {endpoint} - {message}")]
    ConnectionError { endpoint: String, message: String },
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },
    #[error("Authorization error: {resource} - {message}")]
    AuthorizationError { resource: String, message: String },
    #[error("Rate limit exceeded: {resource} - retry after {retry_after} seconds")]
    RateLimitError { resource: String, retry_after: u64 },
    #[error("Plugin lifecycle error: {state} -> {target_state} - {message}")]
    LifecycleError { state: String, target_state: String, message: String },
    #[error("Command execution error: {command} - {message}")]
    CommandExecutionError { command: String, message: String },
    #[error("Event handling error: {event_type} - {message}")]
    EventHandlingError { event_type: String, message: String },
    #[error("Context error: {context} - {message}")]
    ContextError { context: String, message: String },
    #[error("Storage error: {operation} - {message}")]
    StorageError { operation: String, message: String },
    #[error("Cache error: {operation} - {message}")]
    CacheError { operation: String, message: String },
    #[error("Lock error: {resource} - {message}")]
    LockError { resource: String, message: String },
    #[error("Communication error: {target} - {message}")]
    CommunicationError { target: String, message: String },
    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },
    #[error("Resource already exists: {resource}")]
    ResourceAlreadyExists { resource: String },
    #[error("Temporary failure: {operation} - {message}")]
    TemporaryFailure { operation: String, message: String },
    #[error("Permanent failure: {operation} - {message}")]
    PermanentFailure { operation: String, message: String },
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
    #[error("Not supported: {feature}")]
    NotSupported { feature: String },
    #[error("Deprecated feature: {feature}. Please use {alternative}")]
    Deprecated { feature: String, alternative: String },
}
