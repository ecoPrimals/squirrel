// Plugin system examples
//
// This file provides examples of using the plugin system within MCP.
// The examples here are for demonstration purposes only.

use std::sync::Arc;
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;
use async_trait::async_trait;

use crate::tool::{Tool, ToolManager};
use crate::tool::lifecycle::BasicLifecycleHook;
use crate::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin};

/// Example function that sets up a basic plugin environment
pub async fn setup_basic_plugin_environment() -> Result<()> {
    // Create a tool manager
    let tool_manager = Arc::new(ToolManager::builder()
        .lifecycle_hook(BasicLifecycleHook::new())
        .build());
        
    // Create a simple tool
    let tool = Tool::builder()
        .id("sample-tool")
        .name("Sample Tool")
        .version("1.0.0")
        .description("A sample tool for testing")
        .security_level(1)
        .build();
        
    // Register the tool
    tool_manager.register_tool(
        tool,
        crate::tool::executor::BasicToolExecutor::new("sample-tool")
    ).await?;
    
    // At this point, we would normally create and register plugin adapters
    // But since we've stripped out the dependencies, this is just a placeholder
    
    Ok(())
}

/// Example plugin implementation
#[derive(Debug)]
struct ExamplePlugin {
    metadata: PluginMetadata,
}

impl ExamplePlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "An example plugin".to_string(),
                status: PluginStatus::Registered,
            },
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl McpPlugin for ExamplePlugin {
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value> {
        // Simple echo implementation
        Ok(json!({
            "success": true,
            "result": message,
            "message": "Hello from example plugin!",
        }))
    }
    
    fn validate_message_schema(&self, message: &serde_json::Value) -> Result<()> {
        if !message.is_object() {
            return Err(anyhow::anyhow!("Message must be an object"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_example_plugin() {
        let plugin = ExamplePlugin::new();
        
        // Test initialize
        let init_result = plugin.initialize().await;
        assert!(init_result.is_ok());
        
        // Test message handling
        let message = json!({
            "type": "test",
            "data": "Hello, world!"
        });
        
        let result = plugin.handle_message(message.clone()).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], json!(true));
        
        // Test validation
        assert!(plugin.validate_message_schema(&message).is_ok());
        assert!(plugin.validate_message_schema(&json!("invalid")).is_err());
        
        // Test shutdown
        let shutdown_result = plugin.shutdown().await;
        assert!(shutdown_result.is_ok());
    }
} 