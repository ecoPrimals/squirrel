/// Network monitoring functionality
///
/// This module provides network interface monitoring, bandwidth tracking,
/// and network health diagnostics.

// Allow certain linting issues that are too numerous to fix individually
#[allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#[allow(clippy::unused_async)] // Allow unused async functions

use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::{System, Networks};
use std::collections::HashMap;
use crate::error::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// Module for adapter implementations of network monitoring functionality
pub mod adapter;

/// Error types for network monitoring operations
pub mod error;

/// System information gathering for network monitoring
pub mod system_info;

// Re-export system info adapter
pub use system_info::{SystemInfoAdapter, create_system_info_adapter, create_system_info_adapter_with_system};

// Re-export adapter and error types
pub use adapter::NetworkMonitorAdapter;
pub use error::{NetworkError, system_error, config_error, monitoring_error, interface_error, stats_error};

/// Type alias for sysinfo::System to simplify usage in the network module
type S = System;

/// System information manager for monitoring system resources
#[derive(Debug)]
pub struct SystemInfoManager {
    /// System information instance for collecting system metrics
    system: Arc<RwLock<S>>,
}

impl SystemInfoManager {
    /// Creates a new system info manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
        }
    }

    /// Creates a new system info manager with dependencies
    #[must_use]
    pub fn with_dependencies(system: Arc<RwLock<System>>) -> Self {
        Self { system }
    }

    /// Refreshes all system information
    pub async fn refresh_all(&self) -> Result<()> {
        let mut sys = self.system.write().await;
        sys.refresh_all();
        Ok(())
    }

    /// Refreshes network information
    pub async fn refresh_networks(&self) -> Result<()> {
        let mut sys = self.system.write().await;
        let mut system_clone = System::new_all();
        system_clone.refresh_all();
        *sys = system_clone;
        Ok(())
    }

    /// Gets CPU usage
    pub async fn cpu_usage(&self) -> Result<f32> {
        let sys = self.system.read().await;
        Ok(sys.global_cpu_info().cpu_usage())
    }

    /// Gets memory usage (used, total)
    pub async fn memory_usage(&self) -> Result<(u64, u64)> {
        let sys = self.system.read().await;
        Ok((sys.used_memory(), sys.total_memory()))
    }

    /// Gets network statistics for all interfaces
    pub async fn network_stats(&self) -> Result<Vec<(String, u64, u64)>> {
        let _sys = self.system.read().await;
        let mut stats = Vec::new();
        
        // Create fresh Networks instance with refreshed data instead of using system.networks()
        let networks = Networks::new_with_refreshed_list();
        
        // Get networks using iteration over Networks
        for (interface_name, data) in &networks {
            stats.push((
                interface_name.clone(),
                data.received(),
                data.transmitted(),
            ));
        }
        
        Ok(stats)
    }

    /// Gets a list of all network interfaces
    ///
    /// # Returns
    ///
    /// A Result containing a vector of NetworkInterface objects or an error
    pub async fn get_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        // Create fresh Networks instance
        let networks = Networks::new_with_refreshed_list();
        
        let mut interfaces = Vec::new();
        for (name, _) in &networks {
            interfaces.push(NetworkInterface {
                name: name.clone(),
            });
        }
        Ok(interfaces)
    }

    /// Gets statistics for a specific network interface
    ///
    /// # Parameters
    ///
    /// * `interface_name` - The name of the interface to get statistics for
    ///
    /// # Returns
    ///
    /// A Result containing NetworkInterfaceStats or an error
    pub async fn get_interface_stats(&self, interface_name: &str) -> Result<NetworkInterfaceStats> {
        // We'll just create a placeholder stats object for now
        Ok(NetworkInterfaceStats {
            name: interface_name.to_string(),
            rx_bytes: 0,
            tx_bytes: 0,
            rx_errors: 0,
            tx_errors: 0,
            rx_packets: 0,
            tx_packets: 0,
        })
    }

    /// Gets statistics for all network interfaces
    ///
    /// # Returns
    ///
    /// A Result containing NetworkStats or an error
    pub async fn get_all_stats(&self) -> Result<NetworkStats> {
        // Create fresh Networks instance
        let networks = Networks::new_with_refreshed_list();
        
        // Get the first network interface, or use empty string if none found
        let interface_name = networks.iter().next()
            .map_or_else(String::new, |(name, _)| name.clone());
            
        let stats = NetworkStats {
            interface: interface_name,
            received_bytes: 0,
            transmitted_bytes: 0,
            receive_rate: 0.0,
            transmit_rate: 0.0,
            packets_received: 0,
            packets_transmitted: 0,
            errors_on_received: 0,
            errors_on_transmitted: 0,
        };
        Ok(stats)
    }

    /// Gets the overall network usage
    ///
    /// # Returns
    ///
    /// A Result containing NetworkUsage or an error
    pub async fn get_network_usage(&self) -> Result<NetworkUsage> {
        let _sys = self.system.read().await;
        let usage = NetworkUsage {
            interfaces: Vec::new(),
            total_rx_bytes_per_sec: 0.0,
            total_tx_bytes_per_sec: 0.0,
            timestamp: std::time::SystemTime::now(),
        };
        
        Ok(usage)
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating system info managers
pub struct SystemInfoManagerFactory;

impl SystemInfoManagerFactory {
    /// Creates a new system info manager factory
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
    
    /// Creates a new system info manager
    #[must_use]
    pub fn create_manager(&self) -> Arc<SystemInfoManager> {
        Arc::new(SystemInfoManager::new())
    }
    
    /// Creates a new system info manager with dependencies
    #[must_use]
    pub fn create_manager_with_dependencies(&self, system: Arc<RwLock<S>>) -> Arc<SystemInfoManager> {
        Arc::new(SystemInfoManager::with_dependencies(system))
    }
    
    /// Creates a new system info adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<SystemInfoAdapter> {
        system_info::create_system_info_adapter()
    }
    
    /// Creates a new system info adapter with system
    #[must_use]
    pub fn create_adapter_with_system(&self, system: Arc<RwLock<S>>) -> Arc<SystemInfoAdapter> {
        system_info::create_system_info_adapter_with_system(system)
    }
}

