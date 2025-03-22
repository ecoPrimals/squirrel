//! Dashboard server for the MCP monitoring system
//!
//! This module provides a dashboard server for the MCP monitoring system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};
use super::alerts::AlertManager;
use super::metrics::MetricsCollector;

/// Dashboard server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// HTTP port for the dashboard
    pub port: u16,
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Maximum data points to display
    pub max_data_points: usize,
    /// Whether to enable WebSocket for real-time updates
    pub enable_websocket: bool,
    /// Dashboard title
    pub title: String,
    /// Dashboard theme
    pub theme: DashboardTheme,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            update_interval_ms: 5000,
            max_data_points: 100,
            enable_websocket: true,
            title: "MCP Monitoring Dashboard".to_string(),
            theme: DashboardTheme::Light,
        }
    }
}

/// Dashboard theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardTheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Custom theme
    Custom,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget ID
    pub id: String,
    /// Widget title
    pub title: String,
    /// Widget type
    pub widget_type: WidgetType,
    /// Widget size
    pub size: WidgetSize,
    /// Widget position
    pub position: Option<WidgetPosition>,
    /// Widget data source
    pub data_source: WidgetDataSource,
}

/// Widget type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    /// Gauge widget
    Gauge,
    /// Line chart widget
    LineChart,
    /// Bar chart widget
    BarChart,
    /// Table widget
    Table,
    /// Alerts widget
    Alerts,
    /// Text widget
    Text,
    /// Status widget
    Status,
}

/// Widget size
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WidgetSize {
    /// Widget width
    pub width: u32,
    /// Widget height
    pub height: u32,
}

/// Widget position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// X position
    pub x: u32,
    /// Y position
    pub y: u32,
}

/// Widget data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetDataSource {
    /// Metric data source
    Metric(String),
    /// Multiple metrics data source
    MultiMetric(Vec<String>),
    /// Alert data source
    Alerts,
    /// Custom data source
    Custom(String),
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    /// Dashboard widgets
    pub widgets: Vec<Widget>,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            widgets: vec![
                Widget {
                    id: "memory-usage".to_string(),
                    title: "Memory Usage".to_string(),
                    widget_type: WidgetType::Gauge,
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                    },
                    position: Some(WidgetPosition { x: 0, y: 0 }),
                    data_source: WidgetDataSource::Metric("memory_usage_bytes".to_string()),
                },
                Widget {
                    id: "message-latency".to_string(),
                    title: "Message Latency".to_string(),
                    widget_type: WidgetType::LineChart,
                    size: WidgetSize {
                        width: 4,
                        height: 2,
                    },
                    position: Some(WidgetPosition { x: 2, y: 0 }),
                    data_source: WidgetDataSource::Metric("message_latency_ms".to_string()),
                },
                Widget {
                    id: "error-rate".to_string(),
                    title: "Error Rate".to_string(),
                    widget_type: WidgetType::LineChart,
                    size: WidgetSize {
                        width: 4,
                        height: 2,
                    },
                    position: Some(WidgetPosition { x: 0, y: 2 }),
                    data_source: WidgetDataSource::Metric("error_rate".to_string()),
                },
                Widget {
                    id: "active-alerts".to_string(),
                    title: "Active Alerts".to_string(),
                    widget_type: WidgetType::Alerts,
                    size: WidgetSize {
                        width: 6,
                        height: 2,
                    },
                    position: Some(WidgetPosition { x: 0, y: 4 }),
                    data_source: WidgetDataSource::Alerts,
                },
            ],
        }
    }
}

/// Dashboard state
#[derive(Debug, Clone)]
pub struct DashboardState {
    /// Dashboard configuration
    pub config: DashboardConfig,
    /// Dashboard layout
    pub layout: DashboardLayout,
    /// Dashboard server status
    pub server_status: DashboardServerStatus,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// Number of connected clients
    pub connected_clients: usize,
}

/// Dashboard server status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardServerStatus {
    /// Server is starting
    Starting,
    /// Server is running
    Running,
    /// Server is stopping
    Stopping,
    /// Server is stopped
    Stopped,
    /// Server encountered an error
    Error,
}

/// Dashboard server
#[derive(Debug)]
pub struct DashboardServer {
    /// Dashboard state
    state: Arc<RwLock<DashboardState>>,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Alert manager
    alert_manager: Arc<AlertManager>,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(
        config: DashboardConfig,
        metrics_collector: Arc<MetricsCollector>,
        alert_manager: Arc<AlertManager>,
    ) -> Self {
        let state = DashboardState {
            config: config.clone(),
            layout: DashboardLayout::default(),
            server_status: DashboardServerStatus::Stopped,
            last_updated: Utc::now(),
            connected_clients: 0,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
            metrics_collector,
            alert_manager,
        }
    }

