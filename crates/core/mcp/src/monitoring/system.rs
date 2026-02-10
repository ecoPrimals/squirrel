// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Monitoring system orchestration
//!
//! This module provides the main MonitoringSystem that orchestrates all monitoring
//! components including metrics collection, alerting, health monitoring, and dashboard.

use crate::error::Result;
use super::{
    alerts::AlertManager,
    dashboard::DashboardServer,
    health::HealthMonitor,
    metrics::MetricsCollector,
    clients::{MonitoringClient, InMemoryMonitoringClient},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Status of the monitoring system
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringStatus {
    /// Monitoring system is running
    Running,
    /// Monitoring system is stopped
    Stopped,
    /// Monitoring system is starting
    Starting,
    /// Monitoring system is stopping
    Stopping,
    /// Monitoring system is in error state
    Error,
}

/// Comprehensive monitoring system that orchestrates all monitoring components
pub struct MonitoringSystem {
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Alert manager
    alert_manager: Arc<AlertManager>,
    /// Health monitor
    health_monitor: Arc<HealthMonitor>,
    /// Dashboard server (optional)
    dashboard_server: Option<Arc<DashboardServer>>,
    /// Monitoring status
    status: Arc<RwLock<MonitoringStatus>>,
    /// Whether the dashboard is enabled
    dashboard_enabled: bool,
    /// Monitoring client for external integrations
    monitoring_client: Arc<dyn MonitoringClient>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    #[must_use]
    pub fn new() -> Self {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new());
        let health_monitor = Arc::new(HealthMonitor::new());
        let monitoring_client = Arc::new(InMemoryMonitoringClient::new("monitoring_system"));

        Self {
            metrics_collector,
            alert_manager,
            health_monitor,
            dashboard_server: None,
            status: Arc::new(RwLock::new(MonitoringStatus::Stopped)),
            dashboard_enabled: false,
            monitoring_client,
        }
    }

    /// Create a monitoring system with a custom monitoring client
    pub fn with_monitoring_client(monitoring_client: Arc<dyn MonitoringClient>) -> Self {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new());
        let health_monitor = Arc::new(HealthMonitor::new());

        Self {
            metrics_collector,
            alert_manager,
            health_monitor,
            dashboard_server: None,
            status: Arc::new(RwLock::new(MonitoringStatus::Stopped)),
            dashboard_enabled: false,
            monitoring_client,
        }
    }

    /// Get the metrics collector
    #[must_use]
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }

    /// Get the alert manager
    #[must_use]
    pub fn alert_manager(&self) -> Arc<AlertManager> {
        Arc::clone(&self.alert_manager)
    }

    /// Get the health monitor
    #[must_use]
    pub fn health_monitor(&self) -> Arc<HealthMonitor> {
        Arc::clone(&self.health_monitor)
    }

    /// Get the monitoring client
    #[must_use]
    pub fn monitoring_client(&self) -> Arc<dyn MonitoringClient> {
        Arc::clone(&self.monitoring_client)
    }

    /// Enable dashboard on the specified port
    pub fn enable_dashboard(&mut self, port: u16) {
        if self.dashboard_enabled {
            warn!("Dashboard is already enabled");
            return;
        }

        let dashboard_server = Arc::new(DashboardServer::new(
            port,
            Arc::clone(&self.metrics_collector),
            Arc::clone(&self.alert_manager),
        ));

        self.dashboard_server = Some(dashboard_server);
        self.dashboard_enabled = true;
        info!("Dashboard enabled on port {}", port);
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;

        match *status {
            MonitoringStatus::Running => {
                return Err(crate::error::MCPError::MonitoringError(
                    "Monitoring system is already running".to_string(),
                ));
            }
            MonitoringStatus::Starting => {
                return Err(crate::error::MCPError::MonitoringError(
                    "Monitoring system is already starting".to_string(),
                ));
            }
            _ => {}
        }

        *status = MonitoringStatus::Starting;
        drop(status);

        info!("Starting monitoring system...");

        // Start metrics collector
        if let Err(e) = self.metrics_collector.start().await {
            error!("Failed to start metrics collector: {}", e);
            let mut status = self.status.write().await;
            *status = MonitoringStatus::Error;
            return Err(e);
        }

        // Start alert manager
        if let Err(e) = self.alert_manager.start().await {
            error!("Failed to start alert manager: {}", e);
            let mut status = self.status.write().await;
            *status = MonitoringStatus::Error;
            return Err(e);
        }

        // Start dashboard server if enabled
        if let Some(dashboard) = &self.dashboard_server {
            if let Err(e) = dashboard.start().await {
                error!("Failed to start dashboard server: {}", e);
                let mut status = self.status.write().await;
                *status = MonitoringStatus::Error;
                return Err(e);
            }
        }

        // Start background monitoring tasks
        self.start_background_tasks().await;

        let mut status = self.status.write().await;
        *status = MonitoringStatus::Running;
        info!("Monitoring system started successfully");

        Ok(())
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;

        match *status {
            MonitoringStatus::Stopped => {
                return Err(crate::error::MCPError::MonitoringError(
                    "Monitoring system is already stopped".to_string(),
                ));
            }
            MonitoringStatus::Stopping => {
                return Err(crate::error::MCPError::MonitoringError(
                    "Monitoring system is already stopping".to_string(),
                ));
            }
            _ => {}
        }

        *status = MonitoringStatus::Stopping;
        drop(status);

        info!("Stopping monitoring system...");

        // Stop dashboard server
        if let Some(dashboard) = &self.dashboard_server {
            if let Err(e) = dashboard.stop().await {
                error!("Failed to stop dashboard server: {}", e);
            }
        }

        // Stop alert manager
        if let Err(e) = self.alert_manager.stop().await {
            error!("Failed to stop alert manager: {}", e);
        }

        // Stop metrics collector
        if let Err(e) = self.metrics_collector.stop().await {
            error!("Failed to stop metrics collector: {}", e);
        }

        let mut status = self.status.write().await;
        *status = MonitoringStatus::Stopped;
        info!("Monitoring system stopped successfully");

        Ok(())
    }

    /// Get the current status of the monitoring system
    pub async fn get_status(&self) -> MonitoringStatus {
        *self.status.read().await
    }

    /// Check if the monitoring system is running
    pub async fn is_running(&self) -> bool {
        matches!(*self.status.read().await, MonitoringStatus::Running)
    }

    /// Check if the dashboard is enabled
    pub fn is_dashboard_enabled(&self) -> bool {
        self.dashboard_enabled
    }

    /// Get dashboard server reference
    pub fn dashboard_server(&self) -> Option<Arc<DashboardServer>> {
        self.dashboard_server.clone()
    }

    /// Start background monitoring tasks
    async fn start_background_tasks(&self) {
        // Start health monitoring task
        let health_monitor = Arc::clone(&self.health_monitor);
        let alert_manager = Arc::clone(&self.alert_manager);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Update health status
                if let Err(e) = health_monitor.update_health().await {
                    error!("Failed to update health status: {}", e);
                }

                // Check for health alerts
                let alerts = health_monitor.check_health_alerts().await;
                for alert in alerts {
                    if let Err(e) = alert_manager.raise_health_alert(alert).await {
                        error!("Failed to raise health alert: {}", e);
                    }
                }
            }
        });

        // Start metrics collection task
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let monitoring_client = Arc::clone(&self.monitoring_client);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                // Collect and report metrics
                if let Ok(metrics_summary) = metrics_collector.get_metrics_summary().await {
                    for (name, value) in metrics_summary {
                        if let Err(e) = monitoring_client.record_metric(&name, value, None).await {
                            debug!("Failed to record metric '{}': {}", name, e);
                        }
                    }
                }
            }
        });

        debug!("Background monitoring tasks started");
    }

    /// Get a comprehensive system status summary
    pub async fn get_system_summary(&self) -> MonitoringSystemSummary {
        let status = self.get_status().await;
        let health_status = self.health_monitor.get_health().await;
        let metrics_summary = self.metrics_collector.get_metrics_summary().await.unwrap_or_default();
        let alert_summary = self.alert_manager.get_alert_summary().await;

        MonitoringSystemSummary {
            status,
            health_status,
            metrics_count: metrics_summary.len(),
            active_alerts: alert_summary.active_alerts,
            dashboard_enabled: self.dashboard_enabled,
            dashboard_port: self.dashboard_server.as_ref().map(|d| d.port()),
        }
    }

    /// Force a health check and return the result
    pub async fn force_health_check(&self) -> Result<super::health::HealthStatus> {
        self.health_monitor.update_health().await?;
        Ok(self.health_monitor.get_health().await)
    }

    /// Get real-time metrics
    pub async fn get_realtime_metrics(&self) -> Result<std::collections::HashMap<String, super::clients::MetricValue>> {
        self.metrics_collector.get_metrics_summary().await
    }

    /// Trigger a manual alert for testing
    pub async fn trigger_test_alert(&self, message: &str) -> Result<()> {
        use super::clients::{MonitoringEvent, AlertLevel};
        
        let test_event = MonitoringEvent::new(
            "test_alert",
            message,
            AlertLevel::Medium,
            "monitoring_system"
        );

        self.alert_manager.process_event(test_event).await
    }
}

/// Summary of the monitoring system status
#[derive(Debug, Clone)]
pub struct MonitoringSystemSummary {
    /// Current system status
    pub status: MonitoringStatus,
    /// Health status
    pub health_status: super::health::HealthStatus,
    /// Number of metrics being tracked
    pub metrics_count: usize,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Whether dashboard is enabled
    pub dashboard_enabled: bool,
    /// Dashboard port if enabled
    pub dashboard_port: Option<u16>,
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MonitoringStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitoringStatus::Running => write!(f, "Running"),
            MonitoringStatus::Stopped => write!(f, "Stopped"),
            MonitoringStatus::Starting => write!(f, "Starting"),
            MonitoringStatus::Stopping => write!(f, "Stopping"),
            MonitoringStatus::Error => write!(f, "Error"),
        }
    }
}

/// Monitoring system errors
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    /// System is already running
    #[error("Already running: {0}")]
    AlreadyRunning(String),

    /// System is not running
    #[error("Not running: {0}")]
    NotRunning(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// Alert error
    #[error("Alert error: {0}")]
    Alert(String),

    /// Dashboard error
    #[error("Dashboard error: {0}")]
    Dashboard(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
} 