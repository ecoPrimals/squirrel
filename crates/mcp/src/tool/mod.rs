//! Tool management system for MCP
//!
//! This module provides abstractions and implementations for tool management,
//! including registration, execution, and lifecycle hooks.

// Declare submodules
pub mod executor;
pub mod lifecycle;
pub mod cleanup;

// Re-export implementations from modules
pub use self::executor::{BasicToolExecutor, RemoteToolExecutor};
pub use self::lifecycle::{BasicLifecycleHook, CompositeLifecycleHook};
pub use self::cleanup::{
    ResourceManager, BasicResourceManager, ResourceTracker, 
    ResourceUsage, ResourceLimits, CleanupHook, BasicCleanupHook,
    RecoveryHook, RecoveryStrategy
};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;
use std::time::Instant;

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

impl Tool {
    /// Creates a new builder for Tool
    pub fn builder() -> ToolBuilder {
        ToolBuilder::new()
    }
}

/// Builder for Tool
pub struct ToolBuilder {
    id: Option<String>,
    name: Option<String>,
    version: String,
    description: String,
    capabilities: Vec<Capability>,
    security_level: u8,
}

impl ToolBuilder {
    /// Creates a new ToolBuilder with default values
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            version: "0.1.0".to_string(),
            description: "".to_string(),
            capabilities: Vec::new(),
            security_level: 0,
        }
    }
    
    /// Sets the tool ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Sets the tool name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Sets the tool version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Sets the tool description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    
    /// Adds a capability to the tool
    pub fn capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Sets the tool security level
    pub fn security_level(mut self, level: u8) -> Self {
        self.security_level = level;
        self
    }
    
    /// Builds the Tool
    pub fn build(self) -> Tool {
        Tool {
            id: self.id.expect("Tool ID is required"),
            name: self.name.expect("Tool name is required"),
            version: self.version,
            description: self.description,
            capabilities: self.capabilities,
            security_level: self.security_level,
        }
    }
}

impl Default for ToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl fmt::Display for ToolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => write!(f, "registered"),
            Self::Active => write!(f, "active"),
            Self::Starting => write!(f, "starting"),
            Self::Started => write!(f, "started"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Pausing => write!(f, "pausing"),
            Self::Paused => write!(f, "paused"),
            Self::Resuming => write!(f, "resuming"),
            Self::Updating => write!(f, "updating"),
            Self::Error => write!(f, "error"),
            Self::Unregistered => write!(f, "unregistered"),
            Self::Recovering => write!(f, "recovering"),
            Self::Inactive => write!(f, "inactive"),
        }
    }
}

/// Represents errors that can occur during tool operations
#[derive(Debug, Clone)]
pub enum ToolError {
    /// Required dependency is missing
    DependencyNotFound(String),
    
    /// Tool not found error
    ToolNotFound(String),
    
    /// Tool executor not found
    ExecutorNotFound(String),
    
    /// Tool initialization failed
    InitializationFailed { tool_id: String, reason: String },
    
    /// Tool execution failed
    ExecutionFailed { tool_id: String, reason: String },
    
    /// Execution error with message
    ExecutionError(String),
    
    /// Tool is already registered
    AlreadyRegistered(String),
    
    /// Tool is already in the requested state
    AlreadyInState { tool_id: String, state: ToolState },
    
    /// Resource limit exceeded
    ResourceLimitExceeded { 
        tool_id: String, 
        resource_type: String, 
        current: u64, 
        limit: u64 
    },
    
    /// Tool has no state history
    NoStateHistory(String),
    
    /// Internal error with message
    InternalError(String),
    
    /// Tool state transition error
    InvalidStateTransition { 
        tool_id: String, 
        from_state: ToolState, 
        to_state: ToolState 
    },
    
    /// Tool manager is not in the expected state
    InvalidManagerState { expected: String, actual: String },
    
    /// Invalid state error
    InvalidState(String),
    
    /// Lifecycle hook error
    LifecycleError(String),
    
    /// Registration failed error
    RegistrationFailed(String),
    
