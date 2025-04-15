use std::sync::Arc;
use std::time::Duration;

use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use crossterm::ExecutableCommand;
use dashboard_core::service::MockDashboardService;
use ui_terminal::run_ui;

/// Main function for the squirrel dashboard
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable logging if needed
    // env_logger::init();
    
    // Set terminal to raw mode and enter alternate screen
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    
    // Create a mock service provider
    let service = Arc::new(MockDashboardService::new());
    
    // Set refresh rates
    let ui_refresh_rate = Duration::from_millis(100);
    let data_update_interval = Duration::from_secs(5);
    
    // Run the UI
    match run_ui(service, ui_refresh_rate, data_update_interval).await {
        Ok(_) => {
            println!("Dashboard exited successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error running dashboard: {}", e);
            Err(e)
        }
    }
} 