    /// Start the dashboard server
    pub async fn start(&self) -> Result<(), String> {
        {
            let mut state = self.state.write().unwrap();

            // Check if already running
            if matches!(
                state.server_status,
                DashboardServerStatus::Running | DashboardServerStatus::Starting
            ) {
                return Err("Dashboard server is already running".to_string());
            }

            // Update status
            state.server_status = DashboardServerStatus::Starting;
            state.last_updated = Utc::now();
        }

        // Start the server
        let config = self.state.read().unwrap().config.clone();
        let state_arc = self.state.clone();
        let metrics = self.metrics_collector.clone();
        let alerts = self.alert_manager.clone();

        tokio::spawn(async move {
            info!("Starting dashboard server on port {}", config.port);

            // In a real implementation, this would start an HTTP server
            // For this example, we'll just simulate the server running

            // Update server status
            {
                let mut state = state_arc.write().unwrap();
                state.server_status = DashboardServerStatus::Running;
                state.last_updated = Utc::now();
            }

            // Monitor for updates
            loop {
                // Check if we should stop
                {
                    let state = state_arc.read().unwrap();
                    if matches!(
                        state.server_status,
                        DashboardServerStatus::Stopping | DashboardServerStatus::Stopped
                    ) {
                        break;
                    }
                }

                // Update dashboard data
                update_dashboard_data(&state_arc, &metrics, &alerts).await;

                // Sleep for update interval
                tokio::time::sleep(std::time::Duration::from_millis(config.update_interval_ms))
                    .await;
            }

            // Final status update
            {
                let mut state = state_arc.write().unwrap();
                state.server_status = DashboardServerStatus::Stopped;
                state.last_updated = Utc::now();
                state.connected_clients = 0;
            }

            info!("Dashboard server stopped");
        });

        Ok(())
    }

    /// Stop the dashboard server
    pub async fn stop(&self) -> Result<(), String> {
        {
            let mut state = self.state.write().unwrap();

            // Check if already stopped
            if matches!(
                state.server_status,
                DashboardServerStatus::Stopped | DashboardServerStatus::Stopping
            ) {
                return Ok(());
            }

            // Update status
            state.server_status = DashboardServerStatus::Stopping;
            state.last_updated = Utc::now();
        }

        // In a real implementation, we would wait for the server to stop
        // For this example, we'll just simulate waiting
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(())
    }

    /// Get the dashboard state
    pub fn get_state(&self) -> DashboardState {
        let state = self.state.read().unwrap();
        DashboardState {
            config: state.config.clone(),
            layout: state.layout.clone(),
            server_status: state.server_status,
            last_updated: state.last_updated,
            connected_clients: state.connected_clients,
        }
    }

    /// Update the dashboard layout
    pub fn update_layout(&self, layout: DashboardLayout) {
        let mut state = self.state.write().unwrap();
        state.layout = layout;
        state.last_updated = Utc::now();
    }

    /// Add a widget to the dashboard
    pub fn add_widget(&self, widget: Widget) {
        let mut state = self.state.write().unwrap();
        state.layout.widgets.push(widget);
        state.last_updated = Utc::now();
    }

    /// Remove a widget from the dashboard
    pub fn remove_widget(&self, widget_id: &str) -> Result<(), String> {
        let mut state = self.state.write().unwrap();
        let index = state
            .layout
            .widgets
            .iter()
            .position(|w| w.id == widget_id)
            .ok_or_else(|| format!("Widget with ID {} not found", widget_id))?;

        state.layout.widgets.remove(index);
        state.last_updated = Utc::now();

        Ok(())
    }
}

/// Update dashboard data
async fn update_dashboard_data(
    state: &Arc<RwLock<DashboardState>>,
    metrics: &Arc<MetricsCollector>,
    alerts: &Arc<AlertManager>,
) {
    // Get latest metrics
    let all_metrics = metrics.get_all_metrics();

    // Get active alerts
    let active_alerts = alerts.get_active_alerts();

    // Update dashboard last updated timestamp
    {
        let mut state = state.write().unwrap();
        state.last_updated = Utc::now();
    }

    // In a real implementation, this would update a real-time dashboard
    debug!(
        "Dashboard update: {} metrics, {} active alerts",
        all_metrics.len(),
        active_alerts.len()
    );
}
