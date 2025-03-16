// Dashboard module for monitoring system
//
// This module provides functionality for:
// - Real-time metrics visualization
// - Health status display
// - Alert management interface
// - Performance graphs
// - Resource usage charts
// - Custom dashboards
// - Data visualization
// - Interactive controls

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::{OnceCell, RwLock};
use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::monitoring::{
    alerts::{AlertSeverity, AlertStatus, Alert, AlertConfig, AlertManager, DefaultAlertManager},
    health::{HealthChecker, HealthConfig, HealthStatus, DefaultHealthChecker},
    metrics::{performance::OperationType, MetricCollector, MetricConfig, DefaultMetricCollector, Metric},
};
use crate::error::{Result, SquirrelError};
use tokio::time::sleep;

/// Dashboard configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// HTTP server port
    pub port: u16,
    /// Enable dashboard
    pub enabled: bool,
    /// Refresh interval
    pub refresh_interval: Duration,
    /// Enable WebSocket updates
    pub websocket_enabled: bool,
    /// Custom styling options
    pub style: Option<DashboardStyle>,
    pub title: String,
    pub description: String,
    pub health_config: HealthConfig,
    pub metric_config: MetricConfig,
    pub alert_config: AlertConfig,
    pub metrics_port: Option<u16>,
    pub dashboard_port: Option<u16>,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStyle {
    pub theme: String,
    pub custom_css: Option<String>,
}

/// Dashboard component type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardComponent {
    /// Metric chart
    MetricChart {
        /// Component ID
        id: String,
        /// Chart title
        title: String,
        /// Metric names to display
        metrics: Vec<String>,
        /// Chart type (line, bar, etc.)
        chart_type: String,
        /// Time range in seconds
        time_range: u64,
    },
    /// Health status panel
    HealthPanel {
        /// Component ID
        id: String,
        /// Panel title
        title: String,
        /// Components to monitor
        components: Vec<String>,
    },
    /// Alert list
    AlertList {
        /// Component ID
        id: String,
        /// List title
        title: String,
        /// Filter by severity
        severity_filter: Option<AlertSeverity>,
        /// Filter by status
        status_filter: Option<AlertStatus>,
        /// Maximum items to display
        max_items: usize,
    },
    /// Resource usage gauge
    ResourceGauge {
        /// Component ID
        id: String,
        /// Gauge title
        title: String,
        /// Resource type
        resource_type: String,
        /// Team name
        team_name: String,
    },
    /// Performance graph
    PerformanceGraph {
        /// Component ID
        id: String,
        /// Graph title
        title: String,
        /// Operation type
        operation_type: OperationType,
        /// Time range in seconds
        time_range: u64,
    },
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    /// Layout ID
    pub id: String,
    /// Layout name
    pub name: String,
    /// Layout description
    pub description: String,
    /// Dashboard components
    pub components: Vec<DashboardComponent>,
    /// Layout grid configuration
    pub grid: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DashboardLayout {
    pub fn new(id: String, name: String, description: String, grid: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            components: Vec::new(),
            grid,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Dashboard data update
#[derive(Debug, Clone, Serialize)]
pub struct DashboardUpdate {
    /// Component ID
    pub component_id: String,
    /// Update timestamp
    pub timestamp: time::OffsetDateTime,
    /// Updated data
    pub data: serde_json::Value,
}

/// Dashboard errors
#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Component error: {0}")]
    ComponentError(String),
    #[error("Update error: {0}")]
    UpdateError(String),
    #[error("System error: {0}")]
    SystemError(String),
}

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub health: HealthStatus,
    pub metrics: Vec<Metric>,
    pub alerts: Vec<Alert>,
    pub refresh_interval: Duration,
}

/// Dashboard manager for handling monitoring visualization
#[derive(Debug)]
pub struct DashboardManager {
    health_checker: Arc<dyn HealthChecker + Send + Sync>,
    metric_collector: Arc<dyn MetricCollector + Send + Sync>,
    alert_manager: Arc<dyn AlertManager + Send + Sync>,
    layouts: Arc<RwLock<Vec<DashboardLayout>>>,
    config: DashboardConfig,
}

