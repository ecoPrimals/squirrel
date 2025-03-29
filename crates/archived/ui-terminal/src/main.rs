use std::io;
use std::time::Duration;
use std::fmt;
use std::sync::Arc;
use clap::Parser;
use tokio::time;
use chrono::Utc;
use dashboard_core::{
    DefaultDashboardService,
    DashboardConfig,
    DashboardService,
    data::{DashboardData, ProtocolStatus}
};

#[cfg(feature = "mcp-integration")]
use squirrel_mcp;

use ui_terminal::{
    mcp_adapter::{RealMcpMetricsProvider, McpMetricsConfig, create_mcp_metrics_provider},
    app::App,
    adapter::McpMetricsProviderTrait
};

// Wrapper type to avoid orphan rule violations
pub struct LocalProtocolStatus(ProtocolStatus);

impl From<ProtocolStatus> for LocalProtocolStatus {
    fn from(status: ProtocolStatus) -> Self {
        LocalProtocolStatus(status)
    }
}

// Implement Display for our local wrapper
impl fmt::Display for LocalProtocolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ProtocolStatus::Connected => write!(f, "Connected"),
            ProtocolStatus::Disconnected => write!(f, "Disconnected"),
            ProtocolStatus::Connecting => write!(f, "Connecting"),
            ProtocolStatus::Error => write!(f, "Error"),
            ProtocolStatus::Running => write!(f, "Running"),
            ProtocolStatus::Degraded => write!(f, "Degraded"),
            ProtocolStatus::Stopped => write!(f, "Stopped"),
            ProtocolStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Command line arguments for the terminal UI
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Enable monitoring mode (connect to real system metrics)
    #[clap(short, long)]
    monitoring: bool,
    
    /// Update interval in seconds
    #[clap(short, long, default_value = "5")]
    interval: u64,
    
    /// Maximum number of history points to keep
    #[clap(short, long, default_value = "100")]
    history_points: usize,
    
    /// Demo mode (use fake data)
    #[clap(short, long)]
    demo: bool,
    
    /// Enable MCP integration (connect to MCP protocol)
    #[clap(long)]
    mcp: bool,
    
    /// MCP server address
    #[clap(long, default_value = "127.0.0.1:8778")]
    mcp_server: String,
    
    /// MCP update interval in milliseconds
    #[clap(long, default_value = "1000")]
    mcp_interval: u64,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create dashboard configuration with builder pattern
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(args.history_points);
    
    // Create dashboard service with explicit type
    let (dashboard_service, _rx) = DefaultDashboardService::new(config);
    
    // Start dashboard service
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // If monitoring mode is enabled, use real system metrics
    if args.monitoring {
        // Create monitoring adapter
        // Note: We skipped implementing the monitoring functionality
        // because monitoring crate integration is not available yet
        eprintln!("Monitoring mode is not implemented yet. Using demo mode instead.");
        
        // Fall back to demo mode
        start_demo_mode(&dashboard_service, args.interval, args.history_points).await;
    } else if args.demo {
        // Start a task to periodically update with demo data
        start_demo_mode(&dashboard_service, args.interval, args.history_points).await;
    }
    
    // Initialize MCP metrics provider if enabled
    let mcp_provider = if args.mcp {
        println!("Initializing MCP metrics provider with server {}", args.mcp_server);
        
        // Create MCP metrics configuration
        let mcp_config = McpMetricsConfig {
            update_interval_ms: args.mcp_interval,
            server_address: args.mcp_server.clone(),
            ..Default::default()
        };
        
        // Create MCP metrics provider
        let provider = create_mcp_metrics_provider();
        
        // Try to initialize MCP client
        let mcp_client = match create_mcp_client(&args.mcp_server).await {
            Ok(client) => {
                println!("Successfully connected to MCP server");
                Some(client)
            },
            Err(e) => {
                eprintln!("Failed to connect to MCP server: {}. MCP metrics will be unavailable.", e);
                None
            }
        };
        
        // Set MCP client if available
        if let Some(client) = mcp_client {
            // Create a new provider with config
            let _provider = Arc::new(RealMcpMetricsProvider::with_config(mcp_config.clone()));
            
            // Since we can't mutate the Arc directly, we'll create a new provider
            let mut provider_clone = RealMcpMetricsProvider::with_config(mcp_config);
            
            // Set the client on the clone
            provider_clone.set_client(client);
            
            // Replace our provider with the new one that has the client set
            Some(Arc::new(provider_clone))
        } else {
            Some(provider)
        }
    } else {
        None
    };
    
    // Create app with MCP provider
    let mut app = App::new();
    
    // Set MCP provider if available
    if let Some(provider) = mcp_provider {
        app.mcp_metrics_provider = Some(provider.clone());
        
        // Start a task to update MCP metrics if provider is available
        // Using clone of the provider directly to avoid Send bound issues with App
        let provider_clone = provider.clone();
        let interval_ms = args.mcp_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = time::interval(Duration::from_millis(interval_ms));
            
            loop {
                interval_timer.tick().await;
                
                // Use the trait methods directly
                if let Ok(_metrics) = McpMetricsProviderTrait::get_metrics(&*provider_clone).await {
                    // Update performance metrics
                    if let Ok(mut perf) = McpMetricsProviderTrait::get_performance_metrics(&*provider_clone).await {
                        perf.metrics_requests += 1;
                        // No need to update the metrics further in this simplified example
                    }
                }
            }
        });
    }
    
    // Run terminal UI with dashboard service
    tokio::spawn(async move {
        // In a real implementation, we'd run the full UI here
        if let Err(e) = ui_terminal::run_simplified(dashboard_service, args.demo).await {
            eprintln!("Error running UI: {}", e);
        }
    });

    // Wait for user input to exit
    println!("Press Ctrl+C to exit");
    signal_wait().await?;

    Ok(())
}

