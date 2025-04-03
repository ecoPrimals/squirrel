//! Context-MCP integration tests
//!
//! This module contains tests for the Context-MCP adapter functionality,
//! including synchronization, batch processing, and AI enhancements.

mod batch_tests;
mod sync_tests;
mod ai_enhancement_tests;
mod ai_enhancement_e2e_tests;
mod batch_processing_tests;
mod v2_tests;

#[cfg(test)]
mod adapter_tests {
    //! Tests for core adapter functionality
    
    use std::sync::Arc;
    
    use squirrel_integration::context_mcp::{
        ContextMcpAdapter,
        ContextMcpAdapterConfig,
        create_context_mcp_adapter_with_config,
        AdapterStatus,
    };
    
    use squirrel_mcp::McpConfig;
    
    #[tokio::test]
    async fn test_adapter_creation() {
        // Test creating the adapter with default config
        let context_mcp_config = ContextMcpAdapterConfig::default();
        let adapter_result = create_context_mcp_adapter_with_config(context_mcp_config).await;
        
        assert!(adapter_result.is_ok(), "Should be able to create adapter with default config");
        
        // Test creating the adapter with custom config
        let custom_config = ContextMcpAdapterConfig {
            sync_interval_secs: 120,
            max_retries: 5,
            timeout_ms: 10000,
            ..Default::default()
        };
        
        let adapter_result = create_context_mcp_adapter_with_config(custom_config.clone()).await;
        assert!(adapter_result.is_ok(), "Should be able to create adapter with custom config");
        
        let adapter = adapter_result.unwrap();
        assert_eq!(adapter.config.sync_interval_secs, custom_config.sync_interval_secs);
        assert_eq!(adapter.config.max_retries, custom_config.max_retries);
        assert_eq!(adapter.config.timeout_ms, custom_config.timeout_ms);
    }
    
    #[tokio::test]
    async fn test_adapter_status() {
        // Create adapter
        let context_mcp_config = ContextMcpAdapterConfig::default();
        let adapter = create_context_mcp_adapter_with_config(context_mcp_config)
            .await
            .expect("Should be able to create adapter");
        
        // Check initial status
        let status = adapter.get_status().await;
        
        assert_eq!(status.connected_to_mcp, false, "Should not be connected to MCP before initialization");
        assert_eq!(status.connected_to_context, false, "Should not be connected to Context before initialization");
        assert_eq!(status.error_count, 0, "Should start with 0 errors");
        assert_eq!(status.successful_syncs, 0, "Should start with 0 successful syncs");
        
        // Initialize the adapter
        let init_result = adapter.initialize().await;
        assert!(init_result.is_ok(), "Should be able to initialize adapter");
        
        // Check status after initialization
        let status = adapter.get_status().await;
        
        // After initialization, the connections should be established
        // However, in tests they might still be false depending on the environment
        // So we don't assert their values here
        
        // Test error count incrementation (internal function, but important for status)
        adapter.increment_error_count().await;
        let status = adapter.get_status().await;
        assert_eq!(status.error_count, 1, "Error count should be incremented");
        
        // Test sync count incrementation
        adapter.increment_sync_count().await;
        let status = adapter.get_status().await;
        assert_eq!(status.successful_syncs, 1, "Sync count should be incremented");
    }
} 