# ToolHandler Implementation Example

This file demonstrates how to implement the proposed solution for the `ToolHandler` trait refactoring.

## Updated Trait Definitions

```rust
/// Updated MCPInterface with explicit Send + Sync bounds
pub trait MCPInterface: Send + Sync {
    fn initialize(&self) -> Result<(), SquirrelError>;
    fn is_initialized(&self) -> bool;
    fn get_config(&self) -> Result<MCPConfig, SquirrelError>;
    fn send_message(&self, message: &str) -> Result<String, SquirrelError>;
    async fn register_callback(&self, callback: Box<dyn Fn(String) -> Result<(), MCPError> + Send + Sync>) -> Result<(), MCPError>;
}

/// Callbacks available to tools
pub struct ToolCallbacks {
    /// Add a message to a conversation
    pub add_message: Box<dyn Fn(&str, &str, AiMessageType) -> Result<String, McpAiToolsAdapterError> + Send + Sync>,
    
    /// Get conversation history
    pub get_conversation: Box<dyn Fn(&str) -> Result<Vec<ConversationMessage>, McpAiToolsAdapterError> + Send + Sync>,
    
    /// Send a message to MCP
    pub send_mcp_message: Box<dyn Fn(&str) -> Result<String, McpAiToolsAdapterError> + Send + Sync>,
}

/// New v2 trait without adapter dependency
#[async_trait]
pub trait ToolHandlerV2: Send + Sync + std::fmt::Debug {
    /// Handle a tool invocation
    async fn handle(
        &self,
        invocation: AiToolInvocation,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError>;
    
    /// Register callbacks for adapter interaction
    fn register_callbacks(&mut self, callbacks: ToolCallbacks) {
        // Default empty implementation
    }
}
```

## Adapter Changes

```rust
impl McpAiToolsAdapter {
    // ... existing code ...
    
    /// Register a v2 tool with the adapter
    pub fn register_tool_v2<H>(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
        mut handler: H,
    ) -> Result<(), McpAiToolsAdapterError>
    where
        H: ToolHandlerV2 + 'static,
    {
        // Create callbacks
        let self_clone = self.clone();
        let callbacks = ToolCallbacks {
            add_message: Box::new(move |conversation_id, content, message_type| {
                self_clone.add_message(conversation_id, content, message_type)
            }),
            
            get_conversation: Box::new(move |conversation_id| {
                self_clone.get_conversation(conversation_id)
            }),
            
            send_mcp_message: Box::new(move |message| {
                match self_clone.mcp_adapter.send_message(message) {
                    Ok(response) => Ok(response),
                    Err(err) => Err(McpAiToolsAdapterError::MCPError(format!("{:?}", err))),
                }
            }),
        };
        
        // Register callbacks with handler
        handler.register_callbacks(callbacks);
        
        // Create tool definition
        let name = name.into();
        let tool_def = ToolDefinition {
            name: name.clone(),
            description: description.into(),
            parameters_schema,
            handler: Arc::new(ToolHandlerWrapper::new(handler)),
        };
        
        // Register tool
        let mut tools = self.tools.lock().unwrap();
        tools.insert(name, tool_def);
        Ok(())
    }
    
    // ... rest of implementation ...
}

/// Wrapper to adapt ToolHandlerV2 to ToolHandler for backward compatibility
#[derive(Debug)]
struct ToolHandlerWrapper {
    inner: Arc<dyn ToolHandlerV2>,
}

impl ToolHandlerWrapper {
    fn new<H: ToolHandlerV2 + 'static>(handler: H) -> Self {
        Self {
            inner: Arc::new(handler),
        }
    }
}

#[async_trait]
impl ToolHandler for ToolHandlerWrapper {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Delegate to inner handler without passing adapter
        self.inner.handle(invocation).await
    }
}
```

## Example Implementation

```rust
/// Weather tool handler using the new pattern
#[derive(Debug, Default)]
struct WeatherToolHandlerV2 {
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
            .ok_or_else(|| McpAiToolsAdapterError::InvalidArguments(
                "Missing location parameter".to_string()
            ))?;
        
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
        Ok(AiToolResponse::success("weather", serde_json::json!({
            "location": location,
            "temperature": weather_data.temperature,
            "conditions": weather_data.conditions,
            "humidity": weather_data.humidity,
        })))
    }
    
    fn register_callbacks(&mut self, callbacks: ToolCallbacks) {
        // Store only the callbacks we need
        self.add_message = Some(callbacks.add_message);
    }
}

/// Helper function to simulate a weather API call
async fn simulate_weather_api_call(location: &str) -> Result<WeatherData, McpAiToolsAdapterError> {
    // In a real implementation, this would call an actual weather API
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
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

/// Weather data structure
#[derive(Debug)]
struct WeatherData {
    temperature: f64,
    conditions: String,
    humidity: u8,
}
```

## Usage Example

```rust
#[tokio::test]
async fn test_weather_tool() {
    // Create MCP adapter
    let mcp = Arc::new(MockMCP::new("mock_response"));
    let config = McpAiToolsConfig::default();
    let adapter = Arc::new(McpAiToolsAdapter::with_config(mcp, config));
    
    // Register weather tool using new pattern
    adapter.register_tool_v2(
        "weather",
        "Get current weather for a location",
        serde_json::json!({
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
    ).expect("Failed to register weather tool");
    
    // Create a conversation
    let conversation_id = adapter.create_conversation();
    
    // Invoke the weather tool
    let response = adapter.invoke_tool(
        &conversation_id,
        "weather",
        serde_json::json!({
            "location": "New York"
        }),
    ).await.expect("Failed to invoke weather tool");
    
    // Verify the response
    assert_eq!(response.status, AiToolResponseStatus::Success);
    let data = response.data;
    assert_eq!(data.get("location").and_then(|v| v.as_str()), Some("New York"));
    assert!(data.get("temperature").is_some());
    assert!(data.get("conditions").is_some());
    assert!(data.get("humidity").is_some());
}
```

## Migration Path

The approach shown provides backward compatibility by:

1. Keeping the original `ToolHandler` trait unchanged
2. Creating a wrapper that adapts `ToolHandlerV2` to work with existing code
3. Adding new registration methods that use the updated pattern

This allows teams to migrate gradually:

1. New tools can use the improved pattern
2. Existing tools can be migrated gradually
3. The system works with both types of handlers simultaneously

## Benefits

1. **Thread Safety**: Explicit `Send + Sync` bounds ensure all components work correctly in async contexts
2. **Testability**: Tools are easier to test without complex mocking
3. **Minimized Dependencies**: Tools only access the adapter functionality they actually need
4. **Performance**: Reduced overhead from not passing entire adapter structures
5. **Clarity**: Clear separation between tool logic and adapter interaction 