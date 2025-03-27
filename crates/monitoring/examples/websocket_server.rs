use squirrel_monitoring::{
    websocket::{WebSocketConfig, server::WebSocketServer},
    metrics::{Metric, MetricType},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use serde_json::json;

/// This example demonstrates how to use the WebSocket API to stream metrics and other monitoring data
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the WebSocket server
    let ws_config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8765,
        update_interval: 1,
        enable_compression: true,
        auth_required: false,
        ..Default::default()
    };
    
    // Initialize and start the WebSocket server
    let websocket = WebSocketServer::new(ws_config.clone());
    websocket.start().await?;
    
    println!("WebSocket server started on {}:{}", ws_config.host, ws_config.port);
    println!("To connect: `wscat -c ws://{}:{}`", ws_config.host, ws_config.port);
    println!("Example subscription message:");
    println!("{{\"action\":\"subscribe\",\"topic\":\"cpu_usage\"}}");
    
    // Start generating mock metrics
    let ws_server = websocket;
    
    // Generate some initial data
    ws_server.update_component_data("cpu_usage", json!({
        "value": 0.0,
        "timestamp": chrono::Utc::now().timestamp(),
        "unit": "%"
    })).await?;
    
    ws_server.update_component_data("memory_usage", json!({
        "value": 0.0,
        "timestamp": chrono::Utc::now().timestamp(),
        "unit": "%"
    })).await?;
    
    // Start the update loop
    let mut interval = time::interval(Duration::from_secs(1));
    let mut i = 0;
    
    println!("Press Ctrl+C to stop the server");
    loop {
        interval.tick().await;
        i += 1;
        
        // Generate CPU usage metric (random value between 0-100)
        let cpu_usage = rand::random::<f64>() * 100.0;
        generate_metric("cpu_usage", cpu_usage, MetricType::Gauge).await?;
        
        // Generate memory usage metric (increasing and wrapping pattern)
        let memory_usage = (i % 100) as f64;
        generate_metric("memory_usage", memory_usage, MetricType::Gauge).await?;
        
        // Update data in WebSocket server
        ws_server.update_component_data("cpu_usage", json!({
            "value": cpu_usage,
            "timestamp": chrono::Utc::now().timestamp(),
            "unit": "%"
        })).await?;
        
        ws_server.update_component_data("memory_usage", json!({
            "value": memory_usage,
            "timestamp": chrono::Utc::now().timestamp(),
            "unit": "%"
        })).await?;
        
        println!("Updated metrics - CPU: {:.1}%, Memory: {:.1}%", cpu_usage, memory_usage);
    }
}

async fn generate_metric(name: &str, value: f64, metric_type: MetricType) -> Result<(), Box<dyn std::error::Error>> {
    let metric = Metric::new(
        name.to_string(),
        value,
        metric_type,
        HashMap::from([("source".to_string(), "example".to_string())])
    );
    
    // In a real implementation, you would record this metric:
    // monitoring.metrics().record(metric).await?;
    
    Ok(())
} 