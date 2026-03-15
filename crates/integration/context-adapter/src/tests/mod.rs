// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;

use serde_json::json;
use tokio::test;

use crate::{
    create_context_adapter, create_context_adapter_with_config, ContextAdapter,
    ContextAdapterConfig, ContextAdapterFactory,
};

/// Test data structure used for context adapter testing
pub struct TestData {
    /// The test message content
    pub message: String,
    /// The test numeric value
    pub value: i32,
}

impl TestData {
    /// Creates a new test data value as JSON
    #[must_use]
    pub fn new(message: &str, value: i32) -> serde_json::Value {
        serde_json::json!({
            "message": message,
            "value": value
        })
    }
}

#[test]
async fn test_adapter_initialization() {
    let adapter = ContextAdapter::default();

    // Verify adapter state
    let initial_config = adapter.get_config().await.unwrap();
    assert_eq!(initial_config.max_contexts, 1000);

    // Test context operations
    let context_id = "test-init";
    let data = json!({ "test": true });

    // Create a context
    let create_result = adapter
        .create_context(context_id.to_string(), data.clone())
        .await;
    assert!(create_result.is_ok());

    // Verify context was created
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 1);
}

#[test]
async fn test_adapter_with_config() {
    // Create test config
    let config = ContextAdapterConfig {
        max_contexts: 200,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
        enable_plugins: true,
    };

    // Initialize with config
    let adapter = ContextAdapter::new(config.clone());

    // Check config is applied
    let retrieved_config = adapter.get_config().await.unwrap();
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(
        retrieved_config.enable_auto_cleanup,
        config.enable_auto_cleanup
    );
    assert_eq!(retrieved_config.enable_plugins, config.enable_plugins);
}

#[test]
async fn test_state_operations() {
    let adapter = ContextAdapter::default();

    // Create test context
    let context_id = "test-state";
    let test_state = json!({
        "message": "Test message",
        "value": 42
    });

    // Set state by creating a context
    adapter
        .create_context(context_id.to_string(), test_state.clone())
        .await
        .unwrap();

    // Get state by retrieving the context
    let context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(context.data, test_state);

    // Update state
    let updated_state = json!({
        "message": "Updated message",
        "value": 100
    });

    adapter
        .update_context(context_id, updated_state.clone())
        .await
        .unwrap();

    // Verify state was updated
    let updated_context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(updated_context.data, updated_state);
}

#[test]
async fn test_multiple_contexts() {
    let adapter = ContextAdapter::default();

    // Create multiple contexts
    adapter
        .create_context("context1".to_string(), json!({"id": 1}))
        .await
        .unwrap();
    adapter
        .create_context("context2".to_string(), json!({"id": 2}))
        .await
        .unwrap();
    adapter
        .create_context("context3".to_string(), json!({"id": 3}))
        .await
        .unwrap();

    // Verify all contexts were created
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 3);

    // Verify context retrieval
    let context1 = adapter.get_context("context1").await.unwrap();
    let context2 = adapter.get_context("context2").await.unwrap();
    let context3 = adapter.get_context("context3").await.unwrap();

    assert_eq!(context1.data, json!({"id": 1}));
    assert_eq!(context2.data, json!({"id": 2}));
    assert_eq!(context3.data, json!({"id": 3}));

    // Delete a context
    adapter.delete_context("context2").await.unwrap();

    // Verify context was deleted
    let remaining_contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(remaining_contexts.len(), 2);
    assert!(adapter.get_context("context2").await.is_err());
}

