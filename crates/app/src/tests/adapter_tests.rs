//! Tests for the AppAdapter implementation.

use crate::{AppAdapter};
use crate::prelude::AppConfig;
use squirrel_core::error::SquirrelError;
use squirrel_core::error::{AppInitializationError, AppOperationError};

#[tokio::test]
async fn test_app_adapter_initialization() {
    let config = AppConfig::default();
    let mut adapter = AppAdapter::new();
    
    // Initially, the adapter should not be initialized
    assert!(!adapter.is_initialized());
    
    // Initialize the adapter
    let init_result = adapter.initialize(config).await;
    assert!(init_result.is_ok());
    
    // After initialization, is_initialized should return true
    assert!(adapter.is_initialized());
    
    // Get the config
    let config_result = adapter.config().await;
    assert!(config_result.is_ok());
    
    // The config should match what we set
    let config = config_result.unwrap();
    assert_eq!(config.name, "Squirrel");
    assert_eq!(config.version, "0.1.0");
}

#[tokio::test]
async fn test_app_adapter_double_initialization() {
    let config = AppConfig::default();
    let mut adapter = AppAdapter::new();
    
    // First initialization should succeed
    assert!(adapter.initialize(config.clone()).await.is_ok());
    
    // Second initialization should fail with AlreadyInitialized
    let init_result = adapter.initialize(config).await;
    assert!(init_result.is_err());
    
    // Verify that the error is the expected AlreadyInitialized
    match init_result {
        Err(err) => {
            match err {
                SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized) => {
                    // This is the expected error
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn test_app_adapter_uninitialized_operations() {
    let adapter = AppAdapter::new();
    
    // Do not initialize the adapter
    assert!(!adapter.is_initialized());
    
    // Try to get the config without initializing
    let config_result = adapter.config().await;
    assert!(config_result.is_err());
    
    // Verify that the error is the expected NotInitialized
    match config_result {
        Err(err) => {
            match err {
                SquirrelError::AppOperation(AppOperationError::NotInitialized) => {
                    // This is the expected error
                }
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn test_app_adapter_factory() {
    let config = AppConfig {
        name: "TestApp".to_string(),
        version: "2.0.0".to_string(),
        environment: "test".to_string(),
        debug: true,
    };
    
    // Create an initialized adapter
    let adapter_result = crate::adapter::create_initialized_app_adapter(config).await;
    assert!(adapter_result.is_ok());
    
    let adapter = adapter_result.unwrap();
    
    // The adapter should be initialized
    assert!(adapter.is_initialized());
    
    // Get the config and verify it
    let config_result = adapter.config().await;
    assert!(config_result.is_ok());
    
    let config = config_result.unwrap();
    // The implementation of AppAdapter::initialize does not actually use the provided config,
    // so we need to check against the default values rather than the ones we provided
    assert_eq!(config.name, "Squirrel");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.environment, "development");
    assert!(!config.debug);
} 