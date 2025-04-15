//! Integration tests for Context-MCP AI tools integration
//!
//! These tests verify the integration between Context-MCP and AI tools functionality.

use std::sync::Arc;
use std::env;

use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
    ContextAiEnhancementOptions,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};
use serde_json::json;

#[tokio::test]
async fn test_context_ai_enhancements() {
    // Skip this test if no API key is available
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_context_ai_enhancements: OPENAI_API_KEY not set");
            return;
        }
    };
    
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test context
    let context_id = "test-ai-enhancements";
    let test_context = Context {
        id: context_id.to_string(),
        account_id: "test-account".to_string(),
        metadata: Some(json!({
            "test_data": {
                "name": "Test Project",
                "metrics": {
                    "value1": 42,
                    "value2": 73
                }
            }
        })),
        ..Default::default()
    };
    
    // Create the context in MCP
    if let Err(e) = mcp_adapter.create_context(&test_context).await {
        panic!("Failed to create test context: {}", e);
    }
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test insights enhancement
    let options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Insights,
        "openai",
        api_key,
    )
    .with_model("gpt-3.5-turbo") // Use cheaper model for tests
    .with_timeout(20000);
    
    match context_adapter.apply_ai_enhancements(context_id, options).await {
        Ok(_) => {
            // Verify the context was updated with AI insights
            match mcp_adapter.get_context(context_id).await {
                Ok(context) => {
                    let metadata = context.metadata.expect("Context should have metadata");
                    let ai_insights = metadata.get("ai_insights").expect("Context should have ai_insights");
                    
                    assert!(ai_insights.is_object(), "AI insights should be an object");
                    assert!(ai_insights.as_object().unwrap().contains_key("Insights"), 
                           "AI insights should contain the enhancement type");
                },
                Err(e) => panic!("Failed to get updated context: {}", e),
            }
        },
        Err(e) => panic!("Failed to apply AI enhancements: {}", e),
    }
    
    // Clean up
    if let Err(e) = mcp_adapter.delete_context(context_id).await {
        panic!("Failed to delete test context: {}", e);
    }
}

#[test]
fn test_enhancement_options() {
    // Test option builder pattern
    let options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Recommendations,
        "openai",
        "test-key",
    )
    .with_model("gpt-4o")
    .with_timeout(15000)
    .with_parameter("temperature", 0.7)
    .with_system_prompt("Custom prompt");
    
    assert_eq!(options.provider, "openai");
    assert_eq!(options.api_key, "test-key");
    assert_eq!(options.model, Some("gpt-4o".to_string()));
    assert_eq!(options.timeout_ms, Some(15000));
    assert_eq!(options.system_prompt, Some("Custom prompt".to_string()));
    
    // Test parameter addition
    assert!(options.parameters.contains_key("temperature"));
    let temp_value = options.parameters.get("temperature").unwrap();
    assert_eq!(temp_value.as_f64().unwrap(), 0.7);
}

#[test]
fn test_enhancement_types() {
    // Test all enhancement types
    let types = vec![
        ContextEnhancementType::Summarize,
        ContextEnhancementType::Insights,
        ContextEnhancementType::TrendAnalysis,
        ContextEnhancementType::Recommendations,
        ContextEnhancementType::AnomalyDetection,
        ContextEnhancementType::Custom("Test custom instruction".to_string()),
    ];
    
    for enhancement_type in types {
        let options = ContextAiEnhancementOptions::new(
            enhancement_type,
            "test-provider",
            "test-key",
        );
        
        match options.enhancement_type {
            ContextEnhancementType::Custom(ref instruction) => {
                assert_eq!(instruction, "Test custom instruction");
            },
            _ => {},
        }
    }
} 