#[test]
async fn test_thread_safety() {
    // Create shared adapter
    let adapter = Arc::new(ContextAdapter::default());

    // Test concurrent operations
    let adapter_clone1 = adapter.clone();
    let adapter_clone2 = adapter.clone();

    // Create contexts from different threads
    let handle1 = tokio::spawn(async move {
        adapter_clone1
            .create_context("thread1".to_string(), json!({"source": "thread1"}))
            .await
    });

    let handle2 = tokio::spawn(async move {
        adapter_clone2
            .create_context("thread2".to_string(), json!({"source": "thread2"}))
            .await
    });

    // Wait for both operations to complete
    let _ = handle1.await.unwrap();
    let _ = handle2.await.unwrap();

    // Verify both contexts were created successfully
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 2);

    // Verify we can retrieve both contexts
    let context1 = adapter.get_context("thread1").await.unwrap();
    let context2 = adapter.get_context("thread2").await.unwrap();

    assert_eq!(context1.data, json!({"source": "thread1"}));
    assert_eq!(context2.data, json!({"source": "thread2"}));
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
        enable_plugins: true,
    };
    let adapter = ContextAdapter::new(custom_config.clone());
    let adapter_config = adapter.get_config().await.unwrap();
    assert_eq!(adapter_config.max_contexts, 500);
    assert_eq!(adapter_config.ttl_seconds, 1800);
    assert!(!adapter_config.enable_auto_cleanup);
    assert!(adapter_config.enable_plugins);
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
        enable_plugins: true,
    };
    let adapter = ContextAdapterFactory::create_adapter_with_config(custom_config.clone());
    let adapter_config = adapter.get_config().await.unwrap();
    assert_eq!(adapter_config.max_contexts, 200);
    assert_eq!(adapter_config.ttl_seconds, 900);
    assert!(adapter_config.enable_auto_cleanup);
    assert!(adapter_config.enable_plugins);

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

    let result = adapter
        .create_context(context_id.to_string(), data.clone())
        .await;
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

    let update_result = adapter
        .update_context(context_id, updated_data.clone())
        .await;
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
async fn test_configuration_update() {
    // Create adapter with default config
    let adapter = create_context_adapter();

    // Verify default config
    let default_config = adapter.get_config().await.unwrap();
    assert!(default_config.max_contexts > 0);

    // Create a new config
    let config = ContextAdapterConfig {
        max_contexts: 200,
        ttl_seconds: 7200,
        enable_auto_cleanup: false,
        enable_plugins: true,
    };

    // Update the config
    adapter.update_config(config.clone()).await.unwrap();

    // Retrieve the updated config
    let retrieved_config = adapter.get_config().await.unwrap();

    // Verify config values match
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(
        retrieved_config.enable_auto_cleanup,
        config.enable_auto_cleanup
    );
    assert_eq!(retrieved_config.enable_plugins, config.enable_plugins);
}

#[test]
async fn test_cleanup_expired_contexts() {
    use tokio::time::{sleep, Duration};

    // Create adapter with short TTL for testing
    let config = ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 2, // 2 second TTL
        enable_auto_cleanup: true,
        enable_plugins: true,
    };

    let adapter = ContextAdapter::new(config);

    // Create contexts
    adapter
        .create_context("context1".to_string(), json!({"test": 1}))
        .await
        .unwrap();
    adapter
        .create_context("context2".to_string(), json!({"test": 2}))
        .await
        .unwrap();

    // Verify contexts exist
    assert_eq!(adapter.list_contexts().await.unwrap().len(), 2);

    // Wait for TTL to expire
    sleep(Duration::from_secs(3)).await; // Wait 3 seconds to ensure expiry

    // Run cleanup
    adapter.cleanup_expired_contexts().await.unwrap();

    // Contexts should be removed
    assert_eq!(adapter.list_contexts().await.unwrap().len(), 0);
}