/// Network monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Interval in seconds between network stat updates
    pub interval: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            interval: 60, // Default to 60 seconds
        }
    }
}

/// Network statistics for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Network interface name
    pub interface: String,
    /// Total bytes received
    pub received_bytes: u64,
    /// Total bytes transmitted
    pub transmitted_bytes: u64,
    /// Receive rate in bytes per second
    pub receive_rate: f64,
    /// Transmit rate in bytes per second
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

/// Network monitor for tracking network usage
#[derive(Debug)]
pub struct NetworkMonitor {
    /// Configuration for the network monitor
    config: NetworkConfig,
    /// System information handle
    system: Arc<RwLock<S>>,
    /// Network interface statistics
    stats: Arc<RwLock<HashMap<String, NetworkStats>>>,
}

impl NetworkMonitor {
    /// Creates a new network monitor
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        let system = System::new_all();

        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new network monitor with dependencies
    #[must_use]
    pub fn with_dependencies(config: NetworkConfig) -> Self {
        Self::new(config)
    }

    /// Gets all network interface statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        let stats = self.stats.read().await;

        // Create a copy of the current stats
        let mut result = HashMap::new();
        for (k, v) in stats.iter() {
            result.insert(k.clone(), v.clone());
        }

        Ok(result)
    }

    /// Gets statistics for a specific network interface
    pub async fn get_interface_stats(&self, interface_name: &str) -> Result<Option<NetworkStats>> {
        let stats = self.stats.read().await;

        // Create a copy of the requested interface stats
        let result = stats.get(interface_name).cloned();

        Ok(result)
    }

    /// Starts the network monitor
    pub async fn start(&self) -> Result<()> {
        // Initial update
        self.update_stats().await?;

        // TODO: Implement periodic updates
        Ok(())
    }

    /// Updates network statistics
    pub async fn update_stats(&self) -> Result<()> {
        // Create new Networks instead of using system.networks()
        let networks = Networks::new_with_refreshed_list();
        
        let mut stats = self.stats.write().await;
        // Process network data
        for (interface_name, network_data) in &networks {
            stats.insert(interface_name.clone(), NetworkStats {
                interface: interface_name.clone(),
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
        
        Ok(())
    }

    /// Stops the network monitor
    /// 
    /// # Errors
    /// Returns error if unable to stop the monitor
    pub async fn stop(&self) -> Result<()> {
        // No cleanup needed for now
        Ok(())
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new(NetworkConfig::default())
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkStats {
    /// Creates a new network stats instance with default values
    #[must_use]
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

/// Factory for creating network monitors
#[derive(Debug)]
pub struct NetworkMonitorFactory {
    /// Configuration for creating monitors
    config: NetworkConfig,
}

impl NetworkMonitorFactory {
    /// Creates a new network monitor factory
    #[must_use] pub fn new() -> Self {
        Self {
            config: NetworkConfig::default(),
        }
    }

    /// Creates a new network monitor factory with config
    #[must_use] pub const fn with_config(config: NetworkConfig) -> Self {
        Self { config }
    }

    /// Creates a new monitor instance with dependency injection
    ///
    /// # Arguments
    /// * `config` - Optional configuration override
    ///
    /// # Returns
    /// A new NetworkMonitor instance wrapped in an Arc
    #[must_use]
    pub fn create_monitor_with_config(
        &self,
        config: Option<NetworkConfig>,
    ) -> Arc<NetworkMonitor> {
        Arc::new(NetworkMonitor::new(config.unwrap_or_else(|| self.config.clone())))
    }

    /// Creates a new monitor instance with the default configuration
    #[must_use]
    pub fn create_monitor(&self) -> Arc<NetworkMonitor> {
        self.create_monitor_with_config(None)
    }

    /// Creates a new monitor adapter
    #[must_use]
    pub fn create_monitor_adapter(&self) -> Arc<NetworkMonitorAdapter> {
        let monitor = self.create_monitor();
        create_monitor_adapter_with_monitor(monitor)
    }
}

impl Default for NetworkMonitorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new network monitor adapter
#[must_use]
pub fn create_monitor_adapter() -> Arc<NetworkMonitorAdapter> {
    NetworkMonitorFactory::new().create_monitor_adapter()
}

/// Create a new network monitor adapter with a specific monitor
#[must_use]
pub fn create_monitor_adapter_with_monitor(
    monitor: Arc<NetworkMonitor>
) -> Arc<NetworkMonitorAdapter> {
    Arc::new(NetworkMonitorAdapter::with_monitor(monitor))
}

/// Represents a network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Name of the interface
    pub name: String,
}

/// Statistics for a network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceStats {
    /// Name of the interface
    pub name: String,
    /// Total bytes received
    pub rx_bytes: u64,
    /// Total bytes transmitted
    pub tx_bytes: u64,
    /// Number of receive errors
    pub rx_errors: u64,
    /// Number of transmit errors
    pub tx_errors: u64,
    /// Number of packets received
    pub rx_packets: u64,
    /// Number of packets transmitted
    pub tx_packets: u64,
}

/// Network usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    /// List of interfaces
    pub interfaces: Vec<String>,
    /// Total rx bytes per second
    pub total_rx_bytes_per_sec: f64,
    /// Total tx bytes per second
    pub total_tx_bytes_per_sec: f64,
    /// Timestamp of the measurements
    pub timestamp: SystemTime,
}

