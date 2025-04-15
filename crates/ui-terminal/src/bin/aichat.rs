use std::sync::Arc;
use std::time::Duration;
use std::fs::File;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand
};
use dashboard_core::service::MockDashboardService;
use ui_terminal::run_ai_chat_ui;
use log::{LevelFilter, info};
use env_logger::Builder;

/// Main function for the squirrel AI chat interface
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let use_real_api = std::env::args().any(|arg| arg == "--real-api");
    
    // Initialize logging to file instead of stdout
    let log_path = "/tmp/aichat_debug.log";
    let log_file = File::create(log_path)?;
    
    let mut builder = Builder::new();
    builder
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(LevelFilter::Debug)
        .init();
    
    // Log the mode we're running in
    if use_real_api {
        info!("Running with real OpenAI API");
    } else {
        info!("Running with mock API (use --real-api flag to use real API)");
    }
    
    // Set terminal to raw mode and enter alternate screen
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    
    // Create a mock service for now
    // In a real implementation, we would connect to the actual AI service
    let service = Arc::new(MockDashboardService::new());
    
    // Store the API mode in an environment variable that our service can check
    if use_real_api {
        std::env::set_var("USE_REAL_OPENAI_API", "1");
    } else {
        std::env::remove_var("USE_REAL_OPENAI_API");
    }
    
    // Run the chat UI with the service
    // Using 100ms for UI updates and 1s for data updates
    run_ai_chat_ui(
        service,
        Duration::from_millis(100),
        Duration::from_secs(1),
    ).await?;
    
    Ok(())
} 