//! Health monitoring for MCP systems
//!
//! This module provides comprehensive health monitoring capabilities including
//! system resource tracking, sync health, persistence health, and overall health status.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use crate::error::Result;

/// Health status information for the MCP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the system is considered healthy overall
    pub is_healthy: bool,
    /// When the health status was last checked
    pub last_check: DateTime<Utc>,
    /// Synchronization health metrics
    pub sync_status: SyncHealth,
    /// Persistence layer health metrics
    pub persistence_status: PersistenceHealth,
    /// System resource usage health metrics
    pub resource_status: ResourceHealth,
}

/// Health metrics related to synchronization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealth {
    /// Whether a sync operation is currently in progress
    pub is_syncing: bool,
    /// When the last successful sync operation completed
    pub last_successful_sync: DateTime<Utc>,
    /// Number of consecutive sync failures
    pub consecutive_failures: u32,
}

/// Health metrics related to the persistence layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceHealth {
    /// Whether the storage is currently available
    pub storage_available: bool,
    /// When the last successful write operation completed
    pub last_write_success: DateTime<Utc>,
    /// Percentage of storage space used
    pub storage_usage_percent: f64,
}

/// Health metrics related to system resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
}

/// Health monitor for tracking system health status
#[derive(Debug)]
pub struct HealthMonitor {
    /// Current health status
    health_status: Arc<RwLock<HealthStatus>>,
    /// System information collector
    system_info: Arc<RwLock<System>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        let now = Utc::now();
        let initial_health = HealthStatus {
            is_healthy: true,
            last_check: now,
            sync_status: SyncHealth {
                is_syncing: false,
                last_successful_sync: now,
                consecutive_failures: 0,
            },
            persistence_status: PersistenceHealth {
                storage_available: true,
                last_write_success: now,
                storage_usage_percent: 0.0,
            },
            resource_status: ResourceHealth {
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_percent: 0.0,
            },
        };

        Self {
            health_status: Arc::new(RwLock::new(initial_health)),
            system_info: Arc::new(RwLock::new(System::new_all())),
        }
    }

    /// Update the overall health status
    pub async fn update_health(&self) -> Result<()> {
        let mut health = self.health_status.write().await;
        let mut system = self.system_info.write().await;
        
        // Refresh system information
        system.refresh_all();
        
        // Update resource health
        health.resource_status = self.collect_resource_metrics(&system);
        
        // Update overall health based on component health
        health.is_healthy = self.calculate_overall_health(&health);
        health.last_check = Utc::now();
        
        debug!("Health status updated: healthy={}, cpu={:.1}%, memory={:.1}%, disk={:.1}%",
               health.is_healthy,
               health.resource_status.cpu_usage_percent,
               health.resource_status.memory_usage_percent,
               health.resource_status.disk_usage_percent);
        
        Ok(())
    }

    /// Get the current health status
    pub async fn get_health(&self) -> HealthStatus {
        self.health_status.read().await.clone()
    }

    /// Update sync health status
    pub async fn update_sync_status(&self, is_syncing: bool, success: bool) {
        let mut health = self.health_status.write().await;
        
        health.sync_status.is_syncing = is_syncing;
        
        if success {
            health.sync_status.last_successful_sync = Utc::now();
            health.sync_status.consecutive_failures = 0;
        } else {
            health.sync_status.consecutive_failures += 1;
        }
        
        debug!("Sync health updated: syncing={}, consecutive_failures={}", 
               is_syncing, health.sync_status.consecutive_failures);
    }

    /// Update persistence health status
    pub async fn update_persistence_status(&self, available: bool, write_success: bool) {
        let mut health = self.health_status.write().await;
        
        health.persistence_status.storage_available = available;
        
        if write_success {
            health.persistence_status.last_write_success = Utc::now();
        }
        
        debug!("Persistence health updated: available={}, last_write_success={:?}", 
               available, health.persistence_status.last_write_success);
    }

    /// Check if the system is healthy
    pub async fn is_healthy(&self) -> bool {
        self.health_status.read().await.is_healthy
    }

    /// Get sync health status
    pub async fn get_sync_health(&self) -> SyncHealth {
        self.health_status.read().await.sync_status.clone()
    }

    /// Get persistence health status
    pub async fn get_persistence_health(&self) -> PersistenceHealth {
        self.health_status.read().await.persistence_status.clone()
    }

    /// Get resource health status
    pub async fn get_resource_health(&self) -> ResourceHealth {
        self.health_status.read().await.resource_status.clone()
    }

    /// Collect system resource metrics
    fn collect_resource_metrics(&self, system: &System) -> ResourceHealth {
        // Calculate CPU usage
        let cpu_usage = system.cpus().iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>() / system.cpus().len() as f64;

        // Calculate memory usage
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        // Calculate disk usage (use first disk for simplicity)
        let disk_usage = system.disks().first()
            .map(|disk| {
                let total_space = disk.total_space() as f64;
                let available_space = disk.available_space() as f64;
                if total_space > 0.0 {
                    ((total_space - available_space) / total_space) * 100.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        ResourceHealth {
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            disk_usage_percent: disk_usage,
        }
    }

    /// Calculate overall health status based on component health
    fn calculate_overall_health(&self, health: &HealthStatus) -> bool {
        let sync_healthy = health.sync_status.consecutive_failures < 5;
        let persistence_healthy = health.persistence_status.storage_available;
        let cpu_healthy = health.resource_status.cpu_usage_percent < 90.0;
        let memory_healthy = health.resource_status.memory_usage_percent < 90.0;
        let disk_healthy = health.resource_status.disk_usage_percent < 95.0;

        sync_healthy && persistence_healthy && cpu_healthy && memory_healthy && disk_healthy
    }

    /// Get a health summary string
    pub async fn get_health_summary(&self) -> String {
        let health = self.health_status.read().await;
        
        format!(
            "Health: {} | CPU: {:.1}% | Memory: {:.1}% | Disk: {:.1}% | Sync failures: {} | Storage: {}",
            if health.is_healthy { "✓" } else { "✗" },
            health.resource_status.cpu_usage_percent,
            health.resource_status.memory_usage_percent,
            health.resource_status.disk_usage_percent,
            health.sync_status.consecutive_failures,
            if health.persistence_status.storage_available { "Available" } else { "Unavailable" }
        )
    }

    /// Record a sync operation result
    pub async fn record_sync_result(&self, success: bool, duration_ms: f64) {
        if success {
            info!("Sync operation completed successfully in {:.2}ms", duration_ms);
        } else {
            error!("Sync operation failed after {:.2}ms", duration_ms);
        }
        
        self.update_sync_status(false, success).await;
    }

    /// Start monitoring a sync operation
    pub async fn start_sync_operation(&self) {
        info!("Starting sync operation");
        self.update_sync_status(true, true).await;
    }

    /// Check for health alerts that should be raised
    pub async fn check_health_alerts(&self) -> Vec<HealthAlert> {
        let health = self.health_status.read().await;
        let mut alerts = Vec::new();

        // Check CPU usage
        if health.resource_status.cpu_usage_percent > 90.0 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::HighCpuUsage,
                message: format!("High CPU usage: {:.1}%", health.resource_status.cpu_usage_percent),
                severity: if health.resource_status.cpu_usage_percent > 95.0 {
                    HealthAlertSeverity::Critical
                } else {
                    HealthAlertSeverity::Warning
                },
                timestamp: Utc::now(),
            });
        }

        // Check memory usage
        if health.resource_status.memory_usage_percent > 90.0 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::HighMemoryUsage,
                message: format!("High memory usage: {:.1}%", health.resource_status.memory_usage_percent),
                severity: if health.resource_status.memory_usage_percent > 95.0 {
                    HealthAlertSeverity::Critical
                } else {
                    HealthAlertSeverity::Warning
                },
                timestamp: Utc::now(),
            });
        }

        // Check disk usage
        if health.resource_status.disk_usage_percent > 90.0 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::HighDiskUsage,
                message: format!("High disk usage: {:.1}%", health.resource_status.disk_usage_percent),
                severity: if health.resource_status.disk_usage_percent > 98.0 {
                    HealthAlertSeverity::Critical
                } else {
                    HealthAlertSeverity::Warning
                },
                timestamp: Utc::now(),
            });
        }

        // Check sync failures
        if health.sync_status.consecutive_failures >= 5 {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::SyncFailures,
                message: format!("Multiple sync failures: {}", health.sync_status.consecutive_failures),
                severity: if health.sync_status.consecutive_failures >= 10 {
                    HealthAlertSeverity::Critical
                } else {
                    HealthAlertSeverity::Warning
                },
                timestamp: Utc::now(),
            });
        }

        // Check storage availability
        if !health.persistence_status.storage_available {
            alerts.push(HealthAlert {
                alert_type: HealthAlertType::StorageUnavailable,
                message: "Storage is unavailable".to_string(),
                severity: HealthAlertSeverity::Critical,
                timestamp: Utc::now(),
            });
        }

        alerts
    }
}

