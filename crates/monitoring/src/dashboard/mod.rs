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
use crate::{
    health::{HealthChecker, HealthConfig, status::HealthStatus},
    metrics::{performance::OperationType, MetricCollector, MetricConfig, Metric},
    alerts::{AlertSeverity, AlertStatus, Alert, AlertConfig},
};
use squirrel_core::error::{Result, SquirrelError};
use serde_json::{Value, json};
use tracing::{info, error, debug};

/// Module for adapter implementations of dashboard functionality
/// 
/// This module provides adapters for connecting dashboard managers to dependency injection systems,
/// allowing for proper initialization and management of monitoring dashboards.
pub mod adapter;
use adapter::{DashboardManagerAdapter, create_dashboard_manager_adapter_with_manager};

/// Module for WebSocket server implementation
pub mod server;
use server::start_server;

/// Configuration for the dashboard's WebSocket functionality
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
    /// Metric configuration
    pub metric_config: MetricConfig,
    /// WebSocket server port
    pub websocket_port: u16,
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
            websocket_port: 8765,
        }
    }
}

impl From<DashboardConfig> for Config {
    fn from(config: DashboardConfig) -> Self {
        Self {
            websocket_enabled: config.enabled,
            update_interval: config.refresh_interval,
            max_data_points: config.max_metrics,
            websocket_port: config.websocket_port,
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
    /// Component error
    #[error("Component error: {0}")]
    ComponentError(String),
    /// Server error
    #[error("Server error: {0}")]
    ServerError(String),
}

impl From<DashboardError> for SquirrelError {
    fn from(e: DashboardError) -> Self {
        SquirrelError::monitoring(e.to_string())
    }
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
    /// WebSocket server port
    pub websocket_port: u16,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 60,
            max_metrics: 100,
            websocket_port: 8765,
        }
    }
}

/// Alert manager trait for dashboard integration
#[async_trait::async_trait]
pub trait AlertManagerTrait: Send + Sync + std::fmt::Debug {
    /// Get active alerts
    async fn get_active_alerts(&self) -> Result<Vec<Alert>>;
    
    /// Process alerts
    async fn process_alerts(&self) -> Result<()>;
    
    /// Add an alert
    async fn add_alert(&self, alert: Alert) -> Result<()>;
    
    /// Get all alerts
    async fn get_alerts(&self) -> Result<Vec<Alert>>;
    
    /// Acknowledge an alert
    async fn acknowledge_alert(&self, alert_id: &str) -> Result<()>;
    
    /// Start the alert manager
    async fn start(&self) -> Result<()>;
    
    /// Stop the alert manager
    async fn stop(&self) -> Result<()>;
}

/// Dashboard manager for system monitoring
#[derive(Debug)]
pub struct DashboardManager {
    /// Dashboard configuration
    config: DashboardConfig,
    /// WebSocket server handle
    websocket_handle: Option<tokio::task::JoinHandle<()>>,
}

impl DashboardManager {
    /// Creates a new dashboard manager with a specific config
    #[must_use]
    pub const fn new(config: DashboardConfig) -> Self {
        Self { 
            config,
            websocket_handle: None
        }
    }

    /// Start the dashboard manager
    ///
    /// Initializes the dashboard, sets up data collection, and begins
    /// periodic updates of dashboard components.
    ///
    /// # Errors
    /// Returns an error if the dashboard cannot be started due to
    /// configuration issues, resource constraints, or if
    /// required components are not available
    pub async fn start(&mut self) -> Result<()> {
        if self.config.enabled {
            info!("Starting dashboard manager with refresh interval of {} seconds", self.config.refresh_interval);
            
            // Convert DashboardConfig to Config
            let config = Config::from(self.config.clone());
            
            // Create a simple default dashboard manager with the converted config
            let manager = Manager::new(
                config,
                Box::new(MockMetricCollector {}),
                Box::new(MockAlertManager {}),
                Box::new(MockHealthChecker {})
            );
            
            // Start the WebSocket server if enabled
            if self.config.enabled {
                let port = self.config.websocket_port;
                let manager_clone = Arc::new(manager);
                
                // Create socket address
                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
                
                self.websocket_handle = Some(tokio::spawn(async move {
                    match start_server(addr, manager_clone).await {
                        Ok(_) => info!("Dashboard WebSocket server stopped gracefully"),
                        Err(e) => error!("Dashboard WebSocket server error: {}", e),
                    }
                }));
                
                info!("Dashboard WebSocket server started on port {}", port);
            }
            
            Ok(())
        } else {
            debug!("Dashboard manager is disabled, not starting");
            Ok(())
        }
    }

