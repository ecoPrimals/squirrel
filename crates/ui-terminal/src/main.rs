use std::io;
use std::time::Duration;
use std::fmt;
use clap::Parser;
use tokio::time;
use chrono::Utc;
use dashboard_core::{
    DefaultDashboardService,
    DashboardConfig,
    DashboardService,
    data::{DashboardData, ProtocolStatus}
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
}

#[tokio::main]
async fn main() -> io::Result<()> {
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
    
    // Run terminal UI with dashboard service
    tokio::spawn(async move {
        if let Err(e) = ui_terminal::run_simplified(dashboard_service, args.demo).await {
            eprintln!("Error running UI: {}", e);
        }
    });

    // Wait for user input to exit
    println!("Press Ctrl+C to exit");
    signal_wait().await?;

    Ok(())
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
    
    // Update protocol data with string directly
    data.protocol.status = "Connected".to_string();
    
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