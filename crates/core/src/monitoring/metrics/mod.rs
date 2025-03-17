// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::cast_possible_wrap)] // Allow u64 to i64 casts for timestamps
#![allow(clippy::missing_errors_doc)] // Temporarily allow missing error documentation
#![allow(clippy::unused_async)] // Allow unused async functions
#![allow(clippy::doc_markdown)] // Allow documentation markdown issues

//! Metrics collection and management
//! 
//! This module provides functionality for:
//! - Resource usage metrics
//! - Performance metrics
//! - Custom metrics
//! - Metric export

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::Debug;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use crate::SquirrelError;
use crate::error::Result;
pub mod export;
pub mod performance;
pub mod resource;
/// Tool-specific metrics collection and tracking
pub mod tool;

// Re-export important types
pub use tool::ToolMetrics;
pub use export::MetricExporter;

/// Metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    /// Enable metric collection
    pub enabled: bool,
    /// Metric collection interval in seconds
    pub interval: u64,
    /// Maximum number of metrics to store
    pub max_metrics: usize,
}

impl Default for MetricConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            max_metrics: 1000,
        }
    }
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value that only increases
    Counter(u64),
    /// Gauge value that can go up and down
    Gauge(f64),
    /// Histogram value with buckets
    Histogram(Vec<f64>),
}

/// Metric type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    /// Counter metric that only increases
    Counter,
    /// Gauge metric that can go up and down
    Gauge,
    /// Histogram metric for distributions
    Histogram,
    /// Summary metric for distributions with quantiles
    Summary,
}

/// Metric data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: f64,
    /// Metric labels
    pub labels: HashMap<String, String>,
    /// Timestamp in seconds since Unix epoch
    pub timestamp: i64,
}

impl Metric {
    /// Create a new metric
    pub fn new(name: String, value: f64, metric_type: MetricType, labels: Option<HashMap<String, String>>) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels: labels.unwrap_or_default(),
            timestamp: system_time_to_timestamp(SystemTime::now()),
        }
    }
}

/// Metric collector trait
#[async_trait]
pub trait MetricCollector: Debug + Send + Sync {
    /// Collect metrics from the system
    async fn collect_metrics(&self) -> Result<Vec<Metric>>;

    /// Record a new metric
    async fn record_metric(&self, metric: Metric) -> Result<()>;

    /// Start the metric collector
    async fn start(&self) -> Result<()>;

    /// Stop the metric collector
    async fn stop(&self) -> Result<()>;
}

/// Metric source trait for components that provide metrics
#[async_trait]
pub trait MetricSource: Debug + Send + Sync {
    /// Get metrics from this source
    async fn get_metrics(&self) -> Result<Vec<Metric>>;
}

/// Default metric collector implementation
#[derive(Debug)]
pub struct DefaultMetricCollector {
    metrics: Arc<RwLock<Vec<Metric>>>,
    config: MetricConfig,
}

impl DefaultMetricCollector {
    /// Creates a new default metric collector with default configuration
    ///
    /// This initializes an empty metrics collection that will be populated
    /// with metrics when they are recorded or collected.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            config: MetricConfig::default(),
        }
    }
}

impl Default for DefaultMetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricCollector for DefaultMetricCollector {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await.clone();
        Ok(metrics)
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Enforce max metrics limit
        if metrics.len() > self.config.max_metrics {
            metrics.remove(0);
        }
        
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

/// Errors that can occur during metric operations
#[derive(Error, Debug)]
pub enum MetricError {
    /// The metric system has not been initialized
    #[error("Metric not initialized: {0}")]
    NotInitialized(String),
    
    /// An error occurred during metric export
    #[error("Export error: {0}")]
    ExportError(String),
    
    /// An error occurred during metric collection
    #[error("Collection error: {0}")]
    CollectionError(String),
    
