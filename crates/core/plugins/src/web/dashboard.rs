// SPDX-License-Identifier: AGPL-3.0-only
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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::PluginStatus;
use crate::web::{
    ComponentType, HttpMethod, HttpStatus, WebComponent, WebEndpoint, WebRequest, WebResponse,
};
use crate::{DefaultPluginManager, PluginManagerTrait, PluginRegistry};

/// Plugin management dashboard component
#[derive(Clone)]
pub struct PluginDashboard {
    /// Plugin manager instance
    manager: Arc<DefaultPluginManager>,
    /// Dashboard configuration
    config: DashboardConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Refresh interval in seconds
    pub refresh_interval: u64,
    /// Number of recent activities to show
    pub recent_activities_limit: usize,
    /// Whether to show system metrics
    pub show_system_metrics: bool,
    /// Whether to show marketplace integration
    pub show_marketplace: bool,
    /// Theme settings
    pub theme: DashboardTheme,
}

/// Dashboard theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTheme {
    /// Primary color
    pub primary_color: String,
    /// Secondary color
    pub secondary_color: String,
    /// Background color
    pub background_color: String,
    /// Text color
    pub text_color: String,
    /// Dark mode enabled
    pub dark_mode: bool,
}

/// Dashboard overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    /// Plugin statistics
    pub plugin_stats: PluginStatistics,
    /// System health
    pub system_health: SystemHealth,
    /// Recent activities
    pub recent_activities: Vec<ActivityItem>,
    /// Quick actions
    pub quick_actions: Vec<QuickAction>,
    /// Alerts and notifications
    pub alerts: Vec<Alert>,
}

/// Plugin statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatistics {
    /// Total plugins
    pub total_plugins: usize,
    /// Active plugins
    pub active_plugins: usize,
    /// Inactive plugins
    pub inactive_plugins: usize,
    /// Failed plugins
    pub failed_plugins: usize,
    /// Pending updates
    pub pending_updates: usize,
    /// Memory usage by plugins
    pub memory_usage_mb: f64,
    /// CPU usage by plugins
    pub cpu_usage_percent: f64,
}

/// System health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage
    pub memory_usage: MemoryUsage,
    /// CPU usage
    pub cpu_usage: CpuUsage,
    /// Disk usage
    pub disk_usage: DiskUsage,
    /// Network statistics
    pub network_stats: NetworkStats,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings
    Warning,
    /// System has errors
    Error,
    /// System is critical
    Critical,
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Used memory in MB
    pub used_mb: f64,
    /// Total memory in MB
    pub total_mb: f64,
    /// Usage percentage
    pub usage_percent: f64,
}

/// CPU usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    /// Current CPU usage percentage
    pub current_percent: f64,
    /// Average CPU usage over last minute
    pub avg_1min_percent: f64,
    /// Average CPU usage over last 5 minutes
    pub avg_5min_percent: f64,
}

/// Disk usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    /// Used disk space in MB
    pub used_mb: f64,
    /// Total disk space in MB
    pub total_mb: f64,
    /// Usage percentage
    pub usage_percent: f64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
}

/// Activity item for recent activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    /// Activity ID
    pub id: Uuid,
    /// Activity type
    pub activity_type: String,
    /// Activity description
    pub description: String,
    /// Plugin involved (if any)
    pub plugin_id: Option<Uuid>,
    /// Plugin name
    pub plugin_name: Option<String>,
    /// Activity timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Activity status
    pub status: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Quick action for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    /// Action ID
    pub id: String,
    /// Action title
    pub title: String,
    /// Action description
    pub description: String,
    /// Action icon
    pub icon: String,
    /// Action URL
    pub url: String,
    /// Action method
    pub method: String,
    /// Whether action is enabled
    pub enabled: bool,
}

/// Alert for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: Uuid,
    /// Alert level
    pub level: AlertLevel,
    /// Alert title
    pub title: String,
    /// Alert message
    pub message: String,
    /// Alert timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether alert is dismissed
    pub dismissed: bool,
    /// Alert actions
    pub actions: Vec<AlertAction>,
}

/// Alert level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Information alert
    Info,
    /// Success alert
    Success,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
}

/// Alert action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAction {
    /// Action ID
    pub id: String,
    /// Action title
    pub title: String,
    /// Action URL
    pub url: String,
    /// Action method
    pub method: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 30,
            recent_activities_limit: 10,
            show_system_metrics: true,
            show_marketplace: true,
            theme: DashboardTheme {
                primary_color: "#007bff".to_string(),
                secondary_color: "#6c757d".to_string(),
                background_color: "#ffffff".to_string(),
                text_color: "#333333".to_string(),
                dark_mode: false,
            },
        }
    }
}

impl PluginDashboard {
    /// Create a new plugin dashboard
    pub fn new(manager: Arc<DefaultPluginManager>) -> Self {
        Self {
            manager,
            config: DashboardConfig::default(),
        }
    }

    /// Create dashboard with custom configuration
    pub const fn with_config(manager: Arc<DefaultPluginManager>, config: DashboardConfig) -> Self {
        Self { manager, config }
    }

