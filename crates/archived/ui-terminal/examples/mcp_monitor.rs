use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use clap::Parser;
use chrono::Utc;

use ui_terminal::{
    adapter::McpMetricsProviderTrait,
    mcp_adapter::{McpMetricsConfig, RealMcpMetricsProvider},
    run_simplified,
};
use dashboard_core::{
    DefaultDashboardService,
    DashboardConfig,
    DashboardService,
};

/// Command line arguments for the MCP Monitor example
#[derive(Parser, Debug)]
struct Args {
    /// MCP server address
    #[clap(long, default_value = "127.0.0.1:8778")]
    mcp_server: String,
    
    /// MCP update interval in milliseconds
    #[clap(long, default_value = "1000")]
    mcp_interval: u64,
    
    /// Simulate connection issues
    #[clap(long)]
    simulate_issues: bool,
    
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
    
    /// Time to run in seconds (0 = run until Ctrl+C)
    #[clap(short, long, default_value = "0")]
    time: u64,
}

/// MCP Monitor Example
///
/// This example showcases our MCP integration features. It sets up an MCP metrics provider
/// and displays connection health information.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();

    println!("Starting MCP Monitor Example");
    println!("============================");
    println!("- MCP Server: {}", args.mcp_server);
    println!("- Update interval: {} ms", args.mcp_interval);
    println!("- Simulate issues: {}", args.simulate_issues);
    println!("- Verbose output: {}", args.verbose);
    if args.time > 0 {
        println!("- Runtime: {} seconds", args.time);
    } else {
        println!("- Runtime: Until interrupted");
    }
    println!("");

    // Create MCP metrics configuration
    let mcp_config = McpMetricsConfig {
        update_interval_ms: args.mcp_interval,
        server_address: args.mcp_server.clone(),
        ..Default::default()
    };

    // Create MCP metrics provider
    let provider = Arc::new(RealMcpMetricsProvider::with_config(mcp_config));
    let provider_clone = provider.clone();
    
    // Create dashboard service
    let config = DashboardConfig::default()
        .with_update_interval(2)
        .with_max_history_points(100);
    
    let (dashboard_service, _) = DefaultDashboardService::new(config);
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // Start MCP metrics collector task
    tokio::spawn(async move {
        // Create a ticker for periodic updates
        let mut interval = time::interval(Duration::from_millis(args.mcp_interval));
        let mut iteration = 0;
        
        // If simulating issues, create a failure pattern
        let mut should_fail = false;
        
        loop {
            interval.tick().await;
            iteration += 1;
            
            // Simulate connection issues if requested
            if args.simulate_issues && iteration % 10 == 0 {
                should_fail = !should_fail;
                provider_clone.set_should_fail(should_fail).await;
                println!("{}Toggling connection simulation to: {}", 
                        if args.verbose { "[DEBUG] " } else { "" },
                        if should_fail { "FAILING" } else { "SUCCESS" });
            }
            
            // Retrieve and display connection status
            match provider_clone.get_connection_status().await {
                Ok(status) => {
                    if args.verbose {
                        println!("[{}] Connection status: {:?}", 
                                Utc::now().format("%H:%M:%S%.3f"), 
                                status);
                    }
                },
                Err(e) => {
                    if args.verbose {
                        println!("[{}] Failed to get connection status: {}", 
                                Utc::now().format("%H:%M:%S%.3f"), 
                                e);
                    }
                }
            }
            
            // Retrieve and display connection health
            match provider_clone.get_connection_health().await {
                Ok(health) => {
                    println!("[{}] Connection Health:",
                            Utc::now().format("%H:%M:%S%.3f"));
                    println!("  - Latency: {:.2} ms", health.latency_ms);
                    println!("  - Stability: {:.1}%", health.stability);
                    println!("  - Signal Strength: {:.1}%", health.signal_strength);
                    println!("  - Packet Loss: {:.1}%", health.packet_loss);
                    println!("  - Last Checked: {}", health.last_checked.format("%H:%M:%S%.3f"));
                },
                Err(e) => {
                    println!("[{}] Failed to get connection health: {}",
                            Utc::now().format("%H:%M:%S%.3f"),
                            e);
                }
            }
            
            // If requested, get metrics data
            if args.verbose {
                match provider_clone.get_metrics().await {
                    Ok(metrics) => {
                        println!("[{}] Metrics:",
                                Utc::now().format("%H:%M:%S%.3f"));
                        println!("  - Message Stats:");
                        println!("    - Total Requests: {}", metrics.message_stats.total_requests);
                        println!("    - Total Responses: {}", metrics.message_stats.total_responses);
                        println!("    - Request Rate: {:.2}/s", metrics.message_stats.request_rate);
                        println!("    - Response Rate: {:.2}/s", metrics.message_stats.response_rate);
                        println!("  - Latency Stats:");
                        println!("    - Average Latency: {:.2} ms", metrics.latency_stats.average_latency_ms);
                    },
                    Err(e) => {
                        println!("[{}] Failed to get metrics: {}",
                                Utc::now().format("%H:%M:%S%.3f"),
                                e);
                    }
                }
            }
            
            println!("");
            
            // Periodically display performance metrics
            if iteration % 5 == 0 && args.verbose {
                if let Ok(perf) = provider_clone.get_performance_metrics().await {
                    println!("[{}] Performance Metrics:",
                            Utc::now().format("%H:%M:%S%.3f"));
                    println!("  - Metrics Requests: {}", perf.metrics_requests);
                    println!("  - Cache Hits: {}", perf.cache_hits);
                    println!("  - Cache Misses: {}", perf.cache_misses);
                    println!("  - Avg Request Time: {:.2} ms", perf.average_request_time_ms);
                    println!("");
                }
            }
        }
    });
    
    // Run the terminal UI in a separate task
    tokio::spawn(async move {
        if let Err(e) = run_simplified(dashboard_service, true).await {
            eprintln!("Error running UI: {}", e);
        }
    });
    
    // Wait for specified time or forever
    if args.time > 0 {
        time::sleep(Duration::from_secs(args.time)).await;
    } else {
        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;
    }
    
    println!("MCP Monitor Example completed");
    Ok(())
} 