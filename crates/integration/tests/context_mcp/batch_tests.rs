//! Integration tests for Context-MCP batch processing functionality
//!
//! These tests verify the batch processing capabilities of the Context-MCP adapter.

use std::sync::Arc;
use std::env;
use std::collections::HashMap;

use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
    ContextAiEnhancementOptions,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};
use serde_json::json;

/// Helper function to create test contexts in MCP
async fn create_test_contexts(mcp_adapter: &Arc<MCPAdapter>, count: usize, prefix: &str) -> Vec<String> {
    let mut context_ids = Vec::new();
    
    for i in 0..count {
        let context_id = format!("{}-{}", prefix, i);
        context_ids.push(context_id.clone());
        
        let test_context = Context {
            id: context_id.clone(),
            account_id: "test-account".to_string(),
            metadata: Some(json!({
                "test_data": {
                    "name": format!("Test Project {}", i),
                    "index": i,
                    "tags": ["test", if i % 2 == 0 { "even" } else { "odd" }],
                    "metrics": {
                        "value1": 40 + i,
                        "value2": 70 + i
                    }
                }
            })),
            ..Default::default()
        };
        
        if let Err(e) = mcp_adapter.create_context(&test_context).await {
            panic!("Failed to create test context {}: {}", context_id, e);
        }
    }
    
    context_ids
}

/// Helper function to clean up test contexts
async fn delete_test_contexts(mcp_adapter: &Arc<MCPAdapter>, context_ids: &[String]) {
    for context_id in context_ids {
        if let Err(e) = mcp_adapter.delete_context(context_id).await {
            eprintln!("Warning: Failed to delete test context {}: {}", context_id, e);
        }
    }
}

#[tokio::test]
async fn test_batch_enhancement() {
    // Skip this test if no API key is available
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_batch_enhancement: OPENAI_API_KEY not set");
            return;
        }
    };
    
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test contexts
    let context_ids = create_test_contexts(&mcp_adapter, 3, "test-batch").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test batch enhancements
    let options = ContextAiEnhancementOptions::new(
        ContextEnhancementType::Summarize,
        "openai",
        api_key,
    )
    .with_model("gpt-3.5-turbo") // Use cheaper model for tests
    .with_timeout(20000);
    
    match context_adapter.batch_enhance_contexts(context_ids.clone(), options, Some(2)).await {
        Ok(results) => {
            // Verify all contexts were processed
            assert_eq!(results.len(), context_ids.len(), "All contexts should be processed");
            
            // Verify all operations were successful
            for (context_id, result) in &results {
                assert!(result.is_ok(), "Enhancement of {} should succeed", context_id);
            }
            
            // Verify the contexts were updated with AI summaries
            for context_id in &context_ids {
                match mcp_adapter.get_context(context_id).await {
                    Ok(context) => {
                        let metadata = context.metadata.expect("Context should have metadata");
                        let ai_insights = metadata.get("ai_insights").expect("Context should have ai_insights");
                        
                        assert!(ai_insights.is_object(), "AI insights should be an object");
                        assert!(ai_insights.as_object().unwrap().contains_key("Summarize"), 
                               "AI insights should contain the enhancement type");
                    },
                    Err(e) => panic!("Failed to get updated context {}: {}", context_id, e),
                }
            }
        },
        Err(e) => panic!("Failed to apply batch AI enhancements: {}", e),
    }
    
    // Clean up
    delete_test_contexts(&mcp_adapter, &context_ids).await;
}

#[tokio::test]
async fn test_find_contexts_by_tags() {
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test contexts with different tags
    let context_ids = create_test_contexts(&mcp_adapter, 6, "test-tags").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test finding contexts by "even" tag
    match context_adapter.find_contexts_by_tags(vec!["even".to_string()], true).await {
        Ok(found_contexts) => {
            assert_eq!(found_contexts.len(), 3, "Should find 3 contexts with 'even' tag");
            
            // Check that all are even-indexed contexts
            for context_id in &found_contexts {
                assert!(context_id.contains("test-tags-0") || 
                       context_id.contains("test-tags-2") ||
                       context_id.contains("test-tags-4"),
                       "Found unexpected context: {}", context_id);
            }
        },
        Err(e) => panic!("Failed to find contexts by tags: {}", e),
    }
    
    // Test finding contexts by multiple tags (any match)
    match context_adapter.find_contexts_by_tags(
        vec!["even".to_string(), "nonexistent".to_string()], 
        false // any match
    ).await {
        Ok(found_contexts) => {
            assert_eq!(found_contexts.len(), 3, "Should find 3 contexts with 'even' tag (any match)");
        },
        Err(e) => panic!("Failed to find contexts by tags (any match): {}", e),
    }
    
    // Clean up
    delete_test_contexts(&mcp_adapter, &context_ids).await;
}