    /// The metric data is invalid or malformed
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
}

impl From<MetricError> for SquirrelError {
    fn from(err: MetricError) -> Self {
        SquirrelError::metric(&err.to_string())
    }
}

/// Initialize metrics system
pub async fn initialize(_config: Option<MetricConfig>) -> Result<()> {
    // Implementation would go here
    Ok(())
}

/// Get the metric collector instance
pub fn get_collector() -> Option<Arc<DefaultMetricCollector>> {
    None // Placeholder
}

/// Check if metrics system is initialized
pub fn is_initialized() -> bool {
    false // Placeholder
}

/// Shutdown metrics system
pub async fn shutdown() -> Result<()> {
    // Implementation would go here
    Ok(())
}

/// Convert SystemTime to Unix timestamp
fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Records metrics for a tool
///
/// # Parameters
/// * `collector` - The metric collector to record metrics with
/// * `metrics` - A map of tool names to their metrics
///
/// # Errors
/// Returns an error if the metric collector fails to record the metrics
pub async fn record_tool_metrics<S: ::std::hash::BuildHasher>(
    collector: &dyn MetricCollector,
    metrics: &HashMap<String, ToolMetrics, S>,
) -> Result<()> {
    for (tool_name, tool_metrics) in metrics {
        let metric = Metric::new(
            format!("tool.{tool_name}.count"),
            tool_metrics.usage_count as f64,
            MetricType::Counter,
            None,
        );
        
        collector.record_metric(metric).await?;
        
        // Tool success rate
        let mut labels = HashMap::new();
        labels.insert("tool".to_string(), tool_name.clone());
        labels.insert("type".to_string(), "success_rate".to_string());
        
        let metric = Metric::new(
            "tool_success_rate".to_string(),
            tool_metrics.success_rate(),
            MetricType::Gauge,
            Some(labels),
        );
        
        collector.record_metric(metric).await?;
        
        // Tool average duration
        let mut labels = HashMap::new();
        labels.insert("tool".to_string(), tool_name.clone());
        labels.insert("type".to_string(), "average_duration".to_string());
        
        let metric = Metric::new(
            "tool_average_duration".to_string(),
            tool_metrics.average_duration,
            MetricType::Gauge,
            Some(labels),
        );
        
        collector.record_metric(metric).await?;
    }
    
    Ok(())
}

/// Gets all metrics from the monitoring system
///
/// # Returns
/// A vector of all metrics collected from the monitoring system
///
/// # Errors
/// Returns an error if the metrics cannot be collected or if the monitoring service is not initialized
pub async fn get_all_metrics() -> Result<Vec<Metric>> {
    if let Some(collector) = get_collector() {
        collector.collect_metrics().await
    } else {
        Ok(Vec::new())
    }
}

/// Metrics manager for coordinating multiple collectors and exporters
#[derive(Debug)]
pub struct MetricsManager {
    collectors: Arc<RwLock<Vec<Arc<dyn MetricCollector + Send + Sync>>>>,
    exporters: Arc<RwLock<Vec<Arc<dyn MetricExporter + Send + Sync>>>>,
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(Vec::new())),
            exporters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a metric collector to the manager
    ///
    /// # Parameters
    /// * `collector` - The collector to add
    ///
    /// # Errors
    /// Returns an error if the collectors lock cannot be acquired
    pub async fn add_collector(&self, collector: Arc<dyn MetricCollector + Send + Sync>) -> Result<()> {
        let mut collectors = self.collectors.write().await;
        collectors.push(collector);
        Ok(())
    }

    /// Adds a metric exporter to the manager
    ///
    /// # Parameters
    /// * `exporter` - The exporter to add
    ///
    /// # Errors
    /// Returns an error if the exporters lock cannot be acquired
    pub async fn add_exporter(&self, exporter: Arc<dyn MetricExporter + Send + Sync>) -> Result<()> {
        let mut exporters = self.exporters.write().await;
        exporters.push(exporter);
        Ok(())
    }

