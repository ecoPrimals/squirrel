//! Example dashboard plugins
//!
//! This module contains examples of dashboard plugins that can be used
//! as templates for creating custom plugins.

use tokio::sync::{Mutex, broadcast};
use async_trait::async_trait;
use serde_json::{json, Value};
use super::types::{DashboardPlugin, DashboardPluginType, PluginEvent, VisualizationPlugin, DataSourcePlugin, PluginMetadata};
use tracing::info;
use chrono::Utc;
use std::time::Instant;

use squirrel_core::error::Result;
use squirrel_core::error::SquirrelError;

use crate::dashboard::DashboardComponent;
use crate::dashboard::Update;

/// Example dashboard plugin for demonstrating the plugin system
/// 
/// This plugin provides a simple visualization of system metrics.
#[derive(Debug)]
pub struct ExamplePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin data (protected by mutex)
    data: Mutex<Value>,
    /// Last update time
    last_update: Mutex<Option<Instant>>,
    /// Plugin ID as string for DashboardComponent
    id: String,
}

impl ExamplePlugin {
    /// Create a new example plugin
    pub fn new() -> Self {
        let id = "example-plugin-id".to_string();
        let metadata = PluginMetadata {
            id: id.clone(),
            name: "Example Plugin".to_string(),
            description: "An example dashboard plugin for demonstration".to_string(),
            version: "0.1.0".to_string(),
            author: "Squirrel Team".to_string(),
            plugin_type: DashboardPluginType::Visualization,
        };

        let data = json!({
            "cpu": 10.0,
            "memory": 30.0,
            "disk": 40.0,
            "network": 20.0,
            "timestamp": Utc::now().to_rfc3339(),
            "test_field": "Test value for test field"
        });

        Self {
            metadata,
            data: Mutex::new(data),
            last_update: Mutex::new(None),
            id,
        }
    }
    
