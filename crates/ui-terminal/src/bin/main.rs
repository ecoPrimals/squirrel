use std::io;
use std::time::Duration;

use clap::{Parser};
use tokio::time;

use dashboard_core::{
    config::DashboardConfig,
    service::{DashboardService, DefaultDashboardService},
};

use ui_terminal::adapter::MonitoringToDashboardAdapter;
use ui_terminal::TuiDashboard;

/// Terminal UI dashboard for Squirrel system with MCP protocol integration
#[derive(Parser)]
struct Args {
    /// Data update interval in seconds
    #[arg(short, long, default_value_t = 5)]
    interval: u64,
    
    /// Number of history points to keep
    #[arg(short = 'p', long, default_value_t = 1000)]
    history_points: usize,
    
    /// Use integrated monitoring (no arguments needed)
    #[arg(short, long)]
    monitoring: bool,
    
    /// Use MCP integration with mock client
    #[arg(short, long)]
    mcp: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Check if MCP integration mode is enabled
    if args.mcp {
        println!("Starting Terminal UI Dashboard with MCP Protocol integration");
        println!("- Update interval: {} seconds", args.interval);
        println!("- Max history points: {}", args.history_points);
        println!("- MCP metrics: Enabled (using mock client)");
        println!("- Press 'q' to quit, '?' for help");
        
        // Create TUI Dashboard with MCP integration
        let mut tui = TuiDashboard::new_with_mcp();
        return tui.run().await;
    }
    
    // Check if integrated monitoring mode is enabled
    if args.monitoring {
        println!("Starting Terminal UI Dashboard with integrated monitoring");
        println!("- Update interval: {} seconds", args.interval);
        println!("- Max history points: {}", args.history_points);
        println!("- System monitoring: Enabled");
        println!("- Press 'q' to quit, '?' for help");
        
        // Create TUI Dashboard with integrated monitoring
        let mut tui = TuiDashboard::new_with_monitoring();
        return tui.run().await;
    }
    
    // Standard mode with custom configuration
    println!("Starting Terminal UI Dashboard");
    println!("- Update interval: {} seconds", args.interval);
    println!("- Max history points: {}", args.history_points);
    println!("- Press 'q' to quit, '?' for help");
    
    // Create dashboard configuration with builder pattern
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(args.history_points);
    
    // Create dashboard service
    let dashboard_service_with_rx = DefaultDashboardService::new(config);
    let (dashboard_service, _rx) = &dashboard_service_with_rx;
    
    // Create monitoring adapter
    let mut adapter = MonitoringToDashboardAdapter::new_with_defaults();
    
    // Start a task to periodically collect metrics and update the dashboard
    let dashboard_service_clone = dashboard_service.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(args.interval));
        
        loop {
            interval.tick().await;
            
            // Collect metrics from system and generate protocol metrics
            let data = adapter.collect_dashboard_data();
            
            // Update the dashboard with new data
            if let Err(e) = dashboard_service_clone.update_dashboard_data(data).await {
                eprintln!("Failed to update dashboard data: {}", e);
            }
        }
    });
    
    // Start dashboard service
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // Create and run terminal UI
    let mut tui = TuiDashboard::new_from_default_service(dashboard_service_with_rx);
    tui.run().await
} 