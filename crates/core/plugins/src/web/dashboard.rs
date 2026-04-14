// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Management Dashboard
//!
//! This module provides web dashboard components for plugin management including:
//! - Plugin overview and metrics
//! - Real-time status monitoring
//! - Installation and configuration management
//! - Marketplace integration
//! - System health monitoring

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::PluginStatus;
use crate::web::{
    ComponentType, HttpMethod, HttpStatus, WebComponent, WebEndpoint, WebRequest, WebResponse,
};
use crate::{DefaultPluginManager, PluginManagerTrait, PluginRegistry};

pub use super::dashboard_types::{
    ActivityItem, Alert, AlertAction, AlertLevel, CpuUsage, DashboardConfig, DashboardOverview,
    DashboardTheme, DiskUsage, HealthStatus, MemoryUsage, NetworkStats, PluginStatistics,
    QuickAction, SystemHealth,
};

/// Plugin management dashboard component
#[derive(Clone)]
pub struct PluginDashboard {
    manager: Arc<DefaultPluginManager>,
    config: Arc<RwLock<DashboardConfig>>,
    started_at: Instant,
    dismissed_alerts: Arc<RwLock<HashSet<Uuid>>>,
}

impl PluginDashboard {
    /// Create a new plugin dashboard
    pub fn new(manager: Arc<DefaultPluginManager>) -> Self {
        Self {
            manager,
            config: Arc::new(RwLock::new(DashboardConfig::default())),
            started_at: Instant::now(),
            dismissed_alerts: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create dashboard with custom configuration
    pub fn with_config(manager: Arc<DefaultPluginManager>, config: DashboardConfig) -> Self {
        Self {
            manager,
            config: Arc::new(RwLock::new(config)),
            started_at: Instant::now(),
            dismissed_alerts: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Get dashboard endpoints
    #[must_use]
    pub fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/overview".to_string(),
                HttpMethod::Get,
                "Get dashboard overview data".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/stats".to_string(),
                HttpMethod::Get,
                "Get plugin statistics".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/health".to_string(),
                HttpMethod::Get,
                "Get system health status".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/activities".to_string(),
                HttpMethod::Get,
                "Get recent activities".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/alerts".to_string(),
                HttpMethod::Get,
                "Get active alerts".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/alerts/:id/dismiss".to_string(),
                HttpMethod::Post,
                "Dismiss an alert".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/config".to_string(),
                HttpMethod::Get,
                "Get dashboard configuration".to_string(),
            )
            .with_tag("dashboard"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/dashboard/config".to_string(),
                HttpMethod::Put,
                "Update dashboard configuration".to_string(),
            )
            .with_tag("dashboard"),
        ]
    }

    /// Get dashboard web components
    #[must_use]
    pub fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                Uuid::new_v4(),
                "plugin-dashboard".to_string(),
                "Main plugin management dashboard".to_string(),
                ComponentType::Widget,
            ),
            WebComponent::new(
                Uuid::new_v4(),
                "plugin-stats-widget".to_string(),
                "Widget showing plugin statistics".to_string(),
                ComponentType::Widget,
            ),
            WebComponent::new(
                Uuid::new_v4(),
                "system-health-widget".to_string(),
                "Widget showing system health status".to_string(),
                ComponentType::Widget,
            ),
            WebComponent::new(
                Uuid::new_v4(),
                "recent-activities-widget".to_string(),
                "Widget showing recent plugin activities".to_string(),
                ComponentType::Widget,
            ),
            WebComponent::new(
                Uuid::new_v4(),
                "alerts-widget".to_string(),
                "Widget showing system alerts".to_string(),
                ComponentType::Widget,
            ),
        ]
    }

    /// Handle dashboard request
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if routing, deserialization, or handler logic fails.
    pub async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.method, request.path.as_str()) {
            (HttpMethod::Get, "/api/dashboard/overview") => self.get_dashboard_overview().await,
            (HttpMethod::Get, "/api/dashboard/stats") => self.get_plugin_statistics().await,
            (HttpMethod::Get, "/api/dashboard/health") => self.get_system_health().await,
            (HttpMethod::Get, "/api/dashboard/activities") => self.get_recent_activities().await,
            (HttpMethod::Get, "/api/dashboard/alerts") => self.get_active_alerts().await,
            (HttpMethod::Post, path)
                if path.starts_with("/api/dashboard/alerts/") && path.ends_with("/dismiss") =>
            {
                let alert_id = self.extract_alert_id(path)?;
                self.dismiss_alert(alert_id).await
            }
            (HttpMethod::Get, "/api/dashboard/config") => self.get_dashboard_config().await,
            (HttpMethod::Put, "/api/dashboard/config") => {
                let config: DashboardConfig =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.update_dashboard_config(config).await
            }
            _ => Ok(WebResponse {
                status: HttpStatus::NotFound,
                headers: HashMap::new(),
                body: Some(serde_json::json!({
                    "error": "Not Found",
                    "message": format!("No dashboard endpoint found for {} {}", request.method, request.path)
                })),
            }),
        }
    }

    /// Get dashboard overview data
    async fn get_dashboard_overview(&self) -> Result<WebResponse> {
        let overview = DashboardOverview {
            plugin_stats: self.collect_plugin_statistics().await,
            system_health: self.collect_system_health().await,
            recent_activities: self.collect_recent_activities().await,
            quick_actions: self.get_quick_actions().await,
            alerts: self.collect_registry_alerts().await,
        };

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(overview)?),
        })
    }

    /// Get plugin statistics
    async fn get_plugin_statistics(&self) -> Result<WebResponse> {
        let stats = self.collect_plugin_statistics().await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(stats)?),
        })
    }

    /// Get system health
    async fn get_system_health(&self) -> Result<WebResponse> {
        let health = self.collect_system_health().await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(health)?),
        })
    }

    /// Get recent activities
    async fn get_recent_activities(&self) -> Result<WebResponse> {
        let activities = self.collect_recent_activities().await;
        let limit = self.config.read().await.recent_activities_limit;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "activities": activities,
                "total": activities.len(),
                "limit": limit
            })),
        })
    }

    /// Get active alerts
    async fn get_active_alerts(&self) -> Result<WebResponse> {
        let alerts = self.collect_registry_alerts().await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "alerts": alerts,
                "total": alerts.len()
            })),
        })
    }

    /// Dismiss an alert
    async fn dismiss_alert(&self, alert_id: Uuid) -> Result<WebResponse> {
        self.dismissed_alerts.write().await.insert(alert_id);

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "alert_id": alert_id,
                "status": "dismissed",
                "message": "Alert dismissed for this dashboard session"
            })),
        })
    }

    /// Get dashboard configuration
    async fn get_dashboard_config(&self) -> Result<WebResponse> {
        let cfg = self.config.read().await;
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(&*cfg)?),
        })
    }

    /// Update dashboard configuration
    async fn update_dashboard_config(&self, config: DashboardConfig) -> Result<WebResponse> {
        *self.config.write().await = config.clone();

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "status": "updated",
                "message": "Dashboard configuration updated successfully",
                "config": config
            })),
        })
    }

    /// Collect plugin statistics
    async fn collect_plugin_statistics(&self) -> PluginStatistics {
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref())
            .await
            .unwrap_or_default();
        let mut active_count = 0;
        let mut inactive_count = 0;
        let mut failed_count = 0;

        for plugin in &plugins {
            match PluginManagerTrait::get_plugin_status(self.manager.as_ref(), plugin.metadata().id)
                .await
            {
                Ok(PluginStatus::Initialized) => active_count += 1,
                Ok(PluginStatus::Registered) => inactive_count += 1,
                Ok(PluginStatus::Failed) => failed_count += 1,
                _ => inactive_count += 1,
            }
        }

        PluginStatistics {
            total_plugins: plugins.len(),
            active_plugins: active_count,
            inactive_plugins: inactive_count,
            failed_plugins: failed_count,
            pending_updates: 0,
            memory_usage_mb: process_rss_mb_linux(),
            cpu_usage_percent: None,
        }
    }

    /// Collect system health information
    async fn collect_system_health(&self) -> SystemHealth {
        let stats = self.collect_plugin_statistics().await;
        let health_status = if stats.failed_plugins > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        let uptime_seconds =
            linux_system_uptime_seconds().unwrap_or_else(|| self.started_at.elapsed().as_secs());
        let memory_usage = linux_meminfo_memory_usage();
        let observability_hints = Some(vec![
            "attach capability.system.metrics (or host integration) for CPU, disk, and network sampling"
                .to_string(),
            "use plugin registry status in /api/dashboard/stats for workload health".to_string(),
        ]);
        SystemHealth {
            status: health_status,
            uptime_seconds,
            memory_usage,
            cpu_usage: None,
            disk_usage: None,
            network_stats: None,
            observability_hints,
        }
    }

    /// Collect recent activities derived from the live plugin registry
    async fn collect_recent_activities(&self) -> Vec<ActivityItem> {
        let limit = self.config.read().await.recent_activities_limit;
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref())
            .await
            .unwrap_or_default();
        let now = chrono::Utc::now();
        let mut items = Vec::new();

        for (i, plugin) in plugins.iter().enumerate() {
            let meta = plugin.metadata();
            let status =
                PluginManagerTrait::get_plugin_status(self.manager.as_ref(), meta.id).await;
            let (activity_type, description, st) = match status {
                Ok(PluginStatus::Initialized | PluginStatus::Running) => (
                    "plugin.active",
                    format!("Plugin '{}' is active", meta.name),
                    "success",
                ),
                Ok(PluginStatus::Registered | PluginStatus::Loaded) => (
                    "plugin.registered",
                    format!("Plugin '{}' is registered or loaded", meta.name),
                    "info",
                ),
                Ok(PluginStatus::Failed) => (
                    "plugin.failed",
                    format!("Plugin '{}' failed", meta.name),
                    "error",
                ),
                Ok(PluginStatus::Unloaded | PluginStatus::Inactive) => (
                    "plugin.inactive",
                    format!("Plugin '{}' is inactive or unloaded", meta.name),
                    "info",
                ),
                Ok(PluginStatus::Stopped | PluginStatus::Stopping) => (
                    "plugin.stopped",
                    format!("Plugin '{}' is stopped or stopping", meta.name),
                    "info",
                ),
                Err(_) => (
                    "plugin.status_unknown",
                    format!("Could not read status for '{}'", meta.name),
                    "warning",
                ),
            };
            let mut metadata = HashMap::new();
            if let Ok(s) = status {
                metadata.insert("plugin_status".to_string(), format!("{s:?}"));
            }
            items.push(ActivityItem {
                id: Uuid::new_v4(),
                activity_type: activity_type.to_string(),
                description,
                plugin_id: Some(meta.id),
                plugin_name: Some(meta.name.clone()),
                timestamp: now - chrono::Duration::seconds(i as i64),
                status: st.to_string(),
                metadata,
            });
        }

        items.truncate(limit);
        items
    }

    /// Quick actions (marketplace entries honor dashboard config)
    async fn get_quick_actions(&self) -> Vec<QuickAction> {
        let show_marketplace = self.config.read().await.show_marketplace;
        vec![
            QuickAction {
                id: "install-plugin".to_string(),
                title: "Install Plugin".to_string(),
                description: "Install a new plugin from marketplace".to_string(),
                icon: "plus".to_string(),
                url: "/api/plugins".to_string(),
                method: "POST".to_string(),
                enabled: show_marketplace,
            },
            QuickAction {
                id: "refresh-plugins".to_string(),
                title: "Refresh Plugins".to_string(),
                description: "Refresh all plugin statuses".to_string(),
                icon: "refresh".to_string(),
                url: "/api/plugins/refresh".to_string(),
                method: "POST".to_string(),
                enabled: true,
            },
            QuickAction {
                id: "backup-config".to_string(),
                title: "Backup Configuration".to_string(),
                description: "Create a backup of plugin configurations".to_string(),
                icon: "save".to_string(),
                url: "/api/plugins/backup".to_string(),
                method: "POST".to_string(),
                enabled: true,
            },
        ]
    }

    /// Alerts derived from registry state (e.g. failed plugins)
    async fn collect_registry_alerts(&self) -> Vec<Alert> {
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref())
            .await
            .unwrap_or_default();
        let dismissed: HashSet<Uuid> = self.dismissed_alerts.read().await.clone();
        let mut out = Vec::new();

        for plugin in plugins {
            let meta = plugin.metadata();
            let Ok(PluginStatus::Failed) =
                PluginManagerTrait::get_plugin_status(self.manager.as_ref(), meta.id).await
            else {
                continue;
            };

            let id = Uuid::new_v5(
                &Uuid::NAMESPACE_DNS,
                format!("squirrel.dashboard.alert.failed.{}", meta.id).as_bytes(),
            );
            if dismissed.contains(&id) {
                continue;
            }

            out.push(Alert {
                id,
                level: AlertLevel::Error,
                title: format!("Plugin failed: {}", meta.name),
                message: format!(
                    "Plugin '{}' ({}) is in Failed status in the registry",
                    meta.name, meta.id
                ),
                timestamp: chrono::Utc::now(),
                dismissed: false,
                actions: vec![
                    AlertAction {
                        id: "retry-init".to_string(),
                        title: "Retry initialization".to_string(),
                        url: format!("/api/plugins/{}/restart", meta.id),
                        method: "POST".to_string(),
                    },
                    AlertAction {
                        id: "view-logs".to_string(),
                        title: "View logs".to_string(),
                        url: "/api/plugins/logs".to_string(),
                        method: "GET".to_string(),
                    },
                ],
            });
        }

        out
    }

    /// Extract alert ID from path
    fn extract_alert_id(&self, path: &str) -> Result<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 5 {
            let id_str = parts[4];
            Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("Invalid alert ID: {e}"))
        } else {
            Err(anyhow::anyhow!("Invalid path format"))
        }
    }
}

