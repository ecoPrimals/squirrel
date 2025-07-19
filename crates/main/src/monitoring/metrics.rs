//! # Metrics Collection Module
//!
//! This module provides comprehensive metrics collection capabilities for the Squirrel AI ecosystem.
//! It supports various metric types including counters, gauges, histograms, and summaries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::{CustomMetricDefinition, MetricType};
use crate::error::PrimalError;

/// Metrics collection engine
pub struct MetricsCollector {
    /// Registered metrics
    metrics: Arc<RwLock<HashMap<String, MetricDefinition>>>,
    /// Metric values storage
    values: Arc<RwLock<HashMap<String, MetricValue>>>,
    /// Component metrics
    component_metrics: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
    /// System metrics
    system_metrics: Arc<RwLock<SystemMetrics>>,
    /// Collection history
    history: Arc<RwLock<Vec<MetricSnapshot>>>,
    /// Maximum history size
    max_history_size: usize,
}

/// Internal metric definition
#[derive(Debug, Clone)]
struct MetricDefinition {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub labels: Vec<String>,
    pub unit: String,
    pub source: String,
}

/// Metric value with labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub metric_type: MetricType,
}

/// System-wide metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage percentage
    pub memory_percentage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network bytes sent per second
    pub network_bytes_sent: f64,
    /// Network bytes received per second
    pub network_bytes_received: f64,
    /// Number of active connections
    pub active_connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// System uptime in seconds
    pub uptime: u64,
}

/// Metric snapshot for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, MetricValue>,
    pub system_metrics: SystemMetrics,
}

