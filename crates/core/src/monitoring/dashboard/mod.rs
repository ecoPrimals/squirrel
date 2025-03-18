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
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock;
use std::time::Duration;
use time::OffsetDateTime;
use crate::monitoring::{
    alerts::{AlertSeverity, AlertStatus, Alert, AlertConfig, AlertManager},
    health::{HealthChecker, HealthConfig, status::HealthStatus},
    metrics::{performance::OperationType, MetricCollector, MetricConfig, Metric},
};
use crate::error::{Result, SquirrelError};
use serde_json::{Value, to_value};

// Make the adapter module public
pub mod adapter;
use adapter::{DashboardManagerAdapter, create_dashboard_manager_adapter, create_dashboard_manager_adapter_with_manager};

/// Configuration for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Whether to enable `WebSocket` updates
    pub websocket_enabled: bool,
    /// Update interval in seconds
    pub update_interval: u64,
    /// Maximum number of data points to store per metric
    pub max_data_points: usize,
    /// Alert configuration
    pub alert_config: AlertConfig,
    /// Health check configuration
    pub health_config: HealthConfig,
    /// Metric collection configuration
    pub metric_config: MetricConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            websocket_enabled: false,
            update_interval: 60,
            max_data_points: 100,
            alert_config: AlertConfig::default(),
            health_config: HealthConfig::default(),
            metric_config: MetricConfig::default(),
        }
    }
}

/// Component types available in the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    /// Performance graph showing metrics over time
    PerformanceGraph {
        id: String,
        title: String,
        description: String,
        operation_type: OperationType,
        time_range: Duration,
    },
    /// Alert list showing current alerts
    AlertList {
        id: String,
        title: String,
        description: String,
        severity: Option<AlertSeverity>,
        status: Option<AlertStatus>,
    },
    /// Health status display
    HealthStatus {
        id: String,
        title: String,
        description: String,
        service: String,
    },
    /// Custom component with arbitrary data
    Custom {
        id: String,
        title: String,
        description: String,
        data: Value,
    },
}

/// Layout configuration for a dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    /// Unique identifier for the layout
    pub id: String,
    /// Display name of the layout
    pub name: String,
    /// Detailed description of the layout
    pub description: String,
    /// List of components included in the layout
    pub components: Vec<Component>,
    /// Grid configuration for component placement
    pub grid: Value,
    /// Timestamp when the layout was created
    pub created_at: OffsetDateTime,
    /// Timestamp when the layout was last updated
    pub updated_at: OffsetDateTime,
}

/// Data update for the dashboard
#[derive(Debug, Clone, Serialize)]
pub struct Update {
    /// Component ID
    pub component_id: String,
    /// Update timestamp
    pub timestamp: OffsetDateTime,
    /// Updated data
    pub data: Value,
}

/// Current state of the dashboard
#[derive(Debug)]
pub struct Data {
    /// Current health status of monitored components
    pub health: HealthStatus,
    /// List of collected metrics
    pub metrics: Vec<Metric>,
    /// List of active alerts
    pub alerts: Vec<Alert>,
    /// Interval at which dashboard data is refreshed
    pub refresh_interval: Duration,
}

/// Error types for dashboard operations
#[derive(Debug, Error)]
pub enum DashboardError {
    /// Layout not found
    #[error("Layout not found: {0}")]
    LayoutNotFound(String),
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    /// Data error
    #[error("Data error: {0}")]
    DataError(String),
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Whether the dashboard is enabled
    pub enabled: bool,
    /// Dashboard refresh interval in seconds
    pub refresh_interval: u64,
    /// Maximum number of metrics to display
    pub max_metrics: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 60,
            max_metrics: 100,
        }
    }
}

/// Dashboard manager for system monitoring
#[derive(Debug)]
pub struct DashboardManager {
    /// Dashboard configuration
    config: DashboardConfig,
}

impl DashboardManager {
    /// Create a new dashboard manager with the given configuration
    #[must_use]
    pub const fn new(config: DashboardConfig) -> Self {
        Self { config }
    }

    /// Create a new dashboard manager with default configuration
    #[must_use]
    pub fn default() -> Self {
        Self::new(DashboardConfig::default())
    }

    /// Start the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to start
    pub async fn start(&self) -> Result<()> {
        // Implementation will be added in future PRs
        Ok(())
    }

    /// Stop the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to stop
    pub async fn stop(&self) -> Result<()> {
        // Implementation will be added in future PRs
        Ok(())
    }
}

/// Factory for creating and managing dashboard manager instances
#[derive(Debug, Clone)]
pub struct DashboardManagerFactory {
    /// Configuration for creating dashboard managers
    config: DashboardConfig,
}

impl DashboardManagerFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DashboardConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: DashboardConfig) -> Self {
        Self { config }
    }

    /// Creates a dashboard manager with the factory's configuration
    #[must_use]
    pub fn create_manager(&self) -> Arc<DashboardManager> {
        Arc::new(DashboardManager::new(self.config.clone()))
    }

    /// Creates a dashboard manager with dependencies
    ///
    /// This method supports dependency injection by accepting
    /// external dependencies for the dashboard manager.
    #[must_use]
    pub fn create_manager_with_dependencies(
        &self,
        // Add any required dependencies here in the future
    ) -> Arc<DashboardManager> {
        // For now, the dashboard manager doesn't have external dependencies
        self.create_manager()
    }

    /// Creates an adapter for the dashboard manager with default configuration
    ///
    /// This method is used for backward compatibility during the
    /// transition to dependency injection.
    #[must_use]
    pub fn create_adapter(&self) -> Arc<DashboardManagerAdapter> {
        let manager = self.create_manager();
        create_dashboard_manager_adapter_with_manager(manager)
    }
}

