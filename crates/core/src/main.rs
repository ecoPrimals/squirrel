//! Main entry point for the Squirrel application

#![allow(unused_crate_dependencies)]

use squirrel_core::{app::App, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize app
    let app_config = squirrel_core::app::AppConfig::default();
    let app = App::from_config(app_config).await?;
    
    println!("Squirrel initialized successfully!");
    
    // Start the app
    app.start().await?;
    
    Ok(())
} 