//! Tests for the MCPAdapter implementation.

use crate::{MCPAdapter, MCPConfig, adapter::MCPInterface};
use crate::error::{AppInitializationError, AppOperationError};

#[test]
fn test_mcp_adapter_initialization() {
    let config = MCPConfig::default();
    let adapter = MCPAdapter::new(config);
    
    // Initially, the adapter should not be initialized
    assert!(!adapter.is_initialized());
    
    // Initialize the adapter
    let init_result = adapter.initialize();
    assert!(init_result.is_ok());
    
    // After initialization, is_initialized should return true
    assert!(adapter.is_initialized());
    
    // Get the config
    let config_result = adapter.get_config();
    assert!(config_result.is_ok());
    
    // The config should match what we set
    let config = config_result.unwrap();
    assert_eq!(config.version, "1.0");
    assert_eq!(config.timeout_ms, 5000);
}

#[test]
fn test_mcp_adapter_double_initialization() {
    let config = MCPConfig::default();
    let adapter = MCPAdapter::new(config);
    
    // First initialization should succeed
    assert!(adapter.initialize().is_ok());
    
    // Second initialization should fail with AlreadyInitialized
    let init_result = adapter.initialize();
    assert!(init_result.is_err());
    
    // Verify that the error is the expected AlreadyInitialized
    match init_result {
        Err(err) => {
            match err {
                crate::error::SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized) => {
                    // This is the expected error
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_mcp_adapter_uninitialized_operations() {
    let config = MCPConfig::default();
    let adapter = MCPAdapter::new(config);
    
    // Do not initialize the adapter
    assert!(!adapter.is_initialized());
    
    // Try to get the config without initializing
    let config_result = adapter.get_config();
    assert!(config_result.is_err());
    
    // Verify that the error is the expected NotInitialized
    match config_result {
        Err(err) => {
            match err {
                crate::error::SquirrelError::AppOperation(AppOperationError::NotInitialized) => {
                    // This is the expected error
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
    
    // Try to send a message without initializing
    let message_result = adapter.send_message("Hello");
    assert!(message_result.is_err());
    
    // Verify that the error is the expected NotInitialized
    match message_result {
        Err(err) => {
            match err {
                crate::error::SquirrelError::AppOperation(AppOperationError::NotInitialized) => {
                    // This is the expected error
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_mcp_adapter_send_message() {
    let config = MCPConfig::default();
    let adapter = MCPAdapter::new(config);
    
    // Initialize the adapter
    adapter.initialize().unwrap();
    
    // Send a message
    let message = "Hello, World!";
    let result = adapter.send_message(message);
    
    // Check that the message was processed correctly
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), format!("Processed: {}", message));
}

#[test]
fn test_new_initialized_factory() {
    let config = MCPConfig {
        version: "2.0".to_string(),
        max_message_size: 2048,
        timeout_ms: 10000,
        encryption_enabled: false,
    };
    
    // Create an initialized adapter
    let adapter_result = MCPAdapter::new_initialized(config);
    assert!(adapter_result.is_ok());
    
    let adapter = adapter_result.unwrap();
    
    // The adapter should be initialized
    assert!(adapter.is_initialized());
    
    // Get the config and verify it
    let config_result = adapter.get_config();
    assert!(config_result.is_ok());
    
    let config = config_result.unwrap();
    assert_eq!(config.version, "2.0");
    assert_eq!(config.max_message_size, 2048);
    assert_eq!(config.timeout_ms, 10000);
    assert!(!config.encryption_enabled);
} 