    /// Stop the dashboard manager
    ///
    /// Stops all dashboard activities, including data collection
    /// and component updates.
    ///
    /// # Errors
    /// Returns an error if the dashboard cannot be stopped gracefully,
    /// if there are pending operations that cannot be completed,
    /// or if resources cannot be properly released
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.websocket_handle.take() {
            info!("Stopping dashboard WebSocket server");
            handle.abort();
            info!("Dashboard WebSocket server stopped");
        }
        
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
    /// Creates a new dashboard manager factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DashboardConfig::default(),
        }
    }

    /// Creates a new dashboard manager factory with specific configuration
    #[must_use]
    pub const fn with_config(config: DashboardConfig) -> Self {
        Self {
            config,
        }
    }

    /// Create a dashboard manager with the factory's configuration
    #[must_use]
    pub fn create_manager(&self) -> Arc<DashboardManager> {
        Arc::new(DashboardManager::new(self.config.clone()))
    }

    /// Creates a dashboard manager with the factory's configuration and dependencies
    ///
    /// This method allows for future dependency injection when needed.
    #[must_use]
    pub fn create_manager_with_dependencies(
        &self,
        // Add any required dependencies here in the future
    ) -> Arc<DashboardManager> {
        self.create_manager()
    }

    /// Creates a dashboard manager adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<DashboardManagerAdapter> {
        create_adapter(Some(self.config.clone()))
    }
}

impl Default for DashboardManagerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a dashboard manager adapter with optional configuration
///
/// If no configuration is provided, default configuration is used.
#[must_use]
pub fn create_adapter(config: Option<DashboardConfig>) -> Arc<DashboardManagerAdapter> {
    let config = config.unwrap_or_default();
    let manager = DashboardManager::new(config);
    create_dashboard_manager_adapter_with_manager(manager)
}

/// Dashboard manager implementation with full functionality
#[derive(Debug)]
pub struct Manager {
    /// Health checker for monitoring system components
    pub health_checker: Arc<RwLock<Box<dyn HealthChecker + Send + Sync>>>,
    /// Metric collector for gathering system performance metrics
    pub metric_collector: Arc<RwLock<Box<dyn MetricCollector + Send + Sync>>>,
    /// Alert manager for handling and processing system alerts
    pub alert_manager: Arc<RwLock<Box<dyn AlertManagerTrait + Send + Sync>>>,
    /// Collection of dashboard layouts indexed by their IDs
    pub layouts: Arc<RwLock<HashMap<String, Layout>>>,
    /// Dashboard configuration settings
    pub config: Arc<RwLock<Config>>,
    /// Storage for component update history, indexed by component ID
    pub data_store: Arc<RwLock<HashMap<String, Vec<Update>>>>,
    /// WebSocket server handle
    pub websocket_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl Manager {
    /// Creates a new dashboard manager
    #[must_use]
    pub fn new(
        config: Config,
        metric_collector: Box<dyn MetricCollector + Send + Sync>,
        alert_manager: Box<dyn AlertManagerTrait + Send + Sync>,
        health_checker: Box<dyn HealthChecker + Send + Sync>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            layouts: Arc::new(RwLock::new(HashMap::new())),
            metric_collector: Arc::new(RwLock::new(metric_collector)),
            alert_manager: Arc::new(RwLock::new(alert_manager)),
            health_checker: Arc::new(RwLock::new(health_checker)),
            data_store: Arc::new(RwLock::new(HashMap::new())),
            websocket_handle: Arc::new(RwLock::new(None)),
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
    pub async fn start(&self) -> Result<()> {
        // Read the config once to avoid multiple lock acquisitions
        let websocket_enabled: bool;
        let websocket_port: u16;
        
        {
            let config = self.config.read().await;
            websocket_enabled = config.websocket_enabled;
            websocket_port = config.websocket_port;
        }
        
        // Start the WebSocket server if enabled
        if websocket_enabled {
            info!("Starting dashboard WebSocket server on port {}", websocket_port);
            
            // Create a clone for the server task
            let self_clone = Arc::new(self.clone());
            
            // Create socket address
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], websocket_port));
            
            // Start the server in a background task
            let server_handle = tokio::spawn(async move {
                match start_server(addr, self_clone).await {
                    Ok(_) => info!("Dashboard WebSocket server stopped gracefully"),
                    Err(e) => error!("Dashboard WebSocket server error: {}", e),
                }
            });
            
            // Store the server handle
            let mut handle = self.websocket_handle.write().await;
            *handle = Some(server_handle);
            
            info!("Dashboard manager started successfully");
        } else {
            info!("WebSocket updates are disabled, running in polling mode only");
        }
        
