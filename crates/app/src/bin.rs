//! Binary entry point for the Squirrel application
//!
//! This module provides a demonstration of the core application functionality
//! with the new performance monitoring and context synchronization features.

use squirrel_app::context::{Context, ContextConfig};
use squirrel_app::metrics::perf::{PerfCategory, PerfMonitor};
use squirrel_app::context::sync::LatestWinsResolution;
use squirrel_app::error::Result;
use tracing::{info, error};
use serde_json::json;

/// Run a simple demonstration of the application with the new features
async fn run_demo() -> Result<()> {
    // Initialize performance monitoring
    info!("Initializing performance monitoring");
    let monitor = PerfMonitor::new();
    
    // Create contexts with performance monitoring
    info!("Creating contexts");
    let timing_guard = monitor.time("context_creation", PerfCategory::Context).await;
    let mut context1 = Context::new(ContextConfig::default())?;
    let mut context2 = Context::new(ContextConfig::default())?;
    drop(timing_guard);
    
    // Enable synchronization
    info!("Enabling synchronization");
    context1.enable_sync(LatestWinsResolution::new()).await?;
    context2.enable_sync(LatestWinsResolution::new()).await?;
    
    // Update data with timing
    info!("Updating data in contexts");
    {
        let timing_guard = monitor.time("data_update", PerfCategory::Context).await;
        
        // Update context1
        context1.update_data("user", json!({
            "name": "Alice",
            "role": "Admin",
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        })).await?;
        
        // Get context state and sync with context2
        let state = context1.get_sync_snapshot().await?;
        context2.sync_with(state).await?;
        
        // Update context2 and sync back
        context2.update_data("user.settings.language", json!("en-US")).await?;
        let state = context2.get_sync_snapshot().await?;
        context1.sync_with(state).await?;
        
        drop(timing_guard);
    }
    
    // Generate performance report
    info!("Generating performance report");
    let report = monitor.generate_report().await?;
    
    // Print metrics
    info!("Performance Report:");
    for metric in &report.metrics {
        info!(
            "  {}: count={}, avg={}μs, min={}μs, max={}μs, total={}μs",
            metric.name, metric.count, metric.avg_us(), metric.min_us, metric.max_us, metric.total_us
        );
    }
    
    // Print memory usage
    info!(
        "Memory Usage: current={}KB, peak={}KB, allocated={}KB",
        report.memory.current_bytes / 1024,
        report.memory.peak_bytes / 1024,
        report.memory.allocated_bytes / 1024
    );
    
    info!("Demo completed successfully");
    Ok(())
}

/// Main application entry point
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();
    
    // Run the demo
    info!("Starting Squirrel Application");
    
    if let Err(e) = run_demo().await {
        error!("Demo failed: {}", e);
        return Err(Box::new(e) as Box<dyn std::error::Error>);
    }
    
    info!("Squirrel Application completed successfully");
    Ok(())
} 