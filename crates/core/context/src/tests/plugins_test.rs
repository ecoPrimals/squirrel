// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for plugin integration

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use serde_json::json;
    use tokio::test;
    
    use crate::{create_manager_with_config, ContextManagerConfig};
    use crate::plugins::ContextPluginManager;
    
    // This test checks if the plugin manager can be initialized
    #[test]
    async fn test_plugin_initialization() {
        // Create context manager with plugins enabled
        let config = ContextManagerConfig {
            enable_plugins: true,
            ..Default::default()
        };
        
        let manager = create_manager_with_config(config);
        
        // Initialize the manager
        manager.initialize().await.expect("Failed to initialize manager");
        
        // Get the plugin manager
        let plugin_manager = manager.get_plugin_manager().await;
        assert!(plugin_manager.is_some(), "Plugin manager should be available");
    }
    
    // This test checks if plugins can be disabled correctly
    #[test]
    async fn test_plugin_disable() {
        // Create context manager with plugins disabled
        let config = ContextManagerConfig {
            enable_plugins: false,
            ..Default::default()
        };
        
        let manager = create_manager_with_config(config);
        
        // Initialize the manager
        manager.initialize().await.expect("Failed to initialize manager");
        
        // Get the plugin manager
        let plugin_manager = manager.get_plugin_manager().await;
        assert!(plugin_manager.is_none(), "Plugin manager should not be available");
    }
    
    // Test direct plugin manager creation and methods
    #[test]
    async fn test_direct_plugin_manager() {
        // Create a plugin manager directly
        let plugin_manager = Arc::new(ContextPluginManager::new());
        
        // Verify empty transformations and adapters
        let transformations = plugin_manager.get_transformations().await;
        assert!(transformations.is_empty(), "Should have no transformations initially");
        
        let adapters = plugin_manager.get_adapters().await;
        assert!(adapters.is_empty(), "Should have no adapters initially");
    }
    
    // The following test is only run when the 'with-plugins' feature is enabled,
    // as it depends on the actual plugin implementations from the plugins crate
    #[cfg(feature = "with-plugins")]
    #[test]
    async fn test_data_transformation() {
        // Create context manager with plugins enabled
        let config = ContextManagerConfig {
            enable_plugins: true,
            ..Default::default()
        };
        
        let manager = create_manager_with_config(config);
        
        // Initialize the manager
        manager.initialize().await.expect("Failed to initialize manager");
        
        // Test data transformation
        let test_data = json!({
            "data": {
                "key": "value"
            }
        });
        
        let result = manager.transform_data("context.standard", test_data).await.expect("Transform failed");
        
        // Check that the result contains the expected structure
        assert!(result.is_object(), "Result should be an object");
        assert!(result.get("result").is_some(), "Result should contain 'result' field");
        assert!(result.get("metadata").is_some(), "Result should contain 'metadata' field");
    }
} 