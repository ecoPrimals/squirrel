//! Network bandwidth monitoring for system monitoring
//! 
//! Tracks network usage including:
//! - Bandwidth usage per team
//! - Network I/O statistics
//! - Connection tracking

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, NetworkExt};
use std::time::{Duration, Instant};

/// Network bandwidth statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes transmitted
    pub bytes_transmitted: u64,
    /// Receive rate in bytes per second
    pub receive_rate: f64,
    /// Transmit rate in bytes per second
    pub transmit_rate: f64,
    /// Total packets received
    pub packets_received: u64,
    /// Total packets transmitted
    pub packets_transmitted: u64,
    /// Active connections count
    pub active_connections: u32,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            bytes_received: 0,
            bytes_transmitted: 0,
            receive_rate: 0.0,
            transmit_rate: 0.0,
            packets_received: 0,
            packets_transmitted: 0,
            active_connections: 0,
        }
    }
}

/// Network bandwidth monitor
#[derive(Debug)]
pub struct NetworkMonitor {
    /// System information collector
    sys: Arc<RwLock<System>>,
    /// Previous network statistics
    prev_stats: Arc<RwLock<HashMap<String, NetworkStats>>>,
    /// Last update timestamp
    last_update: Arc<RwLock<Instant>>,
}

impl NetworkMonitor {
    /// Create a new network monitor
    pub fn new() -> Self {
        let sys = System::new();
        
        Self {
            sys: Arc::new(RwLock::new(sys)),
            prev_stats: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Update network statistics
    pub async fn update_stats(&self) -> HashMap<String, NetworkStats> {
        let mut sys = self.sys.write().await;
        
        // Create a new Networks instance with refreshed data
        let networks = sysinfo::Networks::new_with_refreshed_list();

        let mut current_stats = HashMap::new();
        let mut prev_stats = self.prev_stats.write().await;
        let mut last_update = self.last_update.write().await;
        let elapsed = last_update.elapsed().as_secs_f64();
        *last_update = Instant::now();

        // Collect network interface statistics
        for (interface_name, data) in networks.iter() {
            let prev = prev_stats
                .get(interface_name)
                .cloned()
                .unwrap_or_default();

            let stats = NetworkStats {
                bytes_received: data.total_received(),
                bytes_transmitted: data.total_transmitted(),
                receive_rate: (data.total_received() - prev.bytes_received) as f64 / elapsed,
                transmit_rate: (data.total_transmitted() - prev.bytes_transmitted) as f64 / elapsed,
                packets_received: data.total_packets_received(),
                packets_transmitted: data.total_packets_transmitted(),
                active_connections: self.count_active_connections(&sys, interface_name).await,
            };

            current_stats.insert(interface_name.to_string(), stats.clone());
            prev_stats.insert(interface_name.to_string(), stats);
        }

        current_stats
    }

    /// Count active network connections for an interface
    async fn count_active_connections(&self, sys: &System, interface: &str) -> u32 {
        // Note: sysinfo doesn't provide connection information
        // In a production system, we would use platform-specific APIs
        // or tools like netstat for accurate connection counting
        0
    }

    /// Get network statistics for all interfaces
    pub async fn get_stats(&self) -> HashMap<String, NetworkStats> {
        let prev_stats = self.prev_stats.read().await;
        prev_stats.clone()
    }

    /// Get network statistics for a specific interface
    pub async fn get_interface_stats(&self, interface: &str) -> Option<NetworkStats> {
        let prev_stats = self.prev_stats.read().await;
        prev_stats.get(interface).cloned()
    }

    /// Start periodic network monitoring
    pub async fn start_monitoring(&self) {
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                monitor.update_stats().await;
            }
        });
    }
}

impl Clone for NetworkMonitor {
    fn clone(&self) -> Self {
        Self {
            sys: self.sys.clone(),
            prev_stats: self.prev_stats.clone(),
            last_update: self.last_update.clone(),
        }
    }
}

// Module state
static NETWORK_MONITOR: tokio::sync::OnceCell<Arc<NetworkMonitor>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the network monitor
pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = Arc::new(NetworkMonitor::new());
    monitor.start_monitoring().await;
    NETWORK_MONITOR.set(monitor)
        .map_err(|_| "Network monitor already initialized")?;
    Ok(())
}

/// Get network statistics for all interfaces
pub async fn get_network_stats() -> Result<HashMap<String, NetworkStats>, crate::error::SquirrelError> {
    NETWORK_MONITOR
        .get()
        .ok_or_else(|| crate::error::SquirrelError::State("Network monitor not initialized".to_string()))?
        .get_stats()
        .await
}

/// Get network statistics for a specific interface
pub async fn get_interface_stats(interface: &str) -> Result<Option<NetworkStats>, crate::error::SquirrelError> {
    Ok(NETWORK_MONITOR
        .get()
        .ok_or_else(|| crate::error::SquirrelError::State("Network monitor not initialized".to_string()))?
        .get_interface_stats(interface)
        .await)
} 