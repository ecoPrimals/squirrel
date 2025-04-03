use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use serde_json::json;
use squirrel_mcp::{MCPError, MCPInterface};
use squirrel_core::error::SquirrelError;
use squirrel_mcp::config::MCPConfig;
use squirrel_integration::{
    McpAiToolsAdapter, McpAiToolsConfig, AiMessageType, 
    McpAiToolsAdapterError, AiToolInvocation, AiToolResponse,
    ToolHandlerV2, ToolCallbacks,
};

/// Mock implementation for V2 testing
#[derive(Debug, Clone)]
struct MockMCPV2 {
    /// Last message received
    last_message: Arc<Mutex<Option<String>>>,
    /// Response to return
    response: String,
}

impl MockMCPV2 {
    /// Create a new MockMCPV2 instance
    fn new(response: impl Into<String>) -> Self {
        Self {
            last_message: Arc::new(Mutex::new(None)),
            response: response.into(),
        }
    }
    
    /// Get the last message that was sent
    fn get_last_message(&self) -> Option<String> {
        self.last_message.lock().unwrap().clone()
    }
}

#[async_trait]
impl MCPInterface for MockMCPV2 {
    fn initialize(&self) -> Result<(), SquirrelError> {
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        true
    }

    fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
        Ok(MCPConfig::default())
    }

    fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
        // Store the message and return the configured response
        let mut last_message = self.last_message.lock().unwrap();
        *last_message = Some(message.to_string());
        Ok(self.response.clone())
    }
    
    async fn register_callback(&self, _callback: Box<dyn Fn(String) -> Result<(), MCPError> + Send + Sync>) -> Result<(), MCPError> {
        // No-op implementation for mock
        Ok(())
    }
}

/// Example V2 tool handler implementation for testing
#[derive(Debug, Default)]
struct TestToolHandlerV2 {
    /// The conversation ID where messages should be added
    conversation_id: String,
    /// Last invocation received
    last_invocation: Arc<Mutex<Option<AiToolInvocation>>>,
    /// Flag to track if callbacks were registered
    callbacks_registered: Arc<Mutex<bool>>,
    /// Add message callback
    add_message: Option<Box<dyn Fn(&str, &str, AiMessageType) -> Result<String, McpAiToolsAdapterError> + Send + Sync>>,
}

impl TestToolHandlerV2 {
    /// Create a new TestToolHandlerV2
    fn new(conversation_id: &str) -> Self {
        Self {
            conversation_id: conversation_id.to_string(),
            last_invocation: Arc::new(Mutex::new(None)),
            callbacks_registered: Arc::new(Mutex::new(false)),
            add_message: None,
        }
    }
    
    /// Get the last invocation that was handled
    fn get_last_invocation(&self) -> Option<AiToolInvocation> {
        self.last_invocation.lock().unwrap().clone()
    }
    
    /// Check if callbacks were registered
    fn were_callbacks_registered(&self) -> bool {
        *self.callbacks_registered.lock().unwrap()
    }
}

#[async_trait]
impl ToolHandlerV2 for TestToolHandlerV2 {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Store the invocation
        let mut last_invocation = self.last_invocation.lock().unwrap();
        *last_invocation = Some(invocation.clone());
        
        // Log the invocation using the callback if available
        if let Some(add_message) = &self.add_message {
            let _ = add_message(
                &self.conversation_id,
                &format!("TestToolHandlerV2 received invocation: {}", invocation.name),
                AiMessageType::System,
            );
        }
        
        // Return a success response
        Ok(AiToolResponse::success(&invocation.name, json!({
            "result": "Success from TestToolHandlerV2"
        })))
    }
    
    fn register_callbacks(&mut self, callbacks: ToolCallbacks) {
        // Mark callbacks as registered
        let mut registered = self.callbacks_registered.lock().unwrap();
        *registered = true;
        
        // Store the add_message callback
        self.add_message = Some(callbacks.add_message);
    }
}

/// Helper to create a basic test adapter for V2 testing
fn create_test_adapter_v2() -> Arc<McpAiToolsAdapter> {
    // Create a mock MCP interface
    let mcp = Arc::new(MockMCPV2::new("mock_response"));
    
    // Create a basic configuration
    let config = McpAiToolsConfig::default();
    
    // Create the adapter
    Arc::new(McpAiToolsAdapter::with_config(mcp, config))
}

#[tokio::test]
async fn test_register_v2_tool() {
    let adapter = create_test_adapter_v2();
    let conversation_id = adapter.create_conversation();
    
    // Create a test tool handler
    let handler = TestToolHandlerV2::new(&conversation_id);
    
    // Register the tool using the V2 method
    adapter.register_tool_v2(
        "test_v2_tool",
        "A test tool using V2 trait",
        json!({
            "type": "object",
            "properties": {
                "param1": { "type": "string" }
            }
        }),
        handler.clone(),
    ).expect("Failed to register tool");
    
    // Verify callbacks were registered
    assert!(handler.were_callbacks_registered(), "Callbacks should be registered");
}

#[tokio::test]
async fn test_invoke_v2_tool() {
    let adapter = create_test_adapter_v2();
    let conversation_id = adapter.create_conversation();
    
    // Create a test tool handler
    let handler = TestToolHandlerV2::new(&conversation_id);
    
    // Register the tool using the V2 method
    adapter.register_tool_v2(
        "test_v2_tool",
        "A test tool using V2 trait",
        json!({
            "type": "object",
            "properties": {
                "param1": { "type": "string" }
            }
        }),
        handler.clone(),
    ).expect("Failed to register tool");
    
    // Invoke the tool
    let args = json!({
        "param1": "value1"
    });
    let response = adapter.invoke_tool(
        &conversation_id,
        "test_v2_tool",
        args.clone(),
    ).await.expect("Failed to invoke tool");
    
    // Verify the response
    assert_eq!(response.status, squirrel_integration::AiToolResponseStatus::Success);
    assert_eq!(response.data.get("result").and_then(|v| v.as_str()),
               Some("Success from TestToolHandlerV2"));
    
    // Verify the invocation was recorded
    let invocation = handler.get_last_invocation().expect("No invocation recorded");
    assert_eq!(invocation.name, "test_v2_tool");
    assert_eq!(invocation.arguments, args);
    
    // Verify system message was added through callback
    let conversation = adapter.get_conversation(&conversation_id).unwrap();
    assert!(conversation.iter().any(|msg| 
        msg.message_type == AiMessageType::System && 
        msg.content.contains("TestToolHandlerV2 received invocation")
    ));
} 