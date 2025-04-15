use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use squirrel_integration::mcp_ai_tools::{
    adapter::{ToolCallbacks, ToolHandlerV2},
    types::{AiMessageType, AiToolInvocation, AiToolResponse},
    error::McpAiToolsAdapterError,
    config::McpAiToolsConfig,
    adapter::McpAiToolsAdapter,
};
use squirrel_mcp::{adapter::create_default_mcp_adapter, MCPInterface};
use std::time::Duration;
use tokio::time::sleep;

/// Weather data structure
#[derive(Debug)]
struct WeatherData {
    temperature: f64,
    conditions: String,
    humidity: u8,
}

/// Weather tool handler using the new pattern
#[derive(Debug, Default)]
struct WeatherToolHandlerV2 {
    // Use Option<Box<...>> since this is how ToolCallbacks stores it
    add_message: Option<Box<dyn Fn(&str, &str, AiMessageType) -> Result<String, McpAiToolsAdapterError> + Send + Sync>>,
}

#[async_trait]
impl ToolHandlerV2 for WeatherToolHandlerV2 {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Extract location from arguments
        let arguments = invocation.arguments;
        let location = arguments
            .get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpAiToolsAdapterError::ToolInvocation {
                tool: invocation.name.clone(),
                message: "Missing location parameter".to_string(),
            })?;
        
        // Simulate weather API call
        let weather_data = simulate_weather_api_call(location).await?;
        
        // Log the operation if callbacks are available
        if let Some(add_message) = &self.add_message {
            let _ = add_message(
                "system",
                &format!("Retrieved weather data for {}: {:?}", location, weather_data),
                AiMessageType::System,
            );
        }
        
        // Return the response
        Ok(AiToolResponse::success(&invocation.name, json!({
            "location": location,
            "temperature": weather_data.temperature,
            "conditions": weather_data.conditions,
            "humidity": weather_data.humidity,
        })))
    }
    
    fn register_callbacks(&mut self, callbacks: ToolCallbacks) {
        // Store only the callbacks we need
        self.add_message = callbacks.add_message;
    }
}

/// Helper function to simulate a weather API call
async fn simulate_weather_api_call(location: &str) -> Result<WeatherData, McpAiToolsAdapterError> {
    // In a real implementation, this would call an actual weather API
    sleep(Duration::from_millis(100)).await;
    
    // Return simulated data based on location
    match location.to_lowercase().as_str() {
        "new york" => Ok(WeatherData {
            temperature: 72.5,
            conditions: "Partly Cloudy".to_string(),
            humidity: 65,
        }),
        "london" => Ok(WeatherData {
            temperature: 58.3,
            conditions: "Rainy".to_string(),
            humidity: 80,
        }),
        "tokyo" => Ok(WeatherData {
            temperature: 68.0,
            conditions: "Clear".to_string(),
            humidity: 70,
        }),
        _ => Ok(WeatherData {
            temperature: 65.0,
            conditions: "Unknown".to_string(),
            humidity: 50,
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP adapter
    let mcp_adapter = Arc::new(create_default_mcp_adapter());
    
    // Create MCP-AI Tools adapter
    let config = McpAiToolsConfig::default();
    let ai_tools_adapter = Arc::new(McpAiToolsAdapter::with_config(mcp_adapter, config));
    
    // Register weather tool using new pattern
    ai_tools_adapter.register_tool_v2(
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
        WeatherToolHandlerV2::default(),
    )?;
    
    // Create a conversation
    let conversation_id = ai_tools_adapter.create_conversation();
    
    // Add a user message
    ai_tools_adapter.add_message(
        &conversation_id,
        "What's the weather like in New York?",
        AiMessageType::Human,
    )?;
    
    // Invoke the weather tool
    let response = ai_tools_adapter.invoke_tool(
        &conversation_id,
        "weather",
        json!({
            "location": "New York"
        }),
    ).await?;
    
    // Print the response
    println!("Tool response: {:#?}", response);
    
    // Get the conversation
    let conversation = ai_tools_adapter.get_conversation(&conversation_id)?;
    println!("Conversation: {:#?}", conversation);
    
    Ok(())
} 