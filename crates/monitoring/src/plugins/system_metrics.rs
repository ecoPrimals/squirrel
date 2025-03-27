//! System metrics plugin implementation

use crate::plugins::common::{MonitoringPlugin, PluginMetadata};
use async_trait::async_trait;
use serde_json::Value;
use squirrel_core::error::Result;
use sysinfo::{System, SystemExt, DiskExt, CpuExt, NetworkExt, NetworksExt};
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

impl Default for SystemMetricsPlugin {
    fn default() -> Self {
        Self::new()
    }
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
        let system = &self.system;

        // Update CPU usage calculation
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        serde_json::json!({
            "usage_percent": cpu_usage,
            "count": system.cpus().len(),
        })
    }
    
    /// Get memory metrics
    fn get_memory_metrics(&self) -> Value {
        let system = &self.system;

        // Update memory usage calculation
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
        
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();
        
        serde_json::json!({
            "total_kb": total_memory,
            "used_kb": used_memory,
            "free_kb": total_memory - used_memory,
            "usage_percent": memory_usage,
            "swap_total_kb": total_swap,
            "swap_used_kb": used_swap,
            "swap_usage_percent": if total_swap > 0 { 
                (used_swap as f64 / total_swap as f64) * 100.0 
            } else { 
                0.0 
            },
        })
    }
    
    /// Get disk usage
    fn get_disk_usage(&self) -> Value {
        let mut disk_info = Vec::new();
        
        for disk in self.system.disks() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };
            
            disk_info.push(serde_json::json!({
                "name": disk.name().to_string_lossy(),
                "mount_point": disk.mount_point().to_string_lossy(),
                "total_bytes": total_space,
                "available_bytes": available_space,
                "used_bytes": used_space,
                "usage_percent": usage_percent
            }));
        }
        
        serde_json::json!({
            "disks": disk_info
        })
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
            "disk": plugin.get_disk_usage(),
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
            "disk": plugin.get_disk_usage(),
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