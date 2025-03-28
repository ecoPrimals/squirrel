use std::error::Error;
use std::sync::Arc;

use clap::Parser;
use dashboard_core::{
    config::DashboardConfig, 
    service::{DashboardService, DefaultDashboardService}
};

/// Command line arguments for the terminal UI
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Enable monitoring mode
    #[clap(long, short = 'm')]
    monitoring: bool,
    
    /// Update interval in seconds
    #[clap(long, short = 'i', default_value = "5")]
    interval: u64,
    
    /// Maximum number of history points to keep
    #[clap(long, short = 'p', default_value = "100")]
    history_points: usize,
    
    /// Run in demo mode (no real monitoring)
    #[clap(long, short = 'd')]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Create dashboard configuration
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(args.history_points);
    
    // Create the dashboard service
    let (dashboard_service, _rx) = DefaultDashboardService::new(config);
    
    // Start the dashboard service
    dashboard_service.start().await?;
    
    // Convert to Arc<dyn DashboardService> for the run_simplified function
    let dashboard_service_arc: Arc<dyn DashboardService> = dashboard_service;
    
    // Run the simplified UI with the dashboard service
    ui_terminal::run_simplified(dashboard_service_arc, args.demo).await?;
    
    Ok(())
} 