    /// Get dashboard endpoints
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
            quick_actions: self.get_quick_actions(),
            alerts: self.get_sample_alerts(),
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

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "activities": activities,
                "total": activities.len(),
                "limit": self.config.recent_activities_limit
            })),
        })
    }

    /// Get active alerts
    #[allow(clippy::unused_async)]
    async fn get_active_alerts(&self) -> Result<WebResponse> {
        let alerts = self.get_sample_alerts();

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
    #[allow(clippy::unused_async)]
    async fn dismiss_alert(&self, alert_id: Uuid) -> Result<WebResponse> {
        // In real implementation, this would mark the alert as dismissed
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "alert_id": alert_id,
                "status": "dismissed",
                "message": "Alert dismissed successfully"
            })),
        })
    }

    /// Get dashboard configuration
    #[allow(clippy::unused_async)]
    async fn get_dashboard_config(&self) -> Result<WebResponse> {
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(&self.config)?),
        })
    }

    /// Update dashboard configuration
    #[allow(clippy::unused_async)]
    async fn update_dashboard_config(&self, config: DashboardConfig) -> Result<WebResponse> {
        // In real implementation, this would update the configuration
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
            pending_updates: 2,     // Placeholder
            memory_usage_mb: 128.5, // Placeholder
            cpu_usage_percent: 5.2, // Placeholder
        }
    }

    /// Collect system health information
    #[allow(clippy::unused_async)]
    async fn collect_system_health(&self) -> SystemHealth {
        SystemHealth {
            status: HealthStatus::Healthy,
            uptime_seconds: 86400, // 24 hours
            memory_usage: MemoryUsage {
                used_mb: 512.0,
                total_mb: 1024.0,
                usage_percent: 50.0,
            },
            cpu_usage: CpuUsage {
                current_percent: 15.5,
                avg_1min_percent: 12.3,
                avg_5min_percent: 10.8,
            },
            disk_usage: DiskUsage {
                used_mb: 5120.0,
                total_mb: 10240.0,
                usage_percent: 50.0,
            },
            network_stats: NetworkStats {
                bytes_sent: 1024 * 1024 * 100,     // 100 MB
                bytes_received: 1024 * 1024 * 200, // 200 MB
                packets_sent: 50000,
                packets_received: 75000,
            },
        }
    }

    /// Collect recent activities
    #[allow(clippy::unused_async)]
    async fn collect_recent_activities(&self) -> Vec<ActivityItem> {
        vec![
            ActivityItem {
                id: Uuid::new_v4(),
                activity_type: "plugin.installed".to_string(),
                description: "Plugin installed successfully".to_string(),
                plugin_id: Some(Uuid::new_v4()),
                plugin_name: Some("Security Scanner".to_string()),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
                status: "success".to_string(),
                metadata: HashMap::new(),
            },
            ActivityItem {
                id: Uuid::new_v4(),
                activity_type: "plugin.updated".to_string(),
                description: "Plugin updated to version 2.1.0".to_string(),
                plugin_id: Some(Uuid::new_v4()),
                plugin_name: Some("Code Formatter".to_string()),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(15),
                status: "success".to_string(),
                metadata: HashMap::new(),
            },
            ActivityItem {
                id: Uuid::new_v4(),
                activity_type: "plugin.failed".to_string(),
                description: "Plugin failed to start".to_string(),
                plugin_id: Some(Uuid::new_v4()),
                plugin_name: Some("Database Connector".to_string()),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(30),
                status: "error".to_string(),
                metadata: HashMap::new(),
            },
        ]
    }

    /// Get quick actions
    fn get_quick_actions(&self) -> Vec<QuickAction> {
        vec![
            QuickAction {
                id: "install-plugin".to_string(),
                title: "Install Plugin".to_string(),
                description: "Install a new plugin from marketplace".to_string(),
                icon: "plus".to_string(),
                url: "/api/plugins".to_string(),
                method: "POST".to_string(),
                enabled: true,
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

    /// Get sample alerts
    fn get_sample_alerts(&self) -> Vec<Alert> {
        vec![
            Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Warning,
                title: "Plugin Update Available".to_string(),
                message: "Security Scanner v2.1.0 is available".to_string(),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(10),
                dismissed: false,
                actions: vec![
                    AlertAction {
                        id: "update-plugin".to_string(),
                        title: "Update Now".to_string(),
                        url: "/api/plugins/update".to_string(),
                        method: "POST".to_string(),
                    },
                    AlertAction {
                        id: "view-changelog".to_string(),
                        title: "View Changelog".to_string(),
                        url: "/api/plugins/changelog".to_string(),
                        method: "GET".to_string(),
                    },
                ],
            },
            Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Error,
                title: "Plugin Failed to Start".to_string(),
                message: "Database Connector failed to initialize".to_string(),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(30),
                dismissed: false,
                actions: vec![
                    AlertAction {
                        id: "restart-plugin".to_string(),
                        title: "Restart Plugin".to_string(),
                        url: "/api/plugins/restart".to_string(),
                        method: "POST".to_string(),
                    },
                    AlertAction {
                        id: "view-logs".to_string(),
                        title: "View Logs".to_string(),
                        url: "/api/plugins/logs".to_string(),
                        method: "GET".to_string(),
                    },
                ],
            },
        ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultPluginManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_plugin_dashboard_creation() {
        let manager = Arc::new(DefaultPluginManager::new());
        let dashboard = PluginDashboard::new(manager);

        let endpoints = dashboard.get_endpoints();
        assert!(!endpoints.is_empty());
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path == "/api/dashboard/overview")
        );
    }

    #[tokio::test]
    async fn test_dashboard_overview() {
        let manager = Arc::new(DefaultPluginManager::new());
        let dashboard = PluginDashboard::new(manager);

        let response = dashboard.get_dashboard_overview().await.unwrap();
        assert_eq!(response.status, HttpStatus::Ok);

        let body = response.body.unwrap();
        assert!(body.get("plugin_stats").is_some());
        assert!(body.get("system_health").is_some());
        assert!(body.get("recent_activities").is_some());
    }

    #[tokio::test]
    async fn test_plugin_statistics() {
        let manager = Arc::new(DefaultPluginManager::new());
        let dashboard = PluginDashboard::new(manager);

        let stats = dashboard.collect_plugin_statistics().await;
        assert_eq!(stats.total_plugins, 0); // No plugins registered in test
    }
}
