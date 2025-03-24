//! Prometheus Component Example
//!
//! This example demonstrates how to set up a dashboard with Prometheus metrics.

use std::{net::SocketAddr, sync::Arc};
use serde_json::json;
use squirrel_monitoring::dashboard::config::{DashboardConfig, ComponentSettings};
use squirrel_monitoring::dashboard::manager::{DashboardManager, Component};
use squirrel_monitoring::dashboard::DashboardComponent;
use squirrel_monitoring::dashboard::Update;
use squirrel_core::error::Result as SquirrelResult;
use async_trait::async_trait;

// Define a constant for the server address
const SERVER_ADDR: &str = "[::1]:8765";

/// MockManager implements the Manager trait for testing
struct MockManager {
    // Add any state you need for testing
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Parse the server address
    let addr: SocketAddr = SERVER_ADDR.parse()?;
    
    // Create a dashboard configuration
    let mut config = DashboardConfig::default();
    
    // Configure the server settings
    if config.server.is_none() {
        config.server = Some(Default::default());
    }
    
    let server_config = config.server.as_mut().unwrap();
    server_config.host = addr.ip().to_string();
    server_config.port = addr.port();
    
    // Create a manager instance
    let dashboard = Arc::new(DashboardManager::new(config));
    
    // Create a sample Prometheus component
    let prometheus_component = Arc::new(PrometheusComponent {
        component: Component {
            id: "prometheus_metrics".to_string(),
            name: "Prometheus Metrics".to_string(),
            component_type: "metrics".to_string(),
            config: ComponentSettings {
                show_metrics: Some(true),
                show_alerts: Some(true),
                show_health: Some(true),
                show_network: Some(true),
                show_analytics: Some(true),
            },
            data: Some(json!({
                "metrics": [
                    {
                        "name": "http_requests_total",
                        "help": "Total number of HTTP requests",
                        "type": "counter",
                        "value": 42
                    },
                    {
                        "name": "http_request_duration_seconds",
                        "help": "HTTP request duration in seconds",
                        "type": "histogram",
                        "buckets": [0.1, 0.5, 1.0, 2.0, 5.0],
                        "values": [0.2, 0.7, 1.5, 3.0]
                    },
                    {
                        "name": "memory_usage_bytes",
                        "help": "Current memory usage in bytes",
                        "type": "gauge",
                        "value": 1024 * 1024 * 100 // 100 MB
                    }
                ]
            })),
            last_updated: Some(chrono::Utc::now().timestamp_millis() as u64),
        }
    });
    
    // Register the Prometheus component
    dashboard.register_component(prometheus_component).await?;
    
    // Start the dashboard
    dashboard.start().await?;
    
    println!("Dashboard server running at http://{}", SERVER_ADDR);
    println!("Press Ctrl+C to stop");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    // Stop the dashboard
    dashboard.stop().await?;
    
    println!("Server stopped");
    
    Ok(())
}

// Prometheus Component Implementation
#[derive(Debug)]
struct PrometheusComponent {
    component: Component,
}

#[async_trait]
impl DashboardComponent for PrometheusComponent {
    fn id(&self) -> &str {
        &self.component.id
    }
    
    async fn start(&self) -> SquirrelResult<()> {
        println!("Starting Prometheus metrics collection...");
        // In a real implementation, this would start scraping Prometheus metrics
        Ok(())
    }
    
    async fn get_data(&self) -> SquirrelResult<serde_json::Value> {
        // In a real implementation, this would query Prometheus for the latest metrics
        Ok(self.component.data.clone().unwrap_or(json!({})))
    }
    
    async fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.component.last_updated.map(|ts| {
            chrono::DateTime::from_timestamp_millis(ts as i64).unwrap_or_default()
        })
    }
    
    async fn get_update(&self) -> SquirrelResult<Update> {
        // Generate a simulated update with the current time
        Ok(Update {
            component_id: self.id().to_string(),
            data: self.component.data.clone().unwrap_or(json!({})),
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn handle_event(&self, event: serde_json::Value) -> SquirrelResult<()> {
        println!("Received event: {}", event);
        // In a real implementation, this would handle events like configuration changes
        Ok(())
    }
    
    async fn stop(&self) -> SquirrelResult<()> {
        println!("Stopping Prometheus metrics collection...");
        // In a real implementation, this would stop the metrics collection process
        Ok(())
    }
}