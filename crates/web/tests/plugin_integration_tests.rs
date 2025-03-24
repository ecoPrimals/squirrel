use anyhow::Result;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use squirrel_web::plugins::{
    Plugin, PluginMetadata, PluginStatus, WebPlugin,
    model::{WebRequest, WebResponse, WebEndpoint, HttpMethod, ComponentType, WebComponent},
    WebPluginRegistry,
    example::ExamplePlugin,
};

/// Test the example plugin directly
#[tokio::test]
async fn test_example_plugin() -> Result<()> {
    // Create a WebPluginRegistry
    let registry = WebPluginRegistry::new();
    
    // Register the example plugin
    let example_plugin = ExamplePlugin::new();
    registry.register_plugin(example_plugin).await?;
    
    // Verify that the plugin was registered
    let plugins = registry.get_plugins().await;
    assert_eq!(plugins.len(), 1);
    
    let plugin = &plugins[0];
    assert_eq!(plugin.metadata().name, "Example Plugin");
    
    // Get the endpoints
    let endpoints = registry.get_endpoints().await;
    assert!(endpoints.len() > 0);
    
    // Verify we can find the hello endpoint
    let hello_endpoint = endpoints.iter()
        .find(|(_, endpoint)| endpoint.path == "/api/example/hello")
        .expect("Hello endpoint not found");
    
    assert_eq!(hello_endpoint.1.method, HttpMethod::Get);
    
    // Test a request to the hello endpoint
    let request = WebRequest::new("/api/example/hello".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status, squirrel_web::plugins::model::HttpStatus::Ok);
    assert!(response.body.is_some());
    
    // Check the response content
    let body = response.body.unwrap();
    let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    
    assert_eq!(message, "Hello from Example Plugin!");
    
    // Test the counter functionality
    // First get initial count
    let request = WebRequest::new("/api/example/counter".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status, squirrel_web::plugins::model::HttpStatus::Ok);
    let body = response.body.unwrap();
    let initial_count = body.get("counter").and_then(|c| c.as_u64()).unwrap_or(0);
    
    // Now increment the counter
    let request = WebRequest::new("/api/example/counter".to_string(), HttpMethod::Post);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status, squirrel_web::plugins::model::HttpStatus::Ok);
    
    // Verify the counter was incremented
    let request = WebRequest::new("/api/example/counter".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    let body = response.body.unwrap();
    let new_count = body.get("counter").and_then(|c| c.as_u64()).unwrap_or(0);
    
    assert_eq!(new_count, initial_count + 1, "Counter should have been incremented");
    
    Ok(())
}

/// Test creating a custom plugin
#[derive(Clone)]
struct TestPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "test_plugin_id".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                author: "Test Author".to_string(),
                repository: None,
                license: None,
                tags: vec!["test".to_string()],
            },
            status: PluginStatus::Active,
        }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
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

#[async_trait]
impl WebPlugin for TestPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                "/test/test_plugin_id".to_string(),
                HttpMethod::Get,
                "Test endpoint".to_string()
            ),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                "Test Component".to_string(),
                ComponentType::Widget,
                "Test component".to_string()
            ),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.path.as_str(), request.method) {
            ("/test/test_plugin_id", HttpMethod::Get) => {
                // Return test response
                Ok(WebResponse::ok().with_body(json!({
                    "plugin_id": self.metadata.id,
                    "path": request.path,
                    "message": "Test plugin response"
                })))
            },
            _ => {
                // Default not found response
                Ok(WebResponse::not_found().with_body(json!({
                    "error": "Endpoint not found",
                    "path": request.path
                })))
            }
        }
    }
    
    async fn get_component_markup(&self, _id: Uuid, props: serde_json::Value) -> Result<String> {
        // Simple markup generation for tests
        Ok(format!(
            "<div class='test-component' data-plugin-id='{}'><pre>{}</pre></div>",
            self.metadata.id,
            serde_json::to_string_pretty(&props).unwrap_or_default()
        ))
    }
}

/// Test a custom plugin
#[tokio::test]
async fn test_custom_plugin() -> Result<()> {
    // Create a WebPluginRegistry
    let registry = WebPluginRegistry::new();
    
    // Register the custom plugin
    let test_plugin = TestPlugin::new();
    registry.register_plugin(test_plugin).await?;
    
    // Verify that the plugin was registered
    let plugins = registry.get_plugins().await;
    assert_eq!(plugins.len(), 1);
    
    // Get the endpoints
    let endpoints = registry.get_endpoints().await;
    assert_eq!(endpoints.len(), 1);
    
    // Verify we can find the test endpoint
    let (_plugin_id, test_endpoint) = endpoints.iter()
        .find(|(_, endpoint)| endpoint.path == "/test/test_plugin_id")
        .expect("Test endpoint not found");
    
    assert_eq!(test_endpoint.method, HttpMethod::Get);
    
    // Test a request to the test endpoint
    let request = WebRequest::new("/test/test_plugin_id".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status, squirrel_web::plugins::model::HttpStatus::Ok);
    assert!(response.body.is_some());
    
    // Check the response content
    let body = response.body.unwrap();
    let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    
    assert_eq!(message, "Test plugin response");
    
    // Test component markup generation
    let components = registry.get_components().await;
    assert_eq!(components.len(), 1);
    
    let (_plugin_id, component) = components.iter()
        .find(|(_, c)| c.name == "Test Component")
        .expect("Test component not found");
    
    let props = json!({
        "content": "Custom Content"
    });
    
    let markup = registry.get_component_markup(component.id, props).await?;
    assert!(markup.contains("test-component"), "Markup should contain component class");
    assert!(markup.contains("test_plugin_id"), "Markup should contain plugin ID");
    assert!(markup.contains("Custom Content"), "Markup should contain custom content");
    
    Ok(())
} 