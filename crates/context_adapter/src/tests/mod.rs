use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::context_adapter::{ContextAdapter, ContextAdapterConfig, ContextAdapterFactory, create_context_adapter, create_context_adapter_with_config};
use crate::mcp::protocol::adapter::MCPContextAdapter;
use crate::test_utils::TestData;

#[test]
async fn test_adapter_initialization() {
    let mut adapter = MCPContextAdapter::new();
    assert!(!adapter.is_initialized());
    
    // Test initialization
    let result = adapter.initialize();
    assert!(result.is_ok());
    assert!(adapter.is_initialized());
}

#[test]
async fn test_adapter_with_config() {
    let mut adapter = MCPContextAdapter::new();
    
    // Create test config
    let config = ContextAdapterConfig {
        persistence_path: Some(String::from("/tmp/test")),
        auto_save_interval: Some(60),
    };
    
    // Initialize with config
    adapter.initialize_with_config(config.clone()).unwrap();
    assert!(adapter.is_initialized());
    
    // Check config is applied
    let retrieved_config = adapter.get_config().await.unwrap();
    assert_eq!(retrieved_config.persistence_path, config.persistence_path);
    assert_eq!(retrieved_config.auto_save_interval, config.auto_save_interval);
}

#[test]
async fn test_state_operations() {
    let mut adapter = MCPContextAdapter::new();
    adapter.initialize().unwrap();
    
    // Create test state
    let test_state = TestData::create_test_state();
    
    // Set state
    adapter.set_state(test_state.clone()).await.unwrap();
    
    // Get state
    let state = adapter.get_state().await.unwrap();
    assert_eq!(state, test_state);
}

#[test]
async fn test_multiple_initialization() {
    let mut adapter = MCPContextAdapter::new();
    
    // First initialization
    adapter.initialize().unwrap();
    assert!(adapter.is_initialized());
    
    // Second initialization should still work
    let result = adapter.initialize();
    assert!(result.is_ok());
}

#[test]
async fn test_integration_with_context() {
    // Create adapter and context manager
    let adapter = Arc::new(RwLock::new(MCPContextAdapter::new()));
    
    // Initialize adapter
    {
        let mut write_adapter = adapter.write().await;
        write_adapter.initialize().unwrap();
    }
    
    // Set state
    let test_state = json!({
        "test": true,
        "value": "DI pattern test"
    });
    
    {
        let read_adapter = adapter.read().await;
        read_adapter.set_state(test_state.clone()).await.unwrap();
    }
    
    // Get state
    let state = {
        let read_adapter = adapter.read().await;
        read_adapter.get_state().await.unwrap()
    };
    
    assert_eq!(state, test_state);
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
    assert_eq!(adapter_config.enable_auto_cleanup, false);
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
    assert_eq!(adapter_config.enable_auto_cleanup, true);
    
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
async fn test_config_update() {
    // Create an adapter
    let adapter = ContextAdapter::default();
    
    // Initial config
    let initial_config = adapter.get_config().await.unwrap();
    assert_eq!(initial_config.max_contexts, 1000);
    
    // Update config
    let new_config = ContextAdapterConfig {
        max_contexts: 2000,
        ttl_seconds: 7200,
        enable_auto_cleanup: false,
    };
    
    let update_result = adapter.update_config(new_config).await;
    assert!(update_result.is_ok());
    
    // Check updated config
    let updated_config = adapter.get_config().await.unwrap();
    assert_eq!(updated_config.max_contexts, 2000);
    assert_eq!(updated_config.ttl_seconds, 7200);
    assert_eq!(updated_config.enable_auto_cleanup, false);
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