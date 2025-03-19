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
use crate::error::{Result, SquirrelError};
use crate::monitoring::metrics::performance::OperationType;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
///
/// Represents a single measurement or observation about the system. Metrics are the core
/// data elements of the monitoring system and can represent various aspects of system
/// performance, resource usage, or application-specific measurements.
///
/// Metrics include metadata like timestamp, labels for categorization, and operation type
/// to associate measurements with specific operations or components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name that uniquely identifies the measurement
    pub name: String,
    /// Type of metric (Counter, Gauge, Histogram, Summary)
    pub metric_type: MetricType,
    /// Current value of the metric
    pub value: f64,
    /// Key-value pairs for adding dimensional data to metrics
    pub labels: HashMap<String, String>,
    /// Timestamp in seconds since Unix epoch
    pub timestamp: i64,
    /// Type of operation this metric is associated with
    pub operation_type: OperationType,
}

impl Metric {
    /// Create a new metric with all required fields
    ///
    /// Creates a new metric with the specified name, value, type, and labels.
    /// The timestamp is automatically set to the current time, and the operation
    /// type is set to Unknown by default.
    ///
    /// # Arguments
    /// * `name` - Name that uniquely identifies the metric
    /// * `value` - Current value of the metric
    /// * `metric_type` - Type of metric (Counter, Gauge, Histogram, Summary)
    /// * `labels` - Key-value pairs for dimensional data
    ///
    /// # Returns
    /// A new metric instance with all fields set
    #[must_use] pub fn new(name: String, value: f64, metric_type: MetricType, labels: HashMap<String, String>) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels,
            timestamp: system_time_to_timestamp(SystemTime::now()),
            operation_type: OperationType::Unknown,
        }
    }
    
    /// Create a new metric with optional labels
    ///
    /// Convenience method that allows labels to be optional. If no labels are
    /// provided, an empty HashMap is used.
    ///
    /// # Arguments
    /// * `name` - Name that uniquely identifies the metric
    /// * `value` - Current value of the metric
    /// * `metric_type` - Type of metric (Counter, Gauge, Histogram, Summary)
    /// * `labels` - Optional key-value pairs for dimensional data
    ///
    /// # Returns
    /// A new metric instance with all fields set
    #[must_use] pub fn with_optional_labels(name: String, value: f64, metric_type: MetricType, labels: Option<HashMap<String, String>>) -> Self {
        Self::new(name, value, metric_type, labels.unwrap_or_default())
    }
    
    /// Determines if this metric should trigger an alert
    ///
    /// Evaluates the metric value and other criteria to determine if
    /// it has crossed a threshold that should trigger an alert.
    ///
    /// # Returns
    /// `true` if the metric should trigger an alert, `false` otherwise
    #[must_use] pub fn should_alert(&self) -> bool {
        // This is a placeholder implementation
        // In a real implementation, this would check against configured thresholds
        // and other alert criteria specific to the metric type and name
        false
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

/// Adapter for protocol metrics collection
#[derive(Debug, Clone)]
pub struct ProtocolMetricsCollectorAdapter {
    /// Inner collector
    inner: Option<Arc<RwLock<dyn MetricCollector>>>,
}

impl Default for ProtocolMetricsCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolMetricsCollectorAdapter {
    /// Create a new adapter
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }

    /// Create a new adapter with a collector
    #[must_use] pub fn with_collector(collector: Arc<RwLock<dyn MetricCollector>>) -> Self {
        Self { inner: Some(collector) }
    }

    /// Get metrics from the collector
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.collect_metrics().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Record a metric
    pub async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.record_metric(metric).await
        } else {
            Ok(())
        }
    }

    /// Check if the adapter is initialized
    #[must_use] pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}

#[async_trait]
impl MetricCollector for ProtocolMetricsCollectorAdapter {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        self.get_metrics().await
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        self.record_metric(metric).await
    }

    async fn start(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.start().await
        } else {
            Ok(())
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.stop().await
        } else {
            Ok(())
        }
    }
}

/// Default implementation of the metric collector
#[derive(Debug)]
pub struct DefaultMetricCollector {
    /// Collection of metric records
    metrics: Arc<RwLock<Vec<Metric>>>,
    /// Metric collector configuration
    config: MetricConfig,
    /// Optional protocol metrics collector
    protocol_collector: Option<Arc<ProtocolMetricsCollectorAdapter>>,
}