    /// Unregistration failed error
    UnregistrationFailed(String),
    
    /// Validation failed error
    ValidationFailed(String),
    
    /// Resource error with message
    ResourceError(String),
    
    /// Tool error with message
    ToolError(String),
    
    /// Security violation error
    SecurityViolation(String),
    
    /// Tool needs reset to recover
    NeedsReset(String),
    
    /// Too many errors occurred
    TooManyErrors(String),
    
    /// Capability not found error
    CapabilityNotFound(String, String),
    
    /// Permission denied error
    PermissionDenied(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::DependencyNotFound(msg) => write!(f, "Dependency not found: {}", msg),
            ToolError::ToolNotFound(msg) => write!(f, "Tool not found: {}", msg),
            ToolError::ExecutorNotFound(msg) => write!(f, "Executor not found: {}", msg),
            ToolError::InitializationFailed { tool_id, reason } => 
                write!(f, "Tool '{}' initialization failed: {}", tool_id, reason),
            ToolError::ExecutionFailed { tool_id, reason } => 
                write!(f, "Tool '{}' execution failed: {}", tool_id, reason),
            ToolError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            ToolError::AlreadyRegistered(msg) => write!(f, "Tool already registered: {}", msg),
            ToolError::AlreadyInState { tool_id, state } => 
                write!(f, "Tool '{}' is already in state: {:?}", tool_id, state),
            ToolError::ResourceLimitExceeded { tool_id, resource_type, current, limit } => 
                write!(f, "Tool '{}' exceeded {} limit: {} > {}", tool_id, resource_type, current, limit),
            ToolError::NoStateHistory(msg) => write!(f, "No state history for tool: {}", msg),
            ToolError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ToolError::InvalidStateTransition { tool_id, from_state, to_state } => 
                write!(f, "Invalid state transition for tool '{}': {:?} -> {:?}", tool_id, from_state, to_state),
            ToolError::InvalidManagerState { expected, actual } => 
                write!(f, "Invalid manager state: expected {}, actual {}", expected, actual),
            ToolError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            ToolError::LifecycleError(msg) => write!(f, "Lifecycle error: {}", msg),
            ToolError::RegistrationFailed(msg) => write!(f, "Registration failed: {}", msg),
            ToolError::UnregistrationFailed(msg) => write!(f, "Unregistration failed: {}", msg),
            ToolError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ToolError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            ToolError::ToolError(msg) => write!(f, "Tool error: {}", msg),
            ToolError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            ToolError::NeedsReset(msg) => write!(f, "Tool needs reset: {}", msg),
            ToolError::TooManyErrors(msg) => write!(f, "Too many errors: {}", msg),
            ToolError::CapabilityNotFound(cap, tool) => 
                write!(f, "Capability '{}' not found for tool '{}'", cap, tool),
            ToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}

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

/// Tool context for execution
#[derive(Debug, Clone)]
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

/// Tool executor
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

/// Tool lifecycle hook to handle different phases of tool operation
#[async_trait::async_trait]
pub trait ToolLifecycleHook: fmt::Debug + Send + Sync {
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
        // Default implementation does nothing
        Ok(())
    }

