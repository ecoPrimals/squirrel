//! Tests for the context manager module

use super::{ContextManager, ContextManagerConfig};
use serde_json::json;
use squirrel_interfaces::context::ContextManager as InterfaceContextManager;

#[test]
fn test_context_manager_config_default() {
    let config = ContextManagerConfig::default();

    assert!(config.enable_plugins);
    assert!(config.plugin_paths.is_none());
}

#[test]
fn test_context_manager_config_custom() {
    let config = ContextManagerConfig {
        enable_plugins: false,
        plugin_paths: Some(vec!["/path/to/plugins".to_string()]),
    };

    assert!(!config.enable_plugins);
    assert!(config.plugin_paths.is_some());
    assert_eq!(
        config
            .plugin_paths
            .as_ref()
            .expect("test: should succeed")
            .len(),
        1
    );
}

#[test]
fn test_context_manager_config_clone() {
    let config1 = ContextManagerConfig {
        enable_plugins: true,
        plugin_paths: Some(vec!["/test".to_string()]),
    };

    let config2 = config1.clone();
    assert_eq!(config1.enable_plugins, config2.enable_plugins);
}

#[test]
fn test_context_manager_config_debug() {
    let config = ContextManagerConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ContextManagerConfig"));
}

#[test]
fn test_context_manager_new() {
    let manager = ContextManager::new();
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("ContextManager"));
}

#[test]
fn test_context_manager_with_config() {
    let config = ContextManagerConfig {
        enable_plugins: false,
        plugin_paths: None,
    };

    let manager = ContextManager::with_config(config);
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("ContextManager"));
}

#[test]
fn test_context_manager_default() {
    let manager = ContextManager::default();
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("ContextManager"));
}

#[tokio::test]
async fn test_context_manager_initialize() {
    let manager = ContextManager::new();
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_context_manager_initialize_idempotent() {
    let manager = ContextManager::new();

    let result1 = manager.initialize().await;
    assert!(result1.is_ok());

    let result2 = manager.initialize().await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_context_manager_get_plugin_manager_before_init() {
    let manager = ContextManager::new();
    let plugin_manager = manager.get_plugin_manager().await;
    assert!(plugin_manager.is_none());
}

#[tokio::test]
async fn test_context_manager_get_plugin_manager_after_init() {
    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let plugin_manager = manager.get_plugin_manager().await;
    assert!(plugin_manager.is_some());
}

#[tokio::test]
async fn test_context_manager_plugins_disabled() {
    let config = ContextManagerConfig {
        enable_plugins: false,
        plugin_paths: None,
    };

    let manager = ContextManager::with_config(config);
    manager.initialize().await.expect("Failed to initialize");

    let plugin_manager = manager.get_plugin_manager().await;
    assert!(plugin_manager.is_none());
}

#[tokio::test]
async fn test_context_manager_transform_data_not_initialized() {
    let manager = ContextManager::new();
    let result = manager.transform_data("test", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_context_manager_get_transformations_not_initialized() {
    let manager = ContextManager::new();
    let result = manager.get_transformations().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_context_manager_transform_data_plugins_disabled() {
    let config = ContextManagerConfig {
        enable_plugins: false,
        plugin_paths: None,
    };

    let manager = ContextManager::with_config(config);
    manager.initialize().await.expect("Failed to initialize");

    let result = manager.transform_data("test", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_context_manager_get_transformations_plugins_disabled() {
    let config = ContextManagerConfig {
        enable_plugins: false,
        plugin_paths: None,
    };

    let manager = ContextManager::with_config(config);
    manager.initialize().await.expect("Failed to initialize");

    let result = manager.get_transformations().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_context_manager_create_recovery_point() {
    use std::collections::HashMap;
    use std::time::SystemTime;

    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let state = crate::ContextState {
        id: "test-state".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let result = manager.create_recovery_point(&state).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_context_manager_get_context_state() {
    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let result = manager.get_context_state("test-id").await;
    assert!(result.is_ok());

    let state = result.expect("test: should succeed");
    assert_eq!(state.id, "test-id");
    assert_eq!(state.version, 1);
}

#[tokio::test]
async fn test_context_manager_update_context_state() {
    use std::collections::HashMap;
    use std::time::SystemTime;

    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let state = crate::ContextState {
        id: "test-state".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({"key": "value"}),
        metadata: HashMap::new(),
        synchronized: true,
        last_modified: SystemTime::now(),
    };

    let result = manager.update_context_state("test-state", state).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_context_manager_concurrent_initialization() {
    let manager = std::sync::Arc::new(ContextManager::new());

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move { manager_clone.initialize().await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_context_manager_concurrent_get_plugin_manager() {
    let manager = std::sync::Arc::new(ContextManager::new());
    manager.initialize().await.expect("Failed to initialize");

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move { manager_clone.get_plugin_manager().await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert!(result.is_some());
    }
}

#[tokio::test]
async fn test_context_manager_with_plugin_paths() {
    let config = ContextManagerConfig {
        enable_plugins: true,
        plugin_paths: Some(vec![
            "/path/to/plugins1".to_string(),
            "/path/to/plugins2".to_string(),
        ]),
    };

    let manager = ContextManager::with_config(config);
    // Initialize should work even if paths don't exist (stub implementation)
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_context_manager_get_plugin_manager_multiple_times() {
    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let pm1 = manager.get_plugin_manager().await;
    let pm2 = manager.get_plugin_manager().await;

    assert!(pm1.is_some());
    assert!(pm2.is_some());
}

#[tokio::test]
async fn test_context_manager_get_context_state_different_ids() {
    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    let state1 = manager.get_context_state("id-1").await.expect("Failed");
    let state2 = manager.get_context_state("id-2").await.expect("Failed");

    assert_eq!(state1.id, "id-1");
    assert_eq!(state2.id, "id-2");
}

#[tokio::test]
async fn test_context_manager_multiple_recovery_points() {
    use std::collections::HashMap;
    use std::time::SystemTime;

    let manager = ContextManager::new();
    manager.initialize().await.expect("Failed to initialize");

    for i in 1..=5 {
        let state = crate::ContextState {
            id: format!("state-{}", i),
            version: i,
            timestamp: i * 1000,
            data: json!({"iteration": i}),
            metadata: HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::now(),
        };

        let result = manager.create_recovery_point(&state).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_context_manager_config_with_empty_paths() {
    let config = ContextManagerConfig {
        enable_plugins: true,
        plugin_paths: Some(vec![]),
    };

    let manager = ContextManager::with_config(config);
    let result = manager.initialize().await;
    assert!(result.is_ok());
}