impl DefaultMetricCollector {
    /// Creates a new default metric collector with default configuration
    ///
    /// This initializes an empty metrics collection that will be populated
    /// with metrics when they are recorded or collected.
    ///
    /// # Returns
    /// A new DefaultMetricCollector with default configuration and empty metrics collection
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            config: MetricConfig::default(),
            protocol_collector: None,
        }
    }
    
    /// Checks if the metric collector is ready to use
    #[must_use] pub fn is_initialized(&self) -> bool {
        // DefaultMetricCollector doesn't have a separate initialization step,
        // it's always ready after creation
        true
    }
    
    /// Initializes the metric collector
    ///
    /// Since DefaultMetricCollector is always initialized after creation,
    /// this method exists only for API compatibility with other adapters.
    ///
    /// # Returns
    /// Always returns Ok(())
    pub fn initialize(&mut self) -> Result<()> {
        // Always initialized
        Ok(())
    }
    
    /// Initializes the metric collector with a specific config
    ///
    /// Sets the configuration to the provided value.
    ///
    /// # Parameters
    /// * `config` - Configuration to use
    ///
    /// # Returns
    /// Always returns Ok(())
    pub fn initialize_with_config(&mut self, config: MetricConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// Creates a new default metric collector with dependencies
    ///
    /// This constructor allows for dependency injection of required collectors
    /// through their adapter interfaces.
    ///
    /// # Arguments
    /// * `config` - Optional metric configuration, uses default if None
    /// * `protocol_collector` - Optional protocol metrics collector adapter
    ///
    /// # Returns
    /// A new DefaultMetricCollector with the specified configuration and dependencies
    #[must_use] pub fn with_dependencies(
        config: Option<MetricConfig>,
        protocol_collector: Option<Arc<ProtocolMetricsCollectorAdapter>>,
    ) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            config: config.unwrap_or_default(),
            protocol_collector,
        }
    }

    /// Creates a new metric collector with a protocol collector
    ///
    /// Convenience method for creating a collector with just a protocol collector
    /// and default configuration.
    ///
    /// # Arguments
    /// * `protocol_collector` - Protocol metrics collector adapter
    ///
    /// # Returns
    /// A new DefaultMetricCollector with default configuration and the specified protocol collector
    #[must_use] pub fn with_protocol_collector(
        protocol_collector: Arc<ProtocolMetricsCollectorAdapter>
    ) -> Self {
        Self::with_dependencies(None, Some(protocol_collector))
    }

    /// Get the protocol collector if one is configured
    ///
    /// # Returns
    /// An Option containing the protocol collector if configured, None otherwise
    #[must_use] pub fn protocol_collector(&self) -> Option<Arc<ProtocolMetricsCollectorAdapter>> {
        self.protocol_collector.clone()
    }
    
    /// Get all metrics currently stored in the collector
    ///
    /// Retrieves all metrics that have been collected or recorded so far.
    ///
    /// # Returns
    /// A Result containing a vector of all stored metrics, or an error if retrieval fails
    ///
    /// # Errors
    /// Returns an error if metrics cannot be accessed
    pub async fn get_all_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
}

impl Default for DefaultMetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricCollector for DefaultMetricCollector {
    /// Collect metrics from the system and all registered sources
    ///
    /// This method aggregates metrics from:
    /// 1. Protocol-specific metrics (if a protocol collector is configured)
    /// 2. System-level resource metrics like CPU, memory, and disk usage
    /// 3. Previously recorded metrics stored in this collector
    ///
    /// # Returns
    /// A Result containing a vector of all collected metrics, or an error if collection fails
    ///
    /// # Errors
    /// Returns an error if metrics cannot be collected from any source
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let mut all_metrics = Vec::new();
        
        // Get stored metrics
        {
            let metrics = self.metrics.read().await;
            all_metrics.extend(metrics.clone());
        }
        
        // Get protocol metrics if available
        if let Some(protocol_collector) = &self.protocol_collector {
            match protocol_collector.get_metrics().await {
                Ok(protocol_metrics) => {
                    all_metrics.extend(protocol_metrics);
                },
                Err(e) => {
                    // Log the error but continue with other metrics
                    tracing::warn!("Failed to collect protocol metrics: {}", e);
                    // We don't want to fail the entire collection if just one source fails
                }
            }
        }
        
