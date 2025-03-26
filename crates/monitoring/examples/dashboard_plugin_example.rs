//! Dashboard Plugin Example
//!
//! This example demonstrates how to use the dashboard plugin system
//! to extend the dashboard with custom visualization and data source plugins.

use anyhow::Result;
use std::sync::Arc;
use squirrel_monitoring::dashboard::{DashboardManager, config::DashboardConfig};
use squirrel_monitoring::dashboard::plugins::{
    DashboardPlugin, 
    VisualizationPlugin, 
    DashboardPluginType,
    PluginEvent,
    PluginMetadata,
    example::{ExamplePlugin, ExampleDataSourcePlugin},
};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use squirrel_core::error::SquirrelError;
use squirrel_monitoring::dashboard::Update;
use squirrel_monitoring::dashboard::DashboardComponent;

/// Custom visualization plugin implementation
#[derive(Debug)]
struct CustomVisualizationPlugin {
    metadata: PluginMetadata,
    data: tokio::sync::Mutex<serde_json::Value>,
    id: String,
}

impl CustomVisualizationPlugin {
    fn new() -> Self {
        let id = "custom-visualization-plugin".to_string();
        Self {
            metadata: PluginMetadata {
                id: id.clone(),
                name: "Custom Visualization".to_string(),
                description: "Custom visualization plugin for the dashboard".to_string(),
                version: "1.0.0".to_string(),
                author: "DataScienceBioLab".to_string(),
                plugin_type: DashboardPluginType::Visualization,
            },
            data: tokio::sync::Mutex::new(json!({
                "chart_type": "pie",
                "series": [
                    {
                        "name": "CPU Utilization",
                        "data": [
                            { "name": "User", "value": 40 },
                            { "name": "System", "value": 25 },
                            { "name": "Idle", "value": 35 }
                        ]
                    }
                ],
                "options": {
                    "title": "CPU Utilization",
                    "colors": ["#4C9AFF", "#FF5630", "#36B37E"]
                }
            })),
            id,
        }
    }
}

#[async_trait::async_trait]
impl DashboardComponent for CustomVisualizationPlugin {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn start(&self) -> Result<(), SquirrelError> {
        info!("Starting custom visualization plugin");
        Ok(())
    }
    
    async fn get_data(&self) -> Result<serde_json::Value, SquirrelError> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        Some(chrono::Utc::now())
    }
    
    async fn get_update(&self) -> Result<Update, SquirrelError> {
        let data = self.data.lock().await.clone();
        Ok(Update {
            component_id: self.id().to_string(),
            data,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn handle_event(&self, event: serde_json::Value) -> Result<(), SquirrelError> {
        info!("Received event in custom component: {:?}", event);
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), SquirrelError> {
        info!("Stopping custom visualization plugin");
        Ok(())
    }
}

#[async_trait::async_trait]
impl DashboardPlugin for CustomVisualizationPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn plugin_type(&self) -> DashboardPluginType {
        DashboardPluginType::Visualization
    }
    
    async fn initialize(&self) -> Result<(), SquirrelError> {
        info!("Initializing custom visualization plugin");
        Ok(())
    }
    
    async fn get_data(&self) -> Result<serde_json::Value, SquirrelError> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }
    
    async fn update(&self, data: serde_json::Value) -> Result<(), SquirrelError> {
        let mut current_data = self.data.lock().await;
        *current_data = data;
        Ok(())
    }
    
    async fn handle_event(&self, event: PluginEvent) -> Result<(), SquirrelError> {
        match event {
            PluginEvent::ConfigUpdate(data) => {
                info!("Received config update event");
                self.update(data).await?;
            },
            PluginEvent::DataUpdate(data) => {
                info!("Received data update event");
                self.update(data).await?;
            },
            PluginEvent::Custom(name, data) => {
                info!("Received custom event: {}", name);
                self.update(data).await?;
            },
        }
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), SquirrelError> {
        info!("Shutting down custom visualization plugin");
        Ok(())
    }
}

