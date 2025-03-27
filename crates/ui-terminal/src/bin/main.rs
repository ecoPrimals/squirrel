use std::sync::Arc;
use std::io;

use clap::{Parser};

use dashboard_core::{
    config::DashboardConfig,
    service::{DashboardService, DefaultDashboardService},
};

use ui_terminal::TuiDashboard;

/// Terminal UI dashboard
#[derive(Parser)]
struct Args {
    /// Data update interval in seconds
    #[arg(short, long, default_value_t = 5)]
    interval: u64,
    
    /// Number of history points to keep
    #[arg(short = 'p', long, default_value_t = 1000)]
    history_points: usize,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Create dashboard configuration with builder pattern
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(args.history_points);
    
    // Create dashboard service
    let dashboard_service_with_rx = DefaultDashboardService::new(config);
    let (dashboard_service, _rx) = &dashboard_service_with_rx;
    
    // Start dashboard service
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // Create and run terminal UI
    let mut tui = TuiDashboard::new_from_default_service(dashboard_service_with_rx);
    tui.run().await
} 