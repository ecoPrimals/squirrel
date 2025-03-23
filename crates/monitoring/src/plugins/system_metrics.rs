//! System metrics plugin implementation

use crate::plugins::common::{MonitoringPlugin, PluginMetadata};
use async_trait::async_trait;
use serde_json::Value;
use sysinfo::{System, Disks};
use std::fmt::Debug;
use tracing::info;

/// System metrics plugin for monitoring system resources
#[derive(Debug)]
pub struct SystemMetricsPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// System info
    system: System,
}

impl SystemMetricsPlugin {
    /// Create a new system metrics plugin
    #[must_use]
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "System Metrics Plugin",
            "1.0.0",
            "Monitors system resources including CPU, memory, disk, and processes",
            "DataScienceBioLab",
        )
        .with_capability("metrics.system")
        .with_capability("metrics.cpu")
        .with_capability("metrics.memory")
        .with_capability("metrics.disk")
        .with_capability("metrics.processes");
        
        Self {
            metadata,
            system: System::new_all(),
        }
    }
    
    /// Update system metrics
    fn update_metrics(&mut self) {
        // Refresh all system information
        self.system.refresh_all();
    }
    
    /// Get CPU metrics
    fn get_cpu_metrics(&self) -> Value {
        // Return CPU usage as a percentage
        let global_cpu_info = self.system.global_cpu_info();
        let cpu_usage = global_cpu_info.cpu_usage();
        
        serde_json::json!({
            "usage_percent": cpu_usage,
            "count": self.system.cpus().len(),
        })
    }
    
    /// Get memory metrics
    fn get_memory_metrics(&self) -> Value {
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();
        
        serde_json::json!({
            "total_kb": total_memory,
            "used_kb": used_memory,
            "free_kb": total_memory - used_memory,
            "usage_percent": if total_memory > 0 { 
                (used_memory as f64 / total_memory as f64) * 100.0 
            } else { 
                0.0 
            },
            "swap_total_kb": total_swap,
            "swap_used_kb": used_swap,
            "swap_usage_percent": if total_swap > 0 { 
                (used_swap as f64 / total_swap as f64) * 100.0 
            } else { 
                0.0 
            },
        })
    }
    
    /// Get disk metrics
    fn get_disk_metrics(&self) -> Value {
        let mut disks = Vec::new();
        
        // Create a Disks object to get disk information
        let disks_info = Disks::new_with_refreshed_list();
        
        for disk in &disks_info {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total - free;
            
            disks.push(serde_json::json!({
                "name": disk.name().to_string_lossy(),
                "mount_point": disk.mount_point().to_string_lossy(),
                "total_bytes": total,
                "used_bytes": used,
                "free_bytes": free,
                "usage_percent": if total > 0 { 
                    (used as f64 / total as f64) * 100.0 
                } else { 
                    0.0 
                },
            }));
        }
        
        serde_json::json!({ "disks": disks })
    }
    
    /// Get process metrics
    fn get_process_metrics(&self) -> Value {
        let process_count = self.system.processes().len();
        
        serde_json::json!({
            "count": process_count,
        })
    }
    
    /// Collect all system metrics
    pub async fn collect_all_metrics(&self) -> anyhow::Result<Value> {
        // Update system metrics (requires mutable reference)
        // This is a limitation of the current design that we need to work around
        let mut plugin = SystemMetricsPlugin {
            metadata: self.metadata.clone(),
            system: System::new_all(),
        };
        plugin.update_metrics();
        
        // Collect all system metrics
        let metrics = serde_json::json!({
            "cpu": plugin.get_cpu_metrics(),
            "memory": plugin.get_memory_metrics(),
            "disk": plugin.get_disk_metrics(),
            "processes": plugin.get_process_metrics(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(metrics)
    }
}

#[async_trait]
impl MonitoringPlugin for SystemMetricsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing System Metrics Plugin");
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down System Metrics Plugin");
        Ok(())
    }
    
    async fn collect_metrics(&self) -> anyhow::Result<Value> {
        // Update system metrics (requires mutable reference)
        // This is a limitation of the current design that we need to work around
        let mut plugin = SystemMetricsPlugin {
            metadata: self.metadata.clone(),
            system: System::new_all(),
        };
        plugin.update_metrics();
        
        // Collect all system metrics
        let metrics = serde_json::json!({
            "cpu": plugin.get_cpu_metrics(),
            "memory": plugin.get_memory_metrics(),
            "disk": plugin.get_disk_metrics(),
            "processes": plugin.get_process_metrics(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(metrics)
    }
    
    fn get_monitoring_targets(&self) -> Vec<String> {
        vec![
            "system".to_string(),
            "cpu".to_string(),
            "memory".to_string(),
            "disk".to_string(),
            "processes".to_string(),
        ]
    }
    
    async fn handle_alert(&self, _alert: Value) -> anyhow::Result<()> {
        // System metrics plugin doesn't handle alerts directly
        Ok(())
    }
} 