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
use rand;

/// Module for adapter implementations of dashboard functionality
/// 
/// This module provides adapters for connecting dashboard managers to dependency injection systems,
/// allowing for proper initialization and management of monitoring dashboards.
pub mod adapter;
use adapter::{DashboardManagerAdapter, create_dashboard_manager_adapter_with_manager};

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
        /// Unique identifier for the component
        id: String,
        /// Display title of the component
        title: String,
        /// Detailed description of the component's purpose
        description: String,
        /// Type of operation to display metrics for
        operation_type: OperationType,
        /// Time range to display data for
        time_range: Duration,
    },
    /// Alert list showing current alerts
    AlertList {
        /// Unique identifier for the component
        id: String,
        /// Display title of the component
        title: String,
        /// Detailed description of the component's purpose
        description: String,
        /// Optional filter for alert severity level
        severity: Option<AlertSeverity>,
        /// Optional filter for alert status
        status: Option<AlertStatus>,
    },
    /// Health status display
    HealthStatus {
        /// Unique identifier for the component
        id: String,
        /// Display title of the component
        title: String,
        /// Detailed description of the component's purpose
        description: String,
        /// Service name to display health status for
        service: String,
    },
    /// Custom component with arbitrary data
    Custom {
        /// Unique identifier for the component
        id: String,
        /// Display title of the component
        title: String,
        /// Detailed description of the component's purpose
        description: String,
        /// Custom data to display
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
    #[allow(dead_code)]
    config: DashboardConfig,
}

impl DashboardManager {
    /// Creates a new dashboard manager with a specific config
    #[must_use]
    pub const fn new(config: DashboardConfig) -> Self {
        Self { config }
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

impl Default for DashboardManager {
    fn default() -> Self {
        Self::new(DashboardConfig::default())
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

    /// Creates a new dashboard manager adapter using dependency injection
    ///
    /// This function is a convenience method for creating an adapter
    /// using dependency injection.
    ///
    /// # Arguments
    /// * `config` - Optional dashboard configuration, uses default if `None`
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

/// Creates a new dashboard manager adapter using dependency injection
///
/// This function is a convenience method for creating an adapter
/// using dependency injection.
///
/// # Arguments
/// * `config` - Optional dashboard configuration, uses default if `None`
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
    /// Health checker for monitoring system components
    health_checker: Arc<RwLock<Box<dyn HealthChecker + Send + Sync>>>,
    /// Metric collector for gathering system performance metrics
    metric_collector: Arc<RwLock<Box<dyn MetricCollector + Send + Sync>>>,
    /// Alert manager for handling and processing system alerts
    alert_manager: Arc<RwLock<Box<dyn AlertManager + Send + Sync>>>,
    /// Collection of dashboard layouts indexed by their IDs
    layouts: Arc<RwLock<HashMap<String, Layout>>>,
    /// Dashboard configuration settings
    config: Arc<RwLock<Config>>,
    /// Storage for component update history, indexed by component ID
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

    /// Starts the dashboard manager to refresh and serve data
    ///
    /// This method sets up the WebSocket server for real-time updates
    /// and begins periodic refreshes of dashboard data.
    ///
    /// # Errors
    ///
    /// Returns error if unable to start the manager
    /// 
    /// # Panics
    /// 
    /// Panics if unable to create runtime
    pub async fn start(&self) -> Result<()> {
        // Read the config once to avoid capturing self in the closure
        let websocket_enabled = self.config.read().await.websocket_enabled;
        let update_interval = self.config.read().await.update_interval;
        
        if websocket_enabled {
            // Only clone the data_store which is used in the thread
            let data_store = self.data_store.clone();
            
            // Start WebSocket server in background
            // For simplicity in this example, we're using a new thread with a Tokio runtime
            // In a real implementation, you might want to use a more sophisticated approach
            std::thread::spawn(move || {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(async move {
                        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                            update_interval
                        ));
                        
                        loop {
                            interval.tick().await;
                            
                            // Generate some sample data for the dashboard
                            let mut sample_data = HashMap::new();
                            sample_data.insert(
                                "system_cpu".to_string(), 
                                Value::from(rand::random::<f64>() * 100.0)
                            );
                            sample_data.insert(
                                "system_memory".to_string(), 
                                Value::from(rand::random::<f64>() * 100.0)
                            );
                            
                            // Update data_store directly
                            let now = OffsetDateTime::now_utc();
                            let mut data_lock = data_store.write().await;
                            
                            for (component_id, value) in &sample_data {
                                let update = Update {
                                    component_id: component_id.clone(),
                                    timestamp: now,
                                    data: value.clone(),
                                };
                                
                                data_lock.entry(component_id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(update);
                            }
                            
                            // Log update
                            eprintln!("Dashboard data updated at {now}");
                        }
                    }),
                    Err(e) => {
                        eprintln!("Failed to create Tokio runtime for dashboard updates: {e}");
                    }
                }
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
            return Err(SquirrelError::dashboard(format!("Layout '{layout_id}' not found")));
        }
        Ok(())
    }

    /// Gets layouts from the dashboard
    /// 
    /// # Errors
    /// Returns error if unable to get layouts
    pub async fn get_layouts(&self) -> Result<Vec<Layout>> {
        let layouts = self.layouts.read().await;
        Ok(layouts.values().cloned().collect())
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

    /// Gets data for a specific component
    /// 
    /// # Errors
    /// Returns error if component data cannot be retrieved
    pub async fn get_data(&self, component_id: &str) -> Result<Vec<Update>> {
        let data_store = self.data_store.read().await;
        data_store.get(component_id)
            .cloned()
            .ok_or_else(|| SquirrelError::dashboard(format!("Component data not found: {component_id}")))
    }

    /// Gets widget data for a specific component
    /// 
    /// # Errors
    /// Returns error if component data cannot be retrieved
    pub async fn get_widget_data(&self, component: &Component) -> Result<Value> {
        match component {
            Component::PerformanceGraph { operation_type, time_range, .. } => {
                let metrics = self.metric_collector.read().await
                    .collect_metrics()
                    .await?
                    .into_iter()
                    .filter(|m| m.operation_type == *operation_type)
                    .take(usize::try_from(time_range.as_secs()).unwrap_or(usize::MAX))
                    .collect::<Vec<_>>();
                
                to_value(metrics).map_err(|e| SquirrelError::serialization(e.to_string()))
            },
            Component::AlertList { severity, status, .. } => {
                let alerts = self.alert_manager.read().await
                    .get_alerts()
                    .await?
                    .into_iter()
                    .filter(|alert| {
                        severity.as_ref().is_none_or(|s| alert.severity == *s) &&
                        status.as_ref().is_none_or(|s| alert.status == *s)
                    })
                    .collect::<Vec<_>>();
                
                to_value(alerts).map_err(|e| SquirrelError::serialization(e.to_string()))
            },
            Component::HealthStatus { service, .. } => {
                let mut status = self.health_checker.read().await
                    .check_health()
                    .await?;
                
                // Add service information to the status
                status.service.clone_from(service);
                
                to_value(status).map_err(|e| SquirrelError::serialization(e.to_string()))
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