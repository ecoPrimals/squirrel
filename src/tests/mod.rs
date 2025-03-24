//! Tests for the context system
//! 
//! This module contains tests for the context system.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use serde_json::json;
    
    use crate::{create_default_manager, create_manager_with_config, ContextManagerConfig};
    
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
} 