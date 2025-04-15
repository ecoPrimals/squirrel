//! MCP-AI Tools Integration Example
//!
//! This example demonstrates how to use the MCP-AI Tools integration
//! to register and invoke AI tools within the MCP ecosystem.

use integration::{
    create_mcp_adapter, create_mcp_ai_tools_adapter, create_mcp_ai_tools_adapter_with_config,
    AiMessageType, AiToolInvocation, AiToolResponse, McpAiToolsAdapter, McpAiToolsAdapterError,
    McpAiToolsConfig, ProviderSettings, AiToolResponseStatus
};
use async_trait::async_trait;
use serde_json::json;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Custom tool handler implementation
struct EchoToolHandler;

#[async_trait]
impl ToolHandler for EchoToolHandler {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // This tool simply echoes back the input
        info!("EchoTool invoked with: {:?}", invocation.arguments);
        
        // Simulate some processing time
        sleep(Duration::from_millis(500)).await;
        
        // Return the echoed response
        Ok(AiToolResponse::success(
            invocation.id,
            json!({
                "echo": invocation.arguments,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        ))
    }
}

/// Weather tool handler implementation
struct WeatherToolHandler;

#[async_trait]
impl ToolHandler for WeatherToolHandler {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        info!("WeatherTool invoked with: {:?}", invocation.arguments);
        
        // Extract location from arguments
        let location = invocation
            .arguments
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
        // Simulate API call to weather service
        sleep(Duration::from_millis(1000)).await;
        
        // Return mock weather data
        Ok(AiToolResponse::success(
            invocation.id,
            json!({
                "location": location,
                "temperature": 22.5,
                "conditions": "Sunny",
                "humidity": 45,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        ))
    }
}

/// Tool handler trait needed for the example
#[async_trait]
trait ToolHandler: Send + Sync {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting MCP-AI Tools integration example");
    
    // Get OpenAI API key from environment variable
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        warn!("OPENAI_API_KEY not set, using empty string");
        String::new()
    });
    
    // Create a custom configuration
    let mut config = McpAiToolsConfig::default();
    
    // Update OpenAI provider with API key
    let openai_settings = ProviderSettings::default_openai()
        .with_parameter("temperature".to_string(), json!(0.3));
    
    config = config
        .with_provider("openai".to_string(), openai_settings)
        .with_timeout(15000)
        .with_streaming(true);
    
    // Create MCP adapter
    let mcp_adapter = create_mcp_adapter()?;
    
    // Create MCP-AI Tools adapter with custom configuration
    let ai_tools_adapter = create_mcp_ai_tools_adapter_with_config(mcp_adapter.clone(), config)?;
    
    // Register tools
    register_tools(&ai_tools_adapter)?;
    
    // Create conversation
    let conversation_id = ai_tools_adapter.create_conversation();
    info!("Created conversation with ID: {}", conversation_id);
    
    // Add system message
    ai_tools_adapter.add_message(
        &conversation_id,
        "You are a helpful AI assistant with access to various tools.",
        AiMessageType::System,
    )?;
    
    // Add user message
    ai_tools_adapter.add_message(
        &conversation_id,
        "Hello, can you help me find the weather in New York?",
        AiMessageType::Human,
    )?;
    
    // Invoke weather tool
    let weather_response = ai_tools_adapter
        .invoke_tool(
            &conversation_id,
            "weather",
            json!({
                "location": "New York"
            }),
        )
        .await?;
    
    info!("Weather tool response: {:?}", weather_response);
    
    // Generate AI response
    let ai_response = ai_tools_adapter
        .generate_response(&conversation_id, None, None, None)
        .await?;
    
    info!("AI response: {}", ai_response);
    
    // Invoke echo tool
    let echo_response = ai_tools_adapter
        .invoke_tool(
            &conversation_id,
            "echo",
            json!({
                "message": "This is a test message",
                "data": {
                    "key1": "value1",
                    "key2": 42
                }
            }),
        )
        .await?;
    
    info!("Echo tool response: {:?}", echo_response);
    
    // Add another user message
    ai_tools_adapter.add_message(
        &conversation_id,
        "Thank you for the information!",
        AiMessageType::Human,
    )?;
    
    // Generate final AI response
    let final_response = ai_tools_adapter
        .generate_response(&conversation_id, None, None, None)
        .await?;
    
    info!("Final AI response: {}", final_response);
    
    info!("MCP-AI Tools integration example completed");
    
    Ok(())
}

/// Register tools with the adapter
fn register_tools(adapter: &Arc<McpAiToolsAdapter>) -> Result<(), Box<dyn std::error::Error>> {
    // Register echo tool
    adapter.register_tool(
        "echo",
        "Echoes back the input message",
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back"
                }
            },
            "required": ["message"]
        }),
        EchoToolHandler,
    )?;
    
    // Register weather tool
    adapter.register_tool(
        "weather",
        "Get current weather for a location",
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city or location to get weather for"
                }
            },
            "required": ["location"]
        }),
        WeatherToolHandler,
    )?;
    
    info!("Registered tools: echo, weather");
    
    Ok(())
} 