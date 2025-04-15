use async_trait::async_trait;
use std::sync::Arc;
use squirrel_mcp::{MCPError, MCPInterface};
use squirrel_core::error::SquirrelError;
use squirrel_mcp::config::MCPConfig;
use squirrel_integration::{
    McpAiToolsAdapter, McpAiToolsConfig, AiMessageType,
    create_mcp_ai_tools_adapter_with_config,
    mcp_ai_tools::config::ProviderSettings,
};
use secrecy::Secret;
use serde_json::json;
use std::collections::HashMap;

/// Simple MCP implementation for live tests
#[derive(Debug, Clone)]
struct SimpleMCP;

#[async_trait]
impl MCPInterface for SimpleMCP {
    fn initialize(&self) -> Result<(), SquirrelError> {
        // No initialization needed for this mock
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        // Always report as initialized
        true
    }

    fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
        // Return default config
        Ok(MCPConfig::default())
    }

    fn send_message(&self, _message: &str) -> Result<String, SquirrelError> {
        // Simple implementation that just returns empty string
        // We're not testing MCP functionality here, just the AI integration
        Ok("".to_string())
    }
    
    async fn register_callback(&self, _callback: Box<dyn Fn(String) -> Result<(), MCPError> + Send + Sync>) -> Result<(), MCPError> {
        // No-op implementation
        Ok(())
    }
}

/// Helper to check if OpenAI API key is available
fn has_openai_api_key() -> bool {
    std::env::var("OPENAI_API_KEY").is_ok()
}

/// Helper to create a live test adapter with OpenAI
fn create_live_adapter() -> Option<Arc<McpAiToolsAdapter>> {
    // Check for OpenAI API key
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return None,
    };
    
    // Create MCP interface
    let mcp = Arc::new(SimpleMCP);
    
    // Create configuration with OpenAI provider
    let mut config = McpAiToolsConfig::default();
    config.default_provider = "openai".to_string();
    
    // Configure OpenAI provider
    config.providers.insert(
        "openai".to_string(),
        ProviderSettings {
            api_key,  // Add the API key directly
            default_model: "gpt-3.5-turbo".to_string(),
            available_models: vec!["gpt-3.5-turbo".to_string()],
            default_parameters: HashMap::from([
                ("temperature".to_string(), serde_json::json!(0.0)),
                ("max_tokens".to_string(), serde_json::json!(100))
            ]),
            timeout_ms: Some(10000),
        },
    );
    
    // Create adapter
    match create_mcp_ai_tools_adapter_with_config(mcp, config) {
        Ok(adapter) => Some(adapter),
        Err(_) => None,
    }
}

// Add extern crate for our accessor extension methods
// This allows us to use the extension methods defined in mock_tests.rs
extern crate self as squirrel_integration_tests;

#[tokio::test]
async fn test_live_conversation() {
    // Skip if no API key is available
    if !has_openai_api_key() {
        println!("Skipping live test: OPENAI_API_KEY not set");
        return;
    }
    
    // Create adapter
    let adapter = match create_live_adapter() {
        Some(adapter) => adapter,
        None => {
            println!("Failed to create live adapter");
            return;
        }
    };
    
    // Create a conversation
    let conversation_id = adapter.create_conversation();
    
    // Add a system message
    let result = adapter.add_message(
        &conversation_id,
        "You are a helpful assistant. Keep responses very short.",
        AiMessageType::System,
    );
    assert!(result.is_ok());
    
    // Add a user message
    let result = adapter.add_message(
        &conversation_id,
        "What is the capital of France?",
        AiMessageType::Human,
    );
    assert!(result.is_ok());
    
    // Generate a response
    let response = adapter.generate_response(
        &conversation_id,
        None,
        None,
        None,
    ).await;
    
    // Verify we got a response
    assert!(response.is_ok());
    let response_text = response.unwrap();
    
    // The response should contain "Paris"
    assert!(
        response_text.to_lowercase().contains("paris"),
        "Expected response to mention Paris, got: {response_text}"
    );
    
    // Verify the message was added to the conversation
    let conversation = adapter.get_conversation(&conversation_id).unwrap();
    assert_eq!(conversation.len(), 3); // System, User, Assistant
    assert_eq!(conversation[2].message_type, AiMessageType::Assistant);
    assert_eq!(conversation[2].content, response_text);
}

