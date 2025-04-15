//! Batch processing tests for Context-MCP AI enhancements
//!
//! These tests verify the adapter's ability to efficiently process
//! multiple context enhancements in parallel.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use serde_json::json;
use futures::future::{join_all, FutureExt};

use squirrel_integration::context_mcp::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    ContextEnhancementType,
    ContextAiEnhancementOptions,
    SyncDirection,
    SquirrelContext,
    ContextManager,
    Result as ContextResult,
};

// Import the MockContextManager from the e2e_tests
mock! {
    pub ContextManager {}
    
    #[async_trait]
    impl ContextManager for ContextManager {
        async fn create_context(
            &self, 
            id: &str, 
            name: &str, 
            data: serde_json::Value, 
            metadata: Option<serde_json::Value>
        ) -> anyhow::Result<()>;
        
        async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext>;
        
        async fn update_context(
            &self, 
            id: &str, 
            data: serde_json::Value, 
            metadata: Option<serde_json::Value>
        ) -> anyhow::Result<()>;
        
        async fn delete_context(&self, id: &str) -> anyhow::Result<()>;
        
        async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>>;
    }
}

/// Create sample test contexts with varying data for batch processing
fn create_test_contexts(count: usize) -> Vec<SquirrelContext> {
    let mut contexts = Vec::with_capacity(count);
    
    for i in 0..count {
        // Create slightly different data for each context
        let context = SquirrelContext {
            id: format!("batch-context-{}", i),
            name: format!("Batch Test Context {}", i),
            data: json!({
                "metrics": [
                    {"name": "cpu_usage", "values": [10+i, 15+i, 25+i, 45+i, 60+i, 30+i, 20+i]},
                    {"name": "memory_usage", "values": [200+i*10, 250+i*10, 300+i*10, 350+i*10, 380+i*10, 400+i*10, 420+i*10]},
                    {"name": "network_traffic", "values": [1000, 1200, 800+i*100, 1500, 1800, 2000, 1600]}
                ],
                "events": [
                    {"timestamp": format!("2023-10-{:02}T10:00:00Z", (i % 30) + 1), "type": "system_start", "details": "Normal startup"},
                    {"timestamp": format!("2023-10-{:02}T12:30:00Z", (i % 30) + 1), "type": "high_load", "details": "CPU spike detected"},
                    {"timestamp": format!("2023-10-{:02}T14:45:00Z", (i % 30) + 1), "type": "error", "details": "Connection timeout"}
                ]
            }),
            metadata: json!({
                "source": "monitoring_system",
                "timestamp": format!("2023-10-{:02}T15:00:00Z", (i % 30) + 1),
                "tags": ["system", "performance", "test", format!("batch-{}", i)]
            }),
        };
        
        contexts.push(context);
    }
    
    contexts
}

/// Setup a test adapter with mocked context manager for batch processing
async fn setup_batch_test_adapter(context_count: usize) -> (ContextMcpAdapter, MockContextManager, Arc<Mutex<Vec<String>>>) {
    // Create mock context manager
    let mut mock_cm = MockContextManager::new();
    
    // Create test contexts
    let contexts = create_test_contexts(context_count);
    
    // Setup context retrieval for each context
    for context in &contexts {
        let context_clone = context.clone();
        let context_id = context.id.clone();
        
        mock_cm.expect_with_context()
            .with(eq(context_id))
            .returning(move |_| Ok(context_clone.clone()));
    }
    
    // Setup list_contexts
    let contexts_clone = contexts.clone();
    mock_cm.expect_list_contexts()
        .returning(move || Ok(contexts_clone.clone()));
    
    // Setup update_context
    let calls = Arc::new(Mutex::new(Vec::new()));
    let calls_clone = calls.clone();
    mock_cm.expect_update_context()
        .returning(move |id, data, metadata| {
            let mut call_list = calls_clone.lock().unwrap();
            call_list.push(format!("update_context({}, {}, {:?})", id, data, metadata));
            
            // Add some random delay to simulate varying processing times
            let delay_ms = (id.len() as u64 % 5) * 10; // 0-40ms delay
            tokio::spawn(async move {
                sleep(Duration::from_millis(delay_ms)).await;
            });
            
            Ok(())
        });
    
    // Create adapter config
    let config = ContextMcpAdapterConfig {
        sync_interval_secs: 5,
        sync_direction: SyncDirection::Bidirectional,
        ..Default::default()
    };
    
    // Create adapter
    let adapter = ContextMcpAdapter::new(config, Box::new(mock_cm.clone()));
    
    (adapter, mock_cm, calls)
}

