//! AI Context Enhancement Example
//!
//! This example demonstrates how to use the unified interface for AI context enhancement
//! with the Context-MCP adapter.
//!
//! To run this example:
//! ```
//! cargo run --example ai_context_enhancement
//! ```

use std::env;
use std::sync::Arc;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;
use serde_json::json;

use squirrel_integration::{
    ContextMcpAdapter, 
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting AI Context Enhancement example");
    
    // Get API key from environment
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        warn!("Warning: OPENAI_API_KEY not set, using a placeholder. Requests will fail.");
        "sk-xxxxxxxxxxxxxxxxxxxx".to_string()
    });
    
    // Create MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    mcp_adapter.initialize()?;
    
    // Create the context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = create_context_mcp_adapter_with_config(context_mcp_config).await?;
    
    // Create a sample context in MCP
    info!("Creating a sample context");
    let context_id = "ai-enhancement-example";
    let sample_context = Context {
        id: context_id.to_string(),
        account_id: "example-account".to_string(),
        metadata: Some(json!({
            "project": {
                "name": "Retail Analytics Platform",
                "description": "Retail analytics and inventory management system",
                "phase": "production",
                "stakeholders": ["Merchandising Team", "Store Operations", "Executive Leadership"]
            },
            "performance": {
                "metrics": {
                    "response_time_ms": 250,
                    "availability": 99.95,
                    "error_rate": 0.03,
                    "concurrent_users": 1250
                },
                "issues": [
                    {
                        "id": "ISS-2401",
                        "description": "High database load during peak hours",
                        "severity": "medium",
                        "resolution": "pending"
                    },
                    {
                        "id": "ISS-2415",
                        "description": "Report generation timeout for large datasets",
                        "severity": "high",
                        "resolution": "in progress"
                    }
                ]
            },
            "inventory": {
                "total_items": 245789,
                "categories": 36,
                "high_demand_items": 1254,
                "low_stock_alert_count": 78
            }
        })),
        ..Default::default()
    };
    
    // Create the context in MCP
    mcp_adapter.create_context(&sample_context).await?;
    info!("Sample context created with ID: {}", context_id);
    
    // Demonstrate different ways to use the unified interface
    
    // 1. Use the convenience method for insights
    info!("Using convenience method for insights...");
    match context_adapter.enhance_with_insights(
        context_id,
        "openai",
        api_key.clone(),
        Some("gpt-4o".to_string()),
    ).await {
        Ok(_) => info!("Successfully applied insights enhancement"),
        Err(e) => {
            error!("Failed to apply insights enhancement: {}", e);
            // Continue with example even if this fails
        }
    }
    
    // 2. Use the convenience method for summary
    info!("Using convenience method for summary...");
    match context_adapter.enhance_with_summary(
        context_id,
        "openai",
        api_key.clone(),
        Some("gpt-4o".to_string()),
    ).await {
        Ok(_) => info!("Successfully applied summary enhancement"),
        Err(e) => {
            error!("Failed to apply summary enhancement: {}", e);
            // Continue with example even if this fails
        }
    }
    
    // 3. Use the convenience method for recommendations
    info!("Using convenience method for recommendations...");
    match context_adapter.enhance_with_recommendations(
        context_id,
        "openai",
        api_key.clone(),
        Some("gpt-4o".to_string()),
    ).await {
        Ok(_) => info!("Successfully applied recommendations enhancement"),
        Err(e) => {
            error!("Failed to apply recommendations enhancement: {}", e);
            // Continue with example even if this fails
        }
    }
    
    // 4. Use the full enhance_context method with custom parameters
    info!("Using full enhance_context method with custom parameters...");
    
    // Create custom parameters
    let mut params = serde_json::Map::new();
    params.insert("temperature".to_string(), json!(0.8));
    params.insert("top_p".to_string(), json!(0.95));
    
    match context_adapter.enhance_context(
        context_id,
        ContextEnhancementType::TrendAnalysis,
        "openai",
        api_key.clone(),
        Some("gpt-4o".to_string()),
        Some(25000), // 25-second timeout
        Some("Analyze performance trends and identify potential optimizations.".to_string()),
        Some(params),
    ).await {
        Ok(_) => info!("Successfully applied custom trend analysis enhancement"),
        Err(e) => {
            error!("Failed to apply custom trend analysis enhancement: {}", e);
            // Continue with example even if this fails
        }
    }
    
    // 5. Use the full enhance_context method with anomaly detection
    info!("Using full enhance_context method for anomaly detection...");
    match context_adapter.enhance_context(
        context_id,
        ContextEnhancementType::AnomalyDetection,
        "openai",
        api_key,
        Some("gpt-4o".to_string()),
        Some(30000),
        None,
        None,
    ).await {
        Ok(_) => info!("Successfully applied anomaly detection enhancement"),
        Err(e) => {
            error!("Failed to apply anomaly detection enhancement: {}", e);
            // Continue with example even if this fails
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
    
    info!("AI Context Enhancement example completed");
    
    Ok(())
} 