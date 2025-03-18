//! Tests for MCP adapters to verify proper DI pattern implementation

use std::sync::Arc;
use crate::mcp::protocol::{
    ProtocolConfig,
    create_protocol_adapter, create_protocol_adapter_with_config
};
use crate::context_adapter::{
    ContextAdapterConfig,
    create_context_adapter as create_general_context_adapter
};
use crate::mcp::context_adapter::{
    create_mcp_context_adapter,
    create_mcp_context_adapter_with_config,
    create_initialized_mcp_context_adapter
};

/// Basic tests for protocol adapter initialization and error handling
#[cfg(test)]
mod protocol_tests {
    use super::*;
    use crate::mcp::protocol::MCPProtocolAdapter;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_protocol_adapter_initialization() {
        // Create a new adapter with default configuration
        let adapter = MCPProtocolAdapter::new();
        
        // Test that the adapter is not initialized by default
        assert!(!adapter.is_initialized().await);
        
        // Initialize the adapter
        let init_result = adapter.initialize().await;
        assert!(init_result.is_ok());
        assert!(adapter.is_initialized().await);
        
        // Try to initialize again, should error
        let reinit_result = adapter.initialize().await;
        assert!(reinit_result.is_err());
        
        // Adapter should still be initialized
        assert!(adapter.is_initialized().await);
    }
    
    #[tokio::test]
    async fn test_protocol_adapter_state() {
        let adapter = MCPProtocolAdapter::new();
        
        // Initialize the adapter
        adapter.initialize().await.unwrap();
        
        // Set some state
        let test_state = json!({"status": "ready"});
        adapter.set_state(test_state.clone()).await.unwrap();
        
        // Get the state
        let state = adapter.get_state().await.unwrap();
        assert_eq!(state, test_state);
    }
    