#[tokio::test]
async fn test_live_conversation_with_context() {
    // Skip if no API key is available
    if !has_openai_api_key() {
        println!("Skipping live test: OPENAI_API_KEY not set");
        return;
    }
    
    // Create adapter
    let adapter = match create_live_adapter() {
        Some(adapter) => adapter,
        None => {
            println!("Failed to create live adapter");
            return;
        }
    };
    
    // Create a conversation
    let conversation_id = adapter.create_conversation();
    
    // Add a system message
    let result = adapter.add_message(
        &conversation_id,
        "You are a helpful assistant. Keep responses very short.",
        AiMessageType::System,
    );
    assert!(result.is_ok());
    
    // Add context information
    let result = adapter.add_message(
        &conversation_id,
        "My name is Alice and I live in New York.",
        AiMessageType::Human,
    );
    assert!(result.is_ok());
    
    // Add assistant acknowledgment
    let result = adapter.add_message(
        &conversation_id,
        "I understand. How can I help you today, Alice?",
        AiMessageType::Assistant,
    );
    assert!(result.is_ok());
    
    // Ask a question that requires the context
    let result = adapter.add_message(
        &conversation_id,
        "What city do I live in?",
        AiMessageType::Human,
    );
    assert!(result.is_ok());
    
    // Generate a response
    let response = adapter.generate_response(
        &conversation_id,
        None,
        None,
        None,
    ).await;
    
    // Verify we got a response
    assert!(response.is_ok());
    let response_text = response.unwrap();
    
    // The response should contain "New York"
    assert!(
        response_text.to_lowercase().contains("new york"),
        "Expected response to mention New York, got: {response_text}"
    );
}

#[tokio::test]
async fn test_live_multiple_conversations() {
    // Skip if no API key is available
    if !has_openai_api_key() {
        println!("Skipping live test: OPENAI_API_KEY not set");
        return;
    }
    
    // Create adapter
    let adapter = match create_live_adapter() {
        Some(adapter) => adapter,
        None => {
            println!("Failed to create live adapter");
            return;
        }
    };
    
    // Create two conversations
    let conversation_id_1 = adapter.create_conversation();
    let conversation_id_2 = adapter.create_conversation();
    
    // Set up first conversation about France
    adapter.add_message(
        &conversation_id_1,
        "You are a helpful assistant. Keep responses very short.",
        AiMessageType::System,
    ).unwrap();
    
    adapter.add_message(
        &conversation_id_1,
        "Let's talk about France.",
        AiMessageType::Human,
    ).unwrap();
    
    // Set up second conversation about Japan
    adapter.add_message(
        &conversation_id_2,
        "You are a helpful assistant. Keep responses very short.",
        AiMessageType::System,
    ).unwrap();
    
    adapter.add_message(
        &conversation_id_2,
        "Let's talk about Japan.",
        AiMessageType::Human,
    ).unwrap();
    
    // Ask about the capital in both conversations
    adapter.add_message(
        &conversation_id_1,
        "What's the capital?",
        AiMessageType::Human,
    ).unwrap();
    
    adapter.add_message(
        &conversation_id_2,
        "What's the capital?",
        AiMessageType::Human,
    ).unwrap();
    
    // Get responses
    let response_1 = adapter.generate_response(
        &conversation_id_1,
        None,
        None,
        None,
    ).await.unwrap();
    
    let response_2 = adapter.generate_response(
        &conversation_id_2,
        None,
        None,
        None,
    ).await.unwrap();
    
    // First response should mention Paris
    assert!(
        response_1.to_lowercase().contains("paris"),
        "Expected response to mention Paris, got: {response_1}"
    );
    
    // Second response should mention Tokyo
    assert!(
        response_2.to_lowercase().contains("tokyo"),
        "Expected response to mention Tokyo, got: {response_2}"
    );
} 