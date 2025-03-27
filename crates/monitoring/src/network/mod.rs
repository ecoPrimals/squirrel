/// Network monitoring functionality)] // Allow u64 to f64 casts for metrics
///
/// This module provides network interface monitoring, bandwidth tracking,
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use sysinfo::{System, SystemExt, NetworkExt, NetworksExt};
use squirrel_core::error::Result;
use tracing::debug;
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};

// Define a type alias for 's' to fix compilation issues
type s = System;

/// Module for adapter implementations of network monitoring functionality
pub mod adapter;

/// Network statistics for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Interface name
    pub interface: String,
    /// Received bytes
    pub rx_bytes: u64,
    /// Transmitted bytes
    pub tx_bytes: u64,
    /// Received bytes per second
    pub rx_bytes_per_sec: f64,
    /// Transmitted bytes per second
    pub tx_bytes_per_sec: f64,
    /// Packets received
    pub rx_packets: Option<u64>,
    /// Packets transmitted
    pub tx_packets: Option<u64>,
    /// Errors on receive
    pub rx_errors: Option<u64>,
    /// Errors on transmit
    pub tx_errors: Option<u64>,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Network monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Update interval in seconds
    pub update_interval: u64,
    /// Interfaces to monitor
    pub interfaces: Vec<String>,
    /// Whether to monitor all interfaces
    pub monitor_all_interfaces: bool,
    /// Whether to collect packet statistics
    pub collect_packet_stats: bool,
    /// Whether to collect error statistics
    pub collect_error_stats: bool,
    /// Whether to automatically start monitoring
    pub auto_start: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            update_interval: 5,
            interfaces: Vec::new(),
            monitor_all_interfaces: true,
            collect_packet_stats: true,
            collect_error_stats: true,
            auto_start: true,
        }
    }
}

/// Network monitor for tracking interface statistics
#[derive(Debug)]
pub struct NetworkMonitor {
    /// Configuration
    config: NetworkConfig,
    /// System information
    system: RwLock<System>,
    /// Network statistics
    stats: RwLock<HashMap<String, NetworkStats>>,
    /// Previous network statistics for calculating rates
    prev_stats: RwLock<HashMap<String, NetworkStats>>,
    /// Running state
    is_running: AtomicBool,
    /// Background task
    background_task: RwLock<Option<JoinHandle<()>>>,
}

impl NetworkMonitor {
    /// Creates a new network monitor with the given configuration
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        let system = System::new_all();
        
        let monitor = Self {
            config: config.clone(),
            system: RwLock::new(system),
            stats: RwLock::new(HashMap::new()),
            prev_stats: RwLock::new(HashMap::new()),
            is_running: AtomicBool::new(false),
            background_task: RwLock::new(None),
        };
        
        if config.auto_start {
            let monitor_clone = Arc::new(monitor.clone());
            tokio::spawn(async move {
                if let Err(e) = monitor_clone.start().await {
                    debug!("Failed to auto-start network monitor: {}", e);
                }
            });
        }
        
        monitor
    }
    
    /// Gets current network statistics for all interfaces
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        Ok(self.stats.read().await.clone())
    }
    
    /// Gets statistics for a specific network interface
    pub async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        Ok(self.stats.read().await.get(interface).cloned())
    }
    
    /// Updates network statistics
    pub async fn update_stats(&self) -> Result<()> {
        let mut system = self.system.write().await;
        system.refresh_networks();
        
        let networks = system.networks();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut prev_stats = self.prev_stats.write().await;
        let mut stats_map = self.stats.write().await;
        
        for (interface_name, network) in networks {
            // Skip if not monitoring all interfaces and this interface is not in the list
            if !self.config.monitor_all_interfaces && !self.config.interfaces.contains(interface_name) {
                continue;
            }
            
            // Get network metrics
            let rx_bytes = network.received();
            let tx_bytes = network.transmitted();
            
            // Calculate rates based on previous readings
            let (rx_bytes_per_sec, tx_bytes_per_sec) = if let Some(prev) = prev_stats.get(interface_name) {
                let time_diff = (now - prev.last_updated) as f64;
                if time_diff > 0.0 {
                    let rx_diff = rx_bytes as f64 - prev.rx_bytes as f64;
                    let tx_diff = tx_bytes as f64 - prev.tx_bytes as f64;
                    (rx_diff / time_diff, tx_diff / time_diff)
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };
            
            // Create network stats for this interface
            let network_stats = NetworkStats {
                interface: interface_name.clone(),
                rx_bytes,
                tx_bytes,
                rx_bytes_per_sec,
                tx_bytes_per_sec,
                rx_packets: Some(network.packets_received()),
                tx_packets: Some(network.packets_transmitted()),
                rx_errors: Some(network.errors_on_received()),
                tx_errors: Some(network.errors_on_transmitted()),
                last_updated: now,
            };
            
            // Update current and previous stats
            stats_map.insert(interface_name.clone(), network_stats.clone());
            prev_stats.insert(interface_name.clone(), network_stats);
        }
        
        Ok(())
    }
    
    /// Starts the network monitor
    pub async fn start(&self) -> Result<()> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err("Network monitor already running".to_string().into());
        }
        
        // Initialize stats on first start
        self.update_stats().await?;
        
        let update_interval = self.config.update_interval;
        let self_clone = Arc::new(self.clone());
        
        let task = tokio::spawn(async move {
            loop {
                if !self_clone.is_running.load(Ordering::SeqCst) {
                    break;
                }
                
                if let Err(e) = self_clone.update_stats().await {
                    debug!("Failed to update network stats: {}", e);
                }
                
                sleep(Duration::from_secs(update_interval)).await;
            }
        });
        
        let mut background_task = self.background_task.write().await;
        *background_task = Some(task);
        
        Ok(())
    }
    
    /// Stops the network monitor
    pub async fn stop(&self) -> Result<()> {
        if !self.is_running.swap(false, Ordering::SeqCst) {
            return Err("Network monitor not running".to_string().into());
        }
        
        let mut background_task = self.background_task.write().await;
        if let Some(task) = background_task.take() {
            task.abort();
        }
        
        Ok(())
    }
}

impl Clone for NetworkMonitor {
    fn clone(&self) -> Self {
        let system = System::new_all();
        
        Self {
            config: self.config.clone(),
            system: RwLock::new(system),
            stats: RwLock::new(HashMap::new()),
            prev_stats: RwLock::new(HashMap::new()),
            is_running: AtomicBool::new(self.is_running.load(Ordering::SeqCst)),
            background_task: RwLock::new(None),
        }
    }
} 