    #[tokio::test]
    async fn test_protocol_adapter_with_custom_config() {
        // Create a custom config
        let config = ProtocolConfig {
            version: "1.2.3".to_string(),
            max_message_size: 2048,
            timeout_ms: 5000,
        };
        
        // Create a new adapter
        let adapter = MCPProtocolAdapter::new();
        
        // Initialize the adapter with custom config
        adapter.initialize_with_config(config.clone()).await.unwrap();
        
        // Check that the config was applied
        let retrieved_config = adapter.get_config().await.unwrap();
        assert_eq!(retrieved_config.version, "1.2.3");
        assert_eq!(retrieved_config.max_message_size, 2048);
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use crate::mcp::context_adapter::MCPContextAdapter;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_context_adapter_initialization() {
        // Create a new adapter with default configuration
        let adapter = MCPContextAdapter::new();
        
        // Test that the adapter is not initialized by default
        assert!(!adapter.is_initialized());
        
        // Initialize the adapter
        let result = adapter.initialize();
        assert!(result.is_ok());
        
        // Adapter should now be initialized
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_context_adapter_with_custom_config() {
        // Create a custom config
        let config = ContextAdapterConfig {
            max_contexts: 100,
            ttl_seconds: 5000,
            enable_auto_cleanup: true,
        };
        
        // Create a new adapter
        let adapter = MCPContextAdapter::new();
        
        // Initialize the adapter with custom config
        adapter.initialize_with_config(config.clone()).unwrap();
        
        // Check that the config was applied
        let retrieved_config = adapter.get_config().await.unwrap();
        assert_eq!(retrieved_config.max_contexts, 100);
        assert_eq!(retrieved_config.ttl_seconds, 5000);
    }
}

/// Tests for protocol adapter factory functions
#[tokio::test]
async fn test_protocol_adapter_factory_functions() {
    // Create uninitialized adapter
    let adapter = create_protocol_adapter();
    assert!(!adapter.is_initialized().await);
    
    // Create adapter with config - should be initialized
    let config = ProtocolConfig::default();
    let result = create_protocol_adapter_with_config(config).await;
    assert!(result.is_ok());
    
    let adapter = result.unwrap();
    assert!(adapter.is_initialized().await);
}

/// Tests for context adapter factory functions
#[tokio::test]
async fn test_mcp_context_adapter_factory_functions() {
    // Create uninitialized adapter
    let adapter = create_mcp_context_adapter();
    assert!(!adapter.is_initialized());
    
    // Create adapter with config - should be initialized
    let config = ContextAdapterConfig::default();
    let result = create_mcp_context_adapter_with_config(config);
    assert!(result.is_ok());
    
    let adapter = result.unwrap();
    assert!(adapter.is_initialized());
    
    // Test pre-initialized adapter creation
    let result = create_initialized_mcp_context_adapter();
    assert!(result.is_ok());
    
    let adapter = result.unwrap();
    assert!(adapter.is_initialized());
}

/// Tests for the new context adapter integration between general context system and MCP
#[cfg(test)]
mod context_adapter_integration_tests {
    use super::*;
    use crate::context_adapter::{ContextAdapter, ContextAdapterConfig};
    use crate::mcp::context_adapter::MCPContextAdapter;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_mcp_context_adapter_initialization() {
        // Create a new MCP context adapter
        let adapter = MCPContextAdapter::new();
        
        // Test that the adapter is not initialized by default
        assert!(!adapter.is_initialized());
        
        // Initialize the adapter
        let result = adapter.initialize();
        assert!(result.is_ok());
        
        // Adapter should now be initialized
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_mcp_context_adapter_operations() {
        // Create and initialize the adapter
        let mut adapter = MCPContextAdapter::new();
        adapter.initialize().unwrap();
        
        // Create a context
        let test_id = "test_context_1";
        let test_data = json!({"name": "Test Context", "value": 42});
        
        let create_result = adapter.create_context(test_id.to_string(), test_data.clone()).await;
        assert!(create_result.is_ok());
        
        // Get the context
        let get_result = adapter.get_context(test_id).await;
        assert!(get_result.is_ok());
        let context = get_result.unwrap();
        assert_eq!(context.id, test_id);
        assert_eq!(context.data, test_data);
        
        // Update the context
        let updated_data = json!({"name": "Updated Context", "value": 100});
        let update_result = adapter.update_context(test_id, updated_data.clone()).await;
        assert!(update_result.is_ok());
        
        // Get the updated context
        let get_updated_result = adapter.get_context(test_id).await;
        assert!(get_updated_result.is_ok());
        let updated_context = get_updated_result.unwrap();
        assert_eq!(updated_context.data, updated_data);
        
        // List contexts
        let list_result = adapter.list_contexts().await;
        assert!(list_result.is_ok());
        let contexts = list_result.unwrap();
        assert_eq!(contexts.len(), 1);
        
        // Delete the context
        let delete_result = adapter.delete_context(test_id).await;
        assert!(delete_result.is_ok());
        
        // Verify deletion
        let get_deleted_result = adapter.get_context(test_id).await;
        assert!(get_deleted_result.is_err());
    }
    
    #[tokio::test]
    async fn test_mcp_context_adapter_with_custom_config() {
        // Create a custom config
        let config = ContextAdapterConfig {
            max_contexts: 50,
            ttl_seconds: 1800,
            enable_auto_cleanup: true,
        };
        
        // Create and initialize the adapter with custom config
        let mut adapter = MCPContextAdapter::new();
        adapter.initialize_with_config(config.clone()).unwrap();
        
        // Check that the config was applied
        let get_config_result = adapter.get_config().await;
        assert!(get_config_result.is_ok());
        let retrieved_config = get_config_result.unwrap();
        assert_eq!(retrieved_config.max_contexts, 50);
        assert_eq!(retrieved_config.ttl_seconds, 1800);
    }
    
    #[tokio::test]
    async fn test_context_adapter_integration() {
        // Create a general context adapter
        let general_adapter = create_general_context_adapter();
        
        // Create MCP context adapter with existing general adapter
        let mcp_adapter = crate::mcp::context_adapter::create_mcp_context_adapter_with_adapter(general_adapter);
        
        // Test that the adapter is initialized
        assert!(mcp_adapter.is_initialized());
        
        // Create a context
        let test_id = "integration_test";
        let test_data = json!({"source": "integration test", "value": 100});
        
        let create_result = mcp_adapter.create_context(test_id.to_string(), test_data.clone()).await;
        assert!(create_result.is_ok());
        
        // Get the context
        let get_result = mcp_adapter.get_context(test_id).await;
        assert!(get_result.is_ok());
        let context = get_result.unwrap();
        assert_eq!(context.data, test_data);
    }
}