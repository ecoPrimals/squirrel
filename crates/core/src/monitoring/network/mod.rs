// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::unused_async)] // Allow unused async functions

use std::sync::{Arc, RwLock};
use sysinfo::{NetworkExt, System, SystemExt};
use thiserror::Error;
use std::collections::HashMap;
use crate::error::{Result, SquirrelError};
use serde::{Serialize, Deserialize};

pub mod adapter;
pub use adapter::{NetworkMonitorAdapter, create_monitor_adapter, create_monitor_adapter_with_monitor, SystemInfoAdapter, create_system_info_adapter, create_system_info_adapter_with_system};

/// System information manager for monitoring system resources
#[derive(Debug)]
pub struct SystemInfoManager {
    system: Arc<RwLock<System>>,
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
        let mut sys = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        sys.refresh_all();
        Ok(())
    }

    /// Refreshes network information
    pub async fn refresh_networks(&self) -> Result<()> {
        let mut sys = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        sys.refresh_networks();
        Ok(())
    }

    /// Gets CPU usage
    pub async fn cpu_usage(&self) -> Result<f32> {
        let sys = self.system.read().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        Ok(sys.global_cpu_info().cpu_usage())
    }

    /// Gets memory usage
    pub async fn memory_usage(&self) -> Result<(u64, u64)> {
        let sys = self.system.read().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        Ok((sys.used_memory(), sys.total_memory()))
    }

    /// Gets network statistics
    pub async fn network_stats(&self) -> Result<Vec<(String, u64, u64)>> {
        let sys = self.system.read().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;
        let mut stats = Vec::new();
        for (interface_name, data) in sys.networks() {
            stats.push((
                interface_name.clone(),
                data.received(),
                data.transmitted(),
            ));
        }
        Ok(stats)
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating system info managers
#[derive(Debug, Default)]
pub struct SystemInfoManagerFactory;

impl SystemInfoManagerFactory {
    /// Creates a new factory
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Creates a new system info manager
    #[must_use]
    pub fn create_manager(&self) -> Arc<SystemInfoManager> {
        Arc::new(SystemInfoManager::new())
    }

    /// Creates a new system info manager with dependencies
    #[must_use]
    pub fn create_manager_with_dependencies(&self, system: Arc<RwLock<System>>) -> Arc<SystemInfoManager> {
        Arc::new(SystemInfoManager::with_dependencies(system))
    }

    /// Creates a new system info adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<SystemInfoAdapter> {
        create_system_info_adapter()
    }

    /// Creates a new system info adapter with an existing system
    #[must_use]
    pub fn create_adapter_with_system(&self, system: Arc<RwLock<System>>) -> Arc<SystemInfoAdapter> {
        create_system_info_adapter_with_system(system)
    }
}

/// Configuration for network monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Interval in seconds between network stat updates
    pub interval: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            interval: 60,
        }
    }
}

/// Network interface statistics
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

/// Errors that can occur during network monitoring
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Error accessing system information
    #[error("System error: {0}")]
    System(String),
    /// Error with network interface
    #[error("Interface error: {0}")]
    Interface(String),
}

/// Monitors network interface statistics
#[derive(Debug)]
pub struct NetworkMonitor {
    /// Configuration for the network monitor
    config: NetworkConfig,
    /// System information handle
    system: Arc<RwLock<System>>,
    /// Network interface statistics
    stats: Arc<RwLock<HashMap<String, NetworkStats>>>,
}

impl NetworkMonitor {
    /// Creates a new network monitor with the given configuration
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        let mut system = System::new();
        system.refresh_networks_list();

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

    /// Gets current network statistics for all interfaces
    /// 
    /// # Errors
    /// Returns error if unable to access system information or network interfaces
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        let mut system = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;

        system.refresh_networks();

        let mut result = HashMap::new();
        for (interface_name, network_data) in system.networks() {
            result.insert(interface_name.to_string(), NetworkStats {
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
        drop(system);

        Ok(result)
    }

    /// Gets statistics for a specific network interface
    /// 
    /// # Errors
    /// Returns error if unable to access system information or if interface not found
    pub async fn get_interface_stats(&self, interface_name: &str) -> Result<Option<NetworkStats>> {
        let system = self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?;

        for (name, network) in system.networks() {
            if name == interface_name {
                let stats = NetworkStats {
                    interface: name.clone(),
                    received_bytes: network.received(),
                    transmitted_bytes: network.transmitted(),
                    packets_received: network.packets_received(),
                    packets_transmitted: network.packets_transmitted(),
                    errors_on_received: network.errors_on_received(),
                    errors_on_transmitted: network.errors_on_transmitted(),
                    receive_rate: network.received() as f64,
                    transmit_rate: network.transmitted() as f64,
                };
                drop(system);
                return Ok(Some(stats));
            }
        }
        drop(system);
        Ok(None)
    }

    /// Starts the network monitor
    /// 
    /// # Errors
    /// Returns error if unable to access system information
    pub async fn start(&self) -> Result<()> {
        self.system.write().map_err(|e| {
            NetworkError::System(format!("Failed to acquire system lock: {e}"))
        })?.refresh_networks_list();
        Ok(())
    }

    /// Updates network statistics
    /// 
    /// # Errors
    /// Returns error if unable to access system information
    pub fn update_stats(&self) -> Result<()> {
        let mut system = match self.system.write() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(NetworkError::System(format!(
                    "Failed to acquire system lock: {e}"
                )).into());
            }
        };

        system.refresh_networks();

        for (interface_name, network_data) in system.networks() {
            self.stats.write().map_err(|e| {
                NetworkError::System(format!("Failed to acquire stats lock: {e}"))
            })?.insert(interface_name.to_string(), NetworkStats {
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
        drop(system);

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
    /// Creates a new network stats instance
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
#[derive(Debug, Clone)]
pub struct NetworkMonitorFactory {
    /// Configuration for creating monitors
    config: NetworkConfig,
}

impl NetworkMonitorFactory {
    /// Creates a new factory with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: NetworkConfig::default(),
        }
    }

    /// Creates a new factory with the specified configuration
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