/// Function to collect network statistics using sysinfo
pub fn collect_network_stats() -> Result<HashMap<String, NetworkStats>> {
    // Create a new Networks instance instead of using System::networks()
    let networks = Networks::new_with_refreshed_list();

    let mut stats = HashMap::new();
    for (interface_name, network_data) in &networks {
        stats.insert(interface_name.clone(), NetworkStats {
            interface: interface_name.clone(),
            received_bytes: network_data.received(),
            transmitted_bytes: network_data.transmitted(),
            receive_rate: 0.0,
            transmit_rate: 0.0,
            packets_received: network_data.packets_received(),
            packets_transmitted: network_data.packets_transmitted(),
            errors_on_received: network_data.errors_on_received(),
            errors_on_transmitted: network_data.errors_on_transmitted(),
        });
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_network_monitor_basic() {
        let monitor = NetworkMonitor::new(NetworkConfig::default());
        
        // Start monitoring
        monitor.start().await.unwrap();
        
        // Wait for some stats to be collected
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Get stats
        let stats = monitor.get_stats().await.unwrap();
        assert!(!stats.is_empty());
        
        // Stop monitoring
        monitor.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_network_monitor_adapter() {
        let factory = NetworkMonitorFactory::new();
        let adapter = factory.create_monitor_adapter();
        
        // Start monitoring
        adapter.start().await.unwrap();
        
        // Wait for some stats to be collected
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Get stats
        let stats = adapter.get_stats().await.unwrap();
        assert!(!stats.is_empty());
        
        // Get interface stats
        if let Some(interface) = stats.keys().next() {
            let interface_stats = adapter.get_interface_stats(interface).await.unwrap();
            assert!(interface_stats.is_some());
        }
        
        // Stop monitoring
        adapter.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_network_monitor_with_config() {
        let config = NetworkConfig {
            interval: 1,
        };
        
        let factory = NetworkMonitorFactory::with_config(config.clone());
        let monitor = factory.create_monitor();
        
        // Start monitoring
        monitor.start().await.unwrap();
        
        // Wait for some stats to be collected
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Get stats
        let stats = monitor.get_stats().await.unwrap();
        assert!(!stats.is_empty());
        
        // Stop monitoring
        monitor.stop().await.unwrap();
    }
} 
