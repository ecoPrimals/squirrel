//! Dashboard-Monitoring Integration
//!
//! This module provides adapters and services for integrating the monitoring system
//! with the dashboard interface. It enables visualization of monitoring data through
//! the dashboard UI.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use squirrel_monitoring::api::MonitoringAPI;
use squirrel_monitoring::alerts::status::AlertSeverity as MonitoringAlertSeverity;

use crate::data::{
    DashboardData, Metrics, CpuMetrics, MemoryMetrics, NetworkMetrics, 
    DiskMetrics, Alert, AlertSeverity, Protocol, ProtocolStatus, ProtocolData
};
use crate::error::{Result, DashboardError};
use crate::service::DashboardService;

/// Configuration for the monitoring adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringAdapterConfig {
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Whether to use WebSocket for updates
    pub use_websocket: bool,
    /// Whether to enable data caching
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl Default for MonitoringAdapterConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 5000,
            use_websocket: true,
            enable_caching: true,
            max_cache_size: 100,
        }
    }
}

/// Error types for the monitoring adapter
#[derive(Debug, thiserror::Error)]
pub enum MonitoringAdapterError {
    /// Monitoring API error
    #[error("Monitoring API error: {0}")]
    MonitoringApiError(String),
    
    /// Dashboard service error
    #[error("Dashboard service error: {0}")]
    DashboardServiceError(String),
    
    /// Data transformation error
    #[error("Data transformation error: {0}")]
    DataTransformationError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Result type for the monitoring adapter
pub type MonitoringAdapterResult<T> = std::result::Result<T, MonitoringAdapterError>;

/// Adapter for integrating monitoring data with the dashboard
pub struct MonitoringDataAdapter {
    /// Monitoring API client
    monitoring_api: Arc<dyn MonitoringAPI>,
    
    /// Dashboard service
    dashboard_service: Arc<dyn DashboardService>,
    
    /// Metrics transformation service
    metrics_transformer: Arc<MetricsTransformationService>,
    
    /// Alert visualization adapter
    alert_adapter: Arc<AlertVisualizationAdapter>,
    
    /// Configuration
    config: MonitoringAdapterConfig,
    
    /// Cache of dashboard data
    cache: Arc<RwLock<Vec<DashboardData>>>,
    
    /// Running state
    running: Arc<Mutex<bool>>,
    
    /// Last update time
    last_update: Arc<Mutex<Instant>>,
}

/// Interface for providing dashboard data
#[async_trait]
pub trait DashboardDataProvider: Send + Sync {
    /// Get the latest dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData>;
    
    /// Start the data provider
    async fn start(&self) -> Result<()>;
    
    /// Stop the data provider
    async fn stop(&self) -> Result<()>;
}

impl MonitoringDataAdapter {
    /// Create a new monitoring data adapter
    pub fn new(
        monitoring_api: Arc<dyn MonitoringAPI>,
        dashboard_service: Arc<dyn DashboardService>,
        config: MonitoringAdapterConfig,
    ) -> Self {
        let metrics_transformer = Arc::new(MetricsTransformationService::new(Default::default()));
        let alert_adapter = Arc::new(AlertVisualizationAdapter::new(Default::default()));
        
        Self {
            monitoring_api,
            dashboard_service,
            metrics_transformer,
            alert_adapter,
            config,
            cache: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// Start the adapter
    pub async fn start(&self) -> MonitoringAdapterResult<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Create a cloned value for the background task
        let monitoring_api = self.monitoring_api.clone();
        let dashboard_service = self.dashboard_service.clone();
        let metrics_transformer = self.metrics_transformer.clone();
        let alert_adapter = self.alert_adapter.clone();
        let config = self.config.clone();
        let cache = self.cache.clone();
        let running_clone = self.running.clone();
        let update_interval_ms = self.config.update_interval_ms;
        
        tokio::spawn(async move {
            let mut update_interval = interval(Duration::from_millis(update_interval_ms));
            
            while *running_clone.lock().unwrap() {
                update_interval.tick().await;
                
                // Fetch monitoring data
                let cpu_data = match monitoring_api.get_component_data("cpu").await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error fetching CPU data: {}", e);
                        continue;
                    }
                };
                
                let memory_data = match monitoring_api.get_component_data("memory").await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error fetching memory data: {}", e);
                        continue;
                    }
                };
                
                let network_data = match monitoring_api.get_component_data("network").await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error fetching network data: {}", e);
                        continue;
                    }
                };
                
