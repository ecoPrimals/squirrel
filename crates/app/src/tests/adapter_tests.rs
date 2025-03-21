//! Tests for the AppAdapter implementation.

#![allow(clippy::module_name_repetitions)]

use squirrel_core::error::{SquirrelError, AppInitializationError, AppOperationError};
use crate::{AppAdapter};
use crate::core::AppConfig;
use crate::adapter::{create_initialized_app_adapter, create_default_app_adapter};

#[tokio::test]
async fn test_initialize() {
    let config = AppConfig::default();
    let mut adapter = AppAdapter::new();
    let init_result = adapter.initialize(&config);
    assert!(init_result.is_ok());
}

#[tokio::test]
async fn test_initialize_already_initialized() {
    let config = AppConfig::default();
    let mut adapter = AppAdapter::new();
    assert!(adapter.initialize(&config).is_ok());
    
    let init_result = adapter.initialize(&config);
    assert!(matches!(init_result, Err(SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized))));
}

#[tokio::test]
async fn test_app_adapter_initialization() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT
    let init_result = adapter.initialize(&config);
    
    // ASSERT
    assert!(init_result.is_ok());
    assert!(adapter.is_initialized());
}

#[tokio::test]
async fn test_app_adapter_double_initialization() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // First initialization should succeed
    assert!(adapter.initialize(&config).is_ok());
    
    // ACT
    let init_result = adapter.initialize(&config);
    
    // ASSERT
    assert!(init_result.is_err());
    if let Err(SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized)) = init_result {
        // No need to assert anything here, the error type is correct
    } else {
        panic!("Expected SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized)");
    }
}

#[tokio::test]
async fn test_app_adapter_uninitialized_operations() {
    let adapter = AppAdapter::new();
    
    // Do not initialize the adapter
    assert!(!adapter.is_initialized());
    
    // Try to get the config without initializing
    let config_result = adapter.config();
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
    // ARRANGE & ACT
    let config = AppConfig {
        name: "TestApp".to_string(),
        version: "0.1.0".to_string(),
        environment: "test".to_string(),
        debug: true,
    };
    
    let adapter_result = create_initialized_app_adapter(&config);
    
    // ASSERT
    assert!(adapter_result.is_ok());
    let adapter = adapter_result.unwrap();
    assert!(adapter.is_initialized());
}

#[tokio::test]
async fn test_app_adapter_default_factory() {
    // ARRANGE & ACT
    let adapter_result = create_default_app_adapter();
    
    // ASSERT
    assert!(adapter_result.is_ok());
    let adapter = adapter_result.unwrap();
    assert!(adapter.is_initialized());
}

#[tokio::test]
async fn test_uninitialized_operations() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    
    // ACT & ASSERT
    assert!(!adapter.is_initialized());
    
    let start_result = adapter.start();
    assert!(start_result.is_err());
    
    let stop_result = adapter.stop();
    assert!(stop_result.is_err());
    
    let context_result = adapter.context();
    assert!(context_result.is_err());
    
    let event_emitter_result = adapter.event_emitter();
    assert!(event_emitter_result.is_err());
}

#[tokio::test]
async fn test_stop_not_started() {
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    assert!(adapter.initialize(&config).is_ok());
    
    let start_result = adapter.start();
    assert!(start_result.is_ok());
    
    let stop_result = adapter.stop();
    assert!(stop_result.is_ok());
    
    let context_result = adapter.context();
    assert!(context_result.is_err());
    
    let event_emitter_result = adapter.event_emitter();
    assert!(event_emitter_result.is_err());
}

#[tokio::test]
async fn test_app_adapter_config() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT
    assert!(adapter.initialize(&config).is_ok());
    assert!(adapter.is_initialized());
    
    let config_result = adapter.config();
    assert!(config_result.is_ok());
}

#[tokio::test]
async fn test_app_adapter_version() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT
    assert!(adapter.initialize(&config).is_ok());
    assert!(adapter.is_initialized());
    
    let version_result = adapter.version();
    assert!(version_result.is_ok());
    assert!(!version_result.unwrap().is_empty());
}

#[tokio::test]
async fn test_app_adapter_ensure_initialized() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT - Before initialization
    assert!(!adapter.is_initialized());
    assert!(adapter.ensure_initialized().is_err());
    
    // After initialization
    adapter.initialize(&config).unwrap();
    assert!(adapter.is_initialized());
    assert!(adapter.ensure_initialized().is_ok());
}

#[tokio::test]
async fn test_app_adapter_start_stop() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT
    adapter.initialize(&config).unwrap();
    assert!(adapter.is_initialized());
    
    // Test start
    assert!(adapter.start().is_ok());
    
    // Test stop
    assert!(adapter.stop().is_ok());
}

#[tokio::test]
async fn test_app_adapter_factory_error() {
    // ARRANGE & ACT
    let config = AppConfig {
        name: "TestApp".to_string(),
        version: "0.1.0".to_string(),
        environment: "test".to_string(),
        debug: true,
    };
    
    let adapter_result = create_initialized_app_adapter(&config);
    
    // ASSERT
    assert!(adapter_result.is_ok());
    let adapter = adapter_result.unwrap();
    assert!(adapter.is_initialized());
}

#[tokio::test]
async fn test_app_adapter_context_error() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT
    adapter.initialize(&config).unwrap();
    assert!(adapter.is_initialized());
    
    let context_result = adapter.context();
    assert!(context_result.is_err());
}

#[tokio::test]
async fn test_app_adapter_event_emitter_error() {
    // ARRANGE
    let mut adapter = AppAdapter::new();
    let config = AppConfig::default();
    
    // ACT & ASSERT
    adapter.initialize(&config).unwrap();
    assert!(adapter.is_initialized());
    
    let event_emitter_result = adapter.event_emitter();
    assert!(event_emitter_result.is_err());
}

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert!(!config.debug);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_access() {
        let config = AppConfig::default();
        let mut adapter = AppAdapter::new();
        adapter.initialize(&config).unwrap();
        assert!(adapter.is_initialized());
    }

    #[tokio::test]
    async fn test_adapter_factory() {
        let config = AppConfig {
            name: "TestApp".to_string(),
            version: "0.1.0".to_string(),
            environment: "test".to_string(),
            debug: true,
        };
        
        let adapter_result = crate::adapter::create_initialized_app_adapter(&config);
        assert!(adapter_result.is_ok());
    }

    #[tokio::test]
    async fn test_start_stop() {
        let config = AppConfig::default();
        let mut adapter = AppAdapter::new();
        adapter.initialize(&config).unwrap();
        assert!(adapter.is_initialized());
        
        assert!(adapter.start().is_ok());
        assert!(adapter.stop().is_ok());
    }

    #[test]
    fn test_lifecycle() {
        let config = AppConfig::default();
        let mut adapter = AppAdapter::new();
        adapter.initialize(&config).unwrap();
        assert!(adapter.is_initialized());
    }
} 