// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::unused_async)] // Allow unused async functions

use std::sync::{Arc, RwLock};
use sysinfo::{NetworkExt, System, SystemExt};
use thiserror::Error;
use std::collections::HashMap;
use crate::error::Result;
use serde::{Serialize, Deserialize};

lazy_static::lazy_static! {
    static ref SYSTEM: Arc<RwLock<System>> = Arc::new(RwLock::new(System::new_all()));
}

/// Initialize the network monitoring system
///
/// # Errors
///
/// Returns an error if the system information cannot be initialized
///
/// # Panics
///
/// Panics if the system lock cannot be acquired
pub fn initialize() -> Result<()> {
    let mut system = SYSTEM.write().unwrap();
    system.refresh_networks();
    Ok(())
}

/// Network monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Interval in seconds to collect network statistics
    pub interval: u64,
    /// List of network interfaces to monitor
    pub interfaces: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            interval: 60,
            interfaces: vec![],
        }
    }
}

/// Network statistics for a single interface
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkStats {
    /// Interface name
    pub interface: String,
    /// Total bytes received
    pub received_bytes: u64,
    /// Total bytes transmitted
    pub transmitted_bytes: u64,
    /// Current receive rate in bytes/sec
    pub receive_rate: f64,
    /// Current transmit rate in bytes/sec
    pub transmit_rate: f64,
    /// Total packets received
    pub packets_received: u64,
    /// Total packets transmitted
    pub packets_transmitted: u64,
    /// Errors on received packets
    pub errors_on_received: u64,
    /// Errors on transmitted packets
    pub errors_on_transmitted: u64,
}

/// Errors that can occur during network monitoring operations
#[derive(Debug, Error)]
pub enum NetworkError {
    /// System-level errors related to network operations
    #[error("System error: {0}")]
    System(String),
    /// Errors related to network metrics collection and processing
    #[error("Metrics error: {0}")]
    Metrics(String),
}

/// Network monitor for collecting network interface statistics
#[derive(Debug)]
pub struct NetworkMonitor {
    config: NetworkConfig,
    system: Arc<RwLock<System>>,
    stats: Arc<RwLock<HashMap<String, NetworkStats>>>,
}

impl NetworkMonitor {
    /// Creates a new network monitor with the specified configuration
    ///
    /// # Arguments
    /// * `config` - The network monitoring configuration
    ///
    /// # Returns
    /// A new `NetworkMonitor` instance initialized with the provided configuration
    pub fn new(config: NetworkConfig) -> Self {
        let mut system = System::new();
        system.refresh_networks_list();

        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get network statistics for all monitored interfaces
    ///
    /// # Errors
    ///
    /// Returns an error if the system or stats lock cannot be acquired
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        let mut system = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        system.refresh_networks();
        
        let mut stats = self.stats.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire stats lock: {e}"))
        })?;
        stats.clear();

        for (interface_name, network_data) in system.networks() {
            if self.config.interfaces.is_empty() || self.config.interfaces.contains(&interface_name.to_string()) {
                stats.insert(interface_name.to_string(), NetworkStats {
                    interface: interface_name.to_string(),
                    received_bytes: network_data.received(),
                    transmitted_bytes: network_data.transmitted(),
                    receive_rate: network_data.received() as f64 / self.config.interval as f64,
                    transmit_rate: network_data.transmitted() as f64 / self.config.interval as f64,
                    packets_received: network_data.packets_received(),
                    packets_transmitted: network_data.packets_transmitted(),
                    errors_on_received: network_data.errors_on_received(),
                    errors_on_transmitted: network_data.errors_on_transmitted(),
                });
            }
        }
        
        // Clone the stats before returning
        let result = stats.clone();
        Ok(result)
    }

    /// Get network statistics for a specific interface
    ///
    /// # Errors
    ///
    /// Returns an error if the network statistics cannot be retrieved
    pub async fn get_interface_stats(&self, interface_name: &str) -> Result<Option<NetworkStats>> {
        let system = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;

        for (name, network) in system.networks() {
            if name == interface_name {
                return Ok(Some(NetworkStats {
                    interface: name.clone(),
                    received_bytes: network.received(),
                    transmitted_bytes: network.transmitted(),
                    packets_received: network.packets_received(),
                    packets_transmitted: network.packets_transmitted(),
                    errors_on_received: network.errors_on_received(),
                    errors_on_transmitted: network.errors_on_transmitted(),
                    receive_rate: network.received() as f64,
                    transmit_rate: network.transmitted() as f64,
                }));
            }
        }
        
        Ok(None)
    }

    /// Start monitoring network interfaces
    ///
    /// # Errors
    ///
    /// Returns an error if the system lock cannot be acquired
    pub async fn start(&self) -> Result<()> {
        // Initialize system info
        let mut system = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        system.refresh_networks_list();
        Ok(())
    }

    /// Stop monitoring network interfaces
    ///
    /// # Errors
    ///
    /// Returns an error if the network monitoring cannot be stopped
    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }

    /// Get current network statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the system or stats lock cannot be acquired
    pub fn get_network_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        // Just make sure we can access the system, but we don't need it directly
        let _system = self.system.read().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        
        let stats = self.stats.read().map_err(|e| {
            NetworkError::System(format!("Failed to acquire stats lock: {e}"))
        })?;
        
        Ok(stats.clone())
    }

    /// Update network statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the system or stats lock cannot be acquired
    pub fn update_stats(&self) -> Result<()> {
        let mut system = match self.system.write() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(NetworkError::System(format!("Failed to acquire system lock: {e}")).into());
            }
        };
        
        system.refresh_networks();
        
        let mut stats = self.stats.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire stats lock: {e}"))
        })?;
        
        // Update stats with current network information
        for (interface_name, network_data) in system.networks() {
            if self.config.interfaces.is_empty() || self.config.interfaces.contains(&interface_name.to_string()) {
                stats.insert(interface_name.to_string(), NetworkStats {
                    interface: interface_name.to_string(),
                    received_bytes: network_data.received(),
                    transmitted_bytes: network_data.transmitted(),
                    receive_rate: network_data.received() as f64 / self.config.interval as f64,
                    transmit_rate: network_data.transmitted() as f64 / self.config.interval as f64,
                    packets_received: network_data.packets_received(),
                    packets_transmitted: network_data.packets_transmitted(),
                    errors_on_received: network_data.errors_on_received(),
                    errors_on_transmitted: network_data.errors_on_transmitted(),
                });
            }
        }
        
        Ok(())
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new(NetworkConfig::default())
    }
}

