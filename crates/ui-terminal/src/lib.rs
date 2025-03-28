//! Terminal UI dashboard for Squirrel monitoring
//! 
//! This is a simplified implementation that will be expanded later.

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;

// Import only the modules we need
pub mod service;
pub mod adapter;
pub mod mock_adapter;
mod state;
pub mod widgets;
mod app;
mod ui;
mod config;
mod util;
mod help;
mod widget_manager;
mod alert;
mod events;

#[cfg(test)]
pub mod tests;

// Re-export the dashboard service
pub use service::DashboardService;
use crate::mock_adapter::MockAdapter;
use crate::adapter::McpMetricsProvider;

/// Run the terminal UI dashboard.
/// This is a simplified implementation that will be expanded later.
pub async fn run(dashboard_service: Arc<dyn DashboardService>, demo_mode: bool) -> Result<(), Box<dyn Error>> {
    println!("Starting terminal dashboard in simplified mode...");
    
    if demo_mode {
        println!("Demo mode activated. Using mock adapter for dashboard metrics.");
        
        // Create and initialize mock adapter
        let mock_adapter = Arc::new(MockAdapter::new());
        
        // Display some basic information
        println!("Getting connection status...");
        match mock_adapter.get_connection_status().await {
            Ok(status) => println!("Connection status: {:?}", status),
            Err(err) => println!("Error getting connection status: {}", err),
        }
        
        println!("Getting dashboard data...");
        match mock_adapter.get_dashboard_data().await {
            Ok(data) => println!("Dashboard data retrieved with timestamp: {}", data.timestamp),
            Err(err) => println!("Error getting dashboard data: {}", err),
        }
        
        // Display performance metrics if available
        println!("Getting performance metrics...");
        match mock_adapter.get_performance_metrics().await {
            Ok(metrics) => println!("Performance metrics retrieved: CPU: {}%, Memory: {}MB", 
                             metrics.cpu_usage.unwrap_or(0.0),
                             metrics.memory_usage.unwrap_or(0.0)),
            Err(err) => println!("Error getting performance metrics: {}", err),
        }
        
        println!("\nPress Enter to exit...");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
    } else {
        println!("Non-demo mode is currently not supported in simplified mode.");
        println!("Please restart with --demo flag or use the web UI instead.");
        std::thread::sleep(Duration::from_secs(5));
    }
    
    Ok(())
} 