/// Health alert information
#[derive(Debug, Clone)]
pub struct HealthAlert {
    /// Type of health alert
    pub alert_type: HealthAlertType,
    /// Alert message
    pub message: String,
    /// Alert severity
    pub severity: HealthAlertSeverity,
    /// When the alert was generated
    pub timestamp: DateTime<Utc>,
}

/// Types of health alerts
#[derive(Debug, Clone)]
pub enum HealthAlertType {
    /// High CPU usage alert
    HighCpuUsage,
    /// High memory usage alert
    HighMemoryUsage,
    /// High disk usage alert
    HighDiskUsage,
    /// Sync operation failures
    SyncFailures,
    /// Storage unavailable
    StorageUnavailable,
    /// General system health issue
    SystemHealth,
}

/// Health alert severity levels
#[derive(Debug, Clone)]
pub enum HealthAlertSeverity {
    /// Informational alert
    Info,
    /// Warning level alert
    Warning,
    /// Critical alert requiring immediate attention
    Critical,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            is_healthy: true,
            last_check: now,
            sync_status: SyncHealth::default(),
            persistence_status: PersistenceHealth::default(),
            resource_status: ResourceHealth::default(),
        }
    }
}

impl Default for SyncHealth {
    fn default() -> Self {
        Self {
            is_syncing: false,
            last_successful_sync: Utc::now(),
            consecutive_failures: 0,
        }
    }
}

impl Default for PersistenceHealth {
    fn default() -> Self {
        Self {
            storage_available: true,
            last_write_success: Utc::now(),
            storage_usage_percent: 0.0,
        }
    }
}

impl Default for ResourceHealth {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            disk_usage_percent: 0.0,
        }
    }
}

impl std::fmt::Display for HealthAlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthAlertSeverity::Info => write!(f, "INFO"),
            HealthAlertSeverity::Warning => write!(f, "WARNING"),
            HealthAlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::fmt::Display for HealthAlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthAlertType::HighCpuUsage => write!(f, "High CPU Usage"),
            HealthAlertType::HighMemoryUsage => write!(f, "High Memory Usage"),
            HealthAlertType::HighDiskUsage => write!(f, "High Disk Usage"),
            HealthAlertType::SyncFailures => write!(f, "Sync Failures"),
            HealthAlertType::StorageUnavailable => write!(f, "Storage Unavailable"),
            HealthAlertType::SystemHealth => write!(f, "System Health"),
        }
    }
} 