        Ok(())
    }

    /// Add a new dashboard layout
    ///
    /// # Errors
    ///
    /// Returns error if the layout cannot be added
    pub async fn add_layout(&self, layout: Layout) -> Result<()> {
        let mut layouts = self.layouts.write().await;
        layouts.insert(layout.id.clone(), layout);
        Ok(())
    }

    /// Remove a dashboard layout by ID
    ///
    /// # Errors
    ///
    /// Returns error if the layout is not found or cannot be removed
    pub async fn remove_layout(&self, layout_id: &str) -> Result<()> {
        let mut layouts = self.layouts.write().await;
        if layouts.remove(layout_id).is_none() {
            return Err(SquirrelError::monitoring(format!("Layout not found: {}", layout_id)));
        }
        Ok(())
    }

    /// Get all dashboard layouts
    ///
    /// # Errors
    ///
    /// Returns error if layouts cannot be retrieved
    pub async fn get_layouts(&self) -> Result<Vec<Layout>> {
        let layouts = self.layouts.read().await;
        Ok(layouts.values().cloned().collect())
    }

    /// Update data for components
    ///
    /// # Errors
    ///
    /// Returns error if data cannot be updated
    pub async fn update_data(&self, values: HashMap<String, Value>) -> Result<()> {
        let mut data_store = self.data_store.write().await;
        
        for (component_id, value) in values {
            let update = Update {
                component_id: component_id.clone(),
                timestamp: OffsetDateTime::now_utc(),
                data: value,
            };
            
            // Create entry if it doesn't exist
            if !data_store.contains_key(&component_id) {
                data_store.insert(component_id.clone(), Vec::new());
            }
            
            // Add update to history
            if let Some(history) = data_store.get_mut(&component_id) {
                // Check if we need to trim the history
                let config = self.config.read().await;
                while history.len() >= config.max_data_points {
                    history.remove(0);
                }
                
                history.push(update);
            }
        }
        
        Ok(())
    }

    /// Get data for a specific component
    ///
    /// # Errors
    ///
    /// Returns error if data cannot be retrieved
    pub async fn get_data(&self, component_id: &str) -> Result<Vec<Update>> {
        let data_store = self.data_store.read().await;
        if let Some(history) = data_store.get(component_id) {
            Ok(history.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get data for a widget based on its component type
    ///
    /// # Errors
    ///
    /// Returns error if data cannot be retrieved or processed
    pub async fn get_widget_data(&self, component: &Component) -> Result<Value> {
        match component {
            Component::PerformanceGraph { id, operation_type, .. } => {
                // Get performance metrics for the operation type
                let metrics = self.metric_collector.read().await;
                let data = metrics.collect_metrics().await?;
                
                // Filter and transform metrics
                let filtered = data.iter()
                    .filter(|m| m.operation_type == *operation_type)
                    .map(|m| json!({
                        "timestamp": m.timestamp,
                        "value": m.value,
                        "operation_type": m.operation_type,
                        "labels": m.labels,
                    }))
                    .collect::<Vec<_>>();
                
                Ok(json!({
                    "component_id": id,
                    "data": filtered,
                    "timestamp": OffsetDateTime::now_utc(),
                }))
            },
            Component::AlertList { id, severity: _severity, status: _status, .. } => {
                // Get alerts from alert manager
                let _alert_manager = self.alert_manager.read().await;
                let alerts: Vec<Alert> = vec![];  // Mock implementation, replace with actual alert fetching
                
                // In a real implementation, we would filter alerts properly
                // For now, just return the empty list since we're mocking
                
                Ok(json!({
                    "component_id": id,
                    "data": alerts,
                    "timestamp": OffsetDateTime::now_utc(),
                }))
            },
            Component::HealthStatus { id, service, .. } => {
                // Get health status for the service
                let health_checker = self.health_checker.read().await;
                let health = health_checker.check_health().await?;
                
                Ok(json!({
                    "component_id": id,
                    "data": {
                        "service": service,
                        "status": format!("{:?}", health.status),
                        "message": health.message,
                        "service_name": health.service,
                    },
                    "timestamp": OffsetDateTime::now_utc(),
                }))
            },
            Component::Custom { id, data, .. } => {
                // Custom components just return their data
                Ok(json!({
                    "component_id": id,
                    "data": data,
                    "timestamp": OffsetDateTime::now_utc(),
                }))
            },
        }
    }

    /// Get all component IDs from layouts
    ///
    /// # Errors
    ///
    /// Returns error if layouts cannot be retrieved
    pub async fn get_components(&self) -> Result<Vec<String>> {
        let layouts = self.layouts.read().await;
        let mut component_ids = Vec::new();
        
        for layout in layouts.values() {
            for component in &layout.components {
                match component {
                    Component::PerformanceGraph { id, .. } => component_ids.push(id.clone()),
                    Component::AlertList { id, .. } => component_ids.push(id.clone()),
                    Component::HealthStatus { id, .. } => component_ids.push(id.clone()),
                    Component::Custom { id, .. } => component_ids.push(id.clone()),
                }
            }
        }
        
        Ok(component_ids)
    }
}

impl Default for Manager {
    fn default() -> Self {
        // Create mock implementations for dependencies
        let metric_collector = Box::new(MockMetricCollector {});
        let alert_manager = Box::new(MockAlertManager {});
        let health_checker = Box::new(MockHealthChecker {});
        
        Self::new(
            Config::default(),
            metric_collector,
            alert_manager,
            health_checker,
        )
    }
}

impl Clone for Manager {
    fn clone(&self) -> Self {
        // This is a shallow clone that shares the Arc pointers
        Self {
            health_checker: self.health_checker.clone(),
            metric_collector: self.metric_collector.clone(),
            alert_manager: self.alert_manager.clone(),
            layouts: self.layouts.clone(),
            config: self.config.clone(),
            data_store: self.data_store.clone(),
            websocket_handle: self.websocket_handle.clone(),
        }
    }
}

/// Creates a default dashboard configuration
#[must_use]
pub fn create_default() -> Config {
    Config::default()
}

// Create mock implementations for testing
#[derive(Debug)]
struct MockMetricCollector {}

#[async_trait::async_trait]
impl MetricCollector for MockMetricCollector {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        // Return empty metrics for now
        Ok(Vec::new())
    }
    
    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct MockAlertManager {}

#[async_trait::async_trait]
impl AlertManagerTrait for MockAlertManager {
    async fn process_alerts(&self) -> Result<()> {
        Ok(())
    }
    
    async fn add_alert(&self, _alert: Alert) -> Result<()> {
        Ok(())
    }
    
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        Ok(Vec::new())
    }
    
    async fn acknowledge_alert(&self, _alert_id: &str) -> Result<()> {
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        Ok(Vec::new())
    }
}

#[derive(Debug)]
struct MockHealthChecker {}

#[async_trait::async_trait]
impl HealthChecker for MockHealthChecker {
    async fn check_health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::default())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get_component_health<'a>(&'a self, _component: &'a str) -> Result<Option<crate::health::ComponentHealth>> {
        Ok(None)
    }
}