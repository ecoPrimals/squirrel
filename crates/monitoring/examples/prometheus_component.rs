use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use squirrel_monitoring::dashboard::{Manager, DashboardConfig};
use squirrel_monitoring::dashboard::components::{PrometheusMetrics, PrometheusConfig};
use squirrel_monitoring::dashboard::server::start_server;
use tokio::signal::ctrl_c;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create a dashboard manager
    let config = DashboardConfig::default();
    let manager = Arc::new(Manager::new_from_dashboard_config(config));
    
    // Create Prometheus component configuration
    let mut queries = HashMap::new();
    queries.insert("cpu_usage".to_string(), "rate(node_cpu_seconds_total{mode=\"user\"}[1m])".to_string());
    queries.insert("memory_usage".to_string(), "node_memory_MemFree_bytes / node_memory_MemTotal_bytes".to_string());
    queries.insert("disk_usage".to_string(), "node_filesystem_free_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"}".to_string());
    
    let prom_config = PrometheusConfig {
        url: "http://localhost:9090".to_string(),  // Default Prometheus address
        interval: 30,                              // Query every 30 seconds
        queries,
        auth: None,
        timeout: 5,                               // 5 second timeout
    };
    
    // Create and register the component
    let prom_component = PrometheusMetrics::new("prometheus_metrics", prom_config);
    manager.register_component(Box::new(prom_component)).await?;
    
    // Start the dashboard server
    let addr: SocketAddr = "[::1]:8765".parse()?;
    let server_manager = manager.clone();
    
    // Start the server in a background task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = start_server(server_manager, addr).await {
            eprintln!("Server error: {}", e);
        }
    });
    
    // Initialize the manager
    manager.start().await?;
    
    info!("Dashboard server running at http://localhost:8765");
    info!("WebSocket endpoint available at ws://localhost:8765/ws");
    info!("Prometheus metrics component registered as 'prometheus_metrics'");
    info!("Press Ctrl+C to exit");
    
    // Wait for Ctrl+C
    ctrl_c().await?;
    
    // Cancel the server task
    server_handle.abort();
    
    Ok(())
} 