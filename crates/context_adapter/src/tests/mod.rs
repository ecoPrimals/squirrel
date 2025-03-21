use std::sync::Arc;

use serde_json::json;
use tokio::test;

use crate::{ContextAdapter, ContextAdapterConfig, ContextAdapterFactory, create_context_adapter, create_context_adapter_with_config};

/// Test data structure used for context adapter testing
pub struct TestData {
    /// The test message content
    pub message: String,
    /// The test numeric value
    pub value: i32,
}

impl TestData {
    /// Creates a new test data value as JSON
    #[must_use] pub fn new(message: &str, value: i32) -> serde_json::Value {
        serde_json::json!({
            "message": message,
            "value": value
        })
    }
}

#[test]
async fn test_adapter_initialization() {
    let adapter = ContextAdapter::default();
    
    // Verify adapter state
    let initial_config = adapter.get_config().await.unwrap();
    assert_eq!(initial_config.max_contexts, 1000);
    
    // Test context operations
    let context_id = "test-init";
    let data = json!({ "test": true });
    
    // Create a context
    let create_result = adapter.create_context(context_id.to_string(), data.clone()).await;
    assert!(create_result.is_ok());
    
    // Verify context was created
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 1);
}

#[test]
async fn test_adapter_with_config() {
    // Create test config
    let config = ContextAdapterConfig {
        max_contexts: 200,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
    };
    
    // Initialize with config
    let adapter = ContextAdapter::new(config.clone());
    
    // Check config is applied
    let retrieved_config = adapter.get_config().await.unwrap();
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(retrieved_config.enable_auto_cleanup, config.enable_auto_cleanup);
}

#[test]
async fn test_state_operations() {
    let adapter = ContextAdapter::default();
    
    // Create test context
    let context_id = "test-state";
    let test_state = json!({
        "message": "Test message",
        "value": 42
    });
    
    // Set state by creating a context
    adapter.create_context(context_id.to_string(), test_state.clone()).await.unwrap();
    
    // Get state by retrieving the context
    let context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(context.data, test_state);
    
    // Update state
    let updated_state = json!({
        "message": "Updated message",
        "value": 100
    });
    
    adapter.update_context(context_id, updated_state.clone()).await.unwrap();
    
    // Verify state was updated
    let updated_context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(updated_context.data, updated_state);
}

#[test]
async fn test_multiple_contexts() {
    let adapter = ContextAdapter::default();
    
    // Create multiple contexts
    adapter.create_context("context1".to_string(), json!({"id": 1})).await.unwrap();
    adapter.create_context("context2".to_string(), json!({"id": 2})).await.unwrap();
    adapter.create_context("context3".to_string(), json!({"id": 3})).await.unwrap();
    
    // Verify all contexts were created
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 3);
    
    // Verify context retrieval
    let context1 = adapter.get_context("context1").await.unwrap();
    let context2 = adapter.get_context("context2").await.unwrap();
    let context3 = adapter.get_context("context3").await.unwrap();
    
    assert_eq!(context1.data, json!({"id": 1}));
    assert_eq!(context2.data, json!({"id": 2}));
    assert_eq!(context3.data, json!({"id": 3}));
    
    // Delete a context
    adapter.delete_context("context2").await.unwrap();
    
    // Verify context was deleted
    let remaining_contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(remaining_contexts.len(), 2);
    assert!(adapter.get_context("context2").await.is_err());
}

#[test]
async fn test_thread_safety() {
    // Create shared adapter
    let adapter = Arc::new(ContextAdapter::default());
    
    // Test concurrent operations
    let adapter_clone1 = adapter.clone();
    let adapter_clone2 = adapter.clone();
    
    // Create contexts from different threads
    let handle1 = tokio::spawn(async move {
        adapter_clone1.create_context("thread1".to_string(), json!({"source": "thread1"})).await
    });
    
    let handle2 = tokio::spawn(async move {
        adapter_clone2.create_context("thread2".to_string(), json!({"source": "thread2"})).await
    });
    
    // Wait for both operations to complete
    let _ = handle1.await.unwrap();
    let _ = handle2.await.unwrap();
    
    // Verify both contexts were created successfully
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 2);
    
    // Verify we can retrieve both contexts
    let context1 = adapter.get_context("thread1").await.unwrap();
    let context2 = adapter.get_context("thread2").await.unwrap();
    
    assert_eq!(context1.data, json!({"source": "thread1"}));
    assert_eq!(context2.data, json!({"source": "thread2"}));
}

