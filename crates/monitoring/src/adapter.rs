use std::sync::Arc;
use std::collections::HashMap;
use sysinfo::{System, SystemExt, DiskExt, CpuExt, NetworkExt};
use chrono::Utc;

use crate::MonitoringService;
use crate::MonitoringConfig;
use crate::MonitoringServiceFactory;
use crate::MonitoringError;

/// Adapter for monitoring service factory
/// 
/// This adapter wraps a monitoring service factory to provide a consistent interface
/// and additional functionality while delegating to the underlying implementation.
pub struct MonitoringServiceFactoryAdapter {
    /// Underlying factory instance that will be used to create monitoring services
    /// The adapter delegates creation requests to this inner factory.
    pub inner: Option<Arc<dyn MonitoringServiceFactory>>,
}

impl std::fmt::Debug for MonitoringServiceFactoryAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonitoringServiceFactoryAdapter")
            .field("inner", &if self.inner.is_some() { "Some(MonitoringServiceFactory)" } else { "None" })
            .finish()
    }
}

impl MonitoringServiceFactoryAdapter {
    /// Creates a new factory adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing factory
    #[must_use]
    pub fn with_factory(factory: Arc<dyn MonitoringServiceFactory>) -> Self {
        Self {
            inner: Some(factory),
        }
    }

    /// Creates a service with the default configuration
    pub async fn create_service(&self) -> Result<Arc<dyn MonitoringService>, MonitoringError> {
        let config = MonitoringConfig::default();
        if let Some(factory) = &self.inner {
            match factory.create_service(config).await {
                Ok(service) => Ok(service),
                Err(e) => Err(MonitoringError::SystemError(format!("Factory error: {}", e)))
            }
        } else {
            Err(MonitoringError::SystemError("No factory configured".to_string()))
        }
    }

    /// Creates a service with a custom configuration
    pub async fn create_service_with_config(&self, config: MonitoringConfig) -> Result<Arc<dyn MonitoringService>, MonitoringError> {
        if let Some(factory) = &self.inner {
            match factory.create_service(config).await {
                Ok(service) => Ok(service),
                Err(e) => Err(MonitoringError::SystemError(format!("Factory error: {}", e)))
            }
        } else {
            Err(MonitoringError::SystemError("No factory configured".to_string()))
        }
    }
}

impl Default for MonitoringServiceFactoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new monitoring service factory adapter
#[must_use]
pub fn create_factory_adapter() -> Arc<MonitoringServiceFactoryAdapter> {
    Arc::new(MonitoringServiceFactoryAdapter::new())
}

/// Creates a new monitoring service factory adapter with an existing factory
#[must_use]
pub fn create_factory_adapter_with_factory(
    factory: Arc<dyn MonitoringServiceFactory>
) -> Arc<MonitoringServiceFactoryAdapter> {
    Arc::new(MonitoringServiceFactoryAdapter::with_factory(factory))
}

/// Resource metrics collector adapter for system monitoring
#[derive(Debug)]
pub struct ResourceMetricsCollectorAdapter {
    system: System,
}

