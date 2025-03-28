use std::error::Error;
use std::io;

use clap::Parser;
use crossterm;
use dashboard_core::config::DashboardConfig;
use dashboard_core::service::DefaultDashboardService;
use ui_terminal::TuiDashboard;

#[derive(Parser)]
#[clap(
    name = "MCP Dashboard",
    about = "A terminal dashboard for MCP metrics",
    disable_help_flag = true
)]
struct Args {
    /// Update interval in milliseconds
    #[clap(short, long)]
    interval: Option<u64>,
    
    /// Maximum history points to keep
    #[clap(short = 'p', long)]
    history_points: Option<usize>,
    
    /// Display help in the UI
    #[clap(short = 'h', long = "show-help")]
    show_help: bool,
}

/// Dashboard application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create dashboard configuration
    let mut config = DashboardConfig::default();
    
    // Override config with command line args
    if let Some(interval) = args.interval {
        config.update_interval = interval;
    }
    
    if let Some(points) = args.history_points {
        config.max_history_points = points;
    }
    
    // Create dashboard service
    let (dashboard_service, _) = DefaultDashboardService::new(config);
    
    // Setup the dashboard UI
    let mut dashboard = TuiDashboard::new(dashboard_service);
    
    // Set show help flag (if needed)
    dashboard.set_show_help(args.show_help);
    
    // Run the dashboard
    dashboard.run().await?;
    
    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    
    Ok(())
} 