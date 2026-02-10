// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the context plugin manager module

use super::ContextPluginManager;
use serde_json::json;

#[test]
fn test_context_plugin_manager_new() {
    let manager = ContextPluginManager::new();
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("ContextPluginManager"));
}

#[test]
fn test_context_plugin_manager_default() {
    let manager = ContextPluginManager::default();
    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("ContextPluginManager"));
}

#[tokio::test]
async fn test_get_transformations_empty() {
    let manager = ContextPluginManager::new();
    let transformations = manager.get_transformations().await;
    assert!(transformations.is_empty());
}

#[tokio::test]
async fn test_get_adapters_empty() {
    let manager = ContextPluginManager::new();
    let adapters = manager.get_adapters().await;
    assert!(adapters.is_empty());
}

#[tokio::test]
async fn test_transform_not_found() {
    let manager = ContextPluginManager::new();
    let result = manager.transform("nonexistent", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_adapter_not_found() {
    let manager = ContextPluginManager::new();
    let result = manager.get_adapter("nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_adapter_metadata_not_found() {
    let manager = ContextPluginManager::new();
    let result = manager.get_adapter_metadata("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_load_plugins_from_path() {
    let manager = ContextPluginManager::new();
    let result = manager.load_plugins_from_path("/nonexistent/path").await;
    // Should succeed (stub implementation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_plugins_from_multiple_paths() {
    let manager = ContextPluginManager::new();

    let paths = vec!["/path/1", "/path/2", "/path/3"];

    for path in paths {
        let result = manager.load_plugins_from_path(path).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_get_transformations() {
    let manager = std::sync::Arc::new(ContextPluginManager::new());

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move { manager_clone.get_transformations().await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert!(result.is_empty());
    }
}

#[tokio::test]
async fn test_concurrent_get_adapters() {
    let manager = std::sync::Arc::new(ContextPluginManager::new());

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move { manager_clone.get_adapters().await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert!(result.is_empty());
    }
}

#[tokio::test]
async fn test_transform_with_different_ids() {
    let manager = ContextPluginManager::new();

    let ids = vec!["transform1", "transform2", "transform3"];

    for id in ids {
        let result = manager.transform(id, json!({})).await;
        assert!(result.is_err()); // All should fail (not registered)
    }
}

#[tokio::test]
async fn test_get_adapter_with_different_ids() {
    let manager = ContextPluginManager::new();

    let ids = vec!["adapter1", "adapter2", "adapter3"];

    for id in ids {
        let result = manager.get_adapter(id).await;
        assert!(result.is_none());
    }
}

#[tokio::test]
async fn test_get_adapter_metadata_with_different_ids() {
    let manager = ContextPluginManager::new();

    let ids = vec!["adapter1", "adapter2", "adapter3"];

    for id in ids {
        let result = manager.get_adapter_metadata(id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_transform_with_complex_data() {
    let manager = ContextPluginManager::new();

    let data = json!({
        "user": {
            "name": "test",
            "age": 30,
            "nested": {
                "field": "value"
            }
        },
        "array": [1, 2, 3]
    });

    let result = manager.transform("test_transform", data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plugin_manager_thread_safety() {
    let manager = std::sync::Arc::new(ContextPluginManager::new());

    let mut handles = vec![];

    // Spawn tasks that do different operations concurrently
    for i in 0..20 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            if i % 3 == 0 {
                let _ = manager_clone.get_transformations().await;
            } else if i % 3 == 1 {
                let _ = manager_clone.get_adapters().await;
            } else {
                let _ = manager_clone.get_adapter("test").await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }
}

#[tokio::test]
async fn test_load_plugins_empty_path() {
    let manager = ContextPluginManager::new();
    let result = manager.load_plugins_from_path("").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_transformations_lookups() {
    let manager = ContextPluginManager::new();

    for _ in 0..100 {
        let transformations = manager.get_transformations().await;
        assert!(transformations.is_empty());
    }
}

#[tokio::test]
async fn test_multiple_adapters_lookups() {
    let manager = ContextPluginManager::new();

    for _ in 0..100 {
        let adapters = manager.get_adapters().await;
        assert!(adapters.is_empty());
    }
}

#[tokio::test]
async fn test_transform_with_null_data() {
    let manager = ContextPluginManager::new();
    let result = manager.transform("test", json!(null)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transform_with_empty_object() {
    let manager = ContextPluginManager::new();
    let result = manager.transform("test", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transform_with_array() {
    let manager = ContextPluginManager::new();
    let result = manager.transform("test", json!([1, 2, 3])).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_transform_attempts() {
    let manager = std::sync::Arc::new(ContextPluginManager::new());

    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            manager_clone
                .transform(&format!("transform_{}", i), json!({}))
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_get_transformations_consistency() {
    let manager = ContextPluginManager::new();

    let t1 = manager.get_transformations().await;
    let t2 = manager.get_transformations().await;
    let t3 = manager.get_transformations().await;

    assert_eq!(t1.len(), t2.len());
    assert_eq!(t2.len(), t3.len());
}

#[tokio::test]
async fn test_get_adapters_consistency() {
    let manager = ContextPluginManager::new();

    let a1 = manager.get_adapters().await;
    let a2 = manager.get_adapters().await;
    let a3 = manager.get_adapters().await;

    assert_eq!(a1.len(), a2.len());
    assert_eq!(a2.len(), a3.len());
}