#[test]
async fn test_context_adapter_config() {
    // Create a config
    let config = ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
        enable_plugins: true,
    };

    // Create adapter with config
    let adapter = create_context_adapter_with_config(config.clone());

    // Get the config back
    let retrieved_config = adapter.get_config().await.unwrap();

    // Verify config values match
    assert_eq!(retrieved_config.max_contexts, config.max_contexts);
    assert_eq!(retrieved_config.ttl_seconds, config.ttl_seconds);
    assert_eq!(
        retrieved_config.enable_auto_cleanup,
        config.enable_auto_cleanup
    );
    assert_eq!(retrieved_config.enable_plugins, config.enable_plugins);
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Arc;

    use anyhow;
    use squirrel_interfaces::plugins::Plugin;
    use squirrel_interfaces::plugins::PluginMetadata;

    use crate::adapter::{
        create_context_adapter_with_plugins, ContextAdapter, ContextAdapterConfig,
    };
    use squirrel_context::plugins::ContextPluginManager;
    use squirrel_interfaces::context::{
        AdapterMetadata, ContextAdapterPlugin, ContextPlugin, ContextTransformation,
    };

    // Mock ContextPlugin for testing
    #[derive(Debug)]
    struct MockContextPlugin {
        transformations: Vec<Arc<MockTransformation>>,
        adapters: Vec<Arc<MockAdapterPlugin>>,
    }

    #[derive(Debug)]
    struct MockTransformation {
        id: String,
        name: String,
        description: String,
    }

    #[derive(Debug)]
    struct MockAdapterPlugin {
        metadata: AdapterMetadata,
        plugin_metadata: PluginMetadata,
    }

    impl MockContextPlugin {
        fn new() -> Self {
            Self {
                transformations: vec![Arc::new(MockTransformation {
                    id: "test.transform".to_string(),
                    name: "Test Transformation".to_string(),
                    description: "Test transformation for unit tests".to_string(),
                })],
                adapters: vec![Arc::new(MockAdapterPlugin {
                    metadata: AdapterMetadata {
                        id: "test.adapter".to_string(),
                        name: "Test Adapter".to_string(),
                        description: "Test adapter for unit tests".to_string(),
                        source_format: "source".to_string(),
                        target_format: "target".to_string(),
                    },
                    plugin_metadata: PluginMetadata {
                        id: "test.adapter".to_string(),
                        name: "Test Adapter".to_string(),
                        description: "Test adapter for unit tests".to_string(),
                        version: "1.0.0".to_string(),
                        author: "Squirrel Test Suite".to_string(),
                        capabilities: Vec::new(),
                    },
                })],
            }
        }
    }

    #[async_trait::async_trait]
    impl Plugin for MockContextPlugin {
        fn metadata(&self) -> &squirrel_interfaces::plugins::PluginMetadata {
            // Create a static metadata instance
            static METADATA: std::sync::LazyLock<squirrel_interfaces::plugins::PluginMetadata> =
                std::sync::LazyLock::new(|| {
                    squirrel_interfaces::plugins::PluginMetadata::new(
                        "mock-context-plugin",
                        "1.0.0",
                        "Mock context plugin for testing",
                        "ecoPrimals Contributors",
                    )
                    .with_capability("context")
                });

            &METADATA
        }

        async fn initialize(&self) -> anyhow::Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl ContextPlugin for MockContextPlugin {
        async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
            self.transformations
                .iter()
                .map(|t| t.clone() as Arc<dyn ContextTransformation>)
                .collect()
        }

        async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>> {
            self.adapters
                .iter()
                .map(|a| a.clone() as Arc<dyn ContextAdapterPlugin>)
                .collect()
        }
    }

    #[async_trait::async_trait]
    impl ContextTransformation for MockTransformation {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_description(&self) -> &str {
            &self.description
        }

        async fn transform(
            &self,
            data: serde_json::Value,
        ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            // Simply add a marker field
            let mut result = data.clone();
            if let serde_json::Value::Object(ref mut map) = result {
                map.insert("transformed".to_string(), json!(true));
                map.insert("transformation_id".to_string(), json!(self.id));
            }
            Ok(result)
        }
    }

    #[async_trait::async_trait]
    impl ContextAdapterPlugin for MockAdapterPlugin {
        async fn get_metadata(&self) -> AdapterMetadata {
            self.metadata.clone()
        }

        async fn convert(
            &self,
            data: serde_json::Value,
        ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            // Simply add a marker field
            let mut result = data.clone();
            if let serde_json::Value::Object(ref mut map) = result {
                map.insert("converted".to_string(), json!(true));
                map.insert("adapter_id".to_string(), json!(self.metadata.id));
            }
            Ok(result)
        }
    }

    // Add implementation of Plugin trait for MockAdapterPlugin
    #[async_trait::async_trait]
    impl Plugin for MockAdapterPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.plugin_metadata
        }

        async fn initialize(&self) -> anyhow::Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    // Helper function to create a test adapter with plugins
    async fn create_test_adapter() -> Arc<ContextAdapter> {
        let plugin_manager = Arc::new(ContextPluginManager::new());
        let mock_plugin = Box::new(MockContextPlugin::new());

        plugin_manager.register_plugin(mock_plugin).await.unwrap();

        let config = ContextAdapterConfig {
            max_contexts: 100,
            ttl_seconds: 3600,
            enable_auto_cleanup: true,
            enable_plugins: true,
        };

        let adapter = create_context_adapter_with_plugins(config, plugin_manager);
        adapter.initialize_plugins().await.unwrap();

        adapter
    }

    #[tokio::test]
    async fn test_create_context() {
        let adapter = create_test_adapter().await;

        // Create a simple context
        let context_id = "test-context";
        let context_data = json!({
            "key": "value",
            "number": 42
        });

        adapter
            .create_context(context_id.to_string(), context_data.clone())
            .await
            .unwrap();

        // Verify the context was created
        let retrieved = adapter.get_context(context_id).await.unwrap();
        assert_eq!(retrieved.id, context_id);
        assert_eq!(retrieved.data, context_data);
    }

    #[tokio::test]
    async fn test_transform_data() {
        let adapter = create_test_adapter().await;

        // Create a simple input
        let input_data = json!({
            "key": "value",
            "number": 42
        });

        // Transform the data
        let result = adapter
            .transform_data("test.transform", input_data.clone())
            .await
            .unwrap();

        // Verify the transformation
        assert_eq!(result["key"], "value");
        assert_eq!(result["number"], 42);
        assert_eq!(result["transformed"], true);
        assert_eq!(result["transformation_id"], "test.transform");
    }

    #[tokio::test]
    async fn test_convert_data() {
        let adapter = create_test_adapter().await;

        // Create a simple input
        let input_data = json!({
            "key": "value",
            "number": 42
        });

        // Convert the data
        let result = adapter
            .convert_data("test.adapter", input_data.clone())
            .await
            .unwrap();

        // Verify the conversion
        assert_eq!(result["key"], "value");
        assert_eq!(result["number"], 42);
        assert_eq!(result["converted"], true);
        assert_eq!(result["adapter_id"], "test.adapter");
    }

    #[tokio::test]
    async fn test_get_transformations() {
        let adapter = create_test_adapter().await;

        // Get all transformations
        let transformations = adapter.get_transformations().await.unwrap();

        // Verify the transformations
        assert_eq!(transformations.len(), 1);
        assert_eq!(transformations[0], "test.transform");
    }

    #[tokio::test]
    async fn test_get_adapters() {
        let adapter = create_test_adapter().await;

        // Get all adapters
        let adapters = adapter.get_adapters().await.unwrap();

        // Verify the adapters
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].id, "test.adapter");
    }

    #[tokio::test]
    async fn test_disabled_plugins() {
        // Create an adapter with plugins disabled
        let plugin_manager = Arc::new(ContextPluginManager::new());
        let config = ContextAdapterConfig {
            max_contexts: 100,
            ttl_seconds: 3600,
            enable_auto_cleanup: true,
            enable_plugins: false,
        };

        let adapter = create_context_adapter_with_plugins(config, plugin_manager);

        // Create a simple input
        let input_data = json!({
            "key": "value",
            "number": 42
        });

        // Try to transform the data, it should fail
        let result = adapter
            .transform_data("test.transform", input_data.clone())
            .await;
        assert!(result.is_err());

        // Try to convert the data, it should fail
        let result = adapter
            .convert_data("test.adapter", input_data.clone())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_update() {
        let adapter = create_test_adapter().await;

        // Update the configuration
        let new_config = ContextAdapterConfig {
            max_contexts: 200,
            ttl_seconds: 7200,
            enable_auto_cleanup: false,
            enable_plugins: false,
        };

        adapter.update_config(new_config.clone()).await.unwrap();

        // Verify the configuration was updated
        let retrieved_config = adapter.get_config().await.unwrap();
        assert_eq!(retrieved_config.max_contexts, 200);
        assert_eq!(retrieved_config.ttl_seconds, 7200);
        assert!(!retrieved_config.enable_auto_cleanup);
        assert!(!retrieved_config.enable_plugins);

        // Now that plugins are disabled, transformation should fail
        let input_data = json!({
            "key": "value",
            "number": 42
        });

        let result = adapter
            .transform_data("test.transform", input_data.clone())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_manager_access() {
        let adapter = create_test_adapter().await;

        // Check we can get the plugin manager
        let plugin_manager = adapter.get_plugin_manager();
        assert!(plugin_manager.is_some());
    }
}