/// Create an MCP client for the specified server address
#[cfg(feature = "mcp-integration")]
async fn create_mcp_client(server_address: &str) -> Result<Arc<std::sync::Mutex<Box<dyn std::fmt::Debug + Send + Sync>>>, String> {
    // Try to create an MCP client
    match squirrel_mcp::client::connect(server_address).await {
        Ok(client) => Ok(Arc::new(std::sync::Mutex::new(Box::new(client) as Box<dyn std::fmt::Debug + Send + Sync>))),
        Err(e) => Err(format!("Failed to connect to MCP server: {}", e)),
    }
}

#[cfg(not(feature = "mcp-integration"))]
async fn create_mcp_client(_server_address: &str) -> Result<Arc<std::sync::Mutex<Box<dyn std::fmt::Debug + Send + Sync>>>, String> {
    Err("MCP integration is not enabled".to_string())
}

/// Starts demo mode with simulated data
async fn start_demo_mode(dashboard_service: &DefaultDashboardService, interval: u64, _history_points: usize) {
    let ds = dashboard_service.clone();
    tokio::spawn(async move {
        let mut interval_timer = time::interval(Duration::from_secs(interval));
        
        loop {
            interval_timer.tick().await;
            
            // Create demo data
            let data = create_demo_data();
            
            // Update the dashboard with demo data
            if let Err(e) = ds.update_dashboard_data(data).await {
                eprintln!("Failed to update dashboard data: {}", e);
            }
        }
    });
}

/// Create demo dashboard data
fn create_demo_data() -> DashboardData {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Create simulated data with random values
    let mut data = DashboardData::default();
    
    // Update system metrics
    data.metrics.cpu.usage = rng.gen_range(0.0..100.0);
    data.metrics.memory.total = 16 * 1024 * 1024 * 1024; // 16 GB
    data.metrics.memory.used = rng.gen_range(2 * 1024 * 1024 * 1024..14 * 1024 * 1024 * 1024);
    data.metrics.memory.available = data.metrics.memory.total - data.metrics.memory.used;
    data.metrics.memory.free = data.metrics.memory.available;
    
    // Update CPU cores
    let core_count = 8;
    data.metrics.cpu.cores.clear();
    for _ in 0..core_count {
        data.metrics.cpu.cores.push(rng.gen_range(0.0..100.0));
    }
    
    // Update network metrics
    data.metrics.network.total_rx_bytes = rng.gen_range(1_000_000..100_000_000);
    data.metrics.network.total_tx_bytes = rng.gen_range(1_000_000..100_000_000);
    data.metrics.network.total_rx_packets = rng.gen_range(1_000..10_000);
    data.metrics.network.total_tx_packets = rng.gen_range(1_000..10_000);
    
    // Simulated interfaces
    data.metrics.network.interfaces.clear();
    for i in 1..3 {
        data.metrics.network.interfaces.push(dashboard_core::data::NetworkInterface {
            name: format!("eth{}", i),
            rx_bytes: rng.gen_range(1_000_000..50_000_000),
            tx_bytes: rng.gen_range(1_000_000..50_000_000),
            rx_packets: rng.gen_range(1_000..5_000),
            tx_packets: rng.gen_range(1_000..5_000),
            is_up: true,
            rx_errors: 0,
            tx_errors: 0,
        });
    }
    
    // Update disk metrics
    let disk_total = 512 * 1024 * 1024 * 1024; // 512 GB
    let disk_used = rng.gen_range(100 * 1024 * 1024 * 1024..400 * 1024 * 1024 * 1024);
    let disk_free = disk_total - disk_used;
    let used_percentage = (disk_used as f64 / disk_total as f64) * 100.0;
    
    data.metrics.disk.usage.insert(
        "/".to_string(),
        dashboard_core::data::DiskUsage {
            total: disk_total,
            used: disk_used,
            free: disk_free,
            used_percentage,
            mount_point: "/".to_string(),
        }
    );
    
    // Add MCP protocol metrics
    let mut protocol_metrics = std::collections::HashMap::new();
    protocol_metrics.insert("request_count".to_string(), rng.gen_range(100.0..1000.0));
    protocol_metrics.insert("response_count".to_string(), rng.gen_range(90.0..950.0));
    protocol_metrics.insert("request_rate".to_string(), rng.gen_range(1.0..10.0));
    protocol_metrics.insert("response_rate".to_string(), rng.gen_range(1.0..10.0));
    protocol_metrics.insert("transaction_count".to_string(), rng.gen_range(50.0..500.0));
    protocol_metrics.insert("transaction_rate".to_string(), rng.gen_range(0.5..5.0));
    protocol_metrics.insert("success_rate".to_string(), rng.gen_range(90.0..100.0));
    protocol_metrics.insert("error_count".to_string(), rng.gen_range(0.0..10.0));
    protocol_metrics.insert("error_rate".to_string(), rng.gen_range(0.0..5.0));
    protocol_metrics.insert("average_latency".to_string(), rng.gen_range(10.0..100.0));
    
    // Update protocol data
    data.protocol.status = "Connected".to_string();
    data.protocol.connected = true;
    data.protocol.metrics = protocol_metrics;
    data.protocol.name = "MCP".to_string();
    data.protocol.protocol_type = "TCP".to_string();
    data.protocol.retry_count = 0;
    data.protocol.version = "1.0".to_string();
    data.protocol.last_connected = Some(Utc::now());
    
    // Update timestamp
    data.timestamp = Utc::now();
    
    data
}

/// Wait for a termination signal
async fn signal_wait() -> io::Result<()> {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for ctrl+c: {}", e);
    }
    Ok(())
} 