impl NetworkStats {
    pub fn new() -> Self {
        Self {
            interface: String::new(),
            received_bytes: 0,
            transmitted_bytes: 0,
            receive_rate: 0.0,
            transmit_rate: 0.0,
            packets_received: 0,
            packets_transmitted: 0,
            errors_on_received: 0,
            errors_on_transmitted: 0,
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Get network statistics for all interfaces
///
/// # Errors
///
/// Returns an error if the system lock cannot be acquired
pub fn get_network_stats() -> Result<HashMap<String, NetworkStats>> {
    let mut system = SYSTEM.write().map_err(|e| {
        NetworkError::System(format!("Failed to acquire system lock: {e}"))
    })?;
    system.refresh_networks();
    
    let mut stats = HashMap::new();
    for (interface_name, network) in system.networks() {
        stats.insert(interface_name.clone(), NetworkStats {
            interface: interface_name.clone(),
            received_bytes: network.received(),
            transmitted_bytes: network.transmitted(),
            packets_received: network.packets_received(),
            packets_transmitted: network.packets_transmitted(),
            errors_on_received: network.errors_on_received(),
            errors_on_transmitted: network.errors_on_transmitted(),
            receive_rate: network.received() as f64,
            transmit_rate: network.transmitted() as f64,
        });
    }
    
    Ok(stats)
}

/// Get network statistics for a specific interface
///
/// # Errors
///
/// Returns an error if the system lock cannot be acquired
pub fn get_interface_stats(interface: &str) -> Result<Option<NetworkStats>> {
    let mut system = SYSTEM.write().map_err(|e| {
        NetworkError::System(format!("Failed to acquire system lock: {e}"))
    })?;
    system.refresh_networks();
    
    for (interface_name, network) in system.networks() {
        if interface_name == interface {
            return Ok(Some(NetworkStats {
                interface: interface_name.clone(),
                received_bytes: network.received(),
                transmitted_bytes: network.transmitted(),
                packets_received: network.packets_received(),
                packets_transmitted: network.packets_transmitted(),
                errors_on_received: network.errors_on_received(),
                errors_on_transmitted: network.errors_on_transmitted(),
                receive_rate: network.received() as f64,
                transmit_rate: network.transmitted() as f64,
            }));
        }
    }
    
    Ok(None)
} 