// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use serde_json::json;
use async_trait::async_trait;

use crate::manager::ContextManagerConfig;
use squirrel_interfaces::context::{
    ContextPlugin, 
    ContextTransformation,
    ContextAdapterPlugin,
    ContextManager
};
use squirrel_interfaces::plugins::{Plugin, PluginMetadata};

use crate::{
    create_default_manager, 
    create_manager_with_config, 
};

// Test implementation of a plugin
#[derive(Debug)]
struct TestPlugin {
    metadata: PluginMetadata
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "test.plugin",
                "1.0.0",
                "A test plugin",
                "ecoPrimals Contributors"
            ).with_capability("context")
        }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

#[async_trait]
impl ContextPlugin for TestPlugin {
    async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
        vec![Arc::new(TestTransformation)]
    }
    
    async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>> {
        vec![]
    }
}

// Test implementation of a transformation
#[derive(Debug)]
struct TestTransformation;

#[async_trait]
impl ContextTransformation for TestTransformation {
    fn get_id(&self) -> &str {
        "test.transform"
    }
    
    fn get_name(&self) -> &str {
        "Test Transformation"
    }
    
    fn get_description(&self) -> &str {
        "A test transformation"
    }
    
    async fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Simply wrap the data
        let result = json!({
            "original": data,
            "test": "Test transformation applied"
        });
        
        Ok(result)
    }
}

#[tokio::test]
async fn test_manager_creation() {
    let manager = create_default_manager();
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_manager_with_config() {
    let config = ContextManagerConfig {
        enable_plugins: true,
        plugin_paths: Some(vec!["./plugins".to_string()]),
    };
    
    let manager = create_manager_with_config(config);
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_plugin_registration() {
    let manager = create_default_manager();
    
    // Initialize the manager
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok());
    
    // Register the plugin
    let plugin = Box::new(TestPlugin::new());
    let reg_result = manager.register_plugin(plugin).await;
    assert!(reg_result.is_ok());
}

#[tokio::test]
async fn test_transformation() {
    let manager = create_default_manager();
    
    // Initialize the manager
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok());
    
    // Register the plugin
    let plugin = Box::new(TestPlugin::new());
    let reg_result = manager.register_plugin(plugin).await;
    assert!(reg_result.is_ok());
    
    // Create test data
    let test_data = json!({
        "query": "test query",
        "data": "test data"
    });
    
    // Transform the data
    let transform_result = manager.transform_data("test.transform", test_data.clone()).await;
    assert!(transform_result.is_ok());
    
    let transformed = transform_result.expect("should succeed");
    
    // Verify the transformation was applied
    let original = transformed.get("original");
    assert!(original.is_some());
    assert_eq!(original.expect("should succeed"), &test_data);
    
    let test_field = transformed.get("test");
    assert!(test_field.is_some());
    assert_eq!(test_field.expect("should succeed").as_str().expect("should succeed"), "Test transformation applied");
} 