/// Implementation of batch enhancement for the adapter
/// In a real implementation, this would be implemented in the actual adapter
async fn batch_enhance_contexts(
    adapter: &ContextMcpAdapter,
    context_ids: Vec<String>,
    enhancement_type: ContextEnhancementType,
    max_concurrency: usize,
) -> HashMap<String, anyhow::Result<()>> {
    let mut results = HashMap::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrency));
    
    let futures = context_ids.into_iter().map(|id| {
        let adapter_clone = adapter.clone();
        let enhancement_type_clone = enhancement_type.clone();
        let semaphore_clone = semaphore.clone();
        
        async move {
            // Acquire permit from semaphore to limit concurrency
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            // Create enhancement options
            let options = ContextAiEnhancementOptions::new(
                enhancement_type_clone,
                "openai",
                "test-api-key"
            ).with_model("gpt-4o");
            
            // Apply enhancement
            let result = adapter_clone.apply_ai_enhancements(&id, options).await;
            
            (id, result.map_err(|e| e.into()))
        }.boxed()
    });
    
    // Execute all futures and collect results
    let all_results = join_all(futures).await;
    
    for (id, result) in all_results {
        results.insert(id, result);
    }
    
    results
}

/// Test the batch processing with small number of contexts
#[tokio::test]
async fn test_batch_processing_small() {
    // Setup test environment with 5 contexts
    let (adapter, _, update_calls) = setup_batch_test_adapter(5).await;
    
    // Get all context IDs
    let contexts = create_test_contexts(5);
    let context_ids: Vec<_> = contexts.iter().map(|c| c.id.clone()).collect();
    
    // Process in batch with max concurrency of 2
    let start_time = Instant::now();
    let results = batch_enhance_contexts(&adapter, context_ids, ContextEnhancementType::Summary, 2).await;
    let elapsed = start_time.elapsed();
    
    println!("Processed 5 contexts in {:?} (max concurrency: 2)", elapsed);
    
    // Verify all contexts were processed
    assert_eq!(results.len(), 5);
    assert!(results.values().all(|r| r.is_ok()));
    
    // Verify contexts were updated
    let calls = update_calls.lock().unwrap();
    assert_eq!(calls.len(), 5);
}

/// Test the batch processing with larger number of contexts
#[tokio::test]
async fn test_batch_processing_medium() {
    // Setup test environment with 20 contexts
    let (adapter, _, update_calls) = setup_batch_test_adapter(20).await;
    
    // Get all context IDs
    let contexts = create_test_contexts(20);
    let context_ids: Vec<_> = contexts.iter().map(|c| c.id.clone()).collect();
    
    // Process in batch with max concurrency of 5
    let start_time = Instant::now();
    let results = batch_enhance_contexts(&adapter, context_ids, ContextEnhancementType::Insights, 5).await;
    let elapsed = start_time.elapsed();
    
    println!("Processed 20 contexts in {:?} (max concurrency: 5)", elapsed);
    
    // Verify all contexts were processed
    assert_eq!(results.len(), 20);
    assert!(results.values().all(|r| r.is_ok()));
    
    // Verify contexts were updated
    let calls = update_calls.lock().unwrap();
    assert_eq!(calls.len(), 20);
}