    async fn generate_visualization_data(&self) -> Result<Value> {
        // Get the current timestamp
        let now = Utc::now();
        let timestamp = now.timestamp();
        
        // Mock data generation based on timestamp
        let cpu = 10.0 + (timestamp % 20) as f64; // 10-29%
        let memory = 30.0 + (timestamp % 30) as f64; // 30-59%
        let disk = 40.0 + (timestamp % 20) as f64; // 40-59%
        let network = 200.0 + (timestamp % 300) as f64; // 200-499 MB/s
        
        // Create JSON data structure
        let data = json!({
            "timestamp": now.to_rfc3339(),
            "metrics": {
                "cpu_usage": cpu,
                "memory_usage": memory,
                "disk_usage": disk,
                "network_throughput": network
            }
        });
        
        Ok(data)
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DashboardComponent for ExamplePlugin {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn start(&self) -> Result<()> {
        info!("Starting example plugin: {}", self.metadata.name);
        Ok(())
    }
    
    async fn get_data(&self) -> Result<Value> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<Utc>> {
        // Convert Instant to DateTime if available
        let instant = *self.last_update.lock().await;
        instant.map(|_| Utc::now()) // Using current time instead of Instant conversion
    }
    
    async fn get_update(&self) -> Result<Update> {
        let data = self.data.lock().await.clone();
        Ok(Update {
            component_id: self.id().to_string(),
            data,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_event(&self, event: Value) -> Result<()> {
        info!("Example plugin received event: {:?}", event);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        info!("Stopping example plugin: {}", self.metadata.name);
        Ok(())
    }
}

#[async_trait]
impl DashboardPlugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn plugin_type(&self) -> DashboardPluginType {
        DashboardPluginType::Visualization
    }
    
    async fn initialize(&self) -> Result<()> {
        info!("Initializing example plugin: {}", self.metadata.name);
        Ok(())
    }
    
    async fn get_data(&self) -> Result<Value> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }
    
    async fn update(&self, data: Value) -> Result<()> {
        let mut current_data = self.data.lock().await;
        *current_data = data;
        *self.last_update.lock().await = Some(Instant::now());
        Ok(())
    }
    
    async fn handle_event(&self, event: PluginEvent) -> Result<()> {
        info!("Example plugin received event: {:?}", event);
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down example plugin: {}", self.metadata.name);
        Ok(())
    }
}

#[async_trait]
impl VisualizationPlugin for ExamplePlugin {
    async fn get_visualization_config(&self) -> Result<Value> {
        Ok(json!({
            "type": "system-metrics",
            "title": "System Metrics Visualization",
            "refresh_rate": 5000,
            "metrics": [
                {"name": "CPU", "color": "#FF6384", "unit": "%"},
                {"name": "Memory", "color": "#36A2EB", "unit": "%"},
                {"name": "Disk", "color": "#FFCE56", "unit": "%"},
                {"name": "Network", "color": "#4BC0C0", "unit": "Mbps"}
            ]
        }))
    }
    
    async fn render(&self, data: Value) -> Result<String> {
        // Use the provided data or the plugin's stored data if empty
        let data_to_render = if data.is_null() || data.as_object().is_none_or(|o| o.is_empty()) {
            self.data.lock().await.clone()
        } else {
            data
        };
        
        // Convert data to SVG visualization
        let cpu = data_to_render["cpu"].as_f64().unwrap_or(0.0);
        let memory = data_to_render["memory"].as_f64().unwrap_or(0.0);
        let disk = data_to_render["disk"].as_f64().unwrap_or(0.0);
        let network = data_to_render["network"].as_f64().unwrap_or(0.0);
        
        // Simple SVG bar chart
        let svg = format!(
            "<svg width=\"400\" height=\"200\" xmlns=\"http://www.w3.org/2000/svg\">\
             <rect x=\"10\" y=\"10\" width=\"{}\" height=\"30\" fill=\"#FF6384\"></rect>\
             <text x=\"15\" y=\"30\" fill=\"white\">CPU: {}%</text>\
             <rect x=\"10\" y=\"50\" width=\"{}\" height=\"30\" fill=\"#36A2EB\"></rect>\
             <text x=\"15\" y=\"70\" fill=\"white\">Memory: {}%</text>\
             <rect x=\"10\" y=\"90\" width=\"{}\" height=\"30\" fill=\"#FFCE56\"></rect>\
             <text x=\"15\" y=\"110\" fill=\"white\">Disk: {}%</text>\
             <rect x=\"10\" y=\"130\" width=\"{}\" height=\"30\" fill=\"#4BC0C0\"></rect>\
             <text x=\"15\" y=\"150\" fill=\"white\">Network: {}Mbps</text>\
             </svg>",
            cpu * 3.0, cpu, 
            memory * 3.0, memory,
            disk * 3.0, disk,
            network * 3.0, network
        );
        
        Ok(svg)
    }
}

/// Example data source plugin
#[derive(Debug)]
pub struct ExampleDataSourcePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Broadcast sender for data stream
    sender: Mutex<Option<broadcast::Sender<Value>>>,
    /// Plugin ID as string
    id: String,
}

impl Default for ExampleDataSourcePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExampleDataSourcePlugin {
    /// Create a new data source plugin
    pub fn new() -> Self {
        let id = "example-datasource-plugin".to_string();
        let metadata = PluginMetadata {
            id: id.clone(),
            name: "Example Data Source".to_string(),
            description: "An example data source plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Squirrel Team".to_string(),
            plugin_type: DashboardPluginType::DataSource,
        };
        
        Self {
            metadata,
            sender: Mutex::new(None),
            id,
        }
    }
    
    /// Generate demo system metrics
    fn generate_system_metrics(&self) -> Value {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let cpu = 10.0 + (timestamp % 20) as f64; // 10-29%
        let memory = 30.0 + (timestamp % 30) as f64; // 30-59%
        let disk = 40.0 + (timestamp % 20) as f64; // 40-59%
        
        json!({
            "timestamp": timestamp,
            "system": {
                "cpu_usage": cpu,
                "memory_usage": memory,
                "disk_usage": disk,
                "uptime": timestamp % 86400, // 0-86399 seconds
                "load_average": [cpu / 10.0, cpu / 20.0, cpu / 30.0]
            },
            "network": {
                "rx_bytes": timestamp * 100,
                "tx_bytes": timestamp * 50,
                "active_connections": (timestamp % 10) + 1
            },
            "data_source_field": "Test data source field value"
        })
    }
}

#[async_trait]
impl DashboardComponent for ExampleDataSourcePlugin {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn start(&self) -> Result<()> {
        // Initialize the data source
        self.initialize().await
    }
    
    async fn get_data(&self) -> Result<Value> {
        // Return current metrics
        Ok(self.generate_system_metrics())
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now())
    }
    
    async fn get_update(&self) -> Result<Update> {
        let data = self.generate_system_metrics();
        Ok(Update {
            component_id: self.id().to_string(),
            data,
            timestamp: Utc::now(),
        })
    }
    
    async fn handle_event(&self, event: Value) -> Result<()> {
        info!("Data source plugin received event: {:?}", event);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Cleanup
        self.shutdown().await
    }
}

#[async_trait]
impl DashboardPlugin for ExampleDataSourcePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn plugin_type(&self) -> DashboardPluginType {
        DashboardPluginType::DataSource
    }
    
