//! Context-MCP AI Tools Integration Example
//!
//! This example demonstrates how to use the AI tools integration with the Context-MCP adapter.
//!
//! To run this example:
//! ```
//! cargo run --example context_ai_tools
//! ```

use std::env;
use tracing::{info, warn, error};
use tracing_subscriber::FmtSubscriber;
use serde_json::json;

use squirrel_integration::{
    ContextMcpAdapter, 
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
    ContextAiEnhancementOptions,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Context-MCP AI Tools integration example");
    
    // Get API key from environment (or use a default one for demo)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        warn!("Warning: OPENAI_API_KEY not set, using a placeholder. Requests will fail.");
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
    let context_id = "ai-tools-example";
    let sample_context = Context {
        id: context_id.to_string(),
        account_id: "example-account".to_string(),
        metadata: Some(json!({
            "project": {
                "name": "Smart City Monitoring",
                "description": "IoT sensors deployment for urban environment monitoring",
                "phase": "implementation",
                "stakeholders": ["City Council", "Environmental Agency", "Tech Partners"]
            },
            "infrastructure": {
                "sensors": {
                    "air_quality": 120,
                    "noise_level": 85,
                    "traffic_flow": 45,
                    "power_consumption": 60
                },
                "gateways": 28,
                "cloud_servers": 5
            },
            "metrics": {
                "last_30_days": {
                    "uptime_percentage": 99.2,
                    "data_collection_rate": 98.7,
                    "alert_triggers": 47,
                    "response_time_ms": 230
                }
            },
            "recent_events": [
                {
                    "timestamp": "2024-06-01T08:30:00Z",
                    "type": "alert",
                    "description": "High pollution levels detected in sector 12"
                },
                {
                    "timestamp": "2024-06-02T14:15:00Z",
                    "type": "system",
                    "description": "Gateway 08 offline, maintenance required"
                },
                {
                    "timestamp": "2024-06-03T19:45:00Z",
                    "type": "alert",
                    "description": "Unusual traffic congestion in downtown area"
                }
            ]
        })),
        ..Default::default()
    };
    
    // Create the context in MCP
    mcp_adapter.create_context(&sample_context).await?;
    info!("Sample context created with ID: {}", context_id);
    
    // Demonstrate different types of context enhancements
    
    // 1. Generate insights
    info!("Enhancing context with AI insights...");
    let insights_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Insights,
        "openai",
        api_key.clone(),
    )
    .with_model("gpt-4o")
    .with_timeout(15000);
    
    match context_adapter.apply_ai_enhancements(context_id, insights_options).await {
        Ok(_) => info!("Successfully applied insights enhancement"),
        Err(e) => {
            error!("Failed to apply insights enhancement: {}", e);
            // Continue with the example even if this fails
        }
    }
    
    // 2. Trend analysis
    info!("Enhancing context with trend analysis...");
    let trend_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::TrendAnalysis,
        "openai",
        api_key.clone(),
    )
    .with_model("gpt-4o")
    .with_timeout(15000);
    
    match context_adapter.apply_ai_enhancements(context_id, trend_options).await {
        Ok(_) => info!("Successfully applied trend analysis enhancement"),
        Err(e) => {
            error!("Failed to apply trend analysis enhancement: {}", e);
            // Continue with the example even if this fails
        }
    }
    
    // 3. Recommendations
    info!("Enhancing context with recommendations...");
    let recommendations_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Recommendations,
        "openai",
        api_key.clone(),
    )
    .with_model("gpt-4o")
    .with_timeout(15000)
    .with_parameter("temperature", 0.8);
    
    match context_adapter.apply_ai_enhancements(context_id, recommendations_options).await {
        Ok(_) => info!("Successfully applied recommendations enhancement"),
        Err(e) => {
            error!("Failed to apply recommendations enhancement: {}", e);
            // Continue with the example even if this fails
        }
    }
    
    // 4. Custom enhancement
    info!("Applying custom enhancement...");
    let custom_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Custom(
            "Identify potential efficiency improvements and cost-saving opportunities".to_string()
        ),
        "openai",
        api_key,
    )
    .with_model("gpt-4o")
    .with_timeout(15000);
    
    match context_adapter.apply_ai_enhancements(context_id, custom_options).await {
        Ok(_) => info!("Successfully applied custom enhancement"),
        Err(e) => {
            error!("Failed to apply custom enhancement: {}", e);
            // Continue with the example even if this fails
        }
    }
    
    // Retrieve and display the enhanced context
    let enhanced_context = mcp_adapter.get_context(context_id).await?;
    
    if let Some(metadata) = &enhanced_context.metadata {
        if let Some(ai_insights) = metadata.get("ai_insights") {
            println!("\nAI Context Enhancements:\n-----------------------");
            println!("{}", serde_json::to_string_pretty(ai_insights)?);
        } else {
            println!("No AI insights found in the enhanced context");
        }
    }
    
    // Cleanup - delete the example context
    mcp_adapter.delete_context(context_id).await?;
    info!("Cleaned up sample context");
    
    info!("Context-MCP AI Tools integration example completed");
    
    Ok(())
} 