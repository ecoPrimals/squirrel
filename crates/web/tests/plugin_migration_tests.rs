//! Plugin migration tests
//!
//! These tests verify that the migration from the existing plugin system to
//! the unified plugin system works as expected.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use anyhow::Result;
    use squirrel_web::{
        plugins::{
            Plugin, WebPlugin, PluginMetadata, WebEndpoint, HttpMethod, PluginManager
        },
        plugin_adapter
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use uuid::Uuid;
    
    /// Test plugin implementation
    #[derive(Debug)]
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
        assert_eq!(plugin_manager.list_plugins().await.len(), 0);
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
    
    // The following tests will be enabled later when migrating to the unified plugin system
    
    /*
    /// Test the unified plugin registry
    #[tokio::test]
    async fn test_unified_plugin_registry() {
        // This test will be implemented when migrating to the unified plugin system
        // It will use the squirrel_plugins::registry::PluginRegistry instead of the PluginManager
    }
    
    /// Test web plugin with unified registry
    #[tokio::test]
    async fn test_web_plugin_with_unified_registry() {
        // This test will be implemented when migrating to the unified plugin system
        // It will verify that web plugins work with the unified plugin registry
    }
    */
} 