    /// Called after a tool is started
    async fn post_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called before a tool is stopped
    async fn pre_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called after a tool is stopped
    async fn post_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a tool is paused
    async fn on_pause(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a tool is resumed
    async fn on_resume(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when a tool is updated
    async fn on_update(&self, _tool: &Tool) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Called when a tool is being cleaned up
    async fn on_cleanup(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Called when registering a tool (new name for on_register)
    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        // Default implementation calls on_register
        self.on_register(tool).await
    }
    
    /// Called when initializing a tool after registration
    async fn initialize_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Called before executing a tool
    async fn pre_execute(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Called after executing a tool
    async fn post_execute(&self, _tool_id: &str, result: Result<(), ToolError>) -> Result<(), ToolError> {
        // Default implementation just returns the result
        result
    }
    
    /// Called when resetting a tool
    async fn reset_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Called when cleaning up a tool (new name for on_cleanup)
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Default implementation calls on_cleanup
        self.on_cleanup(tool_id).await
    }
}

/// Tool manager
#[derive(Debug)]
pub struct ToolManager {
    /// Map of tool IDs to tools
    tools: RwLock<HashMap<String, Tool>>,
    /// Map of tool IDs to tool states
    states: RwLock<HashMap<String, ToolState>>,
    /// Map of tool IDs to tool executors
    executors: RwLock<HashMap<String, Arc<dyn ToolExecutor>>>,
    /// Map of capability names to tool IDs
    capability_map: RwLock<HashMap<String, HashSet<String>>>,
    /// Tool lifecycle hook
    lifecycle_hook: Arc<dyn ToolLifecycleHook>,
    /// Resource manager for tools
    resource_manager: Arc<dyn ResourceManager>,
    /// Recovery hook for tool errors
    recovery_hook: Option<Arc<RecoveryHook>>,
}

/// Builder for ToolManager
pub struct ToolManagerBuilder {
    lifecycle_hook: Option<Arc<dyn ToolLifecycleHook>>,
    resource_manager: Option<Arc<dyn ResourceManager>>,
    recovery_hook: Option<Arc<RecoveryHook>>,
}

impl ToolManagerBuilder {
    /// Create a new ToolManagerBuilder
    pub fn new() -> Self {
        Self {
            lifecycle_hook: None,
            resource_manager: None,
            recovery_hook: None,
        }
    }
    
    /// Set the lifecycle hook
    pub fn lifecycle_hook(mut self, hook: impl ToolLifecycleHook + 'static) -> Self {
        self.lifecycle_hook = Some(Arc::new(hook));
        self
    }
    
    /// Set the resource manager
    pub fn resource_manager(mut self, manager: impl ResourceManager + 'static) -> Self {
        self.resource_manager = Some(Arc::new(manager));
        self
    }
    
    /// Set the recovery hook
    pub fn recovery_hook(mut self, hook: RecoveryHook) -> Self {
        self.recovery_hook = Some(Arc::new(hook));
        self
    }
    
    /// Build the ToolManager
    pub fn build(self) -> ToolManager {
        ToolManager {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: self.lifecycle_hook.unwrap_or_else(|| Arc::new(BasicLifecycleHook::new())),
            resource_manager: self.resource_manager.unwrap_or_else(|| Arc::new(BasicResourceManager::new())),
            recovery_hook: self.recovery_hook,
        }
    }
}

impl Default for ToolManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    /// Builder for ToolManager
    pub fn builder() -> ToolManagerBuilder {
        ToolManagerBuilder::new()
    }

    /// Creates a new ToolManager with default configuration
    #[instrument]
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: Arc::new(BasicLifecycleHook::new()),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hook: None,
        }
    }

    /// Creates a new ToolManager with a custom lifecycle hook
    #[instrument(skip(lifecycle_hook))]
    pub fn with_lifecycle_hook(lifecycle_hook: impl ToolLifecycleHook + 'static) -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: Arc::new(lifecycle_hook),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hook: None,
        }
    }

    /// Sets a recovery hook for error handling
    pub fn with_recovery_hook(mut self, recovery_hook: RecoveryHook) -> Self {
        self.recovery_hook = Some(Arc::new(recovery_hook));
        self
    }

    /// Registers a tool with the manager
    #[instrument(skip(self, executor))]
    pub async fn register_tool(
        &self,
        tool: Tool,
        executor: impl ToolExecutor + 'static,
    ) -> Result<(), ToolError> {
        let tool_id = tool.id.clone();

        // Initialize resource management with default limits
        let base_limits = cleanup::ResourceLimits {
            max_memory_bytes: 100_000_000, // 100 MB
            max_cpu_time_ms: 30_000,       // 30 seconds
            max_file_handles: 50,
            max_network_connections: 10,
        };

        let max_limits = cleanup::ResourceLimits {
            max_memory_bytes: 500_000_000, // 500 MB
            max_cpu_time_ms: 120_000,      // 120 seconds
            max_file_handles: 200,
            max_network_connections: 50,
        };

        self.resource_manager
            .initialize_tool(&tool_id, base_limits, max_limits)
            .await?;

        // Validate the executor handles this tool
        if executor.get_tool_id() != tool.id {
            return Err(ToolError::ValidationFailed(format!(
                "Executor is for tool '{}', not '{}'",
                executor.get_tool_id(), tool.id
            )));
        }
        
        // Check if all tool capabilities are handled by the executor
        let executor_capabilities = executor.get_capabilities();
        for capability in &tool.capabilities {
            if !executor_capabilities.contains(&capability.name) {
                return Err(ToolError::ValidationFailed(format!(
                    "Executor does not handle capability '{}' for tool '{}'",
                    capability.name, tool.id
                )));
            }
        }
        
        // Call the lifecycle hook
        self.lifecycle_hook.on_register(&tool).await.map_err(|e| {
            ToolError::LifecycleError(format!("Registration hook failed: {}", e))
        })?;
        
        // Register the tool
        {
            let mut tools = self.tools.write().await;
            let mut states = self.states.write().await;
            let mut executors = self.executors.write().await;
            let mut capability_map = self.capability_map.write().await;
            
            // Store the tool
            tools.insert(tool_id.clone(), tool.clone());
            
            // Set the initial state
            states.insert(tool_id.clone(), ToolState::Registered);
            
            // Store the executor
            executors.insert(tool_id.clone(), Arc::new(executor));
            
            // Update the capability map
            let tool_capabilities = capability_map.entry(tool_id.clone()).or_insert_with(HashSet::new);
            for capability in &tool.capabilities {
                tool_capabilities.insert(capability.name.clone());
            }
        }
        
        info!("Tool registered: {} ({})", tool.name, tool_id);
        Ok(())
    }
    
    /// Unregisters a tool from the manager
    #[instrument(skip(self))]
    pub async fn unregister_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Cleanup resources first
        self.resource_manager.cleanup_tool(tool_id).await?;

        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Call the lifecycle hook
        self.lifecycle_hook.on_unregister(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Unregistration hook failed: {}", e))
        })?;
        
        // Unregister the tool
        {
            let mut tools = self.tools.write().await;
            let mut states = self.states.write().await;
            let mut executors = self.executors.write().await;
            let mut capability_map = self.capability_map.write().await;
            
            // Remove the tool
            tools.remove(tool_id);
            
            // Remove the state
            states.remove(tool_id);
            
            // Remove the executor
            executors.remove(tool_id);
            
            // Remove from the capability map
            capability_map.remove(tool_id);
        }
        
        info!("Tool unregistered: {}", tool_id);
        Ok(())
    }
    
    /// Activates a tool
    #[instrument(skip(self))]
    pub async fn activate_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Call the lifecycle hook
        self.lifecycle_hook.on_activate(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Activation hook failed: {}", e))
        })?;
        
        // Activate the tool
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Active);
        }
        
        info!("Tool activated: {}", tool_id);
        Ok(())
    }
    
    /// Deactivates a tool
    #[instrument(skip(self))]
    pub async fn deactivate_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Call the lifecycle hook
        self.lifecycle_hook.on_deactivate(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Deactivation hook failed: {}", e))
        })?;
        
        // Deactivate the tool
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Inactive);
        }
        
        info!("Tool deactivated: {}", tool_id);
        Ok(())
    }
    
    /// Gets a tool
    pub async fn get_tool(&self, tool_id: &str) -> Option<Tool> {
        let tools = self.tools.read().await;
        tools.get(tool_id).cloned()
    }
    
    /// Gets a tool's state
    pub async fn get_tool_state(&self, tool_id: &str) -> Option<ToolState> {
        let states = self.states.read().await;
        states.get(tool_id).copied()
    }
    
    /// Gets all tools
    pub async fn get_all_tools(&self) -> Vec<Tool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }
    
    /// Gets all tool states
    pub async fn get_all_tool_states(&self) -> HashMap<String, ToolState> {
        let states = self.states.read().await;
        states.clone()
    }
    
    /// Updates a tool's state
    #[instrument(skip(self))]
    pub async fn update_tool_state(&self, tool_id: &str, state: ToolState) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Update the state
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), state);
        }
        
        info!("Tool state updated: {} -> {}", tool_id, state);
        Ok(())
    }
    
    /// Executes a tool with the specified capability and parameters
    #[instrument(skip(self, params))]
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        capability: &str,
        params: JsonValue,
        request_id: Option<String>,
    ) -> Result<ToolExecutionResult, ToolError> {
        let request_id = request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Get the executor - need to read the RwLock
        let executor = {
            let executors_guard = self.executors.read().await;
            match executors_guard.get(tool_id) {
                Some(executor) => executor.clone(),
                None => {
                    return Err(ToolError::ExecutorNotFound(tool_id.to_string()));
                }
            }
        };
        
        info!(
            tool_id = tool_id,
            capability = capability,
            request_id = request_id,
            "Executing tool capability"
        );
        
        let start_time = Instant::now();
        
        // Create a tool context with proper field types
        let context = ToolContext {
            tool_id: tool_id.to_string(),
            capability: capability.to_string(),
            request_id: request_id.clone(),
            parameters: params.clone().as_object()
                .map(|obj| obj.clone().into_iter().collect())
                .unwrap_or_default(),
            security_token: Some("default-token".to_string()), // Wrap in Some
            session_id: Some(Uuid::new_v4().to_string()),     // Wrap in Some
            timestamp: chrono::Utc::now(), // Use correct type
        };
        
        // Execute the tool
        match executor.execute(context).await {
            Ok(result) => {
                let duration = start_time.elapsed();
                info!(
                    tool_id = tool_id,
                    capability = capability,
                    request_id = request_id,
                    duration_ms = duration.as_millis(),
                    status = ?result.status,
                    "Tool execution completed"
                );
                
                // Preserve the result from the executor, just update the timing
                let mut updated_result = result;
                updated_result.execution_time_ms = duration.as_millis() as u64;
                
                Ok(updated_result)
            }
            Err(error) => {
                let duration = start_time.elapsed();
                error!(
                    tool_id = tool_id,
                    capability = capability,
                    request_id = request_id,
                    duration_ms = duration.as_millis(),
                    error = ?error,
                    "Tool execution failed"
                );
                
                // If it's a CapabilityNotFound error, propagate it to the caller
                if let ToolError::CapabilityNotFound(_, _) = &error {
                    return Err(error);
                }
                
                Ok(ToolExecutionResult {
                    tool_id: tool_id.to_string(),
                    capability: capability.to_string(),
                    request_id,
                    status: ExecutionStatus::Failure,
                    output: None,
                    error_message: Some(error.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }

    /// Starts a tool
    #[instrument(skip(self))]
    pub async fn start_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Active) | Some(ToolState::Started) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already started",
                        tool_id
                    )));
                }
                Some(ToolState::Starting) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already starting",
                        tool_id
                    )));
                }
                Some(ToolState::Error) => {
                    return Err(ToolError::ToolError(format!(
                        "Tool '{}' is in error state and cannot be started",
                        tool_id
                    )));
                }
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                _ => {} // Continue with other states
            }
        }
        
        // Update state to Starting
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Starting);
        }
        
        // Call pre-start lifecycle hook
        self.lifecycle_hook.pre_start(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Pre-start hook failed: {}", e))
        })?;
        
        // Call the executor's start method
        {
            let executors = self.executors.read().await;
            if let Some(executor) = executors.get(tool_id) {
                executor.start().await.map_err(|e| {
                    ToolError::ExecutionError(format!("Failed to start tool: {}", e))
                })?;
            } else {
                return Err(ToolError::ExecutorNotFound(tool_id.to_string()));
            }
        }
        
        // Update state to Started
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Started);
        }
        
        // Call post-start lifecycle hook
        self.lifecycle_hook.post_start(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Post-start hook failed: {}", e))
        })?;
        
        info!("Tool started: {}", tool_id);
        Ok(())
    }

    /// Stops a tool
    #[instrument(skip(self))]
    pub async fn stop_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Stopped) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already stopped",
                        tool_id
                    )));
                }
                Some(ToolState::Stopping) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already stopping",
                        tool_id
                    )));
                }
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                _ => {} // Continue with other states
            }
        }
        
        // Update state to Stopping
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Stopping);
        }
        
        // Call pre-stop lifecycle hook
        self.lifecycle_hook.pre_stop(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Pre-stop hook failed: {}", e))
        })?;
        
        // Call the executor's stop method
        {
            let executors = self.executors.read().await;
            if let Some(executor) = executors.get(tool_id) {
                executor.stop().await.map_err(|e| {
                    ToolError::ExecutionError(format!("Failed to stop tool: {}", e))
                })?;
            } else {
                return Err(ToolError::ExecutorNotFound(tool_id.to_string()));
            }
        }
        
        // Update state to Stopped
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Stopped);
        }
        
        // Call post-stop lifecycle hook
        self.lifecycle_hook.post_stop(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Post-stop hook failed: {}", e))
        })?;
        
        info!("Tool stopped: {}", tool_id);
        Ok(())
    }

    /// Pauses a tool
    #[instrument(skip(self))]
    pub async fn pause_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Paused) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already paused",
                        tool_id
                    )));
                }
                Some(ToolState::Pausing) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already pausing",
                        tool_id
                    )));
                }
                Some(ToolState::Stopped) | Some(ToolState::Stopping) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is stopped or stopping",
                        tool_id
                    )));
                }
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                _ => {} // Continue with other states
            }
        }
        
        // Update state to Pausing
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Pausing);
        }
        
        // Call the executor's pause method
        {
            let executors = self.executors.read().await;
            if let Some(executor) = executors.get(tool_id) {
                executor.pause().await.map_err(|e| {
                    ToolError::ExecutionError(format!("Failed to pause tool: {}", e))
                })?;
            } else {
                return Err(ToolError::ExecutorNotFound(tool_id.to_string()));
            }
        }
        
        // Update state to Paused
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Paused);
        }
        
        // Call pause lifecycle hook
        self.lifecycle_hook.on_pause(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Pause hook failed: {}", e))
        })?;
        
        info!("Tool paused: {}", tool_id);
        Ok(())
    }

    /// Resumes a tool
    #[instrument(skip(self))]
    pub async fn resume_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Active) | Some(ToolState::Started) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already active",
                        tool_id
                    )));
                }
                Some(ToolState::Resuming) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already resuming",
                        tool_id
                    )));
                }
                Some(ToolState::Stopped) | Some(ToolState::Stopping) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is stopped or stopping",
                        tool_id
                    )));
                }
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                Some(ToolState::Paused) => {} // Expected state
                _ => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is not paused",
                        tool_id
                    )));
                }
            }
        }
        
        // Update state to Resuming
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Resuming);
        }
        
        // Call the executor's resume method
        {
            let executors = self.executors.read().await;
            if let Some(executor) = executors.get(tool_id) {
                executor.resume().await.map_err(|e| {
                    ToolError::ExecutionError(format!("Failed to resume tool: {}", e))
                })?;
            } else {
                return Err(ToolError::ExecutorNotFound(tool_id.to_string()));
            }
        }
        
        // Update state to Active
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Active);
        }
        
        // Call resume lifecycle hook
        self.lifecycle_hook.on_resume(tool_id).await.map_err(|e| {
            ToolError::LifecycleError(format!("Resume hook failed: {}", e))
        })?;
        
        info!("Tool resumed: {}", tool_id);
        Ok(())
    }

    /// Updates a tool
    #[instrument(skip(self, updated_tool))]
    pub async fn update_tool(&self, updated_tool: Tool) -> Result<(), ToolError> {
        let tool_id = updated_tool.id.clone();
        
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(&tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.clone()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(&tool_id) {
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                Some(ToolState::Updating) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already updating",
                        tool_id
                    )));
                }
                _ => {} // Continue with other states
            }
        }
        
        // Update state to Updating
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.clone(), ToolState::Updating);
        }
        
        // Call the lifecycle hook
        self.lifecycle_hook.on_update(&updated_tool).await.map_err(|e| {
            ToolError::LifecycleError(format!("Update hook failed: {}", e))
        })?;
        
        // Update the tool registry
        {
            let mut tools = self.tools.write().await;
            let mut capability_map = self.capability_map.write().await;
            
            // Update the tool
            tools.insert(tool_id.clone(), updated_tool.clone());
            
            // Update capability map
            // First remove old capabilities
            if let Some(old_capabilities) = capability_map.get_mut(&tool_id) {
                old_capabilities.clear();
            }
            
            // Add new capabilities
            let tool_capabilities = capability_map.entry(tool_id.clone()).or_insert_with(HashSet::new);
            for capability in &updated_tool.capabilities {
                tool_capabilities.insert(capability.name.clone());
            }
        }
        
        // Update state to Active
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.clone(), ToolState::Active);
        }
        
        info!("Tool updated: {} ({})", updated_tool.name, tool_id);
        Ok(())
    }

    /// Resets a tool to its initial state
    #[instrument(skip(self))]
    pub async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Stop the tool first if it's running
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Active) | Some(ToolState::Started) => {
                    drop(states); // Release the lock before calling stop_tool
                    self.stop_tool(tool_id).await?;
                }
                Some(ToolState::Paused) => {
                    drop(states); // Release the lock before calling stop_tool
                    
                    // Resume first, then stop
                    self.resume_tool(tool_id).await?;
                    self.stop_tool(tool_id).await?;
                }
                _ => {} // No need to stop if already stopped
            }
        }
        
        // Update state to Registered (initial state)
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Registered);
        }
        
        // Reset the resource tracking
        self.resource_manager.reset_tool(tool_id).await?;
        
        info!("Tool reset: {}", tool_id);
        Ok(())
    }

    /// Recovers a tool from error state
    #[instrument(skip(self))]
    pub async fn recover_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_id) {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }
        
        // Check current state
        {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(ToolState::Error) => {} // Expected state
                Some(ToolState::Recovering) => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is already recovering",
                        tool_id
                    )));
                }
                Some(ToolState::Unregistered) => {
                    return Err(ToolError::ToolNotFound(format!(
                        "Tool '{}' is unregistered",
                        tool_id
                    )));
                }
                _ => {
                    return Err(ToolError::InvalidState(format!(
                        "Tool '{}' is not in error state",
                        tool_id
                    )));
                }
            }
        }
        
        // Update state to Recovering
        {
            let mut states = self.states.write().await;
            states.insert(tool_id.to_string(), ToolState::Recovering);
        }
        
        // Get the recovery strategy
        let strategy = if let Some(recovery_hook) = self.recovery_hook.as_ref() {
            recovery_hook.get_strategy(tool_id)
        } else {
            RecoveryStrategy::Reset // Default strategy
        };
        
        info!("Recovering tool {} with strategy: {}", tool_id, strategy);
        
        // Apply the recovery strategy
        match strategy {
            RecoveryStrategy::Reset => {
                // Reset the tool to its initial state
                self.reset_tool(tool_id).await?;
                
                // Update state back to Registered
                {
                    let mut states = self.states.write().await;
                    states.insert(tool_id.to_string(), ToolState::Registered);
                }
                
                info!("Tool '{}' has been reset", tool_id);
                Ok(())
            },
            RecoveryStrategy::Terminate => {
                // Unregister the tool completely
                let result = self.unregister_tool(tool_id).await;
                if result.is_ok() {
                    info!("Tool '{}' has been terminated and unregistered", tool_id);
                    Ok(())
                } else {
                    error!("Failed to terminate tool '{}': {:?}", tool_id, result);
                    // Set state back to Error
                    {
                        let mut states = self.states.write().await;
                        states.insert(tool_id.to_string(), ToolState::Error);
                    }
                    Err(ToolError::UnregistrationFailed(format!(
                        "Failed to terminate tool '{}' during recovery",
                        tool_id
                    )))
                }
            },
            RecoveryStrategy::Continue => {
                // Just update the state back to Registered and continue
                {
                    let mut states = self.states.write().await;
                    states.insert(tool_id.to_string(), ToolState::Registered);
                }
                
                info!("Tool '{}' recovery ignored, continuing execution", tool_id);
                Ok(())
            },
        }
    }
}

