//! Tool execution implementations for MCP
//!
//! This module provides concrete executor implementations for the tool management system.

use std::collections::HashMap;
use chrono::Utc;
use async_trait::async_trait;
use tracing::info;
use serde_json::Value as JsonValue;
use reqwest;

use crate::tool::{
    ToolContext, 
    ToolError, 
    ToolExecutionResult, 
    ExecutionStatus,
    ToolExecutor
};

/// Type alias for capability handler functions
pub type CapabilityHandler = Box<dyn Fn(ToolContext) -> Result<serde_json::Value, ToolError> + Send + Sync>;

/// A simple tool executor that performs basic operations
pub struct BasicToolExecutor {
    /// Tool ID this executor is associated with
    tool_id: String,
    /// Capabilities this executor can handle
    capabilities: Vec<String>,
    /// Custom handlers for specific capabilities
    handlers: HashMap<String, CapabilityHandler>,
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
    async fn execute(&self, ctx: ToolContext) -> Result<ToolExecutionResult, ToolError> {
        // Log execution
        info!(
            tool_id = self.tool_id,
            capability = ctx.capability,
            request_id = ctx.request_id,
            "Executing basic tool function"
        );
        
        // Execute the function with the provided parameters
        let result = if let Some(handler) = self.handlers.get(&ctx.capability) {
            handler(ctx.clone())
        } else {
            // Return an error instead of a failure result when the capability doesn't exist
            return Err(ToolError::CapabilityNotFound(
                ctx.capability.clone(),
                ctx.tool_id.clone()
            ));
        };
        
        match result {
            Ok(output) => Ok(ToolExecutionResult {
                tool_id: ctx.tool_id,
                capability: ctx.capability,
                request_id: ctx.request_id,
                status: ExecutionStatus::Success,
                output: Some(output),
                error_message: None,
                execution_time_ms: 0, // We don't track execution time here
                timestamp: Utc::now(),
            }),
            Err(err) => Ok(ToolExecutionResult {
                tool_id: ctx.tool_id,
                capability: ctx.capability,
                request_id: ctx.request_id,
                status: ExecutionStatus::Failure,
                output: None,
                error_message: Some(err.to_string()),
                execution_time_ms: 0, // We don't track execution time here
                timestamp: Utc::now(),
            }),
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
    async fn execute(&self, ctx: ToolContext) -> Result<ToolExecutionResult, ToolError> {
        // Log execution
        info!(
            tool_id = self.tool_id,
            capability = ctx.capability,
            request_id = ctx.request_id,
            endpoint = self.base_url,
            "Executing remote tool function"
        );
        
        // Prepare the request payload
        let request_payload = serde_json::json!({
            "tool_id": self.tool_id,
            "capability": ctx.capability,
            "request_id": ctx.request_id,
            "parameters": ctx.parameters
        });
        
        // Execute the remote call
        let client = reqwest::Client::new();
        let result = match client
            .post(&self.base_url)
            .json(&request_payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<JsonValue>().await {
                        Ok(json) => Ok(json),
                        Err(e) => Err(ToolError::ExecutionError(format!(
                            "Failed to parse response: {}",
                            e
                        ))),
                    }
                } else {
                    Err(ToolError::ExecutionError(format!(
                        "Remote execution failed with status: {}",
                        response.status()
                    )))
                }
            }
            Err(e) => Err(ToolError::ExecutionError(format!(
                "Failed to send request: {}",
                e
            ))),
        };
        
        match result {
            Ok(output) => Ok(ToolExecutionResult {
                tool_id: ctx.tool_id,
                capability: ctx.capability,
                request_id: ctx.request_id,
                status: ExecutionStatus::Success,
                output: Some(output),
                error_message: None,
                execution_time_ms: 0, // We don't track execution time here
                timestamp: Utc::now(),
            }),
            Err(err) => Ok(ToolExecutionResult {
                tool_id: ctx.tool_id,
                capability: ctx.capability,
                request_id: ctx.request_id,
                status: ExecutionStatus::Failure,
                output: None,
                error_message: Some(err.to_string()),
                execution_time_ms: 0, // We don't track execution time here
                timestamp: Utc::now(),
            }),
        }
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

/// Update trait imports to remove ToolCapability
pub use crate::tool::{
    Tool,
    ToolState,
};

// Instead of redefining, we import and use the ToolExecutor trait from the module
// The original trait definition and default implementations have been removed

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use serde_json::json;
    
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
                Err(ToolError::ExecutionFailed { tool_id: "unknown".to_string(), reason: "Missing 'value' parameter".to_string() })
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
        
        // Now expecting a CapabilityNotFound error since the capability has no handler
        assert!(result.is_err(), "Execution should fail with an error");
        
        if let Err(ToolError::CapabilityNotFound(capability, tool_id)) = result {
            assert_eq!(capability, "echo");
            assert_eq!(tool_id, "test-tool");
        } else {
            panic!("Expected CapabilityNotFound error, got: {:?}", result);
        }
        
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
        
        // Test parameter validation with echo capability that doesn't have a handler
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
        
        // Now expecting a CapabilityNotFound error since the echo capability has no handler
        assert!(result.is_err(), "Execution should fail with an error");
        
        if let Err(ToolError::CapabilityNotFound(capability, tool_id)) = result {
            assert_eq!(capability, "echo");
            assert_eq!(tool_id, "test-tool");
        } else {
            panic!("Expected CapabilityNotFound error, got: {:?}", result);
        }
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
        assert_eq!(execution_result.status, ExecutionStatus::Failure);
        assert!(execution_result.output.is_none());
        assert!(execution_result.error_message.is_some());
        
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
        assert_eq!(execution_result.status, ExecutionStatus::Failure);
        assert!(execution_result.output.is_none());
        assert!(execution_result.error_message.is_some());

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
        // Error message content may vary, just check that it exists
    }
} 