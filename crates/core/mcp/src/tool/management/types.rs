//! Core Types for Tool Management
//!
//! This module contains all the fundamental types, enums, and data structures
//! used throughout the tool management system.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::any::Any;

/// Basic tool information for MCP core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
}

/// Tool capability parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Number parameter
    Number,
    /// Boolean parameter
    Boolean,
    /// Object parameter
    Object,
    /// Array parameter
    Array,
    /// Any type parameter
    Any,
}

impl fmt::Display for ParameterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Object => write!(f, "object"),
            Self::Array => write!(f, "array"),
            Self::Any => write!(f, "any"),
        }
    }
}

/// Tool capability parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub parameter_type: ParameterType,
    /// Whether the parameter is required
    pub required: bool,
}

/// Tool capability return type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnType {
    /// Return type description
    pub description: String,
    /// Return type schema
    pub schema: JsonValue,
}

/// Tool capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability parameters
    pub parameters: Vec<Parameter>,
    /// Capability return type
    pub return_type: Option<ReturnType>,
    /// Security level for this capability (0-10, 0 being lowest)
    pub security_level: Option<u8>,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Tool capabilities
    pub capabilities: Vec<Capability>,
    /// Tool security level (0-10, 0 being lowest)
    pub security_level: u8,
}

/// Tool state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolState {
    /// Tool is registered but not active
    Registered,
    /// Tool is active and ready to execute
    Active,
    /// Tool is in the starting process
    Starting,
    /// Tool has started
    Started,
    /// Tool is in the stopping process
    Stopping,
    /// Tool has been stopped
    Stopped,
    /// Tool is in the pausing process
    Pausing,
    /// Tool is paused
    Paused,
    /// Tool is in the resuming process
    Resuming,
    /// Tool is being updated
    Updating,
    /// Tool is in error state
    Error,
    /// Tool is unregistered
    Unregistered,
    /// Tool is in recovery process
    Recovering,
    /// Tool is inactive (but still registered)
    Inactive,
    /// Tool is running (executing)
    Running,
    /// Tool is initializing
    Initializing,
    /// Tool is resetting
    Resetting,
}

impl fmt::Display for ToolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => write!(f, "Registered"),
            Self::Active => write!(f, "Active"),
            Self::Starting => write!(f, "Starting"),
            Self::Started => write!(f, "Started"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Pausing => write!(f, "Pausing"),
            Self::Paused => write!(f, "Paused"),
            Self::Resuming => write!(f, "Resuming"),
            Self::Updating => write!(f, "Updating"),
            Self::Error => write!(f, "Error"),
            Self::Unregistered => write!(f, "Unregistered"),
            Self::Recovering => write!(f, "Recovering"),
            Self::Inactive => write!(f, "Inactive"),
            Self::Running => write!(f, "Running"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Resetting => write!(f, "Resetting"),
        }
    }
}

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

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution was successful
    Success,
    /// Execution failed
    Failure,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    Timeout,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Tool ID
    pub tool_id: String,
    /// Capability name
    pub capability: String,
    /// Request ID
    pub request_id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Execution output
    pub output: Option<JsonValue>,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Timestamp when the execution completed
    pub timestamp: DateTime<Utc>,
}

/// Tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// Tool ID
    pub tool_id: String,
    /// Capability name
    pub capability: String,
    /// Capability parameters
    pub parameters: HashMap<String, JsonValue>,
    /// Security token
    pub security_token: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Trait for tool executors
#[async_trait]
pub trait ToolExecutor: fmt::Debug + Send + Sync {
    /// Executes a capability with the given context
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, ToolError>;

    /// Gets the tool ID this executor is associated with
    fn get_tool_id(&self) -> String;

    /// Gets the capabilities this executor can handle
    fn get_capabilities(&self) -> Vec<String>;

    /// Starts the executor
    async fn start(&self) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Stops the executor
    async fn stop(&self) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Pauses the executor
    async fn pause(&self) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Resumes the executor
    async fn resume(&self) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Trait for tool lifecycle hooks
#[async_trait]
pub trait ToolLifecycleHook: fmt::Debug + Send + Sync {
    /// Converts to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Called when a tool is registered
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError>;

    /// Called when a tool is unregistered
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError>;

    /// Called when a tool is activated
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError>;

    /// Called when a tool is deactivated
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError>;

    /// Called when a tool encounters an error
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError>;

    /// Called before a tool is started
    async fn pre_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called after a tool is started
    async fn post_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called before a tool is stopped
    async fn pre_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called after a tool is stopped
    async fn post_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when a tool is paused
    async fn on_pause(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when a tool is resumed
    async fn on_resume(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when a tool is updated
    async fn on_update(&self, _tool: &Tool) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when a tool is being cleaned up
    async fn on_cleanup(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when registering a tool (new name for on_register)
    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        self.on_register(tool).await
    }

    /// Called when initializing a tool after registration
    async fn initialize_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called before executing a tool
    async fn pre_execute(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called after executing a tool
    async fn post_execute(
        &self,
        _tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        result
    }

    /// Called when resetting a tool
    async fn reset_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    /// Called when cleaning up a tool (new name for on_cleanup)
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        self.on_cleanup(tool_id).await
    }
} 