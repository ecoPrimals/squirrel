use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use squirrel_mcp::{MCPError, MCPInterface};
use squirrel_core::error::SquirrelError;
use squirrel_mcp::config::MCPConfig;
use squirrel_integration::mcp_ai_tools::{
    McpAiToolsAdapter, McpAiToolsConfig, AiMessageType, 
    McpAiToolsAdapterError, AiToolInvocation, AiToolResponse,
};
use std::fmt::Debug;

/// Mock MCP implementation for testing
#[derive(Debug, Clone)]
struct MockMCP {
    /// Last message received
    last_message: Arc<Mutex<Option<String>>>,
    /// Response to return
    response: String,
    /// Initialization flag
    initialized: Arc<Mutex<bool>>,
}

impl MockMCP {
    /// Create a new MockMCP instance
    fn new(response: impl Into<String>) -> Self {
        Self {
            last_message: Arc::new(Mutex::new(None)),
            response: response.into(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get the last message that was sent
    fn get_last_message(&self) -> Option<String> {
        self.last_message.lock().unwrap().clone()
    }
}

#[async_trait]
impl MCPInterface for MockMCP {
    fn initialize(&self) -> Result<(), SquirrelError> {
        let mut initialized = self.initialized.lock().unwrap();
        *initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
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

/// Mock AI client implementation
#[derive(Debug, Clone)]
struct MockAIClient {
    /// Responses to return for each message
    responses: Arc<Mutex<Vec<String>>>,
    /// Messages received
    messages: Arc<Mutex<Vec<String>>>,
}

impl MockAIClient {
    /// Create a new MockAIClient with a single response
    fn new(response: impl Into<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(vec![response.into()])),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new MockAIClient with multiple responses
    fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get all messages that were sent
    fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

/// Mock implementation for a tool handler
#[derive(Debug, Clone)]
struct MockToolHandler {
    /// Response to return for tool invocation
    response: Arc<Mutex<AiToolResponse>>,
    /// Last invocation received
    last_invocation: Arc<Mutex<Option<AiToolInvocation>>>,
}

impl MockToolHandler {
    /// Create a new MockToolHandler
    fn new(response: AiToolResponse) -> Self {
        Self {
            response: Arc::new(Mutex::new(response)),
            last_invocation: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Get the last invocation that was handled
    fn get_last_invocation(&self) -> Option<AiToolInvocation> {
        self.last_invocation.lock().unwrap().clone()
    }
}

#[async_trait]
impl squirrel_integration::ToolHandler for MockToolHandler {
    // TEMPORARY FIX: Ignoring adapter parameter to avoid Send/Sync issues
    // See specs/integration/tool_handler_refactoring.md and specs/integration/temporary_fix.md
    // for details on this workaround and the long-term solution
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>, // Adapter ignored to avoid Send/Sync issues
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Store the invocation and return the configured response
        // We deliberately don't use the _adapter parameter to avoid Send/Sync issues
        let mut last_invocation = self.last_invocation.lock().unwrap();
        *last_invocation = Some(invocation);
        Ok(self.response.lock().unwrap().clone())
    }
}

/// Helper to create a basic test adapter
fn create_test_adapter() -> Arc<McpAiToolsAdapter> {
    // Create a mock MCP interface
    let mcp = Arc::new(MockMCP::new("mock_response"));
    
    // Create a basic configuration
    let config = McpAiToolsConfig::default();
    
    // Create the adapter
    Arc::new(McpAiToolsAdapter::with_config(mcp, config))
}

#[tokio::test]
async fn test_create_conversation() {
    let adapter = create_test_adapter();
    
    // Create a conversation
    let conversation_id = adapter.create_conversation();
    
    // Verify it's not empty
    assert!(!conversation_id.is_empty());
    
    // Verify we can add a message to it
    let result = adapter.add_message(
        &conversation_id,
        "Test message",
        AiMessageType::Human,
    );
    assert!(result.is_ok());
    
    // Verify we can get the conversation
    let conversation = adapter.get_conversation(&conversation_id);
    assert!(conversation.is_ok());
    let messages = conversation.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test message");
    assert_eq!(messages[0].message_type, AiMessageType::Human);
}

#[tokio::test]
async fn test_add_different_message_types() {
    let adapter = create_test_adapter();
    let conversation_id = adapter.create_conversation();
    
    // Add different types of messages
    let human_id = adapter.add_message(
        &conversation_id,
        "Human message",
        AiMessageType::Human,
    ).unwrap();
    
    let assistant_id = adapter.add_message(
        &conversation_id,
        "Assistant message",
        AiMessageType::Assistant,
    ).unwrap();
    
    let system_id = adapter.add_message(
        &conversation_id,
        "System message",
        AiMessageType::System,
    ).unwrap();
    
    // Verify all messages were added
    let conversation = adapter.get_conversation(&conversation_id).unwrap();
    assert_eq!(conversation.len(), 3);
    
    // Verify message types are correct using direct field access
    assert_eq!(conversation[0].message_type, AiMessageType::Human);
    assert_eq!(conversation[1].message_type, AiMessageType::Assistant);
    assert_eq!(conversation[2].message_type, AiMessageType::System);
    
    // Verify message IDs using direct field access
    assert_eq!(conversation[0].id, human_id);
    assert_eq!(conversation[1].id, assistant_id);
    assert_eq!(conversation[2].id, system_id);
}

#[tokio::test]
async fn test_nonexistent_conversation() {
    let adapter = create_test_adapter();
    
    // Try to add a message to a nonexistent conversation
    let result = adapter.add_message(
        "nonexistent",
        "Test message",
        AiMessageType::Human,
    );
    assert!(result.is_err());
    
    // Try to get a nonexistent conversation
    let result = adapter.get_conversation("nonexistent");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_and_invoke_tool() {
    let adapter = create_test_adapter();
    let conversation_id = adapter.create_conversation();
    
    // Create a mock tool response
    let tool_response = AiToolResponse::success("test_invocation", serde_json::json!({
        "result": "Success!"
    }));
    
    // Create a mock tool handler
    let handler = MockToolHandler::new(tool_response.clone());
    
    // Register the tool
    adapter.register_tool(
        "test_tool",
        "A test tool",
        serde_json::json!({
            "type": "object",
            "properties": {
                "param1": { "type": "string" }
            }
        }),
        handler.clone(),
    ).expect("Failed to register tool");
    
    // Invoke the tool
    let args = serde_json::json!({
        "param1": "value1"
    });
    let response = adapter.invoke_tool(
        &conversation_id,
        "test_tool",
        args.clone(),
    ).await.expect("Failed to invoke tool");
    
    // Verify the response
    assert_eq!(response.status, AiToolResponseStatus::Success);
    
    // Verify the invocation was recorded
    let invocation = handler.get_last_invocation().expect("No invocation recorded");
    assert_eq!(invocation.name, "test_tool");
    assert_eq!(invocation.arguments, args);
}

#[tokio::test]
async fn test_invoke_nonexistent_tool() {
    let adapter = create_test_adapter();
    let conversation_id = adapter.create_conversation();
    
    // Try to invoke a non-existent tool
    let result = adapter.invoke_tool(
        &conversation_id,
        "nonexistent_tool",
        serde_json::json!({})
    ).await;
    
    // Verify the error
    assert!(result.is_err());
    match result {
        Err(squirrel_integration::mcp_ai_tools::McpAiToolsAdapterError::ToolNotFound(name)) => {
            assert_eq!(name, "nonexistent_tool");
        }
        other => panic!("Expected ToolNotFound error, got: {:?}", other),
    }
} 