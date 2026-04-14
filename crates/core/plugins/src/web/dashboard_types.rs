// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for the plugin management dashboard.
//!
//! Configuration, overview, health, activity, and alert types used by
//! [`super::dashboard::PluginDashboard`].
//!
//! Fields are self-documenting DTO structs; see the parent module for usage docs.
#![expect(missing_docs, reason = "DTO fields — documented at usage site")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub refresh_interval: u64,
    pub recent_activities_limit: usize,
    pub show_system_metrics: bool,
    pub show_marketplace: bool,
    pub theme: DashboardTheme,
}

/// Dashboard theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub dark_mode: bool,
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

/// Dashboard overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub plugin_stats: PluginStatistics,
    pub system_health: SystemHealth,
    pub recent_activities: Vec<ActivityItem>,
    pub quick_actions: Vec<QuickAction>,
    pub alerts: Vec<Alert>,
}

/// Plugin statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatistics {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub inactive_plugins: usize,
    pub failed_plugins: usize,
    pub pending_updates: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_mb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage_percent: Option<f64>,
}

/// System health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage: Option<MemoryUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<CpuUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_usage: Option<DiskUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_stats: Option<NetworkStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observability_hints: Option<Vec<String>>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
    Critical,
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub used_mb: f64,
    pub total_mb: f64,
    pub usage_percent: f64,
}

/// CPU usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub current_percent: f64,
    pub avg_1min_percent: f64,
    pub avg_5min_percent: f64,
}

/// Disk usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub used_mb: f64,
    pub total_mb: f64,
    pub usage_percent: f64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Activity item for recent activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: Uuid,
    pub activity_type: String,
    pub description: String,
    pub plugin_id: Option<Uuid>,
    pub plugin_name: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub metadata: HashMap<String, String>,
}

/// Quick action for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub url: String,
    pub method: String,
    pub enabled: bool,
}

/// Alert for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dismissed: bool,
    pub actions: Vec<AlertAction>,
}

/// Alert level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Alert action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAction {
    pub id: String,
    pub title: String,
    pub url: String,
    pub method: String,
}