#[async_trait::async_trait]
impl VisualizationPlugin for CustomVisualizationPlugin {
    async fn get_visualization_config(&self) -> Result<serde_json::Value, SquirrelError> {
        Ok(json!({
            "type": "pie_chart",
            "title": "CPU Utilization Visualization",
            "width": 600,
            "height": 400,
            "colors": ["#4C9AFF", "#FF5630", "#36B37E"],
            "animation": true,
            "legend": true
        }))
    }
    
    async fn render(&self, data: serde_json::Value) -> Result<String, SquirrelError> {
        let current_data = self.data.lock().await;
        
        // Combine current data with new data
        let mut combined = current_data.clone();
        
        if let Some(obj) = combined.as_object_mut() {
            if let Some(data_obj) = data.as_object() {
                for (key, value) in data_obj {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Create a simple HTML/SVG visualization
        let svg = format!(
            "<svg width=\"400\" height=\"300\" xmlns=\"http://www.w3.org/2000/svg\">\
                <circle cx=\"200\" cy=\"150\" r=\"120\" fill=\"#36B37E\" />\
                <text x=\"200\" y=\"150\" text-anchor=\"middle\" fill=\"white\" font-size=\"20\">CPU: {}%</text>\
            </svg>",
            65 // placeholder value
        );
        
        Ok(svg)
    }
}

/// Run dashboard with plugins
async fn run_dashboard_with_plugins() -> Result<()> {
    // Create dashboard configuration
    let config = DashboardConfig::default();
    
    // Create dashboard manager
    let manager = Arc::new(DashboardManager::new(config));
    
    // Start dashboard
    manager.start().await.map_err(|e| anyhow::anyhow!("Failed to start dashboard: {}", e))?;
    
    // Wait for dashboard to start
    sleep(Duration::from_secs(1)).await;
    
    // Create plugins
    let example_plugin = Arc::new(ExamplePlugin::new());
    let example_data_source = Arc::new(ExampleDataSourcePlugin::new());
    let custom_visualization = Arc::new(CustomVisualizationPlugin::new());
    
    // Register plugins
    info!("Registering example plugin");
    manager.register_component(example_plugin).await.map_err(|e| anyhow::anyhow!("Failed to register example plugin: {}", e))?;
    
    info!("Registering data source plugin");
    manager.register_component(example_data_source).await.map_err(|e| anyhow::anyhow!("Failed to register data source plugin: {}", e))?;
    
    info!("Registering custom visualization plugin");
    manager.register_component(custom_visualization).await.map_err(|e| anyhow::anyhow!("Failed to register custom plugin: {}", e))?;
    
    // Get all components
    let components = manager.get_components().await;
    info!("Registered {} components", components.len());
    
    for component in &components {
        info!("Component: {} ({})", component.name, component.id);
    }
    
    // Simulate updating plugin data
    for i in 0..5 {
        sleep(Duration::from_secs(5)).await;
        
        // Update visualization data
        info!("Updating visualization data (iteration {})", i);
        
        for component in &components {
            if component.name == "Custom Visualization" {
                manager.update_component(&component.id, json!({
                    "chart_type": "pie",
                    "series": [
                        {
                            "name": "CPU Utilization (Updated)",
                            "data": [
                                { "name": "User", "value": 35 + i * 5 },
                                { "name": "System", "value": 20 + i * 2 },
                                { "name": "Idle", "value": 45 - i * 7 }
                            ]
                        }
                    ],
                    "options": {
                        "title": format!("CPU Utilization (Update {})", i),
                        "colors": ["#4C9AFF", "#FF5630", "#36B37E"]
                    }
                })).await.map_err(|e| anyhow::anyhow!("Failed to update component: {}", e))?;
            }
        }
    }
    
    // Keep running to allow viewing the dashboard
    info!("Dashboard running at http://localhost:8080");
    info!("Press Ctrl+C to stop");
    
    // Wait for signal to stop
    tokio::signal::ctrl_c().await?;
    
    // Stop dashboard
    manager.stop().await.map_err(|e| anyhow::anyhow!("Failed to stop dashboard: {}", e))?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .init();
    
    // Run example
    if let Err(e) = run_dashboard_with_plugins().await {
        error!("Error running dashboard example: {}", e);
        return Err(e);
    }
    
    Ok(())
} 