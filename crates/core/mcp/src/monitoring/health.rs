// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Health monitoring for MCP systems
//!
//! Provides comprehensive health monitoring without external C dependencies.
//! On Linux, reads /proc for real system metrics. On other platforms, returns defaults.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use std::sync::Arc;
use crate::error::Result;

/// Health status information for the MCP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub sync_status: SyncHealth,
    pub persistence_status: PersistenceHealth,
    pub resource_status: ResourceHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealth {
    pub is_syncing: bool,
    pub last_successful_sync: DateTime<Utc>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceHealth {
    pub storage_available: bool,
    pub last_write_success: DateTime<Utc>,
    pub storage_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
}

/// Health monitor using /proc reads on Linux (Pure Rust, no sysinfo)
#[derive(Debug)]
pub struct HealthMonitor {
    health_status: Arc<RwLock<HealthStatus>>,
}

impl HealthMonitor {
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
        }
    }

    pub async fn update_health(&self) -> Result<()> {
        let mut health = self.health_status.write().await;

        health.resource_status = collect_resource_metrics();
        health.is_healthy = calculate_overall_health(&health);
        health.last_check = Utc::now();

        debug!("Health status updated: healthy={}, cpu={:.1}%, memory={:.1}%, disk={:.1}%",
               health.is_healthy,
               health.resource_status.cpu_usage_percent,
               health.resource_status.memory_usage_percent,
               health.resource_status.disk_usage_percent);

        Ok(())
    }

    pub async fn get_health(&self) -> HealthStatus {
        self.health_status.read().await.clone()
    }

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

    pub async fn update_persistence_status(&self, available: bool, write_success: bool) {
        let mut health = self.health_status.write().await;
        health.persistence_status.storage_available = available;
        if write_success {
            health.persistence_status.last_write_success = Utc::now();
        }
        debug!("Persistence health updated: available={}, last_write_success={:?}",
               available, health.persistence_status.last_write_success);
    }

    pub async fn is_healthy(&self) -> bool {
        self.health_status.read().await.is_healthy
    }

    pub async fn get_sync_health(&self) -> SyncHealth {
        self.health_status.read().await.sync_status.clone()
    }

    pub async fn get_persistence_health(&self) -> PersistenceHealth {
        self.health_status.read().await.persistence_status.clone()
    }

    pub async fn get_resource_health(&self) -> ResourceHealth {
        self.health_status.read().await.resource_status.clone()
    }

    pub async fn get_health_summary(&self) -> String {
        let health = self.health_status.read().await;
        format!(
            "Health: {} | CPU: {:.1}% | Memory: {:.1}% | Disk: {:.1}% | Sync failures: {} | Storage: {}",
            if health.is_healthy { "OK" } else { "DEGRADED" },
            health.resource_status.cpu_usage_percent,
            health.resource_status.memory_usage_percent,
            health.resource_status.disk_usage_percent,
            health.sync_status.consecutive_failures,
            if health.persistence_status.storage_available { "Available" } else { "Unavailable" }
        )
    }

    pub async fn record_sync_result(&self, success: bool, duration_ms: f64) {
        if success {
            info!("Sync operation completed successfully in {:.2}ms", duration_ms);
        } else {
            error!("Sync operation failed after {:.2}ms", duration_ms);
        }
        self.update_sync_status(false, success).await;
    }

    pub async fn start_sync_operation(&self) {
        info!("Starting sync operation");
        self.update_sync_status(true, true).await;
    }

    pub async fn check_health_alerts(&self) -> Vec<HealthAlert> {
        let health = self.health_status.read().await;
        let mut alerts = Vec::new();

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

/// Collect resource metrics from /proc on Linux, defaults on other platforms
fn collect_resource_metrics() -> ResourceHealth {
    ResourceHealth {
        cpu_usage_percent: read_cpu_usage(),
        memory_usage_percent: read_memory_usage(),
        disk_usage_percent: 0.0,
    }
}

#[cfg(target_os = "linux")]
fn read_memory_usage() -> f64 {
    let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") else {
        return 0.0;
    };
    let mut total = 0u64;
    let mut available = 0u64;
    for line in meminfo.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            total = val.trim().split_whitespace().next()
                .and_then(|v| v.parse().ok()).unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("MemAvailable:") {
            available = val.trim().split_whitespace().next()
                .and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
    if total > 0 { ((total - available) as f64 / total as f64) * 100.0 } else { 0.0 }
}

#[cfg(not(target_os = "linux"))]
fn read_memory_usage() -> f64 { 0.0 }

#[cfg(target_os = "linux")]
fn read_cpu_usage() -> f64 {
    let Ok(stat) = std::fs::read_to_string("/proc/stat") else { return 0.0 };
    let Some(line) = stat.lines().next() else { return 0.0 };
    let vals: Vec<u64> = line.split_whitespace().skip(1)
        .filter_map(|v| v.parse().ok()).collect();
    if vals.len() < 4 { return 0.0; }
    let idle = vals[3];
    let total: u64 = vals.iter().sum();
    if total > 0 { ((total - idle) as f64 / total as f64) * 100.0 } else { 0.0 }
}

#[cfg(not(target_os = "linux"))]
fn read_cpu_usage() -> f64 { 0.0 }

fn calculate_overall_health(health: &HealthStatus) -> bool {
    let sync_healthy = health.sync_status.consecutive_failures < 5;
    let persistence_healthy = health.persistence_status.storage_available;
    let cpu_healthy = health.resource_status.cpu_usage_percent < 90.0;
    let memory_healthy = health.resource_status.memory_usage_percent < 90.0;
    let disk_healthy = health.resource_status.disk_usage_percent < 95.0;
    sync_healthy && persistence_healthy && cpu_healthy && memory_healthy && disk_healthy
}

#[derive(Debug, Clone)]
pub struct HealthAlert {
    pub alert_type: HealthAlertType,
    pub message: String,
    pub severity: HealthAlertSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum HealthAlertType {
    HighCpuUsage,
    HighMemoryUsage,
    HighDiskUsage,
    SyncFailures,
    StorageUnavailable,
    SystemHealth,
}

#[derive(Debug, Clone)]
pub enum HealthAlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for HealthMonitor {
    fn default() -> Self { Self::new() }
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
        Self { is_syncing: false, last_successful_sync: Utc::now(), consecutive_failures: 0 }
    }
}

impl Default for PersistenceHealth {
    fn default() -> Self {
        Self { storage_available: true, last_write_success: Utc::now(), storage_usage_percent: 0.0 }
    }
}

impl Default for ResourceHealth {
    fn default() -> Self {
        Self { cpu_usage_percent: 0.0, memory_usage_percent: 0.0, disk_usage_percent: 0.0 }
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
