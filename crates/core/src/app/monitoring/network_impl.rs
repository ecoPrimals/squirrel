use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use sysinfo::Networks;
use crate::app::monitoring::network::{NetworkMonitorTrait, NetworkStats};
use crate::error::Result;
use std::sync::Arc;

/// Implementation of the NetworkMonitorTrait
#[derive(Debug)]
pub struct NetworkMonitorImpl {
    /// Network interface statistics
    stats: RwLock<HashMap<String, NetworkStats>>,
    /// Started flag
    started: bool,
    /// Stopped flag
    stopped: bool,
}

impl NetworkMonitorImpl {
    /// Create a new NetworkMonitorImpl
    pub fn new(_config: HashMap<String, String>) -> Self {
        NetworkMonitorImpl {
            stats: RwLock::new(HashMap::new()),
            started: false,
            stopped: false,
        }
    }

    /// Start the network monitor
    pub async fn start(&mut self) -> Result<()> {
        self.started = true;
        self.update_stats().await?;
        Ok(())
    }

    /// Stop the network monitor
    pub async fn stop(&mut self) -> Result<()> {
        self.stopped = true;
        Ok(())
    }

    /// Update the network stats
    async fn update_stats(&self) -> Result<()> {
        // Create a new Networks instance to get fresh network data
        let networks = Networks::new_with_refreshed_list();
        
        let mut stats = self.stats.write().unwrap();
        
        // Update network stats for each interface
        for (interface, network_data) in &networks {
            let interface_name = interface.to_string();
            
            let network_stats = stats.entry(interface_name.clone()).or_insert_with(|| NetworkStats {
                interface: interface_name.clone(),
                received_bytes: 0,
                transmitted_bytes: 0,
                received_packets: 0,
                transmitted_packets: 0,
                errors_on_received: 0,
                errors_on_transmitted: 0,
            });
            
            // Update the stats with the latest data
            network_stats.received_bytes = network_data.total_received();
            network_stats.transmitted_bytes = network_data.total_transmitted();
            
            // Note: sysinfo doesn't provide packet counts or error counts directly
            // We could potentially calculate these from other sources if needed
        }
        
        Ok(())
    }
}

#[async_trait]
impl NetworkMonitorTrait for NetworkMonitorImpl {
    async fn start(&mut self) -> Result<()> {
        self.started = true;
        self.update_stats().await?;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.stopped = true;
        Ok(())
    }

    async fn get_stats(&self) -> Result<Vec<NetworkStats>> {
        // Update stats before returning them
        self.update_stats().await?;
        
        // Clone the stats to return them
        let stats = self.stats.read().unwrap();
        Ok(stats.values().cloned().collect())
    }
    
    async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        // Update stats before returning them
        self.update_stats().await?;
        
        // Clone the stats to return them
        let stats = self.stats.read().unwrap();
        Ok(stats.get(interface).cloned())
    }
}

/// Factory for creating NetworkMonitorImpl instances
#[derive(Debug)]
pub struct NetworkMonitorFactoryImpl {
    config: HashMap<String, String>,
}

impl NetworkMonitorFactoryImpl {
    /// Create a new NetworkMonitorFactoryImpl with the given config
    pub fn new(config: HashMap<String, String>) -> Self {
        Self { config }
    }
    
    /// Create a new NetworkMonitorFactoryImpl with default config
    pub fn default_config() -> Self {
        Self::new(HashMap::new())
    }
}

impl crate::app::monitoring::network::NetworkMonitorFactory for NetworkMonitorFactoryImpl {
    fn create_monitor(&self) -> Arc<dyn NetworkMonitorTrait> {
        Arc::new(NetworkMonitorImpl::new(self.config.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::monitoring::network::NetworkMonitorTrait;

    #[tokio::test]
    async fn test_network_monitor() {
        let factory = NetworkMonitorFactoryImpl::default_config();
        let mut monitor = NetworkMonitorImpl::new(factory.config.clone());
        
        // Start the monitor
        monitor.start().await.unwrap();
        
        // Get stats
        let stats = monitor.get_stats().await.unwrap();
        
        // Verify that we got some network interfaces
        assert!(!stats.is_empty(), "No network interfaces found");
        
        // Stop the monitor
        monitor.stop().await.unwrap();
    }
} 