/// Test different concurrency levels to find optimal settings
#[tokio::test]
async fn test_concurrency_optimization() {
    // Setup test environment with 30 contexts
    let (adapter, _, _) = setup_batch_test_adapter(30).await;
    
    // Get all context IDs
    let contexts = create_test_contexts(30);
    let context_ids: Vec<_> = contexts.iter().map(|c| c.id.clone()).collect();
    
    // Test with different concurrency levels
    let concurrency_levels = [1, 5, 10, 20, 30];
    
    println!("Comparing different concurrency levels for 30 contexts:");
    
    for &concurrency in &concurrency_levels {
        let start_time = Instant::now();
        let results = batch_enhance_contexts(
            &adapter, 
            context_ids.clone(), 
            ContextEnhancementType::TrendAnalysis,
            concurrency
        ).await;
        let elapsed = start_time.elapsed();
        
        println!("  Concurrency {}: {:?}", concurrency, elapsed);
        
        // Verify all contexts were processed
        assert_eq!(results.len(), 30);
        assert!(results.values().all(|r| r.is_ok()));
    }
}

/// Test batch processing with different enhancement types
#[tokio::test]
async fn test_batch_different_enhancement_types() {
    // Setup test environment with 10 contexts
    let (adapter, _, update_calls) = setup_batch_test_adapter(10).await;
    
    // Get context IDs
    let contexts = create_test_contexts(10);
    let context_ids: Vec<_> = contexts.iter().map(|c| c.id.clone()).collect();
    
    // Create different enhancement types for each context
    let enhancement_types = [
        ContextEnhancementType::Insights,
        ContextEnhancementType::Summary,
        ContextEnhancementType::Recommendations,
        ContextEnhancementType::TrendAnalysis,
        ContextEnhancementType::AnomalyDetection,
        ContextEnhancementType::Custom("Security risk assessment".to_string()),
        ContextEnhancementType::Insights,
        ContextEnhancementType::Summary,
        ContextEnhancementType::Recommendations,
        ContextEnhancementType::TrendAnalysis,
    ];
    
    // Process each context with its own enhancement type
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Max 5 concurrent
    
    let futures = context_ids.iter().enumerate().map(|(i, id)| {
        let adapter_clone = adapter.clone();
        let enhancement_type = enhancement_types[i].clone();
        let semaphore_clone = semaphore.clone();
        let id_clone = id.clone();
        
        async move {
            // Acquire permit from semaphore to limit concurrency
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            // Create enhancement options
            let options = ContextAiEnhancementOptions::new(
                enhancement_type,
                "openai",
                "test-api-key"
            ).with_model("gpt-4o");
            
            // Apply enhancement
            adapter_clone.apply_ai_enhancements(&id_clone, options).await
        }.boxed()
    });
    
    // Execute all futures
    let start_time = Instant::now();
    let results = join_all(futures).await;
    let elapsed = start_time.elapsed();
    
    println!("Processed 10 contexts with different enhancement types in {:?}", elapsed);
    
    // Verify all enhancements succeeded
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verify contexts were updated
    let calls = update_calls.lock().unwrap();
    assert_eq!(calls.len(), 10);
}

/// Test error handling during batch processing
#[tokio::test]
async fn test_batch_error_handling() {
    // Setup test environment with 10 contexts
    let (adapter, mut mock_cm, _) = setup_batch_test_adapter(10).await;
    
    // Modify one context retrieval to fail
    mock_cm.expect_with_context()
        .with(eq("batch-context-5"))
        .returning(|_| Err(anyhow::anyhow!("Context not found or access denied")));
    
    // Get context IDs
    let contexts = create_test_contexts(10);
    let context_ids: Vec<_> = contexts.iter().map(|c| c.id.clone()).collect();
    
    // Process in batch
    let results = batch_enhance_contexts(&adapter, context_ids, ContextEnhancementType::Summary, 5).await;
    
    // Verify we got results for all contexts
    assert_eq!(results.len(), 10);
    
    // Verify that context-5 failed but others succeeded
    let success_count = results.values().filter(|r| r.is_ok()).count();
    let error_count = results.values().filter(|r| r.is_err()).count();
    
    assert_eq!(success_count, 9, "9 contexts should have succeeded");
    assert_eq!(error_count, 1, "1 context should have failed");
    
    // Verify the correct context failed
    assert!(results.get("batch-context-5").unwrap().is_err());
} 