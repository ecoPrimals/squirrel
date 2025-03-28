//! Terminal UI implementation for the Squirrel dashboard
//! 
//! This crate provides a terminal user interface for monitoring system resources, 
//! network activity, and protocol metrics.

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::io;
use dashboard_core::service::DashboardService;
use crate::ui::Ui;

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
pub use service::TerminalDashboardService;
pub use adapter::{McpMetricsProvider, MonitoringToDashboardAdapter, DashboardMonitor};
use crate::mock_adapter::MockAdapter;

pub mod monitoring {
    //! Monitoring-related functionality
    //! 
    //! This module contains types and functions for interacting with the monitoring system.
}

/// Run the terminal UI application
/// 
/// This function initializes the terminal UI, sets up the dashboard service, and runs
/// the main application loop until the user exits.
/// 
/// # Arguments
/// 
/// * `dashboard_service` - The dashboard service to use for retrieving metrics
/// 
/// # Returns
/// 
/// Returns a Result indicating success or failure
pub async fn run<S>(dashboard_service: S) -> io::Result<()>
where
    S: DashboardService + 'static
{
    // Initialize the UI
    let mut ui = Ui::new(dashboard_service)?;
    
    // Run the UI
    ui.run().await?;
    
    Ok(())
}

/// Run the terminal UI dashboard.
/// This is a simplified implementation that will be expanded later.
pub async fn run_simplified(_dashboard_service: Arc<dyn DashboardService>, demo_mode: bool) -> Result<(), Box<dyn Error>> 
{
    println!("Starting terminal dashboard in simplified mode...");
    
    if demo_mode {
        println!("Demo mode activated. Using mock adapter for dashboard metrics.");
        
        // Create and initialize mock adapter
        let mock_adapter: Arc<dyn MonitoringToDashboardAdapter> = Arc::new(MockAdapter::new());
        
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