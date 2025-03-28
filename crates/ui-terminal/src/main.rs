use std::error::Error;
use std::sync::Arc;
use ui_terminal::run;
use ui_terminal::service::DashboardServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Squirrel Terminal Dashboard (Demo Mode)");
    
    // Initialize dashboard service with default configuration
    let service = Arc::new(DashboardServiceImpl::default());
    
    // Run the dashboard in demo mode
    run(service, true).await
} 