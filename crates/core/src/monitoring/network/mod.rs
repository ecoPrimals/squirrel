// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::unused_async)] // Allow unused async functions

use std::sync::{Arc, RwLock, OnceLock};
use sysinfo::{NetworkExt, System, SystemExt};
use thiserror::Error;
use std::collections::HashMap;
use crate::error::{Result, SquirrelError};
use serde::{Serialize, Deserialize};

lazy_static::lazy_static! {
    static ref SYSTEM: Arc<RwLock<System>> = Arc::new(RwLock::new(System::new_all()));
}

/// Initialize the network monitoring system (legacy function)
///
/// # Errors
///
/// Returns an error if:
/// - The system information cannot be initialized
/// - The system lock cannot be acquired
pub fn initialize_legacy() -> Result<()> {
    SYSTEM.write()
        .map_err(|e| SquirrelError::Other(format!("Failed to acquire system lock: {e}")))?
        .refresh_networks();
    Ok(())
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
    /// Name of the network interface
    pub interface: String,
    /// Total bytes received
    pub received_bytes: u64,
    /// Total bytes transmitted
    pub transmitted_bytes: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Total packets transmitted
    pub packets_transmitted: u64,
    /// Number of errors on received packets
    pub errors_on_received: u64,
    /// Number of errors on transmitted packets
    pub errors_on_transmitted: u64,
    /// Rate of bytes received per second
    pub receive_rate: f64,
    /// Rate of bytes transmitted per second
    pub transmit_rate: f64,
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
    /// Creates a new empty network stats instance
    #[must_use]
    pub const fn new() -> Self {
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

/// Get network statistics for all interfaces (legacy function)
///
/// # Errors
///
/// Returns an error if the system lock cannot be acquired
pub fn get_network_stats_legacy() -> Result<HashMap<String, NetworkStats>> {
    let mut system = SYSTEM.write().map_err(|e| {
        SquirrelError::Other(format!("Failed to acquire system lock: {e}"))
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

/// Get network statistics for a specific interface (legacy function)
///
/// # Errors
///
/// Returns an error if the system lock cannot be acquired
pub fn get_interface_stats_legacy(interface: &str) -> Result<Option<NetworkStats>> {
    let mut system = SYSTEM.write().map_err(|e| {
        SquirrelError::Other(format!("Failed to acquire system lock: {e}"))
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

/// Factory for creating and managing network monitor instances
#[derive(Debug, Clone)]
pub struct NetworkMonitorFactory {
    /// Configuration for creating network monitors
    config: NetworkConfig,
}

impl NetworkMonitorFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NetworkConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: NetworkConfig) -> Self {
        Self { config }
    }

    /// Creates a network monitor
    #[must_use]
    pub fn create_monitor(&self) -> Arc<NetworkMonitor> {
        Arc::new(NetworkMonitor::new(self.config.clone()))
    }

    /// Initializes and returns a global network monitor instance
    ///
    /// # Errors
    /// Returns an error if the monitor is already initialized
    pub async fn initialize_global_monitor(&self) -> Result<Arc<NetworkMonitor>> {
        static GLOBAL_MONITOR: OnceLock<Arc<NetworkMonitor>> = OnceLock::new();

        let monitor = self.create_monitor();
        match GLOBAL_MONITOR.set(monitor.clone()) {
            Ok(()) => Ok(monitor),
            Err(_) => {
                // Already initialized, return the existing instance
                Ok(GLOBAL_MONITOR.get()
                    .ok_or_else(|| SquirrelError::monitoring("Failed to get global network monitor"))?
                    .clone())
            }
        }
    }

    /// Gets the global network monitor, initializing it if necessary
    ///
    /// # Errors
    /// Returns an error if the network monitor cannot be initialized
    pub async fn get_global_monitor(&self) -> Result<Arc<NetworkMonitor>> {
        static GLOBAL_MONITOR: OnceLock<Arc<NetworkMonitor>> = OnceLock::new();

        if let Some(monitor) = GLOBAL_MONITOR.get() {
            return Ok(monitor.clone());
        }

        self.initialize_global_monitor().await
    }
}

impl Default for NetworkMonitorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory for creating network monitors
static FACTORY: OnceLock<NetworkMonitorFactory> = OnceLock::new();

/// Initialize the network monitor factory
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory(config: Option<NetworkConfig>) -> Result<()> {
    let factory = match config {
        Some(cfg) => NetworkMonitorFactory::with_config(cfg),
        None => NetworkMonitorFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::monitoring("Network monitor factory already initialized"))?;
    Ok(())
}

/// Get the network monitor factory
#[must_use]
pub fn get_factory() -> Option<NetworkMonitorFactory> {
    FACTORY.get().cloned()
}

/// Get or create the network monitor factory
#[must_use]
pub fn ensure_factory() -> NetworkMonitorFactory {
    FACTORY.get_or_init(NetworkMonitorFactory::new).clone()
}

// Static instance for global access (replace with factory access later)
static NETWORK_MONITOR: tokio::sync::OnceCell<Arc<NetworkMonitor>> = tokio::sync::OnceCell::const_new();

/// Initialize the network monitoring system with configuration
///
/// # Errors
/// Returns an error if:
/// - The system information cannot be initialized
/// - The system lock cannot be acquired
/// - The network monitor is already initialized
pub async fn initialize(config: Option<NetworkConfig>) -> Result<Arc<NetworkMonitor>> {
    // Initialize the system first
    SYSTEM.write()
        .map_err(|e| SquirrelError::monitoring(&format!("Failed to acquire system lock: {e}")))?
        .refresh_networks();
    
    // Create and initialize using factory
    let factory = match config {
        Some(cfg) => NetworkMonitorFactory::with_config(cfg),
        None => ensure_factory(),
    };
    
    let monitor = factory.initialize_global_monitor().await?;
    
    // For backward compatibility, also set in the old static
    let _ = NETWORK_MONITOR.set(monitor.clone());
    
    Ok(monitor)
}

/// Get network statistics
///
/// # Errors
/// Returns an error if:
/// - The network monitor is not initialized
/// - The statistics cannot be retrieved
pub async fn get_network_stats() -> Result<HashMap<String, NetworkStats>> {
    if let Some(monitor) = NETWORK_MONITOR.get() {
        monitor.get_stats().await
    } else {
        // Try to initialize on-demand if not already done
        let factory = ensure_factory();
        let monitor = factory.get_global_monitor().await?;
        monitor.get_stats().await
    }
}

/// Get statistics for a specific network interface
///
/// # Errors
/// Returns an error if:
/// - The network monitor is not initialized
/// - The statistics cannot be retrieved
pub async fn get_interface_stats(interface: &str) -> Result<Option<NetworkStats>> {
    if let Some(monitor) = NETWORK_MONITOR.get() {
        monitor.get_interface_stats(interface).await
    } else {
        // Try to initialize on-demand if not already done
        let factory = ensure_factory();
        let monitor = factory.get_global_monitor().await?;
        monitor.get_interface_stats(interface).await
    }
} 