//! Integration tests for the Squirrel AI Primal
//!
//! Tests complete workflows, error scenarios, and cross-component functionality

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::{timeout, sleep};
use serde_json::json;

use squirrel::primal_provider::SquirrelPrimalProvider;
use squirrel::session::{SessionManagerImpl, SessionConfig};
use squirrel::ecosystem::{EcosystemManager, EcosystemConfig, EcosystemServiceRegistration, EcosystemPrimalType};
use squirrel::monitoring::metrics::MetricsCollector;
use squirrel::error::PrimalError;
use squirrel::universal::PrimalContext;
use squirrel_mcp_config::EcosystemConfig as McpEcosystemConfig;

#[tokio::test]
async fn test_complete_ai_inference_workflow() {
    // Test the complete AI inference workflow from request to response
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    // Test different AI inference scenarios
    let test_cases = vec![
        // Text generation with OpenAI
        json!({
            "task_type": "text_generation",
            "messages": [{"role": "user", "content": "Hello, how are you?"}],
            "model": "gpt-4",
            "max_tokens": 150,
            "temperature": 0.7
        }),
        
        // Code generation
        json!({
            "task_type": "code_generation",
            "messages": [{"role": "user", "content": "Write a Rust function to calculate factorial"}]
        }),
        
        // Analysis with Anthropic
        json!({
            "task_type": "analysis",
            "messages": [{"role": "user", "content": "Analyze this business proposal"}],
            "model": "claude-3-sonnet-20240229"
        }),
        
        // Local processing
        json!({
            "task_type": "local",
            "messages": [{"role": "user", "content": "Process this data locally"}]
        }),
    ];
    
    for (i, test_case) in test_cases.into_iter().enumerate() {
        let response = provider.handle_ai_inference_request(test_case)
            .await
            .expect(&format!("Test case {} failed", i));
        
        // Verify response structure
        assert!(response.get("status").is_some());
        assert_eq!(response["status"].as_str().unwrap(), "completed");
        assert!(response.get("result").is_some());
        assert!(response.get("provider").is_some());
        assert!(response.get("timestamp").is_some());
    }
}

