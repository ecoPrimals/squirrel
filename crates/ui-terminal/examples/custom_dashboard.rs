use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::fs::OpenOptions;
use std::io::prelude::*;

use clap::Parser;
use dashboard_core::{
    config::DashboardConfig,
    data::{DashboardData, Metrics, ProtocolData},
    service::DefaultDashboardService,
};
use tokio::time;
use ui_terminal::TuiDashboard;
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
    
    // Create a dashboard with custom widgets
    let mut dashboard = TuiDashboard::new(dashboard_service.clone());
    
    // Create alert manager
    let alert_manager = dashboard.alert_manager();
    
    // Add some initial alerts if enabled
    if args.alerts {
        alert_manager.add_alert(
            ui_terminal::alert::AlertSeverity::Info,
            "Dashboard example started".to_string(),
            "System".to_string(),
            "Startup".to_string(),
        );
        
        alert_manager.add_alert(
            ui_terminal::alert::AlertSeverity::Info,
            format!("Using {} pattern for CPU simulation", args.pattern),
            "System".to_string(),
            "Configuration".to_string(),
        );
    }
    
    // Create a shared data store for latest dashboard data
    let dashboard_data = Arc::new(std::sync::Mutex::new(DashboardData::default()));
    let dashboard_data_clone = dashboard_data.clone();
    
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
                            ui_terminal::alert::AlertSeverity::Warning,
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
                        history: dashboard_core::data::MetricsHistory::default(),
                    },
                    protocol: ProtocolData {
                        name: "MCP".to_string(),
                        protocol_type: "Custom".to_string(),
                        version: "1.0".to_string(),
                        connected: true,
                        last_connected: Some(Utc::now()),
                        status: protocol_status.to_string(),
                        error: None,
                        retry_count: 0,
                        metrics: {
                            let mut metrics = HashMap::new();
                            metrics.insert("throughput".to_string(), 1024.0 * tick_count as f64);
                            metrics.insert("active_clients".to_string(), 2.0);
                            metrics.insert("connections".to_string(), 4.0);
                            metrics
                        },
                    },
                    alerts: vec![],
                    timestamp: Utc::now(),
                };
                
                // Update the shared data store first (this is our fallback approach)
                {
                    if verbose {
                        println!("Updating shared data store");
                    }
                    let mut dashboard_data = dashboard_data_clone.lock().unwrap();
                    *dashboard_data = data.clone();
                }
                
                // Update the dashboard with retries if it fails
                let mut retry_count = 0;
                let mut last_error_msg = None;
                
                while retry_count <= max_retries {
                    if verbose {
                        println!("Updating dashboard service (attempt {})", retry_count + 1);
                    }
                    
                    match service_clone.update_dashboard_data(data.clone()).await {
                        Ok(_) => {
                            if retry_count > 0 {
                                println!("Successfully updated dashboard data after {} retries", retry_count);
                            } else if verbose {
                                println!("Successfully updated dashboard data");
                            }
                            break;
                        },
                        Err(e) => {
                            // Store error message as String to avoid move issues
                            let error_msg = e.to_string();
                            last_error_msg = Some(error_msg.clone());
                            retry_count += 1;
                            
                            if retry_count > max_retries {
                                eprintln!("Failed to update dashboard data after {} retries: {}", 
                                          max_retries, last_error_msg.as_ref().unwrap());
                                
                                // Add an alert about update failure
                                if alerts_enabled {
                                    alert_mgr.add_alert(
                                        ui_terminal::alert::AlertSeverity::Warning,
                                        format!("Dashboard update failed: {}", last_error_msg.as_ref().unwrap()),
                                        "System".to_string(),
                                        "Update Error".to_string(),
                                    );
                                }
                                break;
                            }
                            
                            // Log the error
                            if verbose {
                                println!("Update attempt {} failed: {}", retry_count, error_msg);
                            }
                            
                            // Add a small delay between retries with exponential backoff
                            let delay = Duration::from_millis(100 * (1 << retry_count.min(10)));
                            time::sleep(delay).await;
                        }
                    }
                }
                
                // Add random alerts if enabled
                if alerts_enabled && tick_count % 7 == 0 && last_alert_time.elapsed() > Duration::from_secs(10) {
                    // Only add a new alert every 10+ seconds to avoid flooding
                    let severity = match tick_count % 3 {
                        0 => ui_terminal::alert::AlertSeverity::Info,
                        1 => ui_terminal::alert::AlertSeverity::Warning,
                        _ => ui_terminal::alert::AlertSeverity::Critical,
                    };
                    
                    let message = match severity {
                        ui_terminal::alert::AlertSeverity::Info => "System information event".to_string(),
                        ui_terminal::alert::AlertSeverity::Warning => format!("CPU usage increased to {:.1}%", cpu_usage),
                        ui_terminal::alert::AlertSeverity::Critical => "Memory pressure detected".to_string(),
                        _ => "Unknown event".to_string(),
                    };
                    
                    alert_mgr.add_alert(
                        severity,
                        message,
                        "Simulation".to_string(),
                        "Test Alert".to_string(),
                    );
                    
                    last_alert_time = std::time::Instant::now();
                }
                
                // Add special CPU alerts if extremely high
                if alerts_enabled && cpu_usage > 90.0 {
                    alert_mgr.add_alert(
                        ui_terminal::alert::AlertSeverity::Critical,
                        format!("CPU usage is critically high: {:.1}%", cpu_usage),
                        "System".to_string(),
                        "Performance".to_string(),
                    );
                }
            }
        });
    }
    
    // Modify the dashboard to use our custom data source when the channel fails
    if let Some(ref mut adapter) = dashboard.monitoring_adapter_mut() {
        adapter.set_data_source(dashboard_data);
    }
    
    // Run the dashboard
    dashboard.run().await?;
    
    Ok(())
} 