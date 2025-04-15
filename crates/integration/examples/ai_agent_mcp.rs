//! Example of using the AI Agent adapter with MCP
//!
//! This example demonstrates how to integrate the AI Agent adapter with MCP.
//! 
//! To run this example:
//! ```
//! cargo run --example ai_agent_mcp
//! ```

use std::sync::Arc;
use std::env;
use tokio::time::Duration;
use tracing::{info, error};
use tracing_subscriber::FmtSubscriber;

use squirrel_integration::{
    AIAgentAdapter,
    AIAgentConfig,
    AgentRequest,
    create_ai_agent_adapter_with_config,
};
use squirrel_mcp::{
    MCPAdapter,
    McpConfig,
    protocol::types::{MCPMessage, MessageType},
    Context,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting AI Agent MCP integration example");
    
    // Get API key from environment (or use a default one for demo)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("Warning: OPENAI_API_KEY not set, using a placeholder. Requests will fail.");
        "sk-xxxxxxxxxxxxxxxxxxxx".to_string()
    });
    
    // Create AI agent configuration
    let ai_config = AIAgentConfig::new("openai", api_key)
        .with_model("gpt-4o")
        .with_timeout(30000);
    
    // Create MCP configuration
    let mcp_config = McpConfig::default();
    
    // Create MCP adapter
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    mcp_adapter.initialize()?;
    
    // Create AI agent adapter with MCP
    let mut ai_adapter = create_ai_agent_adapter_with_config(ai_config)
        .with_mcp(mcp_adapter.clone());
    
    // Initialize the AI agent adapter
    if let Err(e) = ai_adapter.initialize().await {
        error!("Failed to initialize AI Agent adapter: {}", e);
        return Err(e.into());
    }
    
    // Check the status
    let status = ai_adapter.get_status().await;
    info!("AI Agent adapter status: initialized={}, operational={}",
          status.initialized, status.operational);
    
    // Create a context for the request
    let context = Context {
        id: "example-session".to_string(),
        account_id: "example-account".to_string(),
        metadata: Some(serde_json::json!({
            "source": "integration-example"
        })),
        ..Default::default()
    };
    
    // Create a message to process
    let mut message = MCPMessage::new(MessageType::Command);
    message.payload = serde_json::json!({
        "prompt": "How does the adapter pattern help with system integration?",
        "system_message": "You are a helpful assistant specializing in software architecture."
    });
    
    // Process the message
    info!("Sending message to AI agent...");
    match ai_adapter.process_mcp_message(&message, &context).await {
        Ok(response) => {
            info!("Successfully processed message through AI agent");
            
            if let Some(content) = response.payload.get("content") {
                println!("\nAI Response:\n-----------\n{}\n", content.as_str().unwrap_or_default());
            } else {
                println!("\nAI Response:\n-----------\n{}\n", response.payload);
            }
        },
        Err(e) => {
            error!("Failed to process message: {}", e);
        }
    }
    
    // Direct usage example
    info!("Sending direct request to AI agent...");
    let request = AgentRequest {
        prompt: "What are the benefits of the circuit breaker pattern in distributed systems?".to_string(),
        system_message: Some("You are a software architecture expert.".to_string()),
        parameters: None,
        context: None,
    };
    
    match ai_adapter.process_request(request).await {
        Ok(response) => {
            println!("\nDirect Request Response:\n-----------\n{}\n", response.content);
        },
        Err(e) => {
            error!("Failed to process direct request: {}", e);
        }
    }
    
    info!("AI Agent MCP integration example completed");
    
    Ok(())
} 