// MCP plugin adaptation layer
//
// This module provides an adaptation layer between the MCP tool system and the new unified 
// plugin system. It allows existing MCP tools to be used as plugins in the new system.

use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, error, info, warn};

// Use local interfaces instead of squirrel-plugins
use crate::plugins::interfaces::{Plugin, PluginMetadata, McpPlugin};
use crate::tool::{Tool, ToolManager, ToolContext, ToolState};

/// An adapter that converts an MCP Tool to a Plugin
pub struct ToolPluginAdapter {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// The wrapped tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The tool ID in the tool manager
    tool_id: String,
}

impl ToolPluginAdapter {
    /// Create a new tool plugin adapter
    pub fn new(tool: &Tool, tool_manager: Arc<ToolManager>) -> Self {
        let plugin_id = Uuid::new_v4();
        let metadata = PluginMetadata {
            id: plugin_id,
            name: tool.name.clone(),
            version: tool.version.clone(),
            description: tool.description.clone(),
            status: crate::plugins::interfaces::PluginStatus::Registered,
        };
        
        Self {
            metadata,
            tool_manager,
            tool_id: tool.id.clone(),
        }
    }
    
    /// Get the tool ID
    pub fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl std::fmt::Debug for ToolPluginAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolPluginAdapter")
            .field("metadata", &self.metadata)
            .field("tool_id", &self.tool_id)
            .finish()
    }
}

#[async_trait]
impl Plugin for ToolPluginAdapter {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn initialize(&self) -> Result<()> {
        // Ensure the tool is activated in the tool manager
        self.tool_manager.activate_tool(&self.tool_id)
            .await
            .map_err(|e| anyhow!("Failed to activate tool: {}", e))
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Deactivate the tool in the tool manager
        self.tool_manager.deactivate_tool(&self.tool_id)
            .await
            .map_err(|e| anyhow!("Failed to deactivate tool: {}", e))
    }
}

#[async_trait]
impl McpPlugin for ToolPluginAdapter {
    async fn handle_message(&self, message: Value) -> Result<Value> {
        // Extract capability and parameters from message
        let capability = message["capability"].as_str()
            .ok_or_else(|| anyhow!("Missing 'capability' field in message"))?;
        
        let params = message["parameters"].clone();
        
        // Generate a request ID if not provided
        let request_id = message["request_id"].as_str()
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Execute the tool and convert the result to a message
        let result = self.tool_manager.execute_tool(
            &self.tool_id,
            capability,
            params,
            Some(request_id),
        ).await
        .map_err(|e| anyhow!("Tool execution failed: {}", e))?;
        
        // Convert the result to a proper response message
        let response = serde_json::json!({
            "success": matches!(result.status, crate::tool::ExecutionStatus::Success),
            "request_id": result.request_id,
            "capability": result.capability,
            "result": result.output,
            "error": result.error_message,
            "execution_time_ms": result.execution_time_ms,
        });
        
        Ok(response)
    }
    
    fn validate_message_schema(&self, message: &Value) -> Result<()> {
        // Basic schema validation
        if !message.is_object() {
            return Err(anyhow!("Message must be a JSON object"));
        }
        
        if message.get("capability").is_none() {
            return Err(anyhow!("Message must contain a 'capability' field"));
        }
        
        if message.get("parameters").is_none() {
            return Err(anyhow!("Message must contain a 'parameters' field"));
        }
        
        Ok(())
    }
}

/// Factory to create plugin adapters from tools
pub struct ToolPluginFactory {
    tool_manager: Arc<ToolManager>,
}

impl ToolPluginFactory {
    /// Create a new tool plugin factory
    pub fn new(tool_manager: Arc<ToolManager>) -> Self {
        Self { tool_manager }
    }
    
    /// Create plugin adapters for all active tools
    pub async fn create_plugin_adapters(&self) -> Result<Vec<ToolPluginAdapter>> {
        let mut adapters = Vec::new();
        
        let tools = self.tool_manager.get_all_tools().await;
        let states = self.tool_manager.get_all_tool_states().await;
        
        for tool in tools {
            // Only create adapters for active tools
            if let Some(state) = states.get(&tool.id) {
                if matches!(state, ToolState::Active | ToolState::Started) {
                    adapters.push(ToolPluginAdapter::new(&tool, self.tool_manager.clone()));
                }
            }
        }
        
        Ok(adapters)
    }
    
    /// Create a plugin adapter for a specific tool
    pub async fn create_plugin_adapter(&self, tool_id: &str) -> Result<ToolPluginAdapter> {
        let tool = self.tool_manager.get_tool(tool_id).await
            .ok_or_else(|| anyhow!("Tool not found: {}", tool_id))?;
        
        Ok(ToolPluginAdapter::new(&tool, self.tool_manager.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Capability, Parameter, ParameterType, ReturnType};
    use crate::tool::executor::BasicToolExecutor;
    use crate::tool::lifecycle::BasicLifecycleHook;
    use serde_json::json;
    
    // Helper function to create a test tool
    fn create_test_tool() -> Tool {
        Tool::builder()
            .id("test-tool")
            .name("Test Tool")
            .version("1.0.0")
            .description("A test tool")
            .capability(Capability {
                name: "test".to_string(),
                description: "Test capability".to_string(),
                parameters: vec![
                    Parameter {
                        name: "input".to_string(),
                        description: "Test input".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    }
                ],
                return_type: Some(ReturnType {
                    description: "Test output".to_string(),
                    schema: json!({}),
                })
            })
            .security_level(1)
            .build()
    }
    
    #[tokio::test]
    async fn test_adapter_metadata() {
        let tool = create_test_tool();
        let tool_manager = Arc::new(ToolManager::builder()
            .lifecycle_hook(BasicLifecycleHook::new())
            .build());
            
        let adapter = ToolPluginAdapter::new(&tool, tool_manager);
        
        // Verify metadata
        let metadata = adapter.metadata();
        assert_eq!(metadata.name, "Test Tool");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "A test tool");
    }
    
    #[tokio::test]
    async fn test_validate_message_schema() {
        let tool = create_test_tool();
        let tool_manager = Arc::new(ToolManager::builder()
            .lifecycle_hook(BasicLifecycleHook::new())
            .build());
            
        let adapter = ToolPluginAdapter::new(&tool, tool_manager);
        
        // Valid message
        let valid_message = json!({
            "capability": "test",
            "parameters": {
                "input": "test value"
            },
            "request_id": "123"
        });
        
        // Invalid messages
        let invalid_message1 = json!({
            "parameters": {
                "input": "test value"
            }
        });
        
        let invalid_message2 = json!({
            "capability": "test"
        });
        
        // Check validation
        assert!(adapter.validate_message_schema(&valid_message).is_ok());
        assert!(adapter.validate_message_schema(&invalid_message1).is_err());
        assert!(adapter.validate_message_schema(&invalid_message2).is_err());
    }
} 