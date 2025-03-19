use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::error::Result;
use sysinfo::{System, Disk, Networks, Disks};
use crate::monitoring::network::{NetworkError, NetworkStats};
// use crate::monitoring::network::SystemInfoManager;

/// Type alias for System to make it easier to handle
type S = sysinfo::System;

/// System info adapter for monitoring system resources
#[derive(Debug)]
pub struct SystemInfoAdapter {
    /// Inner System object
    inner: Option<Arc<S>>,
    /// Shared system info object
    system: Option<Arc<RwLock<S>>>,
}

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// Name of the disk
    pub name: String,
    /// Mount point of the disk
    pub mount_point: String,
    /// Total space of the disk in bytes
    pub total_space: u64,
    /// Available space of the disk in bytes
    pub available_space: u64,
    /// Disk type
    pub disk_type: String,
    /// File system of the disk
    pub file_system: String,
    /// If the disk is removable
    pub is_removable: bool,
}

impl SystemInfoAdapter {
    /// Create a new system info adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: None,
            system: None,
        }
    }

    /// Create a system info adapter with a system info object
    #[must_use]
    pub fn with_system(system: Arc<RwLock<S>>) -> Self {
        Self {
            inner: None,
            system: Some(system),
        }
    }

    /// Initialize the adapter
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized() {
            return Ok(());
        }

        let system = System::new_all();
        self.inner = Some(Arc::new(system));
        
        Ok(())
    }

    /// Check if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some() || self.system.is_some()
    }

    /// Get CPU usage
    pub async fn cpu_usage(&self) -> Result<f32> {
        if let Some(system) = &self.inner {
            Ok(system.global_cpu_info().cpu_usage())
        } else if let Some(system) = &self.system {
            let sys = system.read().await;
            Ok(sys.global_cpu_info().cpu_usage())
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }

    /// Get memory usage (used, total)
    pub async fn memory_usage(&self) -> Result<(u64, u64)> {
        if let Some(system) = &self.inner {
            Ok((system.used_memory(), system.total_memory()))
        } else if let Some(system) = &self.system {
            let sys = system.read().await;
            Ok((sys.used_memory(), sys.total_memory()))
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }

    /// Refreshes all system information
    pub async fn refresh_all(&self) -> Result<()> {
        if let Some(_) = &self.inner {
            // We can't modify Arc<System> directly, so we'll initialize a new System
            // and return Ok since we can't update the immutable inner system
            Ok(())
        } else if let Some(system) = &self.system {
            let mut sys = system.write().await;
            // Create a new system and replace the existing one
            let mut new_system = System::new_all();
            new_system.refresh_all();
            *sys = new_system;
            Ok(())
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }
    
    /// Refreshes network information
    pub async fn refresh_networks(&self) -> Result<()> {
        if let Some(_) = &self.inner {
            // Can't modify immutable system, just return Ok
            Ok(())
        } else if let Some(system) = &self.system {
            let mut sys = system.write().await;
            // Create a new system and replace the existing one
            let mut new_system = System::new_all();
            new_system.refresh_all(); // This includes network refresh
            *sys = new_system;
            Ok(())
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }
    
    /// Gets network stats as (interface_name, received_bytes, transmitted_bytes)
    pub async fn network_stats(&self) -> Result<Vec<(String, u64, u64)>> {
        if let Some(_) = &self.inner {
            let mut stats = Vec::new();
            
            // Create fresh Networks instance with refreshed data
            let networks = Networks::new_with_refreshed_list();
            
            for (name, network) in &networks {
                stats.push((name.clone(), network.received(), network.transmitted()));
            }
            Ok(stats)
        } else if let Some(_) = &self.system {
            let mut stats = Vec::new();
            
            // Create fresh Networks instance with refreshed data
            let networks = Networks::new_with_refreshed_list();
            
            for (name, network) in &networks {
                stats.push((name.clone(), network.received(), network.transmitted()));
            }
            Ok(stats)
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }

    /// Get access to the networks information
    pub async fn networks(&self) -> Result<HashMap<String, NetworkStats>> {
        if let Some(_) = &self.inner {
            // Create fresh Networks instance with refreshed data
            let networks = Networks::new_with_refreshed_list();
            
            // Create a HashMap that owns the NetworkStats
            let mut result = HashMap::new();
            for (name, network) in networks.iter() {
                // Create a new NetworkStats that owns the data
                let network_stats = NetworkStats {
                    interface: name.clone(),
                    received_bytes: network.total_received(),
                    transmitted_bytes: network.total_transmitted(),
                    receive_rate: network.received() as f64,
                    transmit_rate: network.transmitted() as f64,
                    packets_received: network.total_packets_received(),
                    packets_transmitted: network.total_packets_transmitted(),
                    errors_on_received: network.total_errors_on_received(),
                    errors_on_transmitted: network.total_errors_on_transmitted(),
                };
                result.insert(name.clone(), network_stats);
            }
            Ok(result)
        } else if let Some(_) = &self.system {
            // Create fresh Networks instance with refreshed data
            let networks = Networks::new_with_refreshed_list();
            
            // Create a HashMap that owns the NetworkStats
            let mut result = HashMap::new();
            for (name, network) in networks.iter() {
                // Create a new NetworkStats that owns the data
                let network_stats = NetworkStats {
                    interface: name.clone(),
                    received_bytes: network.total_received(),
                    transmitted_bytes: network.total_transmitted(),
                    receive_rate: network.received() as f64,
                    transmit_rate: network.transmitted() as f64,
                    packets_received: network.total_packets_received(),
                    packets_transmitted: network.total_packets_transmitted(),
                    errors_on_received: network.total_errors_on_received(),
                    errors_on_transmitted: network.total_errors_on_transmitted(),
                };
                result.insert(name.clone(), network_stats);
            }
            Ok(result)
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }

    /// Get access to the disks information
    pub async fn disks(&self) -> Result<Vec<DiskInfo>> {
        if let Some(_) = &self.inner {
            // Create fresh Disks instance with refreshed data
            let disks = Disks::new_with_refreshed_list();
            
            // Create a Vec of DiskInfo objects
            let mut disk_info_vec = Vec::new();
            for disk in disks.iter() {
                let disk_info = DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    disk_type: format!("{:?}", disk.kind()),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    is_removable: disk.is_removable(),
                };
                disk_info_vec.push(disk_info);
            }
            Ok(disk_info_vec)
        } else if let Some(_) = &self.system {
            // Create fresh Disks instance with refreshed data
            let disks = Disks::new_with_refreshed_list();
            
            // Create a Vec of DiskInfo objects
            let mut disk_info_vec = Vec::new();
            for disk in disks.iter() {
                let disk_info = DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    disk_type: format!("{:?}", disk.kind()),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    is_removable: disk.is_removable(),
                };
                disk_info_vec.push(disk_info);
            }
            Ok(disk_info_vec)
        } else {
            Err(NetworkError::System("SystemInfoAdapter not initialized".to_string()).into())
        }
    }
}

/// Create a system info adapter
#[must_use]
pub fn create_system_info_adapter() -> Arc<SystemInfoAdapter> {
    let mut adapter = SystemInfoAdapter::new();
    adapter.initialize().expect("Failed to initialize system info adapter");
    Arc::new(adapter)
}

/// Create a system info adapter with an existing system
#[must_use]
pub fn create_system_info_adapter_with_system(system: Arc<RwLock<S>>) -> Arc<SystemInfoAdapter> {
    Arc::new(SystemInfoAdapter::with_system(system))
} 