    /// Collects metrics from all registered collectors
    ///
    /// # Returns
    /// A vector of all collected metrics
    ///
    /// # Errors
    /// Returns an error if the collectors lock cannot be acquired or if any collector fails
    pub async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let collectors = self.collectors.read().await;

        let mut all_metrics = Vec::new();
        for collector in collectors.iter() {
            let metrics = collector.collect_metrics().await?;
            all_metrics.extend(metrics);
        }

        Ok(all_metrics)
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initializes the metric collector with the given configuration
///
/// # Parameters
/// * `_config` - The configuration for the metric collector
///
/// # Errors
/// Returns an error if the collector is already initialized or if initialization fails
pub fn init_collector(_config: MetricConfig) -> Result<()> {
    // Implementation would go here
    Ok(())
}

// Re-export additional types
pub use performance::PerformanceMetrics;
pub use resource::ResourceMetrics;

/// Gets metrics from the specified sources
///
/// # Parameters
/// * `sources` - The metric sources to collect from
///
/// # Returns
/// A vector of metrics collected from the sources
///
/// # Errors
/// Returns an error if any of the sources fail to provide metrics
pub async fn get_metrics(sources: &[Arc<dyn MetricSource + Send + Sync>]) -> Result<Vec<Metric>> {
    let mut metrics = Vec::new();
    for source in sources {
        let source_metrics = source.get_metrics().await?;
        metrics.extend(source_metrics);
    }
    Ok(metrics)
}

/// Records a counter metric with the given collector
///
/// # Parameters
/// * `collector` - The collector to record the metric with
/// * `name` - The name of the metric
/// * `value` - The value of the metric
/// * `labels` - Optional labels for the metric
///
/// # Errors
/// Returns an error if the collector fails to record the metric
pub async fn record_counter<S: ::std::hash::BuildHasher>(
    collector: &dyn MetricCollector,
    name: &str,
    value: f64,
    labels: Option<HashMap<String, String, S>>,
) -> Result<()> {
    let metric = Metric::new(
        name.to_string(),
        value,
        MetricType::Counter,
        labels.map(|l| l.into_iter().collect()),
    );
    collector.record_metric(metric).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_collector() -> impl MetricCollector {
        DefaultMetricCollector::default()
    }
    
    #[tokio::test]
    async fn test_metric_collector() {
        let collector = create_test_collector();
        
        // Create a test metric
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 1.0,
            metric_type: crate::monitoring::metrics::MetricType::Gauge,
            timestamp: 0,
            labels: HashMap::new(),
        };
        
        // Record the metric
        collector.record_metric(metric).await.unwrap();
        
        // Collect metrics and verify our test metric is included
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty(), "Should have collected at least one metric");
        assert!(
            metrics.iter().any(|m| m.name == "test_metric"), 
            "test_metric should be present in collected metrics"
        );
        
        // Add a second metric
        let second_metric = Metric {
            name: "second_metric".to_string(),
            value: 2.0,
            metric_type: crate::monitoring::metrics::MetricType::Counter,
            timestamp: 0,
            labels: HashMap::new(),
        };
        
        collector.record_metric(second_metric).await.unwrap();
        
        // Collect again and verify both metrics are present
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(metrics.len() >= 2, "Should have collected at least two metrics");
        
        // Clean up
        collector.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_metric_collector_lifecycle() {
        let collector = create_test_collector();
        
        // Start the collector
        collector.start().await.unwrap();
        
        // Record a metric
        let metric = Metric::new(
            "lifecycle_test".to_string(),
            1.0,
            MetricType::Gauge,
            None
        );
        
        collector.record_metric(metric).await.unwrap();
        
        // Verify metric was recorded
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(
            metrics.iter().any(|m| m.name == "lifecycle_test"),
            "Metric should be recorded after collector start"
        );
        
        // Stop the collector
        collector.stop().await.unwrap();
        
        // Collecting metrics should still work after stopping
        let metrics_after_stop = collector.collect_metrics().await;
        assert!(metrics_after_stop.is_ok(), "Collecting metrics should work after stopping");
    }
} 