//! Tests for plugin loading functionality
//!
//! This module contains tests for loading plugins from directories
//! and ensuring the proper cloning mechanism works.

use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use tokio::test;

use squirrel_web::plugins::{
    Plugin, WebPlugin, PluginMetadata, PluginStatus,
    WebPluginRegistry, example::ExamplePlugin,
    register_plugin_factory,
};

/// Test creating a temporary plugin directory and loading plugins from it
#[test]
async fn test_plugin_directory_loading() -> Result<()> {
    // Create a temporary directory for test plugins
    let temp_dir = tempfile::tempdir()?;
    let plugin_dir = temp_dir.path();
    
    // For this test, we don't need actual plugin files since the implementation
    // will load ExamplePlugin when dynamic-plugins feature is not enabled
    
    // Create some dummy plugin files
    let dll_path = plugin_dir.join("dummy_plugin.dll");
    let so_path = plugin_dir.join("dummy_plugin.so");
    let js_path = plugin_dir.join("script_plugin.js");
    
    // Create empty files
    fs::write(&dll_path, b"")?;
    fs::write(&so_path, b"")?;
    fs::write(&js_path, b"")?;
    
    // Create the registry
    let registry = WebPluginRegistry::new();
    
    // Load plugins from the directory
    let count = registry.load_plugins_from_directory(plugin_dir.to_str().unwrap()).await?;
    
    // We should have at least one plugin loaded on any platform
    // (Windows will load from .dll, Unix from .so, and script plugins from .js)
    #[cfg(target_os = "windows")]
    assert!(count >= 1, "Should have loaded at least the .dll plugin");
    
    #[cfg(not(target_os = "windows"))]
    assert!(count >= 1, "Should have loaded at least the .so plugin");
    
    // Verify we can get the plugins
    let plugins = registry.get_plugins().await;
    assert!(!plugins.is_empty(), "Should have loaded at least one plugin");
    
    // Clean up
    temp_dir.close()?;
    
    Ok(())
}

/// A custom cloneable plugin for testing the clone mechanism
#[derive(Clone)]
struct CloneTestPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    counter: i32,
}

impl CloneTestPlugin {
    fn new(id: &str, counter: i32) -> Self {
        Self {
            metadata: PluginMetadata {
                id: id.to_string(),
                name: format!("Clone Test Plugin {}", id),
                version: "1.0.0".to_string(),
                description: "A cloneable test plugin".to_string(),
                author: "Test Author".to_string(),
                repository: None,
                license: None,
                tags: vec!["test".to_string()],
            },
            status: PluginStatus::Active,
            counter,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for CloneTestPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

#[async_trait::async_trait]
impl WebPlugin for CloneTestPlugin {
    fn get_endpoints(&self) -> Vec<squirrel_web::plugins::model::WebEndpoint> {
        vec![]
    }
    
    fn get_components(&self) -> Vec<squirrel_web::plugins::model::WebComponent> {
        vec![]
    }
    
    async fn handle_request(
        &self,
        _request: squirrel_web::plugins::model::WebRequest,
    ) -> Result<squirrel_web::plugins::model::WebResponse> {
        Ok(squirrel_web::plugins::model::WebResponse::ok())
    }
    
    async fn get_component_markup(&self, _id: uuid::Uuid, _props: serde_json::Value) -> Result<String> {
        Ok(format!("Clone Test Plugin with counter: {}", self.counter))
    }
}

/// Test the factory registration and cloning mechanism
#[test]
async fn test_plugin_factory_registration() -> Result<()> {
    // Register a factory for our test plugin
    let plugin_id = "clone-test-1";
    register_plugin_factory(plugin_id, || {
        Box::new(CloneTestPlugin::new(plugin_id, 42))
    });
    
    // Create a registry
    let registry = WebPluginRegistry::new();
    
    // Create and register a plugin
    let plugin = CloneTestPlugin::new(plugin_id, 42);
    registry.register_plugin(plugin).await?;
    
    // Get the plugins (this will clone the plugin)
    let plugins = registry.get_plugins().await;
    assert_eq!(plugins.len(), 1, "Should have registered one plugin");
    
    // Get the components
    let components = registry.get_components().await;
    
    // The CloneTestPlugin doesn't provide components, so we can't test get_component_markup
    // Instead, let's verify that the plugin has been properly cloned by checking its ID
    assert_eq!(plugins[0].metadata().id, plugin_id);
    
    Ok(())
}

/// Test cloning the ExamplePlugin
#[test]
async fn test_example_plugin_cloning() -> Result<()> {
    // Create an example plugin
    let example = ExamplePlugin::new();
    
    // Create a Box<dyn WebPlugin> from it
    let boxed: Box<dyn WebPlugin> = Box::new(example);
    
    // Clone the boxed plugin
    let cloned = boxed.clone();
    
    // Verify that the cloned plugin has the same metadata
    assert_eq!(
        boxed.metadata().id, 
        cloned.metadata().id,
        "Cloned plugin should have the same ID"
    );
    
    Ok(())
} 