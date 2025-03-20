//! Tests for the AppAdapter implementation.

use crate::app::{AppAdapter, AppConfig, AppInterface};
use crate::error::{AppInitializationError, AppOperationError};

#[test]
fn test_app_adapter_initialization() {
    let config = AppConfig::default();
    let adapter = AppAdapter::new(config);
    
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
    assert_eq!(config.name, "Squirrel");
    assert_eq!(config.version, "0.1.0");
}

#[test]
fn test_app_adapter_double_initialization() {
    let config = AppConfig::default();
    let adapter = AppAdapter::new(config);
    
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
fn test_app_adapter_uninitialized_operations() {
    let config = AppConfig::default();
    let adapter = AppAdapter::new(config);
    
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
}

#[test]
fn test_new_initialized_factory() {
    let config = AppConfig {
        name: "TestApp".to_string(),
        version: "2.0.0".to_string(),
        options: vec![("test".to_string(), "value".to_string())],
    };
    
    // Create an initialized adapter
    let adapter_result = AppAdapter::new_initialized(config);
    assert!(adapter_result.is_ok());
    
    let adapter = adapter_result.unwrap();
    
    // The adapter should be initialized
    assert!(adapter.is_initialized());
    
    // Get the config and verify it
    let config_result = adapter.get_config();
    assert!(config_result.is_ok());
    
    let config = config_result.unwrap();
    assert_eq!(config.name, "TestApp");
    assert_eq!(config.version, "2.0.0");
    assert_eq!(config.options.len(), 1);
    assert_eq!(config.options[0].0, "test");
    assert_eq!(config.options[0].1, "value");
} 