    async fn initialize(&self) -> Result<()> {
        // Create a broadcast channel
        let (tx, _) = broadcast::channel(100);
        
        let mut sender_guard = self.sender.lock().await;
        *sender_guard = Some(tx);
        
        // Start data generation task
        let tx_clone = sender_guard.as_ref().unwrap().clone();
        
        tokio::spawn(async move {
            let plugin = ExampleDataSourcePlugin::new();
            
            loop {
                // Generate metrics every 5 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                // Generate demo metrics
                let metrics = plugin.generate_system_metrics();
                
                // Send metrics
                if tx_clone.send(metrics).is_err() {
                    // No receivers, exit the task
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    async fn get_data(&self) -> Result<Value> {
        // Return current system metrics
        Ok(self.generate_system_metrics())
    }
    
    async fn update(&self, _data: Value) -> Result<()> {
        // No update logic for this example
        Ok(())
    }
    
    async fn handle_event(&self, _event: PluginEvent) -> Result<()> {
        // No event handling for this example
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Cleanup logic here
        let mut sender_guard = self.sender.lock().await;
        *sender_guard = None;
        
        Ok(())
    }
}

#[async_trait]
impl DataSourcePlugin for ExampleDataSourcePlugin {
    async fn get_schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "system": {
                    "type": "object",
                    "properties": {
                        "cpu_usage": { "type": "number" },
                        "memory_usage": { "type": "number" },
                        "disk_usage": { "type": "number" },
                        "uptime": { "type": "number" },
                        "load_average": { "type": "array" }
                    }
                },
                "network": {
                    "type": "object",
                    "properties": {
                        "rx_bytes": { "type": "number" },
                        "tx_bytes": { "type": "number" },
                        "active_connections": { "type": "number" }
                    }
                }
            }
        }))
    }
    
    async fn query(&self, query: Value) -> Result<Value> {
        // Parse query
        let query_type = query
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");
        
        // Get current metrics
        let metrics = self.generate_system_metrics();
        
        // Filter metrics based on query type
        match query_type {
            "system" => {
                match metrics.get("system") {
                    Some(system) => {
                        Ok(json!({ "system": system }))
                    },
                    None => Err(SquirrelError::dashboard("System metrics not found"))
                }
            }
            "network" => {
                match metrics.get("network") {
                    Some(network) => {
                        Ok(json!({ "network": network }))
                    },
                    None => Err(SquirrelError::dashboard("Network metrics not found"))
                }
            }
            _ => Ok(metrics),
        }
    }
    
    async fn subscribe(&self, _query: Value) -> Result<()> {
        // In a real implementation, this would register a subscription
        Ok(())
    }
    
    async fn unsubscribe(&self, _subscription_id: String) -> Result<()> {
        // In a real implementation, this would remove a subscription
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_visualization_plugin() {
        let plugin = ExamplePlugin::new();
        
        // Test initialization
        assert!(plugin.initialize().await.is_ok());
        
        // Test metadata
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "Example Plugin");
        
        // Test plugin type
        assert_eq!(plugin.plugin_type(), DashboardPluginType::Visualization);
        
        // Test data retrieval - use fully qualified syntax with disambiguated trait implementation
        let data = crate::dashboard::plugins::types::DashboardPlugin::get_data(&plugin).await.unwrap();
        assert!(data.get("cpu").is_some());
        
        // Test visualization config
        let config = plugin.get_visualization_config().await.unwrap();
        assert!(config.get("type").is_some());
    }
    
    #[tokio::test]
    async fn test_data_source_plugin() {
        let plugin = ExampleDataSourcePlugin::new();
        
        // Test initialization
        let init_result = plugin.initialize().await;
        assert!(init_result.is_ok());
        
        // Test metadata
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "Example Data Source");
        
        // Test plugin type
        assert_eq!(plugin.plugin_type(), DashboardPluginType::DataSource);
        
        // Test data retrieval - use fully qualified syntax with disambiguated trait implementation
        let data = crate::dashboard::plugins::types::DashboardPlugin::get_data(&plugin).await.unwrap();
        assert!(data.get("system").is_some());
        
        // Test query
        let query_result = plugin.query(json!({"type": "system"})).await.unwrap();
        assert!(query_result.get("system").is_some());
        
        // Test stream
        let stream_result = plugin.subscribe(json!({"type": "system"})).await;
        assert!(stream_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_example_plugin_get_data() {
        // Create plugin
        let plugin = ExamplePlugin::new();

        // Get data (disambiguate the method calls)
        let data = crate::dashboard::plugins::types::DashboardPlugin::get_data(&plugin).await.unwrap();

        // Assert
        assert!(data.is_object());
        assert!(data.get("test_field").is_some());
    }
    
    #[tokio::test]
    async fn test_example_data_source_plugin_get_data() {
        // Create plugin
        let plugin = ExampleDataSourcePlugin::new();

        // Get data (disambiguate the method calls)
        let data = crate::dashboard::plugins::types::DashboardPlugin::get_data(&plugin).await.unwrap();

        // Assert
        assert!(data.is_object());
        assert!(data.get("data_source_field").is_some());
    }
} 