//! Dashboard Monitoring Integration Example
//!
//! This example demonstrates how to integrate the monitoring system
//! with the dashboard core system.

use std::sync::Arc;
use std::time::Duration;

use dashboard_core::{
    data::DashboardData,
    monitoring::{MonitoringAdapterConfig, initialize_dashboard_monitoring},
    service::DefaultDashboardService,
    config::DashboardConfig,
};
use squirrel_monitoring::api::{MonitoringAPIProvider, MonitoringAPI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger (simple format)
    env_logger::init();
    
    println!("Starting Dashboard-Monitoring integration example...");
    
    // Create a monitoring API provider
    let monitoring_api = Arc::new(MonitoringAPIProvider::new());
    
    // Create a dashboard service
    let dashboard_config = DashboardConfig::default();
    let (dashboard_service, _rx) = DefaultDashboardService::new(dashboard_config);
    
    // Create and configure the monitoring adapter
    let adapter_config = MonitoringAdapterConfig {
        update_interval_ms: 1000,  // Update every second for the example
        use_websocket: false,      // Don't use WebSocket for this example
        enable_caching: true,
        max_cache_size: 10,
    };
    
    // Initialize the monitoring adapter
    let adapter = initialize_dashboard_monitoring(
        monitoring_api.clone() as Arc<dyn MonitoringAPI>,
        dashboard_service.clone(),
        adapter_config,
    );
    
    println!("Monitoring adapter initialized and started");
    
    // Simulate some monitoring data updates
    for i in 1..=10 {
        let component_data = serde_json::json!({
            "usage": 30.0 + (i as f64 * 2.0),
            "cores": vec![25.0, 35.0, 40.0, 45.0],
            "load_1": 1.2,
            "load_5": 1.0,
            "load_15": 0.8,
            "temperature": 45.5
        });
        
        monitoring_api.update_component_data("cpu", component_data).await?;
        
        let memory_data = serde_json::json!({
            "total": 16_000_000_000u64,
            "used": 4_000_000_000u64 + (i as u64 * 100_000_000),
            "available": 12_000_000_000u64 - (i as u64 * 100_000_000),
            "free": 10_000_000_000u64 - (i as u64 * 100_000_000),
            "swap_used": 500_000_000u64,
            "swap_total": 8_000_000_000u64
        });
        
        monitoring_api.update_component_data("memory", memory_data).await?;
        
        let alerts_data = serde_json::json!({
            "alerts": [
                {
                    "id": format!("alert-{}", i),
                    "name": "High CPU Usage",
                    "message": format!("CPU usage is at {}%", 30 + (i * 2)),
                    "severity": "warning",
                    "source": "system-monitor",
                    "timestamp": chrono::Utc::now().timestamp(),
                    "acknowledged": false
                }
            ]
        });
        
        monitoring_api.update_component_data("alerts", alerts_data).await?;
        
        // Provide some empty data for network and disk to prevent errors
        monitoring_api.update_component_data("network", serde_json::json!({})).await?;
        monitoring_api.update_component_data("disk", serde_json::json!({})).await?;
    }
    
    println!("Simulating dashboard monitoring for 10 seconds...");
    
    // Simulate monitoring for 10 seconds
    for i in 1..=10 {
        // Wait for a second
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Get the latest dashboard data
        match adapter.get_latest_dashboard_data().await {
            Ok(data) => {
                println!("\nDashboard update #{}: {}", i, data.timestamp);
                print_dashboard_data(&data);
            }
            Err(e) => {
                println!("Error getting dashboard data: {}", e);
            }
        }
    }
    
    // Stop the adapter
    adapter.stop()?;
    println!("\nMonitoring adapter stopped");
    
    Ok(())
}

// Helper function to print dashboard data
fn print_dashboard_data(data: &DashboardData) {
    // Print CPU metrics
    println!("CPU Usage: {:.1}%", data.metrics.cpu.usage);
    println!("CPU Load: {:.2} {:.2} {:.2}", 
        data.metrics.cpu.load[0], 
        data.metrics.cpu.load[1], 
        data.metrics.cpu.load[2]
    );
    
    // Print memory metrics
    println!("Memory: {:.1}% used ({} MB / {} MB)", 
        100.0 * data.metrics.memory.used as f64 / data.metrics.memory.total as f64,
        data.metrics.memory.used / 1024 / 1024,
        data.metrics.memory.total / 1024 / 1024
    );
    
    // Print alerts
    if !data.alerts.is_empty() {
        println!("Active Alerts: {}", data.alerts.len());
        for (i, alert) in data.alerts.iter().enumerate().take(3) {
            println!("  {}: [{}] {} - {}", 
                i + 1, 
                format!("{:?}", alert.severity), 
                alert.title, 
                alert.message
            );
        }
        
        if data.alerts.len() > 3 {
            println!("  ...and {} more", data.alerts.len() - 3);
        }
    } else {
        println!("No active alerts");
    }
} 