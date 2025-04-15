//! Integration tests for Context AI enhancement functionality

use std::sync::Arc;
use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
    ContextAiEnhancementOptions,
    apply_ai_enhancement,
};
use serde_json::json;
use tracing::{info, debug};

/// Test the creation of AI enhancement options with different enhancement types
#[tokio::test]
async fn test_enhancement_options_creation() {
    // Test standard enhancement types
    let summary_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Summary,
        "openai",
        "test-key-123"
    )
    .with_model("gpt-4")
    .with_timeout(10000);
    
    assert_eq!(summary_options.enhancement_type, ContextEnhancementType::Summary);
    assert_eq!(summary_options.provider, "openai");
    assert_eq!(summary_options.model, Some("gpt-4".to_string()));
    assert_eq!(summary_options.timeout_ms, Some(10000));
    
    // Test equivalent Summarize variant (should behave the same)
    let summarize_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Summarize,
        "openai",
        "test-key-123"
    )
    .with_model("gpt-4");
    
    assert_eq!(summarize_options.provider, "openai");
    assert_eq!(summarize_options.model, Some("gpt-4".to_string()));
    
    // Test trend analysis
    let trend_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::TrendAnalysis,
        "anthropic",
        "test-anthropic-key"
    )
    .with_system_prompt("Analyze trends in the following data:")
    .with_model("claude-3-sonnet");
    
    assert_eq!(trend_options.enhancement_type, ContextEnhancementType::TrendAnalysis);
    assert_eq!(trend_options.provider, "anthropic");
    assert_eq!(trend_options.custom_prompt, Some("Analyze trends in the following data:".to_string()));
    
    // Test custom enhancement type with additional parameters
    let custom_options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Custom("sentiment_analysis".to_string()),
        "openai",
        "test-key-456"
    )
    .with_model("gpt-4")
    .with_parameter("language", "en")
    .with_parameter("max_score", 10)
    .with_parameter("categories", json!(["positive", "negative", "neutral"]));
    
    // Validate that enhancement type is correct
    match &custom_options.enhancement_type {
        ContextEnhancementType::Custom(prompt) => assert_eq!(prompt, "sentiment_analysis"),
        _ => panic!("Expected Custom enhancement type, got {:?}", custom_options.enhancement_type),
    }
    
    // Validate parameters were stored correctly
    assert!(custom_options.parameters.contains_key("language"));
    assert_eq!(custom_options.parameters.get("language").unwrap().as_str().unwrap(), "en");
    
    assert!(custom_options.parameters.contains_key("max_score"));
    assert_eq!(custom_options.parameters.get("max_score").unwrap().as_i64().unwrap(), 10);
    
    assert!(custom_options.parameters.contains_key("categories"));
    let categories = custom_options.parameters.get("categories").unwrap().as_array().unwrap();
    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].as_str().unwrap(), "positive");
}

/// Test that enhancement types can be properly displayed as strings
#[test]
fn test_enhancement_type_display() {
    // Test standard enhancement types
    assert_eq!(ContextEnhancementType::Summary.to_string(), "Summary");
    assert_eq!(ContextEnhancementType::Summarize.to_string(), "Summarize");
    assert_eq!(ContextEnhancementType::Insights.to_string(), "Insights");
    assert_eq!(ContextEnhancementType::Recommendations.to_string(), "Recommendations");
    assert_eq!(ContextEnhancementType::TrendAnalysis.to_string(), "TrendAnalysis");
    assert_eq!(ContextEnhancementType::AnomalyDetection.to_string(), "AnomalyDetection");
    
    // Test custom enhancement type
    assert_eq!(
        ContextEnhancementType::Custom("test_custom".to_string()).to_string(),
        "Custom: test_custom"
    );
}

/// Test enhancement option chaining
#[test]
fn test_enhancement_option_chaining() {
    // Create options with chained method calls
    let options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Insights,
        "openai",
        "test-key"
    )
    .with_model("gpt-4")
    .with_timeout(15000)
    .with_system_prompt("Provide detailed insights:")
    .with_parameter("detailed", true)
    .with_parameter("format", "markdown")
    .with_parameter("focus_areas", json!(["performance", "security", "usability"]));
    
    // Verify all options are set correctly
    assert_eq!(options.enhancement_type, ContextEnhancementType::Insights);
    assert_eq!(options.provider, "openai");
    assert_eq!(options.api_key, "test-key");
    assert_eq!(options.model, Some("gpt-4".to_string()));
    assert_eq!(options.timeout_ms, Some(15000));
    assert_eq!(options.custom_prompt, Some("Provide detailed insights:".to_string()));
    
    // Verify parameters
    assert!(options.parameters.contains_key("detailed"));
    assert!(options.parameters.get("detailed").unwrap().as_bool().unwrap());
    
    assert!(options.parameters.contains_key("format"));
    assert_eq!(options.parameters.get("format").unwrap().as_str().unwrap(), "markdown");
    
    assert!(options.parameters.contains_key("focus_areas"));
    let focus_areas = options.parameters.get("focus_areas").unwrap().as_array().unwrap();
    assert_eq!(focus_areas.len(), 3);
} 