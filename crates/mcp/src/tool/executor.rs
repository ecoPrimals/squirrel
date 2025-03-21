//! Tool execution implementations for MCP
//!
//! This module provides concrete executor implementations for the tool management system.

use std::collections::HashMap;
use chrono::Utc;
use serde_json::json;
use async_trait::async_trait;
use tracing::{debug, info, instrument, warn};

use crate::tool::{
    ToolContext, 
    ToolError, 
    ToolExecutionResult, 
    ToolExecutor, 
    ExecutionStatus
};

/// A simple tool executor that performs basic operations
pub struct BasicToolExecutor {
    /// Tool ID this executor is associated with
    tool_id: String,
    /// Capabilities this executor can handle
    capabilities: Vec<String>,
    /// Custom handlers for specific capabilities
    handlers: HashMap<String, Box<dyn Fn(ToolContext) -> Result<serde_json::Value, ToolError> + Send + Sync>>,
}

impl std::fmt::Debug for BasicToolExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicToolExecutor")
            .field("tool_id", &self.tool_id)
            .field("capabilities", &self.capabilities)
            .field("handlers", &format!("<{} handlers>", self.handlers.len()))
            .finish()
    }
}

impl BasicToolExecutor {
    /// Creates a new basic tool executor
    pub fn new(tool_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            capabilities: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    /// Adds a capability this executor can handle
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Registers a handler for a specific capability
    pub fn register_handler<F>(&mut self, capability: impl Into<String>, handler: F)
    where
        F: Fn(ToolContext) -> Result<serde_json::Value, ToolError> + Send + Sync + 'static,
    {
        let capability_string = capability.into();
        self.capabilities.push(capability_string.clone());
        self.handlers.insert(capability_string, Box::new(handler));
    }
}

#[async_trait]
impl ToolExecutor for BasicToolExecutor {
    #[instrument(skip(self))]
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        debug!("Executing capability '{}' for tool '{}'", context.capability, context.tool_id);
        
        // Check if this executor handles the given tool
        if context.tool_id != self.tool_id {
            return Err(ToolError::ExecutionFailed(format!(
                "This executor handles tool '{}', not '{}'",
                self.tool_id, context.tool_id
            )));
        }
        
        // Check if this executor handles the given capability
        if !self.capabilities.contains(&context.capability) {
            return Err(ToolError::ExecutionFailed(format!(
                "This executor does not handle the capability '{}'",
                context.capability
            )));
        }
        
        // Execute the capability
        let result = if let Some(handler) = self.handlers.get(&context.capability) {
            // Use the registered handler
            handler(context.clone())
        } else {
            // Default implementation for capabilities without specific handlers
            match context.capability.as_str() {
                "echo" => {
                    // Echo back the parameters
                    if let Some(message) = context.parameters.get("message") {
                        Ok(json!({
                            "message": message,
                            "timestamp": Utc::now().to_rfc3339(),
                        }))
                    } else {
                        Err(ToolError::ValidationFailed(
                            "Echo capability requires a 'message' parameter".to_string()
                        ))
                    }
                },
                "info" => {
                    // Return information about the tool
                    Ok(json!({
                        "tool_id": context.tool_id,
                        "capability": context.capability,
                        "timestamp": Utc::now().to_rfc3339(),
                    }))
                },
                _ => Err(ToolError::ExecutionFailed(format!(
                    "No implementation available for capability '{}'",
                    context.capability
                ))),
            }
        };
        
