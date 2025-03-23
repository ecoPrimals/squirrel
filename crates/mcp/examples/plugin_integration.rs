// Example demonstrating the MCP tool system with plugin interfaces
//
// This is a simplified example that doesn't rely on external plugin dependencies

use std::sync::Arc;
use anyhow::Result;
use serde_json::json;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use squirrel_mcp::tool::{Tool, ToolManager};
use squirrel_mcp::tool::executor::BasicToolExecutor;
use squirrel_mcp::tool::lifecycle::BasicLifecycleHook;
use squirrel_mcp::plugins::interfaces::Plugin;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting MCP Tools Example");

    // 1. Set up the MCP Tool system
    let tool_manager = Arc::new(ToolManager::builder()
        .lifecycle_hook(BasicLifecycleHook::new())
        .build());

    // 2. Create a sample tool
    let tool = Tool::builder()
        .id("sample-tool")
        .name("Sample Tool")
        .version("1.0.0")
        .description("A sample tool for demonstration")
        .capability(create_sample_capability())
        .security_level(1)
        .build();

    // 3. Create an executor for the tool
    let executor = BasicToolExecutor::new("sample-tool");

    // 4. Register the tool with the tool manager
    tool_manager.register_tool(tool, executor).await?;
    
    // 5. Activate the tool
    tool_manager.activate_tool("sample-tool").await?;
    
    info!("Tool registered and activated. Running tool...");
    
    // 6. Execute the tool directly via tool manager
    let result = tool_manager.execute_tool(
        "sample-tool",
        "echo",
        json!({
            "message": "Hello from Tool Manager"
        }),
        Some(Uuid::new_v4().to_string())
    ).await?;
    
    info!("Tool execution result: {:?}", result);
    
    info!("Example completed successfully.");
    
    Ok(())
}

// Helper function to create a sample capability
fn create_sample_capability() -> squirrel_mcp::tool::Capability {
    use squirrel_mcp::tool::{Capability, Parameter, ParameterType, ReturnType};
    
    Capability {
        name: "echo".to_string(),
        description: "Echoes a message back".to_string(),
        parameters: vec![
            Parameter {
                name: "message".to_string(),
                description: "The message to echo".to_string(),
                parameter_type: ParameterType::String,
                required: true,
            }
        ],
        return_type: Some(ReturnType {
            description: "The echoed message".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string"
                    },
                    "timestamp": {
                        "type": "string",
                        "format": "date-time"
                    }
                }
            }),
        }),
    }
} 