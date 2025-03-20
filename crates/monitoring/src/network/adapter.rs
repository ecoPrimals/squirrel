use std::sync::Arc;
use squirrel_core::error::Result;
use super::{NetworkMonitor, NetworkStats, NetworkConfig};
use std::collections::HashMap;

/// Adapter for the network monitor to support dependency injection
#[derive(Debug)]
pub struct NetworkMonitorAdapter {
    /// Underlying network monitor implementation
    inner: Option<Arc<NetworkMonitor>>,
}

impl NetworkMonitorAdapter {
    /// Creates a new network monitor adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing network monitor
    #[must_use]
    pub fn with_monitor(monitor: Arc<NetworkMonitor>) -> Self {
        Self {
            inner: Some(monitor),
        }
    }

    /// Checks if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }

    /// Initializes the adapter with default configuration
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized() {
            return Err("Network monitor adapter already initialized".to_string().into());
        }
        
        let config = NetworkConfig::default();
        let monitor = NetworkMonitor::new(config);
        self.inner = Some(Arc::new(monitor));
        Ok(())
    }

    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: NetworkConfig) -> Result<()> {
        if self.is_initialized() {
            return Err("Network monitor adapter already initialized".to_string().into());
        }
        
        let monitor = NetworkMonitor::new(config);
        self.inner = Some(Arc::new(monitor));
        Ok(())
    }

    /// Gets current network statistics for all interfaces
    ///
    /// # Errors
    /// Returns an error if the network monitor is not initialized via dependency injection
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        if let Some(monitor) = &self.inner {
            monitor.get_stats().await
        } else {
            Err("Network monitor not initialized via dependency injection".to_string().into())
        }
    }

    /// Gets statistics for a specific network interface
    ///
    /// # Errors
    /// Returns an error if the network monitor is not initialized via dependency injection
    pub async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        if let Some(monitor) = &self.inner {
            monitor.get_interface_stats(interface).await
        } else {
            Err("Network monitor not initialized via dependency injection".to_string().into())
        }
    }

    /// Updates network statistics
    ///
    /// # Errors
    /// Returns an error if the network monitor is not initialized via dependency injection
    pub async fn update_stats(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.update_stats().await
        } else {
            Err("Network monitor not initialized via dependency injection".to_string().into())
        }
    }

    /// Starts the network monitor
    ///
    /// # Errors
    /// Returns an error if the network monitor is not initialized via dependency injection
    pub async fn start(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.start().await
        } else {
            Err("Network monitor not initialized via dependency injection".to_string().into())
        }
    }

    /// Stops the network monitor
    ///
    /// # Errors
    /// Returns an error if the network monitor is not initialized via dependency injection
    pub async fn stop(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.stop().await
        } else {
            Err("Network monitor not initialized via dependency injection".to_string().into())
        }
    }
}

impl Clone for NetworkMonitorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for NetworkMonitorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new network monitor adapter
#[must_use]
pub fn create_monitor_adapter() -> Arc<NetworkMonitorAdapter> {
    Arc::new(NetworkMonitorAdapter::new())
}

/// Creates a new network monitor adapter with an existing monitor
#[must_use]
pub fn create_monitor_adapter_with_monitor(monitor: Arc<NetworkMonitor>) -> Arc<NetworkMonitorAdapter> {
    Arc::new(NetworkMonitorAdapter::with_monitor(monitor))
} 