        // Create the execution result
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(output) => {
                info!(
                    "Successfully executed capability '{}' for tool '{}' in {}ms",
                    context.capability, context.tool_id, execution_time
                );
                
                Ok(ToolExecutionResult {
                    tool_id: context.tool_id,
                    capability: context.capability,
                    request_id: context.request_id,
                    status: ExecutionStatus::Success,
                    output: Some(output),
                    error_message: None,
                    execution_time_ms: execution_time,
                    timestamp: Utc::now(),
                })
            },
            Err(err) => {
                warn!(
                    "Failed to execute capability '{}' for tool '{}': {}",
                    context.capability, context.tool_id, err
                );
                
                Ok(ToolExecutionResult {
                    tool_id: context.tool_id,
                    capability: context.capability,
                    request_id: context.request_id,
                    status: ExecutionStatus::Failure,
                    output: None,
                    error_message: Some(err.to_string()),
                    execution_time_ms: execution_time,
                    timestamp: Utc::now(),
                })
            },
        }
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

/// A remote tool executor that delegates to an external service
pub struct RemoteToolExecutor {
    /// Tool ID this executor is associated with
    tool_id: String,
    /// Capabilities this executor can Handle
    capabilities: Vec<String>,
    /// Base URL for the remote service
    base_url: String,
    /// Request timeout in milliseconds
    timeout_ms: u64,
}

impl std::fmt::Debug for RemoteToolExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteToolExecutor")
            .field("tool_id", &self.tool_id)
            .field("capabilities", &self.capabilities)
            .field("base_url", &self.base_url)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

impl RemoteToolExecutor {
    /// Creates a new remote tool executor
    pub fn new(tool_id: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            capabilities: Vec::new(),
            base_url: base_url.into(),
            timeout_ms: 30000, // 30 seconds default timeout
        }
    }
    
    /// Adds a capability this executor can handle
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    /// Sets the request timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[async_trait]
impl ToolExecutor for RemoteToolExecutor {
    #[instrument(skip(self))]
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        debug!("Executing remote capability '{}' for tool '{}'", context.capability, context.tool_id);
        
        // Check if this executor handles the given tool
        if context.tool_id != self.tool_id {
            return Err(ToolError::ExecutionFailed(format!(
                "This executor handles tool '{}', not '{}'",
                self.tool_id, context.tool_id
            )));
        }
        
        // Check if this executor handles the given capability
        if !self.capabilities.contains(&context.capability) {
            return Err(ToolError::ExecutionFailed(format!(
                "This executor does not handle the capability '{}'",
                context.capability
            )));
        }
        
        // In a real implementation, we would send an HTTP request to the remote service
        // For demonstration purposes, we'll simulate a remote call with a delay
        
        // Create the request payload
        let payload = json!({
            "tool_id": context.tool_id,
            "capability": context.capability,
            "parameters": context.parameters,
            "request_id": context.request_id,
            "timestamp": context.timestamp,
        });
        