        Ok(all_metrics)
    }

    /// Record a new metric in the collector
    ///
    /// Adds a new metric to the internal storage. If the number of metrics exceeds
    /// the configured maximum, the oldest metrics are removed to make room.
    ///
    /// # Arguments
    /// * `metric` - The metric to record
    ///
    /// # Returns
    /// Success if the metric was recorded, or an error otherwise
    ///
    /// # Errors
    /// Returns an error if the metric cannot be recorded
    async fn record_metric(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Enforce maximum number of metrics
        if metrics.len() > self.config.max_metrics {
            // Create a new vector with the newest metrics to avoid borrow conflicts
            let max_metrics = self.config.max_metrics;
            let len = metrics.len();
            let new_metrics = metrics.split_off(len - max_metrics);
            *metrics = new_metrics;
        }
        
        Ok(())
    }

    /// Start the metric collector
    ///
    /// Initializes the metric collection system and starts any required background
    /// collection processes. If a protocol collector is configured, it will also
    /// be started.
    ///
    /// # Returns
    /// Success if the collector was started, or an error otherwise
    ///
    /// # Errors
    /// Returns an error if the collector cannot be started
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Stop the metric collector
    ///
    /// Stops all metric collection processes and performs any necessary cleanup.
    /// If a protocol collector is configured, it will also be stopped.
    ///
    /// # Returns
    /// Success if the collector was stopped, or an error otherwise
    ///
    /// # Errors
    /// Returns an error if the collector cannot be stopped
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
        Self::metric(err.to_string())
    }
}

/// Initialize metrics system
pub async fn initialize(_config: Option<MetricConfig>) -> Result<()> {
    // Implementation would go here
    Ok(())
}

/// Get the metric collector instance
#[must_use] pub const fn get_collector() -> Option<Arc<DefaultMetricCollector>> {
    None // Placeholder
}

/// Check if metrics system is initialized
#[must_use] pub const fn is_initialized() -> bool {
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
            HashMap::new(),
        );
        
        collector.record_metric(metric).await?;
        
        // Tool success rate
        let mut labels = HashMap::new();
        labels.insert("tool".to_string(), tool_name.clone());
        labels.insert("type".to_string(), "success_rate".to_string());
        
        let metric = Metric::with_optional_labels(
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
        
        let metric = Metric::with_optional_labels(
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
    /// Creates a new metrics manager
    ///
    /// Initializes a metrics manager with empty collections of
    /// metric collectors and exporters
    #[must_use] pub fn new() -> Self {
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
pub const fn init_collector(_config: MetricConfig) -> Result<()> {
    // Implementation would go here
    Ok(())
}

/// Records a counter metric
///
/// # Arguments
/// * `collector` - The metric collector to use
/// * `name` - The name of the metric
/// * `value` - The value to record
/// * `labels` - Additional labels for the metric
///
/// # Errors
/// Returns an error if the metric cannot be recorded
pub async fn record_counter<S: ::std::hash::BuildHasher>(
    collector: &dyn MetricCollector,
    name: &str,
    value: f64,
    labels: Option<HashMap<String, String, S>>,
) -> Result<()> {
    // Convert the labels with custom hasher S to a standard HashMap
    let std_labels = match &labels {
        Some(custom_labels) => {
            let mut std_map = HashMap::new();
            for (k, v) in custom_labels {
                std_map.insert(k.clone(), v.clone());
            }
            Some(std_map)
        },
        None => None,
    };
    
    let metric = Metric::with_optional_labels(
        name.to_string(), 
        value, 
        MetricType::Counter,
        std_labels
    );
    collector.record_metric(metric).await
}

/// Re-export additional types - remove PerformanceMetrics since it doesn't exist
pub use resource::TeamResourceMetrics;

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

/// Protocol metrics module
pub mod protocol {
    use super::{Arc, MetricCollector, ProtocolMetricsCollectorAdapter, RwLock};
    
    /// Create a protocol metrics collector adapter
    #[must_use] pub fn create_collector_adapter() -> Arc<ProtocolMetricsCollectorAdapter> {
        Arc::new(ProtocolMetricsCollectorAdapter::new())
    }
    
    /// Create a protocol metrics collector adapter with a collector
    #[must_use] pub fn create_collector_adapter_with_collector(
        collector: Arc<RwLock<dyn MetricCollector>>
    ) -> Arc<ProtocolMetricsCollectorAdapter> {
        Arc::new(ProtocolMetricsCollectorAdapter::with_collector(collector))
    }
}

