//! Plugin migration tests
//!
//! These tests verify that the migration from the existing plugin system to
//! the unified plugin system works as expected.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use anyhow::Result;
    use squirrel_web::{
        plugins_legacy::{
            Plugin, WebPlugin, PluginMetadata, WebEndpoint, HttpMethod, PluginManager
        },
        plugins,
        plugin_adapter
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use uuid::Uuid;
    
    /// Test plugin implementation
    #[derive(Debug, Clone)]
    struct TestPlugin {
        metadata: PluginMetadata,
    }
    
    impl TestPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "Test Plugin".to_string(),
                    version: "0.1.0".to_string(),
                    description: "Test plugin for migration testing".to_string(),
                    author: "Test Author".to_string(),
                    capabilities: vec!["test".to_string()],
                    dependencies: vec![],
                },
            }
        }
    }
    
    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }
    
    #[async_trait]
    impl WebPlugin for TestPlugin {
        fn get_endpoints(&self) -> Vec<WebEndpoint> {
            vec![
                WebEndpoint {
                    path: "/test".to_string(),
                    method: HttpMethod::Get,
                    permissions: vec![],
                },
                WebEndpoint {
                    path: "/test/create".to_string(),
                    method: HttpMethod::Post,
                    permissions: vec!["create".to_string()],
                },
            ]
        }
        
        fn get_components(&self) -> Vec<squirrel_web::plugins_legacy::WebComponent> {
            // Return an empty list for test purposes
            Vec::new()
        }
        
        async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
            match endpoint.path.as_str() {
                "/test" => {
                    Ok(serde_json::json!({ "status": "ok" }))
                },
                "/test/create" => {
                    Ok(serde_json::json!({ "status": "created" }))
                },
                _ => {
                    Err(anyhow::anyhow!("Unknown endpoint"))
                }
            }
        }
    }
    
    /// Test the initialization of the plugin system through the adapter
    #[tokio::test]
    async fn test_plugin_adapter_init() {
        // Initialize the plugin system through the adapter
        let result = plugin_adapter::init_plugin_system().await;
        
        // Verify initialization succeeded
        assert!(result.is_ok());
        
        // Verify we got a plugin manager
        let plugin_manager = result.unwrap();
        
        // Verify plugin manager is empty
        assert_eq!(plugin_manager.0.list_plugins().await.len(), 0);
    }
    
    /// Test the registration of a plugin through the legacy system
    #[tokio::test]
    async fn test_plugin_registration() {
        // Create a plugin manager
        let plugin_manager = PluginManager::new();
        
        // Create a test plugin
        let plugin = Box::new(TestPlugin::new());
        
        // Register the plugin
        let result = plugin_manager.register_plugin(plugin).await;
        
        // Verify registration succeeded
        assert!(result.is_ok());
        
        // Verify plugin count
        assert_eq!(plugin_manager.list_plugins().await.len(), 1);
    }
    
    /// Test getting plugin endpoints
    #[tokio::test]
    async fn test_plugin_endpoints() {
        // Create a plugin manager
        let plugin_manager = PluginManager::new();
        
        // Create a test plugin
        let plugin = Box::new(TestPlugin::new());
        let plugin_id = plugin.metadata().id;
        
        // Register the plugin
        plugin_manager.register_plugin(plugin).await.unwrap();
        
        // Get endpoints
        let endpoints = plugin_manager.get_endpoints::<()>().await.unwrap();
        
        // Verify endpoint count
        assert_eq!(endpoints.len(), 2);
        
        // Verify endpoint paths
        let endpoint_paths: Vec<String> = endpoints
            .iter()
            .map(|(_, endpoint)| endpoint.path.clone())
            .collect();
        
        assert!(endpoint_paths.contains(&"/test".to_string()));
        assert!(endpoint_paths.contains(&"/test/create".to_string()));
        
        // Verify plugin ID
        for (id, _) in endpoints {
            assert_eq!(id, plugin_id);
        }
    }
    
    // Import the Plugin trait from the new plugins module
    use squirrel_web::plugins::Plugin as NewPlugin;
    
    /// Test unified plugin implementation for backward compatibility
    #[derive(Clone)]
    struct UnifiedTestPlugin {
        metadata: plugins::PluginMetadata,
        status: plugins::PluginStatus,
    }
    
    impl UnifiedTestPlugin {
        fn new(id: &str) -> Self {
            Self {
                metadata: plugins::PluginMetadata {
                    id: id.to_string(),
                    name: "Unified Test Plugin".to_string(),
                    version: "1.0.0".to_string(),
                    description: "A test plugin for the unified plugin system".to_string(),
                    author: "Test Author".to_string(),
                    repository: None,
                    license: None,
                    tags: vec!["test".to_string()],
                },
                status: plugins::PluginStatus::Active,
            }
        }
    }
    
    #[async_trait::async_trait]
    impl plugins::Plugin for UnifiedTestPlugin {
        fn metadata(&self) -> &plugins::PluginMetadata {
            &self.metadata
        }
        
        fn status(&self) -> plugins::PluginStatus {
            self.status
        }
        
        fn set_status(&mut self, status: plugins::PluginStatus) {
            self.status = status;
        }
    }
    
    #[async_trait::async_trait]
    impl plugins::WebPlugin for UnifiedTestPlugin {
        fn get_endpoints(&self) -> Vec<plugins::WebEndpoint> {
            vec![
                plugins::WebEndpoint::new(
                    "/api/unified/test".to_string(),
                    plugins::HttpMethod::Get,
                    "Test endpoint".to_string(),
                ),
                plugins::WebEndpoint::new(
                    "/api/unified/create".to_string(),
                    plugins::HttpMethod::Post,
                    "Create test resource".to_string(),
                )
                .with_permission("create".to_string()),
            ]
        }
        
        fn get_components(&self) -> Vec<plugins::WebComponent> {
            vec![
                plugins::WebComponent::new(
                    "Test Component".to_string(),
                    plugins::ComponentType::Widget,
                    "A test component".to_string(),
                )
                .with_route("/test".to_string())
                .with_priority(10)
            ]
        }
        
        async fn handle_request(&self, request: plugins::WebRequest) -> Result<plugins::WebResponse> {
            match (request.path.as_str(), request.method) {
                ("/api/unified/test", plugins::HttpMethod::Get) => {
                    Ok(plugins::WebResponse::ok().with_body(serde_json::json!({
                        "status": "ok",
                        "plugin": self.metadata.id,
                    })))
                },
                ("/api/unified/create", plugins::HttpMethod::Post) => {
                    Ok(plugins::WebResponse::created().with_body(serde_json::json!({
                        "status": "created",
                        "data": request.body,
                    })))
                },
                _ => {
                    Ok(plugins::WebResponse::not_found())
                }
            }
        }
        
        async fn get_component_markup(&self, id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
            // Return a simple HTML string for testing
            Ok(format!(r#"<div class="test-component" id="{}">{}</div>"#, 
                id, props.to_string()))
        }
    }
    
    /// Test the unified plugin registry
    #[tokio::test]
    async fn test_unified_plugin_registry() {
        // Create a unified plugin registry
        let registry = plugins::WebPluginRegistry::new();
        
        // Create a unified test plugin
        let plugin = UnifiedTestPlugin::new("unified-test-1");
        
        // Register the plugin
        let result = registry.register_plugin(plugin).await;
        
        // Verify registration succeeded
        assert!(result.is_ok());
        
        // Verify we can get plugins
        let plugins = registry.get_plugins().await;
        assert_eq!(plugins.len(), 1);
        
        // Verify plugin ID
        assert_eq!(plugins[0].metadata().id, "unified-test-1");
        
        // Test enabling/disabling
        let disable_result = registry.disable_plugin("unified-test-1").await;
        assert!(disable_result.is_ok());
        
        // Verify plugin is disabled
        let disabled_plugins = registry.get_disabled_plugins().await;
        assert_eq!(disabled_plugins.len(), 1);
        
        // Verify no active plugins
        let active_plugins = registry.get_plugins().await;
        assert_eq!(active_plugins.len(), 0);
        
        // Re-enable the plugin
        let enable_result = registry.enable_plugin("unified-test-1").await;
        assert!(enable_result.is_ok());
        
        // Verify plugin is enabled
        let active_plugins = registry.get_plugins().await;
        assert_eq!(active_plugins.len(), 1);
    }
    
    /// Test web plugin with unified registry
    #[tokio::test]
    async fn test_web_plugin_with_unified_registry() {
        // Create a unified plugin registry
        let registry = plugins::WebPluginRegistry::new();
        
        // Create a unified test plugin
        let plugin = UnifiedTestPlugin::new("unified-test-2");
        
        // Register the plugin
        registry.register_plugin(plugin).await.unwrap();
        
        // Get endpoints
        let endpoints = registry.get_endpoints().await;
        
        // Verify endpoint count
        assert_eq!(endpoints.len(), 2);
        
        // Verify endpoint paths
        let endpoint_paths: Vec<String> = endpoints
            .iter()
            .map(|(_, endpoint)| endpoint.path.clone())
            .collect();
        
        assert!(endpoint_paths.contains(&"/api/unified/test".to_string()));
        assert!(endpoint_paths.contains(&"/api/unified/create".to_string()));
        
        // Test request handling
        let request = plugins::WebRequest {
            path: "/api/unified/test".to_string(),
            method: plugins::HttpMethod::Get,
            headers: std::collections::HashMap::new(),
            query_params: std::collections::HashMap::new(),
            body: None,
        };
        
        let response = registry.handle_request(request).await.unwrap();
        
        // Verify response
        assert_eq!(response.status, plugins::model::HttpStatus::Ok);
        
        if let Some(body) = response.body {
            let json: serde_json::Value = body;
            assert_eq!(json["status"], "ok");
            assert_eq!(json["plugin"], "unified-test-2");
        } else {
            panic!("Expected response body");
        }
        
        // Test components
        let components = registry.get_components().await;
        
        // Verify component count
        assert_eq!(components.len(), 1);
        
        // Get component markup
        let component_id = components[0].1.id;
        let markup = registry.get_component_markup(component_id, serde_json::json!({"test": true})).await.unwrap();
        
        // Verify markup contains component ID
        assert!(markup.contains(&component_id.to_string()));
    }
} 