        debug!("Remote request payload: {:?}", payload);
        
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Simulate remote execution
        // In a real implementation, we would parse the HTTP response
        let result = match context.capability.as_str() {
            "remote_echo" => {
                if let Some(message) = context.parameters.get("message") {
                    Ok(json!({
                        "message": message,
                        "remote": true,
                        "timestamp": Utc::now().to_rfc3339(),
                    }))
                } else {
                    Err(ToolError::ValidationFailed(
                        "Remote echo capability requires a 'message' parameter".to_string()
                    ))
                }
            },
            "remote_compute" => {
                if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
                    if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
                        if let Some(op) = context.parameters.get("operation").and_then(|v| v.as_str()) {
                            match op {
                                "add" => Ok(json!({ "result": a + b })),
                                "subtract" => Ok(json!({ "result": a - b })),
                                "multiply" => Ok(json!({ "result": a * b })),
                                "divide" => {
                                    if b == 0.0 {
                                        Err(ToolError::ExecutionFailed("Division by zero".to_string()))
                                    } else {
                                        Ok(json!({ "result": a / b }))
                                    }
                                },
                                _ => Err(ToolError::ValidationFailed(format!(
                                    "Unsupported operation: {}", op
                                ))),
                            }
                        } else {
                            Err(ToolError::ValidationFailed(
                                "Remote compute capability requires an 'operation' parameter".to_string()
                            ))
                        }
                    } else {
                        Err(ToolError::ValidationFailed(
                            "Remote compute capability requires a 'b' parameter".to_string()
                        ))
                    }
                } else {
                    Err(ToolError::ValidationFailed(
                        "Remote compute capability requires an 'a' parameter".to_string()
                    ))
                }
            },
            _ => Err(ToolError::ExecutionFailed(format!(
                "No implementation available for remote capability '{}'",
                context.capability
            ))),
        };
        
        // Create the execution result
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(output) => {
                info!(
                    "Successfully executed remote capability '{}' for tool '{}' in {}ms",
                    context.capability, context.tool_id, execution_time
                );
                
                Ok(ToolExecutionResult {
                    tool_id: context.tool_id,
                    capability: context.capability,
                    request_id: context.request_id,
                    status: ExecutionStatus::Success,
                    output: Some(output),
                    error_message: None,
                    execution_time_ms: execution_time,
                    timestamp: Utc::now(),
                })
            },
            Err(err) => {
                warn!(
                    "Failed to execute remote capability '{}' for tool '{}': {}",
                    context.capability, context.tool_id, err
                );
                
                Ok(ToolExecutionResult {
                    tool_id: context.tool_id,
                    capability: context.capability,
                    request_id: context.request_id,
                    status: ExecutionStatus::Failure,
                    output: None,
                    error_message: Some(err.to_string()),
                    execution_time_ms: execution_time,
                    timestamp: Utc::now(),
                })
            },
        }
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_basic_executor() {
        let mut executor = BasicToolExecutor::new("test-tool")
            .with_capability("echo")
            .with_capability("info");
        
        // Register a custom handler
        executor.register_handler("custom", |context| {
            if let Some(value) = context.parameters.get("value") {
                Ok(json!({
                    "processed_value": format!("Processed: {}", value),
                    "timestamp": Utc::now().to_rfc3339(),
                }))
            } else {
                Err(ToolError::ValidationFailed("Missing 'value' parameter".to_string()))
            }
        });
        
        // Create a test context
        let context = ToolContext {
            tool_id: "test-tool".to_string(),
            capability: "echo".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("message".to_string(), json!("Hello, world!"));
                params
            },
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Execution failed: {:?}", result);
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Success);
        assert!(execution_result.output.is_some());
        assert_eq!(execution_result.output.as_ref().unwrap()["message"], json!("Hello, world!"));
        
        // Test the custom handler
        let context = ToolContext {
            tool_id: "test-tool".to_string(),
            capability: "custom".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("value".to_string(), json!(42));
                params
            },
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Execution failed: {:?}", result);
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Success);
        assert!(execution_result.output.is_some());
        assert_eq!(execution_result.output.as_ref().unwrap()["processed_value"], json!("Processed: 42"));
        
        // Test parameter validation
        let context = ToolContext {
            tool_id: "test-tool".to_string(),
            capability: "echo".to_string(),
            parameters: HashMap::new(), // Missing required 'message' parameter
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Execution result should be created even for validation failures");
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Failure);
        assert!(execution_result.output.is_none());
        assert!(execution_result.error_message.is_some());
    }
    
    #[tokio::test]
    async fn test_remote_executor() {
        let executor = RemoteToolExecutor::new("remote-tool", "https://example.com/api")
            .with_capability("remote_echo")
            .with_capability("remote_compute")
            .with_timeout(5000);
        
        // Test remote echo
        let context = ToolContext {
            tool_id: "remote-tool".to_string(),
            capability: "remote_echo".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("message".to_string(), json!("Hello from remote!"));
                params
            },
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Remote execution failed: {:?}", result);
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Success);
        assert!(execution_result.output.is_some());
        assert_eq!(execution_result.output.as_ref().unwrap()["message"], json!("Hello from remote!"));
        assert_eq!(execution_result.output.as_ref().unwrap()["remote"], json!(true));
        
        // Test remote compute
        let context = ToolContext {
            tool_id: "remote-tool".to_string(),
            capability: "remote_compute".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("a".to_string(), json!(10));
                params.insert("b".to_string(), json!(5));
                params.insert("operation".to_string(), json!("multiply"));
                params
            },
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Remote execution failed: {:?}", result);
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Success);
        assert!(execution_result.output.is_some());
        
        // Using as_f64() to compare the numeric values directly rather than comparing JSON values
        let result_value = execution_result.output.as_ref().unwrap()["result"].as_f64().unwrap();
        assert_eq!(result_value, 50.0);
        
        // Test error case: division by zero
        let context = ToolContext {
            tool_id: "remote-tool".to_string(),
            capability: "remote_compute".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("a".to_string(), json!(10));
                params.insert("b".to_string(), json!(0));
                params.insert("operation".to_string(), json!("divide"));
                params
            },
            security_token: None,
            session_id: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        };
        
        // Execute the capability
        let result = executor.execute(context).await;
        assert!(result.is_ok(), "Remote execution result should be created even for errors");
        
        let execution_result = result.unwrap();
        assert_eq!(execution_result.status, ExecutionStatus::Failure);
        assert!(execution_result.output.is_none());
        assert!(execution_result.error_message.is_some());
        assert!(execution_result.error_message.as_ref().unwrap().contains("Division by zero"));
    }
} 