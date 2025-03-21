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
    ResourceCleanupHook, BasicResourceCleanupHook, EnhancedResourceCleanupHook,
    ResourceTracker, ResourceStatus, ResourceEvent, ResourceType, 
    ResourceUsage, ResourceLimits
};
// Import RecoveryHook from the cleanup module
pub use self::cleanup::recovery::RecoveryHook;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;
use tracing::{error, info, instrument};

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

/// Tool state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolState {
    /// Tool is registered but not active
    Registered,
    /// Tool is active and ready to use
    Active,
    /// Tool is inactive
    Inactive,
    /// Tool is in an error state
    Error,
    /// Tool is unregistered
    Unregistered,
}

impl fmt::Display for ToolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => write!(f, "registered"),
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Error => write!(f, "error"),
            Self::Unregistered => write!(f, "unregistered"),
        }
    }
}

/// Tool error
#[derive(Debug, thiserror::Error, Clone)]
pub enum ToolError {
    /// Error during tool registration
    #[error("Failed to register tool: {0}")]
    RegistrationFailed(String),
    
    /// Error during tool unregistration
    #[error("Failed to unregister tool: {0}")]
    UnregistrationFailed(String),
    
    /// Error during validation
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    /// Error during execution
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Error during lifecycle hook
    #[error("Lifecycle hook failed: {0}")]
    LifecycleError(String),
    
    /// Error when tool is not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    /// Error when capability is not found
    #[error("Capability not found: {0} for tool {1}")]
    CapabilityNotFound(String, String),
    
    /// Error when permission is denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
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
}

/// Tool lifecycle hook
#[async_trait]
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
}

/// Tool manager
#[derive(Debug)]
pub struct ToolManager {
    /// Registered tools
    tools: RwLock<HashMap<String, Tool>>,
    /// Tool states
    states: RwLock<HashMap<String, ToolState>>,
    /// Tool executors
    executors: RwLock<HashMap<String, Arc<dyn ToolExecutor>>>,
    /// Tool capabilities to executor mapping
    capability_map: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Lifecycle hook
    lifecycle_hook: Arc<dyn ToolLifecycleHook>,
}

impl ToolManager {
    /// Creates a new tool manager
    pub fn new(lifecycle_hook: impl ToolLifecycleHook + 'static) -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: Arc::new(lifecycle_hook),
        }
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        // Create a composite lifecycle hook with basic functionality
        let mut lifecycle_hook = CompositeLifecycleHook::new();
        
        lifecycle_hook.add_hook(BasicLifecycleHook::new());
        lifecycle_hook.add_hook(RecoveryHook::new());
        
        // Add the enhanced resource hook
        lifecycle_hook.add_hook(EnhancedResourceCleanupHook::new());
        
        Self::new(lifecycle_hook)
    }
}

impl ToolManager {
    /// Registers a tool
    #[instrument(skip(self, tool, executor))]
    pub async fn register_tool(
        &self,
        tool: Tool,
        executor: impl ToolExecutor + 'static,
    ) -> Result<(), ToolError> {
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
        let tool_id = tool.id.clone();
        
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
            let tool_capabilities = capability_map.entry(tool_id.clone()).or_insert_with(HashMap::new);
            for capability in &tool.capabilities {
                tool_capabilities.insert(capability.name.clone(), tool_id.clone());
            }
        }
        
        info!("Tool registered: {} ({})", tool.name, tool_id);
        Ok(())
    }
    
    /// Unregisters a tool
    #[instrument(skip(self))]
    pub async fn unregister_tool(&self, tool_id: &str) -> Result<(), ToolError> {
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
    
    /// Executes a tool capability
    #[instrument(skip(self, parameters))]
    pub async fn execute_capability(
        &self,
        tool_id: &str,
        capability: &str,
        parameters: HashMap<String, JsonValue>,
        security_token: Option<String>,
        session_id: Option<String>,
    ) -> Result<ToolExecutionResult, ToolError> {
        // Check if the tool exists and is active
        let tool = {
            let tools = self.tools.read().await;
            match tools.get(tool_id) {
                Some(tool) => tool.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            }
        };
        
        let state = {
            let states = self.states.read().await;
            match states.get(tool_id) {
                Some(state) => *state,
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            }
        };
        
        // Check if the tool is active
        if state != ToolState::Active {
            return Err(ToolError::ValidationFailed(format!(
                "Tool '{}' is not active (current state: {})",
                tool_id, state
            )));
        }
        
        // Check if the capability exists
        let capability_exists = tool.capabilities.iter().any(|c| c.name == capability);
        if !capability_exists {
            return Err(ToolError::CapabilityNotFound(capability.to_string(), tool_id.to_string()));
        }
        
        // Get the executor
        let executor = {
            let executors = self.executors.read().await;
            match executors.get(tool_id) {
                Some(executor) => executor.clone(),
                None => return Err(ToolError::ExecutionFailed(format!(
                    "No executor found for tool '{}'",
                    tool_id
                ))),
            }
        };
        
        // Validate the parameters
        let capability_def = tool.capabilities.iter()
            .find(|c| c.name == capability)
            .unwrap(); // Safe because we checked existence above
        
        for param in &capability_def.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                return Err(ToolError::ValidationFailed(format!(
                    "Required parameter '{}' is missing for capability '{}'",
                    param.name, capability
                )));
            }
        }
        
        // Create the context
        let context = ToolContext {
            tool_id: tool_id.to_string(),
            capability: capability.to_string(),
            parameters,
            security_token,
            session_id,
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        match executor.execute(context).await {
            Ok(result) => {
                // Check if execution failed
                if result.status == ExecutionStatus::Failure {
                    // Call the lifecycle hook on error
                    if let Some(error_message) = &result.error_message {
                        let error = ToolError::ExecutionFailed(error_message.clone());
                        let _ = self.lifecycle_hook.on_error(tool_id, &error).await;
                    }
                }
                
                Ok(result)
            },
            Err(error) => {
                // Call the lifecycle hook on error
                let _ = self.lifecycle_hook.on_error(tool_id, &error).await;
                
                // Update the tool state to error
                let _ = self.update_tool_state(tool_id, ToolState::Error).await;
                
                Err(error)
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