/// Resident set size for this process (MiB) from Linux `/proc/self/status` (`VmRSS`).
fn process_rss_mb_linux() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb as f64 / 1024.0);
            }
        }
    }
    None
}

/// System uptime in whole seconds from Linux `/proc/uptime` (first field).
#[cfg(target_os = "linux")]
#[expect(
    clippy::cast_sign_loss,
    reason = "/proc/uptime first field is non-negative; cast is bounded via clamp"
)]
fn linux_system_uptime_seconds() -> Option<u64> {
    let first = std::fs::read_to_string("/proc/uptime").ok()?;
    let secs = first.split_whitespace().next()?.parse::<f64>().ok()?;
    let bounded = secs.clamp(0.0, u64::MAX as f64).floor();
    Some(bounded as u64)
}

#[cfg(not(target_os = "linux"))]
const fn linux_system_uptime_seconds() -> Option<u64> {
    None
}

/// Host memory usage from Linux `/proc/meminfo` (`MemTotal` vs `MemAvailable` / `MemFree`).
#[cfg(target_os = "linux")]
fn linux_meminfo_memory_usage() -> Option<MemoryUsage> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    let total_kb = parse_meminfo_kb("MemTotal:", &data)?;
    let avail_kb =
        parse_meminfo_kb("MemAvailable:", &data).or_else(|| parse_meminfo_kb("MemFree:", &data))?;
    let used_kb = total_kb.saturating_sub(avail_kb);
    let total_mib = total_kb as f64 / 1024.0;
    let used_mib = used_kb as f64 / 1024.0;
    let usage_percent = if total_kb > 0 {
        (used_kb as f64 / total_kb as f64) * 100.0
    } else {
        0.0
    };
    Some(MemoryUsage {
        used_mb: used_mib,
        total_mb: total_mib,
        usage_percent,
    })
}

#[cfg(not(target_os = "linux"))]
const fn linux_meminfo_memory_usage() -> Option<MemoryUsage> {
    None
}

/// Parses a `Mem*:` line in `/proc/meminfo` and returns the value in kB.
#[cfg(target_os = "linux")]
fn parse_meminfo_kb(prefix: &str, data: &str) -> Option<u64> {
    for line in data.lines() {
        if line.starts_with(prefix) {
            let mut parts = line.split_whitespace();
            parts.next()?;
            let kb = parts.next()?.parse().ok()?;
            return Some(kb);
        }
    }
    None
}

#[cfg(test)]
#[path = "dashboard_tests.rs"]
mod tests;