#[tokio::test]
async fn test_batch_enhance_with_type() {
    // Skip this test if no API key is available
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_batch_enhance_with_type: OPENAI_API_KEY not set");
            return;
        }
    };
    
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test contexts
    let context_ids = create_test_contexts(&mcp_adapter, 2, "test-batch-type").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test batch_enhance_with_type convenience method
    match context_adapter.batch_enhance_with_type(
        context_ids.clone(),
        ContextEnhancementType::Insights,
        "openai",
        api_key,
        Some("gpt-3.5-turbo"),
        Some(2)
    ).await {
        Ok(results) => {
            assert_eq!(results.len(), context_ids.len(), "All contexts should be processed");
            
            // Verify all operations were successful
            for (context_id, result) in &results {
                assert!(result.is_ok(), "Enhancement of {} should succeed", context_id);
            }
        },
        Err(e) => panic!("Failed to apply batch enhancements with type: {}", e),
    }
    
    // Clean up
    delete_test_contexts(&mcp_adapter, &context_ids).await;
}

// This test is more complex and may be omitted in CI environments due to cost
#[tokio::test]
#[ignore] // Mark as ignored by default to avoid running in normal test suite
async fn test_batch_enhance_with_multiple_models() {
    // Skip this test if no API key is available
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_batch_enhance_with_multiple_models: OPENAI_API_KEY not set");
            return;
        }
    };
    
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test contexts
    let context_ids = create_test_contexts(&mcp_adapter, 2, "test-multi-model").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test enhancing with multiple models
    let models = vec!["gpt-3.5-turbo".to_string(), "gpt-4o".to_string()];
    
    match context_adapter.batch_enhance_with_multiple_models(
        context_ids.clone(),
        ContextEnhancementType::Summarize,
        "openai",
        api_key,
        models.clone(),
        Some(2)
    ).await {
        Ok(results) => {
            assert_eq!(results.len(), context_ids.len(), "All contexts should be processed");
            
            // Verify both models were run on each context
            for context_id in &context_ids {
                let context_results = results.get(context_id).expect("Context should have results");
                assert_eq!(context_results.len(), models.len(), "Each context should be processed by all models");
                
                for model in &models {
                    assert!(context_results.contains_key(model), "Context should have results for model {}", model);
                }
            }
        },
        Err(e) => panic!("Failed to apply multi-model batch enhancements: {}", e),
    }
    
    // Clean up
    delete_test_contexts(&mcp_adapter, &context_ids).await;
}

#[tokio::test]
#[ignore] // Mark as ignored by default to avoid running in normal test suite
async fn test_evaluate_models_for_enhancement() {
    // Skip this test if no API key is available
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_evaluate_models_for_enhancement: OPENAI_API_KEY not set");
            return;
        }
    };
    
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test contexts
    let context_ids = create_test_contexts(&mcp_adapter, 2, "test-evaluate").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig::default();
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Test model evaluation
    let models = vec!["gpt-3.5-turbo".to_string(), "gpt-4o".to_string()];
    
    // Custom evaluator that always prefers gpt-4o
    let evaluator = Arc::new(|_context_id: &str, model: &str| -> f64 {
        if model == "gpt-4o" {
            0.9 // Pretend gpt-4o is better
        } else {
            0.5
        }
    });
    
    match context_adapter.evaluate_models_for_enhancement(
        context_ids.clone(),
        ContextEnhancementType::Insights,
        "openai",
        api_key,
        models,
        Some(evaluator)
    ).await {
        Ok(best_model) => {
            assert_eq!(best_model, "gpt-4o", "GPT-4o should be evaluated as the best model");
        },
        Err(e) => panic!("Failed to evaluate models: {}", e),
    }
    
    // Clean up
    delete_test_contexts(&mcp_adapter, &context_ids).await;
} 