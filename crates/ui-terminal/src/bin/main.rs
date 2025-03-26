use std::io;
use std::time::Duration;
use std::error::Error;

use clap::{Parser, ArgAction};
use tokio::runtime::Runtime;
use tokio::signal;

use dashboard_core::{
    config::DashboardConfig,
    service::{DashboardService, DefaultDashboardService},
};
use ui_terminal::TuiDashboard;

/// Command line arguments for the TUI dashboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[clap(short, long, value_name = "FILE")]
    config: Option<String>,
    
    /// Set refresh interval in seconds (overrides config)
    #[clap(short, long, value_name = "SECONDS")]
    refresh: Option<u64>,
    
    /// Enable debug mode
    #[clap(short, long, action=ArgAction::SetTrue)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Create tokio runtime
    let runtime = Runtime::new()?;
    
    // Enter the runtime context
    runtime.block_on(async {
        // Load configuration
        let mut config = match &args.config {
            Some(path) => DashboardConfig::from_file(path).await.unwrap_or_default(),
            None => DashboardConfig::default(),
        };
        
        // Override refresh rate if specified
        if let Some(refresh) = args.refresh {
            config.refresh_interval = Duration::from_secs(refresh);
        }
        
        // Set debug mode if specified
        if args.debug {
            config.debug = true;
        }
        
        // Create dashboard service
        let service = DefaultDashboardService::new(config.clone());
        
        // Create TUI dashboard
        let mut dashboard = TuiDashboard::new(service, config)?;
        
        // Handle Ctrl+C signal
        let ctrl_c = signal::ctrl_c();
        
        tokio::select! {
            _ = dashboard.run() => {
                println!("Dashboard closed");
            }
            _ = ctrl_c => {
                println!("Received Ctrl+C, shutting down");
            }
        }
        
        Ok(())
    })
} 