impl Default for DashboardManagerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a dashboard manager adapter using dependency injection
///
/// This function is a convenience method for creating an adapter
/// using dependency injection.
///
/// # Arguments
/// * `config` - Optional dashboard configuration, uses default if None
#[must_use]
pub fn create_adapter(config: Option<DashboardConfig>) -> Arc<DashboardManagerAdapter> {
    let factory = match config {
        Some(cfg) => DashboardManagerFactory::with_config(cfg),
        None => DashboardManagerFactory::new(),
    };
    factory.create_adapter()
}

/// Manager for dashboard operations
pub struct Manager {
    health_checker: Arc<RwLock<Box<dyn HealthChecker + Send + Sync>>>,
    metric_collector: Arc<RwLock<Box<dyn MetricCollector + Send + Sync>>>,
    alert_manager: Arc<RwLock<Box<dyn AlertManager + Send + Sync>>>,
    layouts: Arc<RwLock<HashMap<String, Layout>>>,
    config: Arc<RwLock<Config>>,
    data_store: Arc<RwLock<HashMap<String, Vec<Update>>>>,
}

impl Manager {
    /// Creates a new dashboard manager
    #[must_use]
    pub fn new(
        config: Config,
        metric_collector: Box<dyn MetricCollector + Send + Sync>,
        alert_manager: Box<dyn AlertManager + Send + Sync>,
        health_checker: Box<dyn HealthChecker + Send + Sync>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            layouts: Arc::new(RwLock::new(HashMap::new())),
            metric_collector: Arc::new(RwLock::new(metric_collector)),
            alert_manager: Arc::new(RwLock::new(alert_manager)),
            health_checker: Arc::new(RwLock::new(health_checker)),
            data_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Starts the dashboard manager
    /// 
    /// # Errors
    /// Returns error if unable to start the manager
    /// 
    /// # Panics
    /// Panics if unable to create runtime
    pub async fn start(&self) -> Result<()> {
        if self.config.read().await.websocket_enabled {
            // Start WebSocket server in background
            tokio::spawn(async move {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async {
                        // WebSocket server implementation
                    });
            });
        }
        Ok(())
    }

    /// Adds a new layout to the dashboard
    /// 
    /// # Errors
    /// Returns error if unable to acquire lock
    pub async fn add_layout(&self, layout: Layout) -> Result<()> {
        let id = layout.id.clone();
        self.layouts.write().await.insert(id, layout);
        Ok(())
    }

    /// Removes a layout from the dashboard
    /// 
    /// # Errors
    /// Returns error if unable to acquire lock or layout not found
    pub async fn remove_layout(&self, layout_id: &str) -> Result<()> {
        let mut layouts = self.layouts.write().await;
        if layouts.remove(layout_id).is_none() {
            return Err(SquirrelError::dashboard(&format!("Layout '{layout_id}' not found")));
        }
        Ok(())
    }

    /// Gets all dashboard layouts
    /// 
    /// # Errors
    /// Returns error if unable to acquire lock
    pub async fn get_layouts(&self) -> Result<Vec<Layout>> {
        Ok(self.layouts.read().await.values().cloned().collect())
    }

    /// Updates dashboard data
    /// 
    /// # Errors
    /// Returns error if unable to acquire lock
    pub async fn update_data(&self, values: HashMap<String, Value>) -> Result<()> {
        let mut data_store = self.data_store.write().await;
        let now = OffsetDateTime::now_utc();

        for (component_id, value) in values {
            let update = Update {
                component_id: component_id.clone(),
                timestamp: now,
                data: value,
            };

            data_store.entry(component_id)
                .or_insert_with(Vec::new)
                .push(update);
        }
        Ok(())
    }

    /// Retrieves dashboard data for `WebSocket` updates
    /// 
    /// # Returns
    /// * `Result<Data>` - Current dashboard state for `WebSocket` clients
    /// 
    /// # Errors
    /// Returns error if unable to acquire lock
    pub async fn get_data(&self, component_id: &str) -> Result<Vec<Update>> {
        Ok(self.data_store.read().await
            .get(component_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Gets widget data
    /// 
    /// # Errors
    /// Returns error if unable to get widget data
    pub async fn get_widget_data(&self, component: &Component) -> Result<Value> {
        match component {
            Component::PerformanceGraph { operation_type, time_range, .. } => {
                let metrics = self.metric_collector.read().await
                    .collect_metrics()
                    .await?
                    .into_iter()
                    .filter(|m| m.operation_type == *operation_type)
                    .take(time_range.as_secs() as usize)
                    .collect::<Vec<_>>();
                
                to_value(metrics).map_err(|e| SquirrelError::serialization(&e.to_string()))
            },
            Component::AlertList { severity, status, .. } => {
                let alerts = self.alert_manager.read().await
                    .get_alerts()
                    .await?
                    .into_iter()
                    .filter(|alert| {
                        severity.as_ref().map_or(true, |s| alert.severity == *s) &&
                        status.as_ref().map_or(true, |s| alert.status == *s)
                    })
                    .collect::<Vec<_>>();
                
                to_value(alerts).map_err(|e| SquirrelError::serialization(&e.to_string()))
            },
            Component::HealthStatus { service, .. } => {
                let mut status = self.health_checker.read().await
                    .check_health()
                    .await?;
                
                // Add service information to the status
                status.service = service.clone();
                
                to_value(status).map_err(|e| SquirrelError::serialization(&e.to_string()))
            },
            Component::Custom { data, .. } => {
                Ok(data.clone())
            },
        }
    }
}

/// Creates a default dashboard configuration
#[must_use]
pub fn create_default() -> Config {
    Config::default()
}

// Re-export adapter types
pub use adapter::{DashboardManagerAdapter, create_dashboard_manager_adapter, create_dashboard_manager_adapter_with_manager};