// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool management errors
//!
//! This module contains all error types for tool management operations.

use super::tool::ToolState;

/// Tool management errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ToolError {
    /// Required dependency is missing
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),

    /// Tool not found error
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Tool executor not found
    #[error("Executor not found: {0}")]
    ExecutorNotFound(String),

    /// Tool initialization failed
    #[error("Tool initialization failed for {tool_id}: {reason}")]
    InitializationFailed { tool_id: String, reason: String },

    /// Tool execution failed
    #[error("Tool execution failed for {tool_id}: {reason}")]
    ExecutionFailed { tool_id: String, reason: String },

    /// Execution error with message
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Tool is already registered
    #[error("Tool already registered: {0}")]
    AlreadyRegistered(String),

    /// Tool is already in the requested state
    #[error("Tool {tool_id} is already in state {state}")]
    AlreadyInState { tool_id: String, state: ToolState },

    /// Resource limit exceeded
    #[error("Resource limit exceeded for {tool_id}: {resource_type} current={current}, limit={limit}")]
    ResourceLimitExceeded {
        tool_id: String,
        resource_type: String,
        current: u64,
        limit: u64,
    },

    /// Tool has no state history
    #[error("No state history for tool: {0}")]
    NoStateHistory(String),

    /// Internal error with message
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Tool state transition error
    #[error("Invalid state transition for {tool_id}: {from_state} -> {to_state}: {message}")]
    InvalidStateTransition {
        tool_id: String,
        from_state: ToolState,
        to_state: ToolState,
        message: String,
    },

    /// Tool manager is not in the expected state
    #[error("Invalid manager state: expected {expected}, got {actual}")]
    InvalidManagerState { expected: String, actual: String },

    /// Invalid state error
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Lifecycle hook error
    #[error("Lifecycle error: {0}")]
    LifecycleError(String),

    /// Registration failed error
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Unregistration failed error
    #[error("Unregistration failed: {0}")]
    UnregistrationFailed(String),

    /// Validation failed error
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    /// Validation error with message
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Resource error with message
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Tool error with message
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Security violation error
    #[error("Security violation: {0}")]
    SecurityViolation(String),

    /// Tool needs reset to recover
    #[error("Tool needs reset: {0}")]
    NeedsReset(String),

    /// Too many errors occurred
    #[error("Too many errors: {0}")]
    TooManyErrors(String),

    /// Capability not found error
    #[error("Capability {1} not found for tool {0}")]
    CapabilityNotFound(String, String),

    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Rollback failed error
    #[error("Rollback failed for {tool_id}: original error: {original_error}, rollback error: {rollback_error}")]
    RollbackFailed {
        tool_id: String,
        original_error: Box<ToolError>,
        rollback_error: Box<ToolError>,
    },
    
    /// No rollback state available error
    #[error("No rollback state available for {tool_id}: {from_state} -> {to_state}")]
    NoRollbackStateAvailable {
        tool_id: String,
        from_state: ToolState,
        to_state: ToolState,
    },
    
    /// Rollback partially successful error
    #[error("Rollback partially successful for {tool_id}: {message} (original error: {original_error})")]
    RollbackPartiallySuccessful {
        tool_id: String,
        original_error: Box<ToolError>,
        message: String,
    },
    
    /// State transition error
    #[error("State transition error: {0}")]
    StateTransition(String),
    
    /// Critical error that requires immediate attention
    #[error("Critical error: {0}")]
    Critical(String),
    
    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

