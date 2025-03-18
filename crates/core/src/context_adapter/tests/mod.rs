use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::context_adapter::{ContextAdapter, ContextAdapterConfig};
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