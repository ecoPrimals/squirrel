//! Plugin system tests
//!
//! This module provides tests for the plugin system.
#![cfg(test)]

use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

use crate::plugins::core::{Plugin, PluginStatus};
use crate::plugins::model::{WebPlugin, WebRequest, WebResponse, HttpMethod};
use crate::plugins::registry::WebPluginRegistry;
use crate::plugins::example::ExamplePlugin;

#[tokio::test]
async fn test_plugin_registration() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Check that the plugin was registered
    let plugins = registry.get_plugins().await;
    assert_eq!(plugins.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_initialization() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    let plugin_id = plugin.metadata().id;
    registry.register_plugin(plugin).await?;
    
    // Check plugin status
    let status = registry.get_plugin_status(&plugin_id).await?;
    assert_eq!(status, PluginStatus::Ready);
    
    Ok(())
}

#[tokio::test]
async fn test_endpoint_registration() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Check registered endpoints
    let endpoints = registry.get_endpoints().await;
    assert_eq!(endpoints.len(), 3);
    
    Ok(())
}

#[tokio::test]
async fn test_endpoint_handling() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Create request
    let request = WebRequest::new(
        "/api/example/greeting",
        HttpMethod::Post,
    ).with_body(json!({
        "name": "Test User"
    }));
    
    // Handle request
    let response = registry.handle_request(request).await?;
    
    // Check response
    assert_eq!(response.status as u16, 200);
    assert!(response.body.is_some());
    if let Some(body) = response.body {
        assert!(body.get("message").is_some());
        assert_eq!(body["message"], "Hello, Test User!");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_component_registration() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Check registered components
    let components = registry.get_components().await;
    assert_eq!(components.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_route_matching() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Test route with parameter
    let request = WebRequest::new(
        "/api/example/data/test-key",
        HttpMethod::Get,
    );
    
    // Find endpoint
    let endpoint = registry.find_endpoint(&request.path, request.method).await;
    assert!(endpoint.is_some());
    
    if let Some((_, endpoint)) = endpoint {
        assert_eq!(endpoint.path, "/api/example/data/:key");
        assert_eq!(endpoint.method, HttpMethod::Get);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_data_set_get() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    registry.register_plugin(plugin).await?;
    
    // Set data
    let set_request = WebRequest::new(
        "/api/example/data/test-key",
        HttpMethod::Post,
    )
    .with_body(json!({
        "value": "test-value",
        "timestamp": 12345
    }))
    .with_route_param("key", "test-key")
    .with_permission("example.data.write");
    
    let set_response = registry.handle_request(set_request).await?;
    assert_eq!(set_response.status as u16, 201);
    
    // Get data
    let get_request = WebRequest::new(
        "/api/example/data/test-key",
        HttpMethod::Get,
    )
    .with_route_param("key", "test-key")
    .with_permission("example.data.read");
    
    let get_response = registry.handle_request(get_request).await?;
    assert_eq!(get_response.status as u16, 200);
    
    if let Some(body) = get_response.body {
        assert_eq!(body["value"], "test-value");
        assert_eq!(body["timestamp"], 12345);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_component_markup() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    
    // Get component IDs
    let plugin_components = plugin.get_components();
    let greeting_id = plugin_components[0].id;
    
    registry.register_plugin(plugin).await?;
    
    // Get component markup
    let markup = registry.get_component_markup(
        greeting_id,
        json!({
            "name": "Test User"
        }),
    ).await?;
    
    // Check markup
    assert!(markup.contains("Hello, Test User!"));
    assert!(markup.contains("greeting-component"));
    
    Ok(())
}

#[tokio::test]
async fn test_shutdown() -> Result<()> {
    // Create plugin registry
    let registry = WebPluginRegistry::new();
    
    // Create and register plugin
    let plugin = ExamplePlugin::new();
    let plugin_id = plugin.metadata().id;
    registry.register_plugin(plugin).await?;
    
    // Check initial status
    let status = registry.get_plugin_status(&plugin_id).await?;
    assert_eq!(status, PluginStatus::Ready);
    
    // Shutdown
    registry.shutdown().await?;
    
    // Check final status
    let status = registry.get_plugin_status(&plugin_id).await?;
    assert_eq!(status, PluginStatus::Disabled);
    
    Ok(())
} 