                let disk_data = match monitoring_api.get_component_data("disk").await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error fetching disk data: {}", e);
                        continue;
                    }
                };
                
                let alerts_data = match monitoring_api.get_component_data("alerts").await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error fetching alerts data: {}", e);
                        continue;
                    }
                };
                
                // Transform metrics and alerts
                let metrics = match metrics_transformer.transform_metrics(
                    cpu_data, memory_data, network_data, disk_data
                ).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Error transforming metrics: {}", e);
                        continue;
                    }
                };
                
                let alerts = match alert_adapter.transform_alerts(alerts_data).await {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Error transforming alerts: {}", e);
                        continue;
                    }
                };
                
                // Create dashboard data
                let dashboard_data = DashboardData {
                    metrics,
                    protocol: ProtocolData::default(),
                    alerts,
                    timestamp: Utc::now(),
                };
                
                // Update the cache if enabled
                if config.enable_caching {
                    let mut cache_lock = cache.write().await;
                    cache_lock.push(dashboard_data.clone());
                    
                    // Trim cache if it exceeds max size
                    if cache_lock.len() > config.max_cache_size {
                        cache_lock.remove(0);
                    }
                }
                
                // Send the data to the dashboard service
                if let Err(e) = dashboard_service.update_dashboard_data(dashboard_data).await {
                    eprintln!("Error updating dashboard service: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the adapter
    pub fn stop(&self) -> MonitoringAdapterResult<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }
    
    /// Update the dashboard data
    async fn update(&self) -> MonitoringAdapterResult<()> {
        // Fetch monitoring data
        let dashboard_data = self.fetch_dashboard_data().await?;
        
        // Update the cache if enabled
        if self.config.enable_caching {
            let mut cache = self.cache.write().await;
            cache.push(dashboard_data.clone());
            
            // Trim cache if it exceeds max size
            if cache.len() > self.config.max_cache_size {
                cache.remove(0);
            }
        }
        
        // Update the last update time
        let mut last_update = self.last_update.lock().unwrap();
        *last_update = Instant::now();
        
        // Send the data to the dashboard service
        self.dashboard_service
            .update_dashboard_data(dashboard_data)
            .await
            .map_err(|e| MonitoringAdapterError::DashboardServiceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Fetch dashboard data from the monitoring system
    async fn fetch_dashboard_data(&self) -> MonitoringAdapterResult<DashboardData> {
        // Get system metrics
        let cpu_data = self.monitoring_api
            .get_component_data("cpu")
            .await
            .map_err(|e| MonitoringAdapterError::MonitoringApiError(e.to_string()))?;
        
        let memory_data = self.monitoring_api
            .get_component_data("memory")
            .await
            .map_err(|e| MonitoringAdapterError::MonitoringApiError(e.to_string()))?;
        
        let network_data = self.monitoring_api
            .get_component_data("network")
            .await
            .map_err(|e| MonitoringAdapterError::MonitoringApiError(e.to_string()))?;
        
        let disk_data = self.monitoring_api
            .get_component_data("disk")
            .await
            .map_err(|e| MonitoringAdapterError::MonitoringApiError(e.to_string()))?;
        
        // Transform metrics
        let metrics = self.metrics_transformer
            .transform_metrics(cpu_data, memory_data, network_data, disk_data)
            .await?;
        
        // Get alerts
        let alerts_data = self.monitoring_api
            .get_component_data("alerts")
            .await
            .map_err(|e| MonitoringAdapterError::MonitoringApiError(e.to_string()))?;
        
        // Transform alerts
        let alerts = self.alert_adapter
            .transform_alerts(alerts_data)
            .await?;
        
        // Create dashboard data
        let dashboard_data = DashboardData {
            metrics,
            protocol: ProtocolData::default(),
            alerts,
            timestamp: Utc::now(),
        };
        
        Ok(dashboard_data)
    }
    
    /// Get the latest dashboard data
    pub async fn get_latest_dashboard_data(&self) -> MonitoringAdapterResult<DashboardData> {
        if self.config.enable_caching {
            let cache = self.cache.read().await;
            if let Some(data) = cache.last() {
                return Ok(data.clone());
            }
        }
        
        self.fetch_dashboard_data().await
    }
}

impl Clone for MonitoringDataAdapter {
    fn clone(&self) -> Self {
        Self {
            monitoring_api: self.monitoring_api.clone(),
            dashboard_service: self.dashboard_service.clone(),
            metrics_transformer: self.metrics_transformer.clone(),
            alert_adapter: self.alert_adapter.clone(),
            config: self.config.clone(),
            cache: self.cache.clone(),
            running: self.running.clone(),
            last_update: self.last_update.clone(),
        }
    }
}

/// Configuration for metrics transformation
#[derive(Debug, Clone, Default)]
pub struct MetricsTransformationConfig {}

/// Service for transforming monitoring metrics to dashboard format
pub struct MetricsTransformationService {
    /// Configuration
    config: MetricsTransformationConfig,
}

impl MetricsTransformationService {
    /// Create a new metrics transformation service
    pub fn new(config: MetricsTransformationConfig) -> Self {
        Self { config }
    }
    
    /// Transform metrics from monitoring format to dashboard format
    pub async fn transform_metrics(
        &self,
        cpu_data: Value,
        memory_data: Value,
        network_data: Value,
        disk_data: Value,
    ) -> MonitoringAdapterResult<Metrics> {
        // Extract CPU metrics
        let cpu_usage = cpu_data.get("usage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let cpu_cores = cpu_data.get("cores")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .collect::<Vec<f64>>()
            })
            .unwrap_or_default();
        
        let cpu_load = [
            cpu_data.get("load_1")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            cpu_data.get("load_5")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            cpu_data.get("load_15")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        ];
        
        let cpu_metrics = CpuMetrics {
            usage: cpu_usage,
            cores: cpu_cores,
            temperature: cpu_data.get("temperature")
                .and_then(|v| v.as_f64()),
            load: cpu_load,
        };
        
        // Extract memory metrics
        let memory_metrics = MemoryMetrics {
            total: memory_data.get("total")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            used: memory_data.get("used")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            available: memory_data.get("available")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            free: memory_data.get("free")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            swap_used: memory_data.get("swap_used")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            swap_total: memory_data.get("swap_total")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        };
        
        // Create empty network and disk metrics for now
        // In a real implementation, these would be populated from the monitoring data
        let network_metrics = NetworkMetrics::default();
        let disk_metrics = DiskMetrics::default();
        
        // Create the metrics object
        let metrics = Metrics {
            cpu: cpu_metrics,
            memory: memory_metrics,
            network: network_metrics,
            disk: disk_metrics,
            history: Default::default(),
        };
        
        Ok(metrics)
    }
}

/// Configuration for alert visualization
#[derive(Debug, Clone, Default)]
pub struct AlertVisualizationConfig {}

/// Adapter for visualizing alerts in the dashboard
pub struct AlertVisualizationAdapter {
    /// Configuration
    config: AlertVisualizationConfig,
}

impl AlertVisualizationAdapter {
    /// Create a new alert visualization adapter
    pub fn new(config: AlertVisualizationConfig) -> Self {
        Self { config }
    }
    
    /// Transform alerts from monitoring format to dashboard format
    pub async fn transform_alerts(&self, alerts_data: Value) -> MonitoringAdapterResult<Vec<Alert>> {
        let alerts_arr = match alerts_data.get("alerts") {
            Some(Value::Array(arr)) => arr,
            _ => return Ok(Vec::new()),
        };
        
        let mut dashboard_alerts = Vec::new();
        
        for alert_value in alerts_arr {
            if let Some(alert_obj) = alert_value.as_object() {
                let id = alert_obj.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let title = alert_obj.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Alert")
                    .to_string();
                
                let message = alert_obj.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                let source = alert_obj.get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Map the severity from monitoring to dashboard format
                let severity_str = alert_obj.get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");
                
                let severity = match severity_str.to_lowercase().as_str() {
                    "critical" => AlertSeverity::Critical,
                    "error" => AlertSeverity::Error,
                    "warning" => AlertSeverity::Warning,
                    _ => AlertSeverity::Info,
                };
                
                let timestamp = alert_obj.get("timestamp")
                    .and_then(|v| v.as_i64())
                    .map(|ts| {
                        let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
                        dt
                    })
                    .unwrap_or_else(Utc::now);
                
                let acknowledged = alert_obj.get("acknowledged")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                let acknowledged_by = alert_obj.get("acknowledged_by")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let acknowledged_at = alert_obj.get("acknowledged_at")
                    .and_then(|v| v.as_i64())
                    .map(|ts| {
                        let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
                        dt
                    });
                
                dashboard_alerts.push(Alert {
                    id,
                    title,
                    message,
                    severity,
                    source,
                    timestamp,
                    acknowledged,
                    acknowledged_by,
                    acknowledged_at,
                });
            }
        }
        
        Ok(dashboard_alerts)
    }
}

/// Map a monitoring alert severity to a dashboard alert severity
fn map_alert_severity(severity: MonitoringAlertSeverity) -> AlertSeverity {
    match severity {
        MonitoringAlertSeverity::Critical => AlertSeverity::Critical,
        MonitoringAlertSeverity::Error => AlertSeverity::Error,
        MonitoringAlertSeverity::Warning => AlertSeverity::Warning,
        MonitoringAlertSeverity::Info => AlertSeverity::Info,
    }
}

/// Initialize the dashboard-monitoring integration
pub fn initialize_dashboard_monitoring(
    monitoring_api: Arc<dyn MonitoringAPI>,
    dashboard_service: Arc<dyn DashboardService>,
    config: MonitoringAdapterConfig,
) -> Arc<MonitoringDataAdapter> {
    let adapter = Arc::new(MonitoringDataAdapter::new(
        monitoring_api,
        dashboard_service,
        config,
    ));
    
    tokio::spawn({
        let adapter = adapter.clone();
        async move {
            if let Err(e) = adapter.start().await {
                eprintln!("Failed to start monitoring adapter: {}", e);
            }
        }
    });
    
    adapter
}

#[async_trait]
impl DashboardDataProvider for MonitoringDataAdapter {
    async fn get_dashboard_data(&self) -> Result<DashboardData> {
        self.get_latest_dashboard_data()
            .await
            .map_err(|e| DashboardError::External(e.to_string()))
    }
    
    async fn start(&self) -> Result<()> {
        MonitoringDataAdapter::start(self)
            .await
            .map_err(|e| DashboardError::External(e.to_string()))
    }
    
    async fn stop(&self) -> Result<()> {
        MonitoringDataAdapter::stop(self)
            .map_err(|e| DashboardError::External(e.to_string()))
    }
} 