//! General tests for web plugin functionality
//!
//! This module contains unit tests for the web plugin system.

#[cfg(test)]
mod general_tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use serde_json::json;
    use uuid::Uuid;

    use crate::web::{
        HttpMethod,
        WebRequest, WebResponse,
        WebComponent, ComponentType,
        WebEndpoint,
        ExampleWebPlugin,
        WebPlugin,
        LegacyWebPluginAdapter,
        WebPluginRegistry,
    };
    use crate::plugin::Plugin;

    #[tokio::test]
    async fn test_example_plugin_endpoints() {
        let plugin = ExampleWebPlugin::new();
        
        // Get endpoints
        let endpoints = plugin.get_endpoints();
        
        // Check that we have the expected number of endpoints
        assert_eq!(endpoints.len(), 5);
        
        // Check that we have a GET endpoint
        let get_endpoint = endpoints.iter().find(|e| e.method == HttpMethod::Get && e.path == "/api/examples");
        assert!(get_endpoint.is_some());
        
        // Check that we have a POST endpoint
        let post_endpoint = endpoints.iter().find(|e| e.method == HttpMethod::Post && e.path == "/api/examples");
        assert!(post_endpoint.is_some());
    }
    
    #[tokio::test]
    async fn test_example_plugin_components() {
        let plugin = ExampleWebPlugin::new();
        
        // Get components
        let components = plugin.get_components();
        
        // Check that we have the expected number of components
        assert_eq!(components.len(), 3);
        
        // Check that we have a page component
        let page_component = components.iter().find(|c| c.component_type == ComponentType::Page);
        assert!(page_component.is_some());
        
        // Check that we have a widget component
        let widget_component = components.iter().find(|c| c.component_type == ComponentType::Widget);
        assert!(widget_component.is_some());
        
        // Check that we have a navigation component
        let nav_component = components.iter().find(|c| c.component_type == ComponentType::Navigation);
        assert!(nav_component.is_some());
    }
    
    #[tokio::test]
    async fn test_example_plugin_request_handling() {
        let plugin = ExampleWebPlugin::new();
        
        // Initialize the plugin
        plugin.initialize().await.unwrap();
        
        // Create a GET request
        let request = WebRequest::new(
            HttpMethod::Get,
            "/api/examples".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
            Some("test-user".to_string()),
            vec![],
        );
        
        // Handle the request
        let response = plugin.handle_request(request).await.unwrap();
        
        // Check response
        assert_eq!(response.status, crate::web::HttpStatus::Ok);
        assert!(response.body.is_some());
        
        if let Some(body) = response.body {
            assert!(body.get("items").is_some());
            assert!(body.get("count").is_some());
            
            if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
                assert_eq!(items.len(), 2); // We initialized with 2 example items
            } else {
                panic!("Expected items array in response");
            }
        }
    }
    
    #[tokio::test]
    async fn test_example_plugin_component_markup() {
        let plugin = ExampleWebPlugin::new();
        
        // Get a component ID
        let components = plugin.get_components();
        let component = components.first().unwrap();
        let component_id = component.id;
        
        // Get markup
        let markup = plugin.get_component_markup(component_id, json!({"test": "value"})).await.unwrap();
        
        // Check markup
        assert!(!markup.is_empty());
        assert!(markup.contains(&component.name));
        assert!(markup.contains(&component.description));
        assert!(markup.contains("test"));
        assert!(markup.contains("value"));
    }
    
    #[tokio::test]
    async fn test_legacy_adapter() {
        // Create and initialize the example plugin
        let example_plugin = Arc::new(ExampleWebPlugin::new());
        example_plugin.initialize().await.unwrap();
        
        // Create the adapter
        let adapter = LegacyWebPluginAdapter::new(example_plugin.clone());
        
        // Test metadata
        assert_eq!(adapter.metadata().name, example_plugin.metadata().name);
        
        // Test endpoints
        let endpoints = adapter.get_endpoints();
        assert_eq!(endpoints.len(), example_plugin.get_endpoints().len());
        
        // Test components
        let components = adapter.get_components();
        assert_eq!(components.len(), example_plugin.get_components().len());
        
        // Test request handling
        let request = WebRequest::new(
            HttpMethod::Get,
            "/api/examples".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
            Some("test-user".to_string()),
            vec![],
        );
        
        let response = adapter.handle_request(request).await.unwrap();
        assert!(response.body.is_some());
        
        // Test component markup
        let component_id = components.first().unwrap().id;
        let markup = adapter.get_component_markup(component_id, json!({})).await.unwrap();
        assert!(!markup.is_empty());
    }
    
    #[tokio::test]
    async fn test_registry() {
        // Create the registry
        let registry = WebPluginRegistry::new(Arc::new(crate::registry::PluginRegistry::new()));
        
        // Create and initialize the example plugin
        let example_plugin = Arc::new(ExampleWebPlugin::new());
        example_plugin.initialize().await.unwrap();
        
        // Create the adapter
        let adapter = Arc::new(LegacyWebPluginAdapter::new(example_plugin.clone()));
        
        // Register the plugin via adapter
        // Note: In a real scenario, this would be done through the PluginRegistry
        // This is simplified for testing
        registry.register_endpoints(adapter.metadata().id, adapter.get_endpoints());
        registry.register_components(adapter.metadata().id, adapter.get_components());
        
        // Test finding endpoints
        let endpoint = registry.find_endpoint("/api/examples", HttpMethod::Get).await.unwrap();
        assert_eq!(endpoint.path, "/api/examples");
        assert_eq!(endpoint.method, HttpMethod::Get);
        
        // Test listing endpoints
        let endpoints = registry.get_all_endpoints().await;
        assert_eq!(endpoints.len(), 5);
        
        // Test listing components
        let components = registry.get_all_components().await;
        assert_eq!(components.len(), 3);
    }
} 