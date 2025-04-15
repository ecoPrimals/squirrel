//! Example of AI-enhanced context
//!
//! This example demonstrates how to use the AI enhancement capabilities
//! of the Context-MCP adapter.
//!
//! To run this example:
//! ```
//! cargo run --example ai_enhanced_context
//! ```

use std::env;
use std::time::Duration;
use tracing::{info, error};
use tracing_subscriber::FmtSubscriber;
use serde_json::json;

use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    AiEnhancementOptions,
    create_context_mcp_adapter_with_config,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting AI-enhanced context example");
    
    // Get API key from environment (or use a default one for demo)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("Warning: OPENAI_API_KEY not set, using a placeholder. Requests will fail.");
        "sk-xxxxxxxxxxxxxxxxxxxx".to_string()
    });
    
    // Create MCP adapter first
    let mcp_config = McpConfig::default();
    let mcp_adapter = std::sync::Arc::new(MCPAdapter::new(mcp_config));
    mcp_adapter.initialize()?;
    
    // Create the context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = create_context_mcp_adapter_with_config(context_mcp_config).await?;
    
    // Create a sample context in MCP
    info!("Creating a sample context");
    let context_id = "ai-enhanced-example";
    let sample_context = Context {
        id: context_id.to_string(),
        account_id: "example-account".to_string(),
        metadata: Some(json!({
            "project": {
                "name": "Sample Project",
                "description": "This is a sample project for testing AI enhancement",
                "type": "web-application",
                "technologies": ["Rust", "React", "GraphQL"],
                "team_size": 5
            },
            "environment": {
                "name": "development",
                "region": "us-west-2",
                "resources": {
                    "cpu": "2",
                    "memory": "4GB",
                    "storage": "100GB"
                }
            },
            "metrics": {
                "api_latency_ms": 250,
                "error_rate": 0.02,
                "user_satisfaction": 4.2
            }
        })),
        ..Default::default()
    };
    
    // Create the context in MCP
    mcp_adapter.create_context(&sample_context).await?;
    info!("Sample context created with ID: {}", context_id);
    
    // Define AI enhancement options
    let ai_options = AiEnhancementOptions::new("openai", api_key)
        .with_model("gpt-4o")
        .with_timeout(20000);
    
    // Enhance the context with AI
    info!("Enhancing context with AI...");
    match context_adapter.enhance_with_ai(context_id, ai_options).await {
        Ok(_) => info!("Context successfully enhanced with AI insights"),
        Err(e) => {
            error!("Failed to enhance context: {}", e);
            return Err(e.into());
        }
    }
    
    // Retrieve and display the enhanced context
    let enhanced_context = mcp_adapter.get_context(context_id).await?;
    
    if let Some(metadata) = &enhanced_context.metadata {
        if let Some(insights) = metadata.get("ai_insights") {
            println!("\nAI Insights:\n------------");
            println!("{}", serde_json::to_string_pretty(insights)?);
        } else {
            println!("No AI insights found in the enhanced context");
        }
    }
    
    // Cleanup - delete the example context
    mcp_adapter.delete_context(context_id).await?;
    info!("Cleaned up sample context");
    
    info!("AI-enhanced context example completed");
    
    Ok(())
} 