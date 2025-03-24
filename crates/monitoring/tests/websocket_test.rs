use std::sync::Arc;
use tokio::time::Duration;
use serde_json::Value;
use async_trait::async_trait;
use time;
use squirrel_monitoring::{
    dashboard::DashboardManager,
    dashboard::config::{DashboardConfig, ComponentSettings},
    dashboard::manager::{Manager, Component},
};
use squirrel_core::error::Result;

// Create a MockManager that implements the Manager trait for testing
#[derive(Debug)]
struct MockManager {
    components: Vec<Component>,
}

#[async_trait]
impl Manager for MockManager {
    async fn get_components(&self) -> Vec<Component> {
        self.components.clone()
    }
    
    async fn get_component_data(&self, _id: &str) -> Option<Value> {
        Some(serde_json::json!({
            "value": 42.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }))
    }
    
    async fn get_health_status(&self) -> Value {
        serde_json::json!({
            "status": "healthy",
            "components": []
        })
    }
}

impl MockManager {
    fn new() -> Self {
        // Create a test component
        let test_component = Component {
            id: "test_performance_graph".to_string(),
            name: "Test Performance Graph".to_string(),
            component_type: "graph".to_string(),
            config: ComponentSettings::default(),
            data: None,
            last_updated: Some(time::OffsetDateTime::now_utc().unix_timestamp() as u64),
        };
        
        Self {
            components: vec![test_component],
        }
    }
    
    async fn update_data(&self, _data: std::collections::HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

/// Integration test for the WebSocket dashboard functionality
#[tokio::test]
async fn test_dashboard_websocket() -> Result<()> {
    // Create a dashboard configuration with WebSocket enabled
    let mut config = DashboardConfig::default();
    config.server = Some(Default::default());
    config.server.as_mut().unwrap().host = "127.0.0.1".to_string();
    config.server.as_mut().unwrap().port = 9898; // Use a different port for testing
    config.update_interval = 1; // Fast refresh for testing
    
    // Create and start the dashboard manager
    let dashboard = DashboardManager::new(config.clone());
    
    // Try to start the dashboard manager
    dashboard.start().await?;
    println!("Dashboard started successfully with WebSocket server");
    
    // Wait a moment to make sure the server is running
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create a MockManager
    let arc_manager = Arc::new(MockManager::new());
    
    // Add some sample data
    let mut data = std::collections::HashMap::new();
    data.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 42.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    arc_manager.update_data(data).await?;
    println!("Added test data");
    
    // Allow time for WebSocket to process
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Stop the dashboard manager
    dashboard.stop().await?;
    println!("Dashboard stopped successfully");
    
    Ok(())
} 