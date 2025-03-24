//! Tests for the web plugin adapters
//!
//! This module contains tests to verify the bidirectional compatibility adapters.

#[cfg(test)]
mod adapter_tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::{Value, json};
    use uuid::Uuid;
    
    use crate::plugin::{Plugin, PluginMetadata, PluginStatus};
    use crate::web::{
        WebPlugin, WebEndpoint, WebComponent, ComponentType,
        WebRequest, WebResponse, HttpMethod, HttpStatus,
        LegacyWebPluginAdapter, NewWebPluginAdapter
    };
    use crate::web::adapter::LegacyWebPluginTrait;
    use std::any::Any;
    
    // Mock legacy plugin that implements the old API
    struct MockLegacyPlugin {
        metadata: PluginMetadata,
        status: PluginStatus,
    }
    
    impl MockLegacyPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "mock_legacy".to_string(),
                    version: "1.0.0".to_string(),
                    description: "A mock legacy plugin".to_string(),
                    author: "Test".to_string(),
                    capabilities: Vec::new(),
                    dependencies: Vec::new(),
                },
                status: PluginStatus::Registered,
            }
        }
        
        // Legacy handler methods
        async fn handle_get(&self, _path: &str) -> Result<WebResponse> {
            Ok(WebResponse {
                status: HttpStatus::Ok,
                headers: HashMap::new(),
                body: Some(json!({"message": "Legacy GET response"})),
            })
        }
        
        async fn handle_post(&self, _path: &str, _body: Option<Value>) -> Result<WebResponse> {
            Ok(WebResponse {
                status: HttpStatus::Created,
                headers: HashMap::new(),
                body: Some(json!({"message": "Legacy POST response"})),
            })
        }
    }
    
    #[async_trait]
    impl Plugin for MockLegacyPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl LegacyWebPluginTrait for MockLegacyPlugin {
        fn get_endpoints(&self) -> Vec<crate::plugin::WebEndpoint> {
            vec![
                crate::plugin::WebEndpoint {
                    path: "/".to_string(),
                    method: "GET".to_string(),
                    permissions: Vec::new(),
                },
                crate::plugin::WebEndpoint {
                    path: "/".to_string(),
                    method: "POST".to_string(),
                    permissions: Vec::new(),
                },
            ]
        }
        
        async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value> {
            match method {
                "GET" => {
                    let response = self.handle_get(path).await?;
                    Ok(response.body.unwrap_or(json!({})))
                },
                "POST" => {
                    let response = self.handle_post(path, Some(body)).await?;
                    Ok(response.body.unwrap_or(json!({})))
                },
                _ => Ok(json!({"error": "Method not supported"})),
            }
        }
        
        fn get_components(&self) -> Vec<crate::web::adapter::LegacyWebComponent> {
            vec![
                crate::web::adapter::LegacyWebComponent {
                    id: Uuid::new_v4().to_string(),
                    name: "Legacy Component".to_string(),
                    description: "A legacy component".to_string(),
                    component_type: "page".to_string(),
                    properties: json!({}),
                },
            ]
        }
        
        async fn get_component_markup(&self, _component_id: &str, _props: Value) -> Result<String> {
            Ok("<div>Legacy Component</div>".to_string())
        }
    }
    
    // Mock new plugin that implements WebPlugin directly
    struct MockNewPlugin {
        metadata: PluginMetadata,
        status: PluginStatus,
    }
    
    impl MockNewPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "mock_new".to_string(),
                    version: "1.0.0".to_string(),
                    description: "A mock new plugin".to_string(),
                    author: "Test".to_string(),
                    capabilities: Vec::new(),
                    dependencies: Vec::new(),
                },
                status: PluginStatus::Registered,
            }
        }
    }
    
    #[async_trait]
    impl Plugin for MockNewPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl WebPlugin for MockNewPlugin {
        fn get_endpoints(&self) -> Vec<WebEndpoint> {
            vec![
                WebEndpoint {
                    id: Uuid::new_v4(),
                    path: "/api/new".to_string(),
                    method: HttpMethod::Get,
                    description: "Get new data".to_string(),
                    permissions: vec![],
                    is_public: false,
                    is_admin: false,
                    tags: vec![],
                },
                WebEndpoint {
                    id: Uuid::new_v4(),
                    path: "/api/new".to_string(),
                    method: HttpMethod::Post,
                    description: "Create new data".to_string(),
                    permissions: vec![],
                    is_public: false,
                    is_admin: false,
                    tags: vec![],
                }
            ]
        }
        
        fn get_components(&self) -> Vec<WebComponent> {
            vec![
                WebComponent {
                    id: Uuid::new_v4(),
                    name: "NewComponent".to_string(),
                    description: "A new component".to_string(),
                    component_type: ComponentType::Page,
                    properties: HashMap::new(),
                    route: None,
                    priority: 0,
                    permissions: vec![],
                    parent: None,
                    icon: None,
                }
            ]
        }
        
        async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
            match (request.method, request.path.as_str()) {
                (HttpMethod::Get, "/api/new") => {
                    Ok(WebResponse {
                        status: HttpStatus::Ok,
                        headers: HashMap::new(),
                        body: Some(json!({"message": "New GET response"})),
                    })
                },
                (HttpMethod::Post, "/api/new") => {
                    Ok(WebResponse {
                        status: HttpStatus::Created,
                        headers: HashMap::new(),
                        body: Some(json!({"message": "New POST response"})),
                    })
                },
                _ => {
                    Ok(WebResponse {
                        status: HttpStatus::NotFound,
                        headers: HashMap::new(),
                        body: Some(json!({"error": "Not Found"})),
                    })
                }
            }
        }
        
        async fn get_component_markup(&self, _component_id: Uuid, _props: Value) -> Result<String> {
            Ok("<div>New Component</div>".to_string())
        }
    }
    
    #[tokio::test]
    async fn test_legacy_adapter() {
        // Create a mock legacy plugin
        let legacy_plugin = Arc::new(MockLegacyPlugin::new());
        
        // Create an adapter for the legacy plugin
        let adapter = LegacyWebPluginAdapter::new(legacy_plugin.clone());
        
        // Test endpoints
        let endpoints = adapter.get_endpoints();
        assert_eq!(endpoints.len(), 2); // GET and POST endpoints for root path
        
        // Test handling GET request
        let get_request = WebRequest {
            path: "/".to_string(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
        };
        
        let get_response = adapter.handle_request(get_request).await.unwrap();
        assert_eq!(get_response.status, HttpStatus::Ok);
        assert_eq!(get_response.body.unwrap(), json!({"message": "Legacy GET response"}));
        
        // Test handling POST request
        let post_request = WebRequest {
            path: "/".to_string(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            body: Some(json!({"data": "test"})),
            user_id: None,
            permissions: vec![],
        };
        
        let post_response = adapter.handle_request(post_request).await.unwrap();
        assert_eq!(post_response.status, HttpStatus::Created);
        assert_eq!(post_response.body.unwrap(), json!({"message": "Legacy POST response"}));
    }
    
    #[tokio::test]
    async fn test_new_adapter() {
        // Create a mock new plugin
        let new_plugin = Arc::new(MockNewPlugin::new());
        
        // Create an adapter for the new plugin
        let adapter = NewWebPluginAdapter::new(new_plugin.clone());
        
        // Test handling GET request
        let get_request = WebRequest {
            path: "/api/new".to_string(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
        };
        
        let get_response = adapter.handle_request(get_request).await.unwrap();
        assert_eq!(get_response.status, HttpStatus::Ok);
        assert_eq!(get_response.body.unwrap(), json!({"message": "New GET response"}));
        
        // Test handling POST request
        let post_request = WebRequest {
            path: "/api/new".to_string(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            body: Some(json!({"data": "test"})),
            user_id: None,
            permissions: vec![],
        };
        
        let post_response = adapter.handle_request(post_request).await.unwrap();
        assert_eq!(post_response.status, HttpStatus::Created);
        assert_eq!(post_response.body.unwrap(), json!({"message": "New POST response"}));
        
        // Test component markup
        let component_id = new_plugin.get_components()[0].id;
        let markup = adapter.get_component_markup(component_id, json!({})).await.unwrap();
        assert_eq!(markup, "<div>New Component</div>");
    }
} 