/// Example usage of the tool manager
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use uuid::Uuid;
/// use serde_json::json;
/// use chrono::Utc;
///
/// use mcp::tool::{
///     Tool, Capability, Parameter, ParameterType, 
///     BasicToolExecutor, BasicLifecycleHook, SecurityLifecycleHook,
///     CompositeLifecycleHook, ToolManager
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create the lifecycle hooks
///     let basic_hook = BasicLifecycleHook::new();
///     let security_hook = SecurityLifecycleHook::new()
///         .with_default_security_level(1)
///         .enforce_allowed_tools(false);
///
///     // Create a composite hook with both hooks
///     let mut composite_hook = CompositeLifecycleHook::new();
///     composite_hook.add_hook(basic_hook);
///     composite_hook.add_hook(security_hook);
///
///     // Create the tool manager
///     let manager = ToolManager::new(composite_hook);
///
///     // Create a tool
///     let tool = Tool {
///         id: "calculator".to_string(),
///         name: "Calculator".to_string(),
///         version: "1.0.0".to_string(),
///         description: "A simple calculator tool".to_string(),
///         capabilities: vec![
///             Capability {
///                 name: "add".to_string(),
///                 description: "Adds two numbers".to_string(),
///                 parameters: vec![
///                     Parameter {
///                         name: "a".to_string(),
///                         description: "First number".to_string(),
///                         parameter_type: ParameterType::Number,
///                         required: true,
///                     },
///                     Parameter {
///                         name: "b".to_string(),
///                         description: "Second number".to_string(),
///                         parameter_type: ParameterType::Number,
///                         required: true,
///                     },
///                 ],
///                 return_type: None,
///             },
///             Capability {
///                 name: "subtract".to_string(),
///                 description: "Subtracts two numbers".to_string(),
///                 parameters: vec![
///                     Parameter {
///                         name: "a".to_string(),
///                         description: "First number".to_string(),
///                         parameter_type: ParameterType::Number,
///                         required: true,
///                     },
///                     Parameter {
///                         name: "b".to_string(),
///                         description: "Second number".to_string(),
///                         parameter_type: ParameterType::Number,
///                         required: true,
///                     },
///                 ],
///                 return_type: None,
///             },
///         ],
///         security_level: 1,
///     };
///
///     // Create a basic executor with custom handlers
///     let mut executor = BasicToolExecutor::new("calculator");
///
///     // Register handlers for the capabilities
///     executor.register_handler("add", |context| {
///         if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
///             if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
///                 Ok(json!({ "result": a + b }))
///             } else {
///                 Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
///             }
///         } else {
///             Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
///         }
///     });
///
///     executor.register_handler("subtract", |context| {
///         if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
///             if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
///                 Ok(json!({ "result": a - b }))
///             } else {
///                 Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
///             }
///         } else {
///             Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
///         }
///     });
///
///     // Register the tool with the manager
///     manager.register_tool(tool, executor).await?;
///
///     // Activate the tool
///     manager.activate_tool("calculator").await?;
///
///     // Execute the add capability
///     let mut parameters = HashMap::new();
///     parameters.insert("a".to_string(), json!(5));
///     parameters.insert("b".to_string(), json!(3));
///
///     let result = manager.execute_capability(
///         "calculator",
///         "add",
///         parameters,
///         None,
///         None,
///     ).await?;
///
///     println!("Execution result: {:?}", result);
///
///     // Deactivate the tool
///     manager.deactivate_tool("calculator").await?;
///
///     // Unregister the tool
///     manager.unregister_tool("calculator").await?;
///
///     Ok(())
/// }
/// ```
#[cfg(test)]
mod tests {
    // Test module implementation
} 