/// All metrics data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMetrics {
    pub metrics: HashMap<String, MetricValue>,
    pub component_metrics: HashMap<String, HashMap<String, f64>>,
    pub system_metrics: SystemMetrics,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            values: Arc::new(RwLock::new(HashMap::new())),
            component_metrics: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    /// Register a custom metric
    pub async fn register_custom_metric(
        &self,
        definition: CustomMetricDefinition,
    ) -> Result<(), PrimalError> {
        let mut metrics = self.metrics.write().await;

        let metric_def = MetricDefinition {
            name: definition.name.clone(),
            metric_type: definition.metric_type,
            description: definition.description,
            labels: definition.labels,
            unit: definition.unit,
            source: definition.source,
        };

        metrics.insert(definition.name.clone(), metric_def);
        info!("Registered custom metric: {}", definition.name);

        Ok(())
    }

    /// Record a metric value
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), PrimalError> {
        let metrics = self.metrics.read().await;

        if let Some(metric_def) = metrics.get(name) {
            let mut values = self.values.write().await;

            let metric_value = MetricValue {
                value,
                labels,
                timestamp: Utc::now(),
                metric_type: metric_def.metric_type.clone(),
            };

            values.insert(name.to_string(), metric_value);
            debug!("Recorded metric: {} = {}", name, value);

            Ok(())
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Metric '{}' not registered",
                name
            )))
        }
    }

    /// Get metrics for a specific component
    pub async fn get_component_metrics(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        let component_metrics = self.component_metrics.read().await;

        if let Some(metrics) = component_metrics.get(component) {
            Ok(metrics.clone())
        } else {
            Ok(HashMap::new())
        }
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Result<AllMetrics, PrimalError> {
        let values = self.values.read().await;
        let component_metrics = self.component_metrics.read().await;
        let system_metrics = self.system_metrics.read().await;

        Ok(AllMetrics {
            metrics: values.clone(),
            component_metrics: component_metrics.clone(),
            system_metrics: system_metrics.clone(),
        })
    }

    /// Collect all metrics from various sources
    pub async fn collect_metrics(&self) -> Result<(), PrimalError> {
        debug!("Collecting metrics from all sources");

        // Collect system metrics
        self.collect_system_metrics().await?;

        // Collect component metrics
        self.collect_component_metrics().await?;

        // Create snapshot
        self.create_snapshot().await?;

        debug!("Metrics collection completed");
        Ok(())
    }

    /// Collect system-wide metrics
    async fn collect_system_metrics(&self) -> Result<(), PrimalError> {
        let mut system_metrics = self.system_metrics.write().await;

        // In a real implementation, these would come from actual system monitoring
        // For now, we'll use simulated values
        system_metrics.cpu_usage = self.get_cpu_usage().await?;
        system_metrics.memory_usage = self.get_memory_usage().await?;
        system_metrics.memory_percentage = self.get_memory_percentage().await?;
        system_metrics.disk_usage = self.get_disk_usage().await?;
        system_metrics.network_bytes_sent = self.get_network_bytes_sent().await?;
        system_metrics.network_bytes_received = self.get_network_bytes_received().await?;
        system_metrics.active_connections = self.get_active_connections().await?;
        system_metrics.request_rate = self.get_request_rate().await?;
        system_metrics.error_rate = self.get_error_rate().await?;
        system_metrics.avg_response_time = self.get_avg_response_time().await?;
        system_metrics.uptime = self.get_uptime().await?;

        Ok(())
    }

    /// Collect component-specific metrics
    async fn collect_component_metrics(&self) -> Result<(), PrimalError> {
        let mut component_metrics = self.component_metrics.write().await;

        // Collect metrics from each component
        let components = vec![
            "ai_intelligence",
            "mcp_integration",
            "context_state",
            "agent_deployment",
            "songbird",
            "toadstool",
            "nestgate",
            "beardog",
        ];

        for component in components {
            let metrics = self.collect_component_specific_metrics(component).await?;
            component_metrics.insert(component.to_string(), metrics);
        }

        Ok(())
    }

    /// Collect metrics for a specific component
    async fn collect_component_specific_metrics(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        let mut metrics = HashMap::new();

        match component {
            "ai_intelligence" => {
                metrics.insert("requests_processed".to_string(), 42.0);
                metrics.insert("avg_processing_time".to_string(), 150.0);
                metrics.insert("success_rate".to_string(), 0.95);
                metrics.insert("memory_usage".to_string(), 256.0);
            }
            "mcp_integration" => {
                metrics.insert("messages_sent".to_string(), 128.0);
                metrics.insert("messages_received".to_string(), 134.0);
                metrics.insert("connection_count".to_string(), 5.0);
                metrics.insert("protocol_errors".to_string(), 2.0);
            }
            "context_state" => {
                metrics.insert("active_sessions".to_string(), 8.0);
                metrics.insert("context_size".to_string(), 1024.0);
                metrics.insert("cache_hit_rate".to_string(), 0.87);
                metrics.insert("persistence_latency".to_string(), 25.0);
            }
            "agent_deployment" => {
                metrics.insert("deployed_agents".to_string(), 12.0);
                metrics.insert("running_agents".to_string(), 10.0);
                metrics.insert("failed_deployments".to_string(), 1.0);
                metrics.insert("deployment_time".to_string(), 30.0);
            }
            "songbird" => {
                metrics.insert("orchestrations_active".to_string(), 3.0);
                metrics.insert("service_discoveries".to_string(), 15.0);
                metrics.insert("load_balancer_requests".to_string(), 89.0);
                metrics.insert("health_checks".to_string(), 24.0);
            }
            "toadstool" => {
                metrics.insert("compute_jobs_queued".to_string(), 6.0);
                metrics.insert("compute_jobs_running".to_string(), 4.0);
                metrics.insert("compute_jobs_completed".to_string(), 45.0);
                metrics.insert("cpu_utilization".to_string(), 0.72);
            }
            "nestgate" => {
                metrics.insert("storage_operations".to_string(), 156.0);
                metrics.insert("storage_size_gb".to_string(), 2.4);
                metrics.insert("backup_operations".to_string(), 3.0);
                metrics.insert("replication_lag".to_string(), 12.0);
            }
            "beardog" => {
                metrics.insert("authentication_requests".to_string(), 67.0);
                metrics.insert("authorization_checks".to_string(), 234.0);
                metrics.insert("security_violations".to_string(), 0.0);
                metrics.insert("token_refreshes".to_string(), 8.0);
            }
            _ => {
                // Default metrics for unknown components
                metrics.insert("status".to_string(), 1.0);
                metrics.insert("uptime".to_string(), 3600.0);
            }
        }

        Ok(metrics)
    }

    /// Create a snapshot of current metrics
    async fn create_snapshot(&self) -> Result<(), PrimalError> {
        let values = self.values.read().await;
        let system_metrics = self.system_metrics.read().await;

        let snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            metrics: values.clone(),
            system_metrics: system_metrics.clone(),
        };

        let mut history = self.history.write().await;
        history.push(snapshot);

        // Limit history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        Ok(())
    }

    /// System metric collection helpers (simplified implementations)
    async fn get_cpu_usage(&self) -> Result<f64, PrimalError> {
        // In a real implementation, this would read from /proc/stat or similar
        Ok(25.5)
    }

    async fn get_memory_usage(&self) -> Result<u64, PrimalError> {
        // In a real implementation, this would read from /proc/meminfo
        Ok(1024 * 1024 * 512) // 512 MB
    }

    async fn get_memory_percentage(&self) -> Result<f64, PrimalError> {
        Ok(32.8)
    }

    async fn get_disk_usage(&self) -> Result<f64, PrimalError> {
        Ok(45.2)
    }

    async fn get_network_bytes_sent(&self) -> Result<f64, PrimalError> {
        Ok(1024.0 * 50.0) // 50 KB/s
    }

    async fn get_network_bytes_received(&self) -> Result<f64, PrimalError> {
        Ok(1024.0 * 75.0) // 75 KB/s
    }

    async fn get_active_connections(&self) -> Result<u32, PrimalError> {
        Ok(12)
    }

    async fn get_request_rate(&self) -> Result<f64, PrimalError> {
        Ok(45.7)
    }

    async fn get_error_rate(&self) -> Result<f64, PrimalError> {
        Ok(0.8)
    }

    async fn get_avg_response_time(&self) -> Result<f64, PrimalError> {
        Ok(125.3)
    }

    async fn get_uptime(&self) -> Result<u64, PrimalError> {
        Ok(3600 * 24 * 5) // 5 days
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_percentage: 0.0,
            disk_usage: 0.0,
            network_bytes_sent: 0.0,
            network_bytes_received: 0.0,
            active_connections: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            avg_response_time: 0.0,
            uptime: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::CustomMetricDefinition;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.metrics.read().await.is_empty());
        assert!(collector.values.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_custom_metric_registration() {
        let collector = MetricsCollector::new();

        let metric_def = CustomMetricDefinition {
            name: "test_counter".to_string(),
            metric_type: MetricType::Counter,
            description: "Test counter metric".to_string(),
            labels: vec!["service".to_string()],
            unit: "count".to_string(),
            source: "test".to_string(),
        };

        let result = collector.register_custom_metric(metric_def).await;
        assert!(result.is_ok());

        let metrics = collector.metrics.read().await;
        assert!(metrics.contains_key("test_counter"));
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let collector = MetricsCollector::new();

        // First register a metric
        let metric_def = CustomMetricDefinition {
            name: "test_gauge".to_string(),
            metric_type: MetricType::Gauge,
            description: "Test gauge metric".to_string(),
            labels: vec!["component".to_string()],
            unit: "bytes".to_string(),
            source: "test".to_string(),
        };

        collector.register_custom_metric(metric_def).await.unwrap();

        // Now record a value
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "test_component".to_string());

        let result = collector.record_metric("test_gauge", 42.0, labels).await;
        assert!(result.is_ok());

        let values = collector.values.read().await;
        assert!(values.contains_key("test_gauge"));
        assert_eq!(values.get("test_gauge").unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        let result = collector.collect_metrics().await;
        assert!(result.is_ok());

        let system_metrics = collector.system_metrics.read().await;
        assert!(system_metrics.cpu_usage > 0.0);
        assert!(system_metrics.memory_usage > 0);

        let component_metrics = collector.component_metrics.read().await;
        assert!(component_metrics.contains_key("ai_intelligence"));
        assert!(component_metrics.contains_key("mcp_integration"));
    }

    #[tokio::test]
    async fn test_component_metrics_retrieval() {
        let collector = MetricsCollector::new();

        // Collect metrics first
        collector.collect_metrics().await.unwrap();

        let ai_metrics = collector
            .get_component_metrics("ai_intelligence")
            .await
            .unwrap();
        assert!(!ai_metrics.is_empty());
        assert!(ai_metrics.contains_key("requests_processed"));
        assert!(ai_metrics.contains_key("avg_processing_time"));
    }
}