impl ResourceMetricsCollectorAdapter {
    /// Create a new resource metrics collector adapter
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ResourceMetricsCollectorAdapter {
            system,
        }
    }
    
    /// Refresh system data
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// Get CPU usage as a percentage
    pub fn cpu_usage(&mut self) -> f32 {
        self.refresh();
        self.system.global_cpu_info().cpu_usage()
    }
    
    /// Get memory usage in bytes
    pub fn memory_used(&mut self) -> u64 {
        self.refresh();
        self.system.used_memory()
    }
    
    /// Get total memory in bytes
    pub fn memory_total(&mut self) -> u64 {
        self.refresh();
        self.system.total_memory()
    }
    
    /// Get memory usage as a percentage
    pub fn memory_usage_percent(&mut self) -> f64 {
        let used = self.memory_used() as f64;
        let total = self.memory_total() as f64;
        
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get disk usage information
    pub fn disk_usage(&mut self) -> (u64, u64) {
        self.refresh();
        self.system.refresh_disks_list();
        
        let mut used = 0;
        let mut total = 0;
        
        for disk in self.system.disks() {
            total += disk.total_space();
            used += disk.total_space() - disk.available_space();
        }
        
        (used, total)
    }
    
    /// Get disk usage as a percentage
    pub fn disk_usage_percent(&mut self) -> f64 {
        let (used, total) = self.disk_usage();
        
        if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get detailed disk information
    pub fn disk_details(&mut self) -> Vec<DiskInfo> {
        self.refresh();
        self.system.refresh_disks_list();
        
        let mut disks = Vec::new();
        
        for disk in self.system.disks() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let used_percentage = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            disks.push(DiskInfo {
                mount_point,
                device: disk.name().to_string_lossy().to_string(),
                total,
                used,
                available,
                used_percentage,
            });
        }
        
        disks
    }
    
    /// Get network usage information
    pub fn network_usage(&mut self) -> (u64, u64, u64, u64) {
        self.refresh();
        self.system.refresh_networks();
        
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        let mut rx_packets = 0;
        let mut tx_packets = 0;
        
        for (_name, network) in self.system.networks() {
            rx_bytes += network.received();
            tx_bytes += network.transmitted();
            rx_packets += network.packets_received();
            tx_packets += network.packets_transmitted();
        }
        
        (rx_bytes, tx_bytes, rx_packets, tx_packets)
    }
    
    /// Get detailed network interface information
    pub fn network_details(&mut self) -> HashMap<String, NetworkInfo> {
        self.refresh();
        self.system.refresh_networks();
        
        let mut interfaces = HashMap::new();
        
        for (name, network) in self.system.networks() {
            interfaces.insert(name.clone(), NetworkInfo {
                name: name.clone(),
                rx_bytes: network.received(),
                tx_bytes: network.transmitted(),
                rx_packets: network.packets_received(),
                tx_packets: network.packets_transmitted(),
            });
        }
        
        interfaces
    }
    
    /// Get CPU usage per core
    pub fn cpu_per_core_usage(&mut self) -> Vec<f64> {
        self.refresh();
        self.system.refresh_cpu();
        
        let mut core_usage = Vec::new();
        for cpu in self.system.cpus() {
            core_usage.push(f64::from(cpu.cpu_usage()));
        }
        
        core_usage
    }
    
    /// Generate health alerts based on system metrics
    pub fn health_alerts(&mut self) -> Vec<HealthAlert> {
        self.refresh();
        
        let mut alerts = Vec::new();
        
        // Check CPU usage
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        if cpu_usage > 90.0 {
            alerts.push(HealthAlert {
                id: format!("cpu-high-{}", Utc::now().timestamp()),
                title: "High CPU Usage".to_string(),
                description: format!("CPU usage is at {:.1}%", cpu_usage),
                severity: AlertSeverity::Critical,
                source: "system.cpu".to_string(),
                timestamp: Utc::now(),
            });
        } else if cpu_usage > 75.0 {
            alerts.push(HealthAlert {
                id: format!("cpu-warn-{}", Utc::now().timestamp()),
                title: "Elevated CPU Usage".to_string(),
                description: format!("CPU usage is at {:.1}%", cpu_usage),
                severity: AlertSeverity::Warning,
                source: "system.cpu".to_string(),
                timestamp: Utc::now(),
            });
        }
        
        // Check memory usage
        let memory_percent = self.memory_usage_percent();
        
        if memory_percent > 90.0 {
            alerts.push(HealthAlert {
                id: format!("memory-high-{}", Utc::now().timestamp()),
                title: "High Memory Usage".to_string(),
                description: format!("Memory usage is at {:.1}%", memory_percent),
                severity: AlertSeverity::Critical,
                source: "system.memory".to_string(),
                timestamp: Utc::now(),
            });
        } else if memory_percent > 80.0 {
            alerts.push(HealthAlert {
                id: format!("memory-warn-{}", Utc::now().timestamp()),
                title: "Elevated Memory Usage".to_string(),
                description: format!("Memory usage is at {:.1}%", memory_percent),
                severity: AlertSeverity::Warning,
                source: "system.memory".to_string(),
                timestamp: Utc::now(),
            });
        }
        
        alerts
    }
}

// Implement Clone manually since System doesn't implement Clone
impl Clone for ResourceMetricsCollectorAdapter {
    fn clone(&self) -> Self {
        // Create a new instance with a fresh System
        Self::new()
    }
}

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub used_percentage: f64,
}

/// Network interface information structure
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

/// Health alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Health alert structure
#[derive(Debug, Clone)]
pub struct HealthAlert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub source: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_usage() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let usage = collector.cpu_usage();
        
        assert!(usage >= 0.0 && usage <= 100.0);
    }
    
    #[test]
    fn test_memory_usage() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let used = collector.memory_used();
        let total = collector.memory_total();
        let percent = collector.memory_usage_percent();
        
        assert!(used <= total);
        assert!(percent >= 0.0 && percent <= 100.0);
    }
    
    #[test]
    fn test_disk_usage() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let (used, total) = collector.disk_usage();
        let percent = collector.disk_usage_percent();
        
        assert!(used <= total);
        assert!(percent >= 0.0 && percent <= 100.0);
    }
    
    #[test]
    fn test_disk_details() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let disks = collector.disk_details();
        
        // Should have at least one disk
        assert!(!disks.is_empty());
        
        // Verify disk metrics
        for disk in &disks {
            assert!(disk.used <= disk.total);
            assert!(disk.used_percentage >= 0.0 && disk.used_percentage <= 100.0);
            assert!(!disk.mount_point.is_empty());
            assert!(!disk.device.is_empty());
        }
    }
    
    #[test]
    fn test_network_usage() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let (rx_bytes, tx_bytes, rx_packets, tx_packets) = collector.network_usage();
        
        assert!(rx_bytes >= 0);
        assert!(tx_bytes >= 0);
        assert!(rx_packets >= 0);
        assert!(tx_packets >= 0);
    }
    
    #[test]
    fn test_cpu_per_core_usage() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let cpu_metrics = collector.cpu_per_core_usage();
        
        // Should have at least one CPU core
        assert!(!cpu_metrics.is_empty());
        
        // Verify CPU metrics
        for usage in cpu_metrics {
            assert!(usage >= 0.0 && usage <= 100.0);
        }
    }
    
    #[test]
    fn test_health_alerts() {
        let mut collector = ResourceMetricsCollectorAdapter::new();
        let alerts = collector.health_alerts();
        
        // Alerts may or may not be present depending on system load
        for alert in alerts {
            // Verify alert structure
            assert!(!alert.id.is_empty());
            assert!(!alert.title.is_empty());
            assert!(!alert.description.is_empty());
            assert!(!alert.source.is_empty());
            
            // Verify alert is for CPU or memory
            assert!(alert.source == "system.cpu" || alert.source == "system.memory");
            
            // Verify severity
            assert!(alert.severity == AlertSeverity::Warning || 
                   alert.severity == AlertSeverity::Critical);
        }
    }
} 