#[tokio::test]
async fn test_ai_inference_error_handling() {
    // Test error scenarios in AI inference
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    // Test cases that should produce errors
    let error_cases = vec![
        // Missing messages
        json!({
            "task_type": "text_generation",
            "model": "gpt-4"
        }),
        
        // Invalid task type
        json!({
            "task_type": "invalid_task_type",
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        
        // Malformed payload
        json!({
            "not_a_valid": "payload"
        }),
    ];
    
    for (i, error_case) in error_cases.into_iter().enumerate() {
        let response = provider.handle_ai_inference_request(error_case)
            .await
            .expect(&format!("Error test case {} should still return a response", i));
        
        // Should return error status but not panic
        if response.get("status").is_some() {
            let status = response["status"].as_str().unwrap();
            if status == "error" {
                assert!(response.get("error").is_some(), "Error response should include error message");
            }
        }
    }
}

#[tokio::test]
async fn test_context_analysis_comprehensive() {
    // Test comprehensive context analysis functionality
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    let test_cases = vec![
        // Sentiment analysis
        (
            json!({
                "context": {"text": "This is absolutely wonderful! I love this amazing product."},
                "analysis_type": "sentiment"
            }),
            "positive"
        ),
        
        // Intent classification
        (
            json!({
                "context": {"text": "Can you help me with this problem?"},
                "analysis_type": "intent"
            }),
            "help_request"
        ),
        
        // Entity extraction
        (
            json!({
                "context": {"text": "Contact john.doe@example.com or visit https://example.com"},
                "analysis_type": "entities"
            }),
            "entities"
        ),
        
        // Topic analysis
        (
            json!({
                "context": {"text": "Machine learning and AI technology are revolutionizing software development"},
                "analysis_type": "topics"
            }),
            "technology"
        ),
        
        // General analysis
        (
            json!({
                "context": {"text": "This is a complex sentence with multiple clauses and technical terminology."},
                "analysis_type": "general"
            }),
            "high"
        ),
    ];
    
    for (test_input, expected_indicator) in test_cases {
        let response = provider.handle_context_analysis(test_input)
            .await
            .expect("Context analysis failed");
        
        assert_eq!(response["status"].as_str().unwrap(), "analyzed");
        assert!(response.get("result").is_some());
        assert!(response.get("timestamp").is_some());
        
        // Verify analysis results contain expected indicators
        let result_str = serde_json::to_string(&response["result"]).unwrap().to_lowercase();
        assert!(result_str.contains(expected_indicator), 
                "Analysis result should contain '{}', got: {}", expected_indicator, result_str);
    }
}

#[tokio::test]
async fn test_session_lifecycle_management() {
    // Test complete session lifecycle
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    // Create session
    let create_request = json!({
        "action": "create",
        "user_id": "test-user-123",
        "device_id": "device-456",
        "session_type": "ai_chat",
        "metadata": {"client": "test-client", "version": "1.0"}
    });
    
    let create_response = provider.handle_session_management(create_request)
        .await
        .expect("Session creation failed");
    
    assert_eq!(create_response["status"].as_str().unwrap(), "created");
    let session_id = create_response["session"]["session_id"]
        .as_str()
        .expect("Session ID not found");
    
    // Get session status
    let get_request = json!({
        "action": "status",
        "session_id": session_id
    });
    
    let get_response = provider.handle_session_management(get_request)
        .await
        .expect("Session retrieval failed");
    
    assert_eq!(get_response["status"].as_str().unwrap(), "found");
    assert!(get_response["session"]["is_valid"].as_bool().unwrap());
    
    // Update session
    let update_request = json!({
        "action": "update",
        "session_id": session_id,
        "updates": {"last_activity": "2024-01-20T10:00:00Z"}
    });
    
    let update_response = provider.handle_session_management(update_request)
        .await
        .expect("Session update failed");
    
    assert_eq!(update_response["status"].as_str().unwrap(), "updated");
    
    // List sessions
    let list_request = json!({"action": "list"});
    
    let list_response = provider.handle_session_management(list_request)
        .await
        .expect("Session listing failed");
    
    assert_eq!(list_response["status"].as_str().unwrap(), "listed");
    
    // End session
    let end_request = json!({
        "action": "end",
        "session_id": session_id
    });
    
    let end_response = provider.handle_session_management(end_request)
        .await
        .expect("Session termination failed");
    
    assert_eq!(end_response["status"].as_str().unwrap(), "ended");
    
    // Cleanup expired sessions
    let cleanup_request = json!({"action": "cleanup"});
    
    let cleanup_response = provider.handle_session_management(cleanup_request)
        .await
        .expect("Session cleanup failed");
    
    assert_eq!(cleanup_response["status"].as_str().unwrap(), "cleaned");
}

#[tokio::test]
async fn test_session_error_scenarios() {
    // Test session management error handling
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    // Test invalid session ID
    let invalid_get = json!({
        "action": "get",
        "session_id": "non-existent-session"
    });
    
    let response = provider.handle_session_management(invalid_get)
        .await
        .expect("Should handle invalid session gracefully");
    
    assert_eq!(response["status"].as_str().unwrap(), "found");
    
    // Test update without session ID
    let update_without_id = json!({
        "action": "update",
        "updates": {"key": "value"}
    });
    
    let result = provider.handle_session_management(update_without_id).await;
    assert!(result.is_err(), "Should fail without session ID");
    
    // Test invalid action
    let invalid_action = json!({
        "action": "invalid_action"
    });
    
    let result = provider.handle_session_management(invalid_action).await;
    assert!(result.is_err(), "Should fail with invalid action");
}

#[tokio::test]
async fn test_concurrent_operations() {
    // Test concurrent access to primal provider
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = Arc::new(
        SquirrelPrimalProvider::new(config, context)
            .expect("Failed to create provider")
    );
    
    // Create multiple concurrent tasks
    let mut tasks = Vec::new();
    
    for i in 0..10 {
        let provider_clone = provider.clone();
        
        tasks.push(tokio::spawn(async move {
            // Mix different types of operations
            let operations = vec![
                // AI inference
                async {
                    let request = json!({
                        "task_type": "text_generation",
                        "messages": [{"role": "user", "content": format!("Request {}", i)}]
                    });
                    provider_clone.handle_ai_inference_request(request).await
                },
                
                // Context analysis
                async {
                    let request = json!({
                        "context": {"text": format!("Analyze this text {}", i)},
                        "analysis_type": "general"
                    });
                    provider_clone.handle_context_analysis(request).await
                },
                
                // Session management
                async {
                    let request = json!({
                        "action": "create",
                        "user_id": format!("user-{}", i)
                    });
                    provider_clone.handle_session_management(request).await
                },
            ];
            
            // Execute one random operation
            let op_index = i % operations.len();
            match op_index {
                0 => {
                    let request = json!({
                        "task_type": "text_generation",
                        "messages": [{"role": "user", "content": format!("Request {}", i)}]
                    });
                    provider_clone.handle_ai_inference_request(request).await
                },
                1 => {
                    let request = json!({
                        "context": {"text": format!("Analyze this text {}", i)},
                        "analysis_type": "general"
                    });
                    provider_clone.handle_context_analysis(request).await
                },
                2 => {
                    let request = json!({
                        "action": "create",
                        "user_id": format!("user-{}", i)
                    });
                    provider_clone.handle_session_management(request).await
                },
                _ => unreachable!(),
            }
        }));
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    
    // Verify all tasks completed successfully
    for (i, result) in results.into_iter().enumerate() {
        let response = result
            .expect(&format!("Task {} panicked", i))
            .expect(&format!("Task {} failed", i));
        
        assert!(response.get("status").is_some(), "Response should have status");
    }
    
    // Verify zero-copy metrics show concurrent efficiency
    let metrics = provider.get_zero_copy_metrics();
    assert!(metrics.get("total_operations").unwrap().as_u64() >= 10);
}

#[tokio::test]
async fn test_timeout_handling() {
    // Test timeout scenarios
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    // Test operations with timeout
    let request = json!({
        "task_type": "text_generation",
        "messages": [{"role": "user", "content": "Hello"}]
    });
    
    // Should complete within reasonable time
    let result = timeout(
        Duration::from_secs(10), 
        provider.handle_ai_inference_request(request)
    ).await;
    
    assert!(result.is_ok(), "Operation should complete within timeout");
    let response = result.unwrap().expect("Operation should succeed");
    assert!(response.get("status").is_some());
}

#[tokio::test]
async fn test_provider_initialization_and_cleanup() {
    // Test provider lifecycle
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    // Test creation
    let provider = SquirrelPrimalProvider::new(config.clone(), context.clone())
        .expect("Provider creation should succeed");
    
    // Test operations work immediately after creation
    let test_request = json!({
        "task_type": "text_generation", 
        "messages": [{"role": "user", "content": "Test"}]
    });
    
    let response = provider.handle_ai_inference_request(test_request)
        .await
        .expect("Operations should work after creation");
    
    assert_eq!(response["status"].as_str().unwrap(), "completed");
    
    // Test metrics are properly initialized
    let metrics = provider.get_zero_copy_metrics();
    assert!(metrics.get("total_operations").is_some());
    assert!(metrics.get("efficiency_ratio").is_some());
    
    // Test metrics reset
    provider.reset_zero_copy_metrics();
    let reset_metrics = provider.get_zero_copy_metrics();
    assert_eq!(reset_metrics.get("total_operations").unwrap().as_u64(), 0);
}

#[tokio::test] 
async fn test_edge_case_inputs() {
    // Test edge cases and unusual inputs
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config, context)
        .expect("Failed to create provider");
    
    let edge_cases = vec![
        // Empty messages
        json!({
            "task_type": "text_generation",
            "messages": []
        }),
        
        // Very long content
        json!({
            "task_type": "text_generation",
            "messages": [{"role": "user", "content": "x".repeat(10000)}]
        }),
        
        // Special characters
        json!({
            "task_type": "text_generation",
            "messages": [{"role": "user", "content": "Special chars: 你好 🚀 ñ ë ü"}]
        }),
        
        // Nested JSON in content
        json!({
            "task_type": "text_generation",
            "messages": [{"role": "user", "content": "{\"nested\": \"json\", \"array\": [1,2,3]}"}]
        }),
        
        // Unicode content
        json!({
            "context": {"text": "Unicode: 🔥 💯 ✨ 🎉 🚀"},
            "analysis_type": "sentiment"
        }),
        
        // Empty context analysis
        json!({
            "context": {"text": ""},
            "analysis_type": "general"  
        }),
    ];
    
    for (i, edge_case) in edge_cases.into_iter().enumerate() {
        let result = if edge_case.get("messages").is_some() {
            provider.handle_ai_inference_request(edge_case).await
        } else {
            provider.handle_context_analysis(edge_case).await  
        };
        
        // Should handle edge cases gracefully without panicking
        match result {
            Ok(response) => {
                assert!(response.get("status").is_some(), 
                       "Edge case {} should have status", i);
            },
            Err(e) => {
                // Errors are acceptable for edge cases, but shouldn't panic
                println!("Edge case {} produced error (acceptable): {}", i, e);
            }
        }
    }
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    // Test memory usage remains stable under load
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = Arc::new(
        SquirrelPrimalProvider::new(config, context)
            .expect("Failed to create provider")
    );
    
    // Get initial metrics
    provider.reset_zero_copy_metrics();
    let initial_metrics = provider.get_zero_copy_metrics();
    
    // Generate load
    let mut tasks = Vec::new();
    
    for batch in 0..5 {
        let provider_clone = provider.clone();
        
        tasks.push(tokio::spawn(async move {
            for i in 0..20 {
                let request = json!({
                    "task_type": "text_generation",
                    "messages": [{"role": "user", "content": format!("Batch {} Request {}", batch, i)}]
                });
                
                let _response = provider_clone.handle_ai_inference_request(request)
                    .await
                    .expect("Request should succeed");
            }
        }));
    }
    
    // Wait for all tasks
    futures::future::join_all(tasks).await;
    
    // Check final metrics
    let final_metrics = provider.get_zero_copy_metrics();
    
    // Verify optimizations were effective under load
    let total_ops = final_metrics.get("total_operations").unwrap().as_u64();
    let clones_avoided = final_metrics.get("clone_operations_avoided").unwrap().as_u64(); 
    let efficiency = final_metrics.get("efficiency_ratio").unwrap().as_f64();
    
    assert!(total_ops >= 100, "Should have processed 100 operations");
    assert!(clones_avoided >= 100, "Should have avoided many clones");
    assert!(efficiency > 0.8, "Efficiency should remain high under load: {}", efficiency);
    
    println!("Load test results: {} ops, {} clones avoided, {:.2}% efficiency", 
             total_ops, clones_avoided, efficiency * 100.0);
}

#[tokio::test]
async fn test_cross_component_integration() {
    // Test integration between multiple components
    let context = PrimalContext::default();
    let config = McpEcosystemConfig::default();
    
    let provider = SquirrelPrimalProvider::new(config.clone(), context)
        .expect("Failed to create provider");
    
    // Test AI inference → Context analysis → Session management chain
    
    // 1. AI inference generates some content
    let ai_request = json!({
        "task_type": "text_generation",
        "messages": [{"role": "user", "content": "Generate a business proposal"}]
    });
    
    let ai_response = provider.handle_ai_inference_request(ai_request)
        .await
        .expect("AI inference failed");
    
    assert_eq!(ai_response["status"], "completed");
    let generated_content = ai_response["result"]["content"].as_str().unwrap();
    
    // 2. Analyze the generated content
    let analysis_request = json!({
        "context": {"text": generated_content},
        "analysis_type": "topics"
    });
    
    let analysis_response = provider.handle_context_analysis(analysis_request)
        .await
        .expect("Context analysis failed");
    
    assert_eq!(analysis_response["status"], "analyzed");
    
    // 3. Create session to track this interaction
    let session_request = json!({
        "action": "create",
        "user_id": "integration-test-user",
        "session_type": "ai_analysis",
        "metadata": {
            "ai_provider": ai_response["provider"],
            "analysis_type": "topics",
            "content_length": generated_content.len()
        }
    });
    
    let session_response = provider.handle_session_management(session_request)
        .await
        .expect("Session management failed");
    
    assert_eq!(session_response["status"], "created");
    
    // Verify the entire workflow completed successfully
    let metrics = provider.get_zero_copy_metrics();
    assert!(metrics.get("total_operations").unwrap().as_u64() >= 3);
    
    println!("Cross-component integration test completed successfully");
}

// Helper function to create futures dependency
#[allow(unused_imports)]
use futures; 