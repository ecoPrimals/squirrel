use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::fs::OpenOptions;
use std::io::prelude::*;

use clap::Parser;
use dashboard_core::{
    config::DashboardConfig,
    data::{DashboardData, Metrics, ProtocolData, MetricsHistory},
    service::DefaultDashboardService,
    DashboardService,
};
use tokio::time;
use ui_terminal::{
    app::App,
    alert, 
    mcp_adapter::{McpMetricsConfig, RealMcpMetricsProvider},
    adapter::McpMetricsProviderTrait
};
use std::collections::HashMap;
use chrono::Utc;
use rand::prelude::*;

// Helper function to log to a file
fn log_to_file(message: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("dashboard_debug.log")?;
        
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    writeln!(file, "[{}] {}", timestamp, message)?;
    Ok(())
}

/// Custom Dashboard Demonstration
#[derive(Parser)]
struct Args {
    /// Update interval in seconds
    #[clap(short, long, default_value_t = 3)]
    interval: u64,
    
    /// Enable dynamic alerts simulation
    #[clap(short, long)]
    alerts: bool,
    
    /// Show MCP protocol simulation
    #[clap(short, long)]
    mcp: bool,
    
    /// Show CPU simulation pattern (sine, spike, random)
    #[clap(short = 'p', long, default_value = "sine")]
    pattern: String,
    
    /// Number of update retries before giving up
    #[clap(short = 'r', long, default_value_t = 3)]
    retries: u32,
    
    /// Enable verbose logging
    #[clap(short = 'v', long)]
    verbose: bool,
    
    /// MCP server address (when mcp is enabled)
    #[clap(long, default_value = "127.0.0.1:8778")]
    mcp_server: String,
    
    /// MCP update interval in milliseconds
    #[clap(long, default_value = "1000")]
    mcp_interval: u64,
}