impl DashboardManager {
    pub fn new(
        health_checker: Arc<dyn HealthChecker + Send + Sync>,
        metric_collector: Arc<dyn MetricCollector + Send + Sync>,
        alert_manager: Arc<dyn AlertManager + Send + Sync>,
        config: DashboardConfig,
    ) -> Self {
        Self {
            health_checker,
            metric_collector,
            alert_manager,
            layouts: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Initializes the dashboard manager with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the dashboard manager is already initialized
    pub fn initialize(config: Option<DashboardConfig>) -> Result<Arc<DashboardManager>> {
        // For backward compatibility, still store in a global static
        static INSTANCE: OnceCell<Arc<DashboardManager>> = OnceCell::const_new();
        
        let config = config.unwrap_or_default();
        let health_checker: Arc<dyn HealthChecker + Send + Sync> = Arc::new(DefaultHealthChecker::new());
        let metric_collector: Arc<dyn MetricCollector + Send + Sync> = Arc::new(DefaultMetricCollector::new());
        let alert_manager: Arc<dyn AlertManager + Send + Sync> = Arc::new(DefaultAlertManager::new(AlertConfig::default()));
        
        let manager = Arc::new(DashboardManager::new(
            health_checker,
            metric_collector,
            alert_manager,
            config,
        ));

        INSTANCE
            .set(manager.clone())
            .map_err(|_| SquirrelError::monitoring("Dashboard manager already initialized"))?;

        Ok(manager)
    }

    /// Starts the dashboard manager and its components
    ///
    /// # Errors
    /// Returns an error if any component fails to start
    ///
    /// # Panics
    /// Panics if a new Tokio runtime cannot be created
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Start the health checker
        let health_start = self.health_checker.start();
        health_start.await?;
        
        // Start the metric collector
        let metrics_start = self.metric_collector.start();
        metrics_start.await?;
        
        // Start the alert manager
        let alerts_start = self.alert_manager.start();
        alerts_start.await?;

        let _layouts = self.layouts.clone();
        let config = self.config.clone();

        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    loop {
                        // Refresh dashboard data
                        sleep(config.refresh_interval).await;
                    }
                });
        });

        Ok(())
    }

    /// Stops the dashboard manager and its components
    ///
    /// # Errors
    ///
    /// Returns an error if any component fails to stop
    pub async fn stop(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Stop the health checker
        let health_stop = self.health_checker.stop();
        health_stop.await?;
        
        // Stop the metric collector
        let metrics_stop = self.metric_collector.stop();
        metrics_stop.await?;
        
        // Stop the alert manager
        let alerts_stop = self.alert_manager.stop();
        alerts_stop.await?;
        
        Ok(())
    }

    /// Retrieves dashboard data including health status, metrics, and alerts
    ///
    /// # Errors
    ///
    /// Returns an error if the data could not be retrieved from any component
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        // Get health status
        let health_future = self.health_checker.check_health();
        let health = health_future.await?;
        
        // Get metrics
        let metrics_future = self.metric_collector.collect_metrics();
        let metrics = metrics_future.await?;
        
        // Get alerts
        let alerts_future = self.alert_manager.get_alerts();
        let alerts = alerts_future.await?;
        
        Ok(DashboardData {
            health,
            metrics,
            alerts,
            refresh_interval: self.config.refresh_interval,
        })
    }

    /// Updates the dashboard configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration could not be updated
    pub async fn update_config(&mut self, config: DashboardConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// Adds a new dashboard layout
    ///
    /// # Errors
    ///
    /// Returns an error if the layout could not be added
    pub async fn add_layout(&self, layout: DashboardLayout) -> Result<()> {
        let mut layouts = self.layouts.write().await;
        layouts.push(layout);
        Ok(())
    }

    /// Removes a dashboard layout by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the layout with the given ID is not found
    pub async fn remove_layout(&self, layout_id: &str) -> Result<()> {
        let mut layouts = self.layouts.write().await;
        let original_len = layouts.len();
        layouts.retain(|layout| layout.id != layout_id);
        
        if layouts.len() == original_len {
            // No layout was removed
            return Err(SquirrelError::monitoring(&format!("Layout '{layout_id}' not found")));
        }
        
        Ok(())
    }

    /// Retrieves all dashboard layouts
    ///
    /// # Errors
    ///
    /// Returns an error if the layouts could not be retrieved
    pub async fn get_layouts(&self) -> Result<Vec<DashboardLayout>> {
        let layouts = self.layouts.read().await;
        Ok(layouts.clone())
    }

    /// Retrieves a dashboard layout by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the layout could not be retrieved
    pub async fn get_layout(&self, layout_id: &str) -> Result<Option<DashboardLayout>> {
        let layouts = self.layouts.read().await;
        Ok(layouts.iter().find(|layout| layout.id == layout_id).cloned())
    }

    /// Retrieves data for a dashboard widget
    ///
    /// # Errors
    ///
    /// Returns an error if the widget data could not be retrieved
    pub async fn get_widget_data(&self, widget: &DashboardWidget) -> Result<serde_json::Value> {
        match &widget.data_source {
            DataSource::Metrics(metric_name) => {
                let metrics_future = self.metric_collector.collect_metrics();
                let metrics = metrics_future.await?;
                
                // Filter metrics by name
                let filtered_metrics: Vec<Metric> = metrics.into_iter()
                    .filter(|m| m.name.contains(metric_name))
                    .collect();
                
                let metrics_value: serde_json::Value = serde_json::to_value(filtered_metrics)?;
                Ok(metrics_value)
            }
            DataSource::Alerts => {
                let alerts_future = self.alert_manager.get_alerts();
                let alerts = alerts_future.await?;
                let alerts_value: serde_json::Value = serde_json::to_value(alerts)?;
                Ok(alerts_value)
            }
            DataSource::Health => {
                let health_future = self.health_checker.check_health();
                let health = health_future.await?;
                let health_value: serde_json::Value = serde_json::to_value(health)?;
                Ok(health_value)
            }
        }
    }

    /// Retrieves dashboard data for WebSocket clients
    ///
    /// # Errors
    ///
    /// Returns an error if the data could not be retrieved from any component
    pub async fn get_dashboard_data_ws(&self) -> Result<DashboardData> {
        // Get metrics
        let metrics_future = self.metric_collector.collect_metrics();
        let metrics = metrics_future.await?;
        
        // Get alerts
        let alerts_future = self.alert_manager.get_alerts();
        let alerts = alerts_future.await?;
        
        // Get health status
        let health_future = self.health_checker.check_health();
        let health = health_future.await?;
        
        Ok(DashboardData {
            health,
            metrics,
            alerts,
            refresh_interval: self.config.refresh_interval,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_dashboard_manager() {
        let config = DashboardConfig::default();
        let health_checker = Arc::new(DefaultHealthChecker::new());
        let metric_collector = Arc::new(DefaultMetricCollector::new());
        let alert_config = AlertConfig::default();
        let alert_manager = Arc::new(DefaultAlertManager::new(alert_config));

        let manager = DashboardManager::new(
            health_checker,
            metric_collector,
            alert_manager,
            config,
        );

        // Test adding a layout
        let layout = DashboardLayout {
            id: "test".to_string(),
            name: "Test Layout".to_string(),
            description: "Test layout".to_string(),
            components: vec![],
            grid: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(manager.add_layout(layout.clone()).await.is_ok());

        // Test getting layouts
        let layouts = manager.get_layouts().await.unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].id, "test");

        // Test removing a layout
        assert!(manager.remove_layout("test").await.is_ok());
        assert!(manager.remove_layout("nonexistent").await.is_err());

        // Test getting a specific layout
        assert!(manager.get_layout("test").await.unwrap().is_none());
    }
}

/// Get the dashboard manager instance
pub fn get_manager() -> Option<Arc<DashboardManager>> {
    DASHBOARD_MANAGER.get().cloned()
}

/// Check if the dashboard system is initialized
pub fn is_initialized() -> bool {
    DASHBOARD_MANAGER.get().is_some()
}

/// Creates a default dashboard with standard layout and components
///
/// # Errors
///
/// Returns an error if the dashboard manager cannot be created
pub async fn create_default_dashboard() -> Result<Arc<DashboardManager>> {
    // Fix the trait object issues by using concrete implementations
    let health_checker: Arc<dyn HealthChecker + Send + Sync> = Arc::new(DefaultHealthChecker::new());
    let metric_collector: Arc<dyn MetricCollector + Send + Sync> = Arc::new(DefaultMetricCollector::new());
    let alert_manager: Arc<dyn AlertManager + Send + Sync> = Arc::new(DefaultAlertManager::new(AlertConfig::default()));
    
    let config = DashboardConfig::default();
    let manager = Arc::new(DashboardManager::new(
        health_checker,
        metric_collector,
        alert_manager,
        config,
    ));

    let default_layout = DashboardLayout::new(
        "default".to_string(),
        "Default Dashboard".to_string(),
        "Default monitoring dashboard".to_string(),
        serde_json::json!({
            "rows": 2,
            "cols": 2
        }),
    );

    manager.add_layout(default_layout).await?;
    Ok(manager)
}

#[derive(Debug, Clone)]
pub enum DataSource {
    Metrics(String),
    Alerts,
    Health,
}

#[derive(Debug, Clone)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub data_source: DataSource,
    pub refresh_interval: Duration,
}

#[derive(Debug, Clone)]
pub enum WidgetType {
    LineChart,
    BarChart,
    Table,
    Gauge,
    Status,
}

// Add the missing DASHBOARD_MANAGER
static DASHBOARD_MANAGER: OnceCell<Arc<DashboardManager>> = OnceCell::const_new(); 