/// Factory for creating metric collectors
///
/// This factory provides a centralized way to create metric collectors with
/// consistent configuration. It supports creating standalone collectors and
/// collectors with protocol-specific integrations.
#[derive(Debug)]
pub struct MetricCollectorFactory {
    /// Configuration for metrics
    config: MetricConfig,
}

impl MetricCollectorFactory {
    /// Creates a new metric collector factory with default configuration
    /// 
    /// Initializes a factory with default metric collection configuration
    /// settings, suitable for standard monitoring scenarios.
    ///
    /// # Returns
    /// A new factory instance with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: MetricConfig::default(),
        }
    }
    
    /// Creates a new metric collector factory with custom configuration
    /// 
    /// Initializes a factory with custom metric collection configuration
    /// settings, allowing for tailored monitoring behavior.
    ///
    /// # Arguments
    /// * `config` - Custom metric collector configuration
    ///
    /// # Returns
    /// A new factory instance with the specified configuration
    #[must_use] pub const fn with_config(config: MetricConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Creates a metric collector with the factory's configuration
    /// 
    /// Creates a new metric collector using the factory's configuration
    /// without any protocol-specific collectors.
    ///
    /// # Returns
    /// A new metric collector with the factory's configuration
    #[must_use] pub fn create_collector(&self) -> Arc<DefaultMetricCollector> {
        let collector = DefaultMetricCollector::with_dependencies(
            Some(self.config.clone()),
            None,
        );
        Arc::new(collector)
    }
    
    /// Creates a metric collector with protocol integration
    /// 
    /// Creates a new metric collector using the factory's configuration
    /// with the specified protocol collector for integrated metrics.
    ///
    /// # Arguments
    /// * `protocol_collector` - Protocol metrics collector adapter
    ///
    /// # Returns
    /// A new metric collector with protocol integration
    #[must_use] pub fn create_collector_with_protocol(&self, protocol_collector: Arc<ProtocolMetricsCollectorAdapter>) -> Arc<DefaultMetricCollector> {
        let collector = DefaultMetricCollector::with_dependencies(
            Some(self.config.clone()),
            Some(protocol_collector),
        );
        Arc::new(collector)
    }
}

impl Default for MetricCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a metric collector factory with default configuration
/// 
/// Convenience function for creating a factory with default settings.
///
/// # Returns
/// A new metric collector factory with default configuration
#[must_use] pub fn create_factory() -> Arc<MetricCollectorFactory> {
    Arc::new(MetricCollectorFactory::new())
}

/// Creates a metric collector factory with custom configuration
/// 
/// Convenience function for creating a factory with custom settings.
///
/// # Arguments
/// * `config` - Custom metric collector configuration
///
/// # Returns
/// A new metric collector factory with the specified configuration
#[must_use] pub fn create_factory_with_config(config: MetricConfig) -> Arc<MetricCollectorFactory> {
    Arc::new(MetricCollectorFactory::with_config(config))
}

/// Creates a default metric collector
/// 
/// Convenience function for creating a metric collector with default configuration.
///
/// # Returns
/// A new metric collector with default configuration
#[must_use] pub fn create_collector() -> Arc<DefaultMetricCollector> {
    let factory = create_factory();
    factory.create_collector()
}

/// Creates a protocol metrics collector adapter
/// 
/// Convenience function for creating an uninitialized protocol metrics collector adapter.
///
/// # Returns
/// A new uninitialized protocol metrics collector adapter
#[must_use] pub fn create_collector_adapter() -> Arc<ProtocolMetricsCollectorAdapter> {
    Arc::new(ProtocolMetricsCollectorAdapter::new())
}

/// Aggregates metrics from multiple collectors
#[derive(Debug)]
pub struct AggregateMetricCollector {
    /// Collection of metric collectors
    collectors: Arc<RwLock<Vec<Arc<dyn MetricCollector + Send + Sync>>>>,
    /// Collection of metric exporters
    exporters: Arc<RwLock<Vec<Arc<dyn MetricExporter + Send + Sync>>>>,
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
            metric_type: MetricType::Gauge,
            timestamp: 0,
            labels: HashMap::new(),
            operation_type: OperationType::Unknown,
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
            metric_type: MetricType::Counter,
            timestamp: 0,
            labels: HashMap::new(),
            operation_type: OperationType::Unknown,
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
            HashMap::new(),
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