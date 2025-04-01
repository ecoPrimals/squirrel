use squirrel_mcp::error::Result;
use squirrel_mcp::transport::memory::MemoryChannel;
use squirrel_mcp::transport::Transport;
use tokio::time::{sleep, Duration};
use tokio::main;
use std::sync::{Arc, Mutex};

/// A simple connection health monitor
struct ConnectionHealthMonitor {
    transport_name: String,
    transport: Arc<dyn Transport>,
    status_history: Arc<Mutex<Vec<bool>>>,
    check_interval: Duration,
}

impl ConnectionHealthMonitor {
    fn new(transport_name: &str, transport: Arc<dyn Transport>, check_interval: Duration) -> Self {
        Self {
            transport_name: transport_name.to_string(),
            transport,
            status_history: Arc::new(Mutex::new(Vec::new())),
            check_interval,
        }
    }
    
    async fn start_monitoring(&self, duration: Duration) -> Result<()> {
        println!("Starting connection monitoring for {}...", self.transport_name);
        
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < duration {
            // Check connection status
            let is_connected = self.transport.is_connected().await;
            
            // Record status
            self.status_history.lock().unwrap().push(is_connected);
            
            println!("[{}] Connection status: {}", self.transport_name, if is_connected { "CONNECTED" } else { "DISCONNECTED" });
            
            // Wait for next check
            sleep(self.check_interval).await;
        }
        
        Ok(())
    }
    
    fn get_uptime_percentage(&self) -> f64 {
        let history = self.status_history.lock().unwrap();
        if history.is_empty() {
            return 0.0;
        }
        
        let connected_count = history.iter().filter(|&&status| status).count();
        (connected_count as f64 / history.len() as f64) * 100.0
    }
}

/// Example demonstrating connection health monitoring
#[main]
async fn main() -> Result<()> {
    // Create a pair of memory transports as Arc<dyn Transport> that are already connected
    let (client, server) = MemoryChannel::create_connected_pair_arc();
    
    // Create monitors
    let client_monitor = ConnectionHealthMonitor::new(
        "Client",
        client.clone(),
        Duration::from_millis(500)
    );
    
    let server_monitor = ConnectionHealthMonitor::new(
        "Server",
        server.clone(),
        Duration::from_millis(500)
    );
    
    println!("Client and server are already connected.");
    
    // Start monitoring in background
    let client_monitor_clone = client_monitor.clone();
    let server_monitor_clone = server_monitor.clone();
    
    let client_task = tokio::spawn(async move {
        client_monitor.start_monitoring(Duration::from_secs(10)).await.unwrap();
    });
    
    let server_task = tokio::spawn(async move {
        server_monitor.start_monitoring(Duration::from_secs(10)).await.unwrap();
    });
    
    // Simulate some connection issues
    sleep(Duration::from_secs(2)).await;
    println!("\nSimulating client disconnection...");
    client.disconnect().await?;
    
    sleep(Duration::from_secs(2)).await;
    println!("\nSimulating server disconnection...");
    server.disconnect().await?;
    
    // Wait for monitoring to complete
    client_task.await.unwrap();
    server_task.await.unwrap();
    
    // Report uptime
    println!("\nConnection Health Report:");
    println!("Client uptime: {:.1}%", client_monitor_clone.get_uptime_percentage());
    println!("Server uptime: {:.1}%", server_monitor_clone.get_uptime_percentage());
    
    println!("\nConnection health monitor example completed!");
    Ok(())
}

// Allow cloning the monitor
impl Clone for ConnectionHealthMonitor {
    fn clone(&self) -> Self {
        Self {
            transport_name: self.transport_name.clone(),
            transport: Arc::clone(&self.transport),
            status_history: Arc::clone(&self.status_history),
            check_interval: self.check_interval,
        }
    }
} 