/// Entry point for the custom dashboard example
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    println!("Starting Custom Dashboard Example");
    println!("- Update interval: {} seconds", args.interval);
    println!("- Alerts simulation: {}", if args.alerts { "Enabled" } else { "Disabled" });
    println!("- MCP protocol: {}", if args.mcp { "Enabled" } else { "Disabled" });
    println!("- CPU pattern: {}", args.pattern);
    println!("- Retries: {}", args.retries);
    println!("- Verbose: {}", args.verbose);
    println!("- Press 'q' to quit, '?' for help");
    
    // Create dashboard configuration
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(100);
    
    // Create dashboard service
    let (dashboard_service, _) = DefaultDashboardService::new(config);
    
    // Start the dashboard service
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // Create app with proper configuration
    let mut app = App::new();
    
    // Create alert manager
    let alert_manager = Arc::new(alert::AlertManager::new());
    
    // Add some initial alerts if enabled
    if args.alerts {
        alert_manager.add_alert(
            alert::AlertSeverity::Info,
            "Dashboard example started".to_string(),
            "System".to_string(),
            "Startup".to_string(),
        );
        
        alert_manager.add_alert(
            alert::AlertSeverity::Info,
            format!("Using {} pattern for CPU simulation", args.pattern),
            "System".to_string(),
            "Configuration".to_string(),
        );
    }
    
    // Create a shared data store for latest dashboard data
    let dashboard_data = Arc::new(std::sync::Mutex::new(DashboardData::default()));
    
    // Initialize MCP metrics provider if enabled
    if args.mcp {
        println!("Initializing MCP metrics provider with server {}", args.mcp_server);
        
        // Create MCP metrics configuration
        let mcp_config = McpMetricsConfig {
            update_interval_ms: args.mcp_interval,
            server_address: args.mcp_server.clone(),
            ..Default::default()
        };
        
        // Initialize the MCP metrics provider in the app
        app.init_mcp_metrics_provider(Some(mcp_config.clone()));
        
        // Get reference to the provider for background task
        if let Some(provider) = &app.mcp_metrics_provider {
            let provider_clone = provider.clone();
            
            // Start a task to update MCP metrics periodically
            tokio::spawn(async move {
                let mut interval_timer = time::interval(Duration::from_millis(args.mcp_interval));
                
                loop {
                    interval_timer.tick().await;
                    
                    // Use the trait methods directly to update metrics
                    if let Ok(_metrics) = McpMetricsProviderTrait::get_metrics(&*provider_clone).await {
                        // We could process the metrics here if needed
                        // For now, just logging if verbose is enabled
                        if args.verbose {
                            println!("Updated MCP metrics");
                        }
                    }
                }
            });
        }
    }
    
    // Start a background task to update metrics periodically
    if args.interval > 0 {
        let service_clone = dashboard_service.clone();
        let alert_mgr = alert_manager.clone();
        let alerts_enabled = args.alerts;
        let mcp_enabled = args.mcp;
        let pattern = args.pattern.clone();
        let max_retries = args.retries;
        let verbose = args.verbose;
        
        tokio::spawn(async move {
            // Simulation state
            let mut tick_count = 0;
            let mut cpu_trend: f64 = 50.0;
            let mut protocol_status = "Connected";
            let mut last_alert_time = std::time::Instant::now();
            
            // Create interval for periodic updates
            let mut interval = time::interval(Duration::from_secs(args.interval));
            
            loop {
                interval.tick().await;
                tick_count += 1;
                
                if verbose {
                    println!("Tick {}: Generating simulated data", tick_count);
                }
                
                // Simulate CPU metrics based on pattern
                let cpu_usage = match pattern.as_str() {
                    "sine" => {
                        // Sine wave pattern between 30% and 70%
                        let angle = (tick_count as f64) * 0.1;
                        50.0 + 20.0 * angle.sin()
                    },
                    "spike" => {
                        // Occasional spikes
                        if tick_count % 10 == 0 {
                            85.0 + (tick_count as f64 % 15.0)
                        } else {
                            30.0 + (tick_count as f64 % 20.0)
                        }
                    },
                    "random" => {
                        // Random walk with constraints - create a new RNG each time
                        let mut rng = rand::thread_rng();
                        let change = rng.gen_range(-5.0..5.0);
                        cpu_trend += change;
                        cpu_trend = cpu_trend.max(10.0).min(95.0);
                        cpu_trend
                    },
                    _ => 50.0, // Default
                };
                
                // Simulate memory metrics
                let memory_used = 4096 + (tick_count as u64 % 4096);
                let memory_total = 16384;
                
                // Simulate disk usage metrics
                let mut disk_usage = HashMap::new();
                disk_usage.insert(
                    "/".to_string(),
                    dashboard_core::data::DiskUsage {
                        mount_point: "/".to_string(),
                        total: 1024 * 1024 * 100, // 100 GB
                        used: 1024 * 1024 * (30 + tick_count % 50), // 30-80 GB
                        free: 1024 * 1024 * (70 - tick_count % 50), // 20-70 GB
                        used_percentage: 30.0 + (tick_count % 50) as f64,
                    },
                );
                
                // Simulate protocol status changes if MCP enabled
                if mcp_enabled && tick_count % 20 == 0 {
                    // Toggle status for demonstration
                    protocol_status = match protocol_status {
                        "Connected" => "Degraded",
                        "Degraded" => "Connected",
                        _ => "Connected",
                    };
                    
                    if alerts_enabled {
                        alert_mgr.add_alert(
                            alert::AlertSeverity::Warning,
                            format!("MCP Protocol status changed to {}", protocol_status),
                            "Protocol".to_string(),
                            "Status Change".to_string(),
                        );
                    }
                }
                
                // Create CPU cores vector
                let cores = vec![
                    cpu_usage - 5.0,
                    cpu_usage,
                    cpu_usage + 5.0,
                    cpu_usage - 2.0,
                    cpu_usage + 2.0,
                    cpu_usage - 1.0,
                    cpu_usage + 1.0,
                    cpu_usage,
                ];
                
                // Create dashboard data
                let data = DashboardData {
                    metrics: Metrics {
                        cpu: dashboard_core::data::CpuMetrics {
                            usage: cpu_usage,
                            cores: cores,
                            temperature: Some(45.0 + (tick_count % 10) as f64),
                            load: [cpu_usage/100.0, cpu_usage/100.0 - 0.1, cpu_usage/100.0 - 0.2],
                        },
                        memory: dashboard_core::data::MemoryMetrics {
                            total: memory_total * 1024 * 1024, // MB to bytes
                            used: memory_used * 1024 * 1024,   // MB to bytes
                            available: (memory_total - memory_used) * 1024 * 1024,
                            free: (memory_total - memory_used) * 1024 * 1024,
                            swap_used: 1024 * 1024 * 1024,
                            swap_total: 8192 * 1024 * 1024,
                        },
                        disk: dashboard_core::data::DiskMetrics {
                            usage: disk_usage,
                            total_reads: tick_count * 100,
                            total_writes: tick_count * 50,
                            read_bytes: tick_count * 1024 * 100,
                            written_bytes: tick_count * 1024 * 50,
                        },
                        network: dashboard_core::data::NetworkMetrics {
                            interfaces: vec![
                                dashboard_core::data::NetworkInterface {
                                    name: "eth0".to_string(),
                                    is_up: true,
                                    rx_bytes: 1024 * 1024 * tick_count,
                                    tx_bytes: 1024 * 512 * tick_count,
                                    rx_packets: tick_count * 100,
                                    tx_packets: tick_count * 80,
                                    rx_errors: 0,
                                    tx_errors: 0,
                                }
                            ],
                            total_rx_bytes: 1024 * 1024 * tick_count,
                            total_tx_bytes: 1024 * 512 * tick_count,
                            total_rx_packets: tick_count * 100,
                            total_tx_packets: tick_count * 80,
                        },
                        history: MetricsHistory::default(),
                    },
                    protocol: ProtocolData {
                        name: "MCP".to_string(),
                        protocol_type: "Custom".to_string(),
                        version: "1.0".to_string(),
                        connected: protocol_status == "Connected",
                        last_connected: Some(Utc::now()),
                        status: protocol_status.to_string(),
                        error: None,
                        retry_count: 0,
                        metrics: {
                            let mut m = HashMap::new();
                            m.insert("request_count".to_string(), tick_count as f64 * 10.0);
                            m.insert("response_count".to_string(), tick_count as f64 * 9.0);
                            m.insert("request_rate".to_string(), 10.0);
                            m.insert("response_rate".to_string(), 9.0);
                            m.insert("error_rate".to_string(), if tick_count % 10 == 0 { 1.0 } else { 0.0 });
                            m
                        },
                    },
                    alerts: alert_mgr.to_dashboard_alerts(),
                    timestamp: Utc::now(),
                };
                
                // Update dashboard data
                *dashboard_data.lock().unwrap() = data.clone();
                
                // Update the dashboard service
                let data_clone = data.clone();
                match service_clone.update_dashboard_data(data_clone).await {
                    Ok(_) => {
                        if verbose {
                            println!("Dashboard data updated successfully");
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to update dashboard data: {}", e);
                        
                        // Try retries
                        let mut retry_count = 0;
                        while retry_count < max_retries {
                            retry_count += 1;
                            println!("Retrying update ({}/{})", retry_count, max_retries);
                            
                            // Wait a bit before retrying
                            time::sleep(Duration::from_millis(500)).await;
                            
                            // Try again with a fresh clone
                            if service_clone.update_dashboard_data(data.clone()).await.is_ok() {
                                println!("Update succeeded on retry {}", retry_count);
                                break;
                            }
                        }
                    }
                }
                
                // Add random alerts if enabled and it's been at least 5 seconds since the last alert
                if alerts_enabled && last_alert_time.elapsed() > Duration::from_secs(5) {
                    let mut rng = rand::thread_rng();
                    
                    if rng.gen_range(0.0..1.0) < 0.3 {
                        // 30% chance of a new alert
                        let severity = match rng.gen_range(0..4) {
                            0 => alert::AlertSeverity::Critical,
                            1 => alert::AlertSeverity::Warning,
                            2 => alert::AlertSeverity::Info,
                            _ => alert::AlertSeverity::Error,
                        };
                        
                        let message = match severity {
                            alert::AlertSeverity::Critical => format!("Critical alert at tick {}", tick_count),
                            alert::AlertSeverity::Warning => format!("Warning alert at tick {}", tick_count),
                            alert::AlertSeverity::Info => format!("Info alert at tick {}", tick_count),
                            alert::AlertSeverity::Error => format!("Error alert at tick {}", tick_count),
                        };
                        
                        alert_mgr.add_alert(
                            severity,
                            message,
                            "System".to_string(),
                            "Simulation".to_string(),
                        );
                        
                        last_alert_time = std::time::Instant::now();
                    }
                }
            }
        });
    }
    
    // Run the terminal UI
    ui_terminal::run_simplified(dashboard_service, true).await?;
    
    println!("Dashboard example completed");
    Ok(())
} 