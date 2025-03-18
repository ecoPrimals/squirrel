use std::sync::Arc;
use crate::error::Result;
use super::{NetworkMonitor, NetworkStats, NetworkConfig, ensure_factory};
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock;
use sysinfo::System;

/// Adapter for the network monitor to support dependency injection
#[derive(Debug)]
pub struct NetworkMonitorAdapter {
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

    /// Gets current network statistics for all interfaces
    pub async fn get_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        if let Some(monitor) = &self.inner {
            monitor.get_stats().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_monitor().await {
                Ok(monitor) => monitor.get_stats().await,
                Err(e) => Err(e),
            }
        }
    }

    /// Gets statistics for a specific network interface
    pub async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        if let Some(monitor) = &self.inner {
            monitor.get_interface_stats(interface).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_monitor().await {
                Ok(monitor) => monitor.get_interface_stats(interface).await,
                Err(e) => Err(e),
            }
        }
    }

    /// Updates network statistics
    pub async fn update_stats(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.update_stats()
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_monitor().await {
                Ok(monitor) => monitor.update_stats(),
                Err(e) => Err(e),
            }
        }
    }

    /// Starts the network monitor
    pub async fn start(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.start().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_monitor().await {
                Ok(monitor) => monitor.start().await,
                Err(e) => Err(e),
            }
        }
    }

    /// Stops the network monitor
    pub async fn stop(&self) -> Result<()> {
        if let Some(monitor) = &self.inner {
            monitor.stop().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_monitor().await {
                Ok(monitor) => monitor.stop().await,
                Err(e) => Err(e),
            }
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