#[test]
async fn test_context_adapter_creation() {
    // Test default creation
    let adapter = ContextAdapter::default();
    let config = adapter.get_config().await.unwrap();
    assert_eq!(config.max_contexts, 1000);
    
    // Test with custom config
    let custom_config = ContextAdapterConfig {
        max_contexts: 500,
        ttl_seconds: 1800,
        enable_auto_cleanup: false,
    };
    let adapter = ContextAdapter::new(custom_config.clone());
    let adapter_config = adapter.get_config().await.unwrap();
    assert_eq!(adapter_config.max_contexts, 500);
    assert_eq!(adapter_config.ttl_seconds, 1800);
    assert!(!adapter_config.enable_auto_cleanup);
}

#[test]
async fn test_context_adapter_factory() {
    // Test factory with default config
    let adapter = ContextAdapterFactory::create_adapter();
    let config = adapter.get_config().await.unwrap();
    assert_eq!(config.max_contexts, 1000);
    
    // Test factory with custom config
    let custom_config = ContextAdapterConfig {
        max_contexts: 200,
        ttl_seconds: 900,
        enable_auto_cleanup: true,
    };
    let adapter = ContextAdapterFactory::create_adapter_with_config(custom_config.clone());
    let adapter_config = adapter.get_config().await.unwrap();
    assert_eq!(adapter_config.max_contexts, 200);
    assert_eq!(adapter_config.ttl_seconds, 900);
    assert!(adapter_config.enable_auto_cleanup);
    
    // Test helper functions
    let adapter1 = create_context_adapter();
    let adapter2 = create_context_adapter_with_config(custom_config);
    
    let config1 = adapter1.get_config().await.unwrap();
    let config2 = adapter2.get_config().await.unwrap();
    
    assert_eq!(config1.max_contexts, 1000);
    assert_eq!(config2.max_contexts, 200);
}

#[test]
async fn test_context_operations() {
    // Create an adapter
    let adapter = ContextAdapter::default();
    
    // Test context creation
    let context_id = "test-context";
    let data = json!({
        "key": "value",
        "number": 42
    });
    
    let result = adapter.create_context(context_id.to_string(), data.clone()).await;
    assert!(result.is_ok());
    
    // Test get context
    let context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(context.id, context_id);
    assert_eq!(context.data, data);
    
    // Test update context
    let updated_data = json!({
        "key": "updated_value",
        "number": 100
    });
    
    let update_result = adapter.update_context(context_id, updated_data.clone()).await;
    assert!(update_result.is_ok());
    
    let updated_context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(updated_context.data, updated_data);
    
    // Test list contexts
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].id, context_id);
    
    // Test delete context
    let delete_result = adapter.delete_context(context_id).await;
    assert!(delete_result.is_ok());
    
    let list_after_delete = adapter.list_contexts().await.unwrap();
    assert_eq!(list_after_delete.len(), 0);
    
    // Test getting non-existent context
    let not_found = adapter.get_context("non-existent").await;
    assert!(not_found.is_err());
}

#[test]
async fn test_configuration_update() {
    // Create adapter with default config
    let adapter = create_context_adapter();
    
    // Verify default config
    let default_config = adapter.get_config().await.unwrap();
    assert!(default_config.max_contexts > 0);
    
    // Create a new config
    let config = ContextAdapterConfig {
        max_contexts: 200,
        ttl_seconds: 7200, 
        enable_auto_cleanup: false,
    };
    
    // Update the config
    adapter.update_config(config.clone()).await.unwrap();
    
    // Retrieve the updated config
    let retrieved_config = adapter.get_config().await.unwrap();
    
    // Verify config values match
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(retrieved_config.enable_auto_cleanup, config.enable_auto_cleanup);
}

#[test]
async fn test_cleanup_expired_contexts() {
    use tokio::time::{sleep, Duration};
    
    // Create adapter with short TTL for testing
    let config = ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 1, // 1 second TTL
        enable_auto_cleanup: true,
    };
    
    let adapter = ContextAdapter::new(config);
    
    // Create contexts
    adapter.create_context("context1".to_string(), json!({"test": 1})).await.unwrap();
    adapter.create_context("context2".to_string(), json!({"test": 2})).await.unwrap();
    
    // Verify contexts exist
    assert_eq!(adapter.list_contexts().await.unwrap().len(), 2);
    
    // Wait for TTL to expire
    sleep(Duration::from_millis(1500)).await;
    
    // Run cleanup
    adapter.cleanup_expired_contexts().await.unwrap();
    
    // Contexts should be removed
    assert_eq!(adapter.list_contexts().await.unwrap().len(), 0);
}

#[test]
async fn test_context_adapter_config() {
    // Create a config
    let config = ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
    };
    
    // Create adapter with config
    let adapter = create_context_adapter_with_config(config.clone());
    
    // Get the config back
    let retrieved_config = adapter.get_config().await.unwrap();
    
    // Verify config values match
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(retrieved_config.enable_auto_cleanup, config.enable_auto_cleanup);
} 