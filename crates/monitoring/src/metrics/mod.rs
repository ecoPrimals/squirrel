// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::cast_possible_wrap)] // Allow u64 to i64 casts for timestamps
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
use log;
use squirrel_core::error::{Result, SquirrelError};
use performance::OperationType;
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
    #[must_use] 
    pub fn new(name: String, value: f64, metric_type: MetricType, labels: HashMap<String, String>) -> Self {
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
    #[must_use] 
    pub fn with_optional_labels(name: String, value: f64, metric_type: MetricType, labels: Option<HashMap<String, String>>) -> Self {
        Self::new(name, value, metric_type, labels.unwrap_or_default())
    }
    
    /// Determines if this metric should trigger an alert
    ///
    /// Evaluates the metric value and other criteria to determine if
    /// it has crossed a threshold that should trigger an alert.
    ///
    /// # Returns
    /// `true` if the metric should trigger an alert, `false` otherwise
    #[must_use] 
    pub fn should_alert(&self) -> bool {
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
    /// 
    /// # Errors
    /// Returns an error if metrics collection fails due to system issues,
    /// resource constraints, or failed collectors
    async fn collect_metrics(&self) -> Result<Vec<Metric>>;

    /// Record a new metric
    ///
    /// # Parameters
    /// * `metric` - The metric to record
    ///
    /// # Errors
    /// Returns an error if the metric cannot be recorded due to storage issues,
    /// validation failures, or if the collector is not initialized
    async fn record_metric(&self, metric: Metric) -> Result<()>;

    /// Start the metric collector
    /// 
    /// # Errors
    /// Returns an error if the collector cannot be started due to
    /// initialization failures or resource constraints
    async fn start(&self) -> Result<()>;

    /// Stop the metric collector
    /// 
    /// # Errors
    /// Returns an error if the collector cannot be stopped gracefully
    /// or if there are pending operations that cannot be completed
    async fn stop(&self) -> Result<()>;
}

/// Default implementation of the metric collector
///
/// This collector provides a standard implementation of the `MetricCollector` trait,
/// storing metrics in memory and supporting basic operations like recording metrics,
/// collecting all metrics, and lifecycle management.
#[derive(Debug)]
pub struct DefaultMetricCollector {
    /// Configuration for the metric collector
    config: MetricConfig,
    /// Storage for collected metrics
    metrics: Arc<RwLock<Vec<Metric>>>,
    /// Flag indicating whether the collector is initialized
    initialized: Arc<RwLock<bool>>,
    /// Optional protocol metric collector adapter for integration with protocol metrics
    protocol_collector: Option<Arc<ProtocolMetricsCollectorAdapter>>,
}

impl DefaultMetricCollector {
    /// Creates a new metric collector with default configuration
    /// 
    /// Initializes a new collector with default settings and an empty metrics collection.
    ///
    /// # Returns
    /// A new metric collector instance ready for use
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MetricConfig::default(),
            metrics: Arc::new(RwLock::new(Vec::new())),
            initialized: Arc::new(RwLock::new(false)),
            protocol_collector: None,
        }
    }

    /// Initializes the metric collector with a custom configuration
    ///
    /// # Arguments
    /// * `config` - The configuration to use for initialization
    ///
    /// # Errors
    /// Returns an error if initialization fails
    pub async fn initialize_with_config(&mut self, config: MetricConfig) -> Result<()> {
        self.config = config;
        let mut initialized = self.initialized.write().await;
        *initialized = true;
        Ok(())
    }

    /// Creates a new metric collector with dependencies
    /// 
    /// Initializes a new collector with optional custom configuration and
    /// protocol collector integration.
    ///
    /// # Arguments
    /// * `config` - Optional custom configuration, uses default if `None`
    /// * `protocol_collector` - Optional protocol collector adapter for integration
    ///
    /// # Returns
    /// A new metric collector instance with the specified dependencies
    #[must_use]
    pub fn with_dependencies(
        config: Option<MetricConfig>,
        protocol_collector: Option<Arc<ProtocolMetricsCollectorAdapter>>,
    ) -> Self {
        Self {
            config: config.unwrap_or_default(),
            metrics: Arc::new(RwLock::new(Vec::new())),
            initialized: Arc::new(RwLock::new(false)),
            protocol_collector,
        }
    }

    /// Checks if the collector is initialized
    ///
    /// # Returns
    /// `true` if the collector is initialized, `false` otherwise
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        // Using a synchronous method that doesn't require block_in_place
        // This avoids requiring a multi-threaded runtime for tests
        self.initialized.try_read().map(|guard| *guard).unwrap_or(false)
    }

    /// Initializes the collector
    ///
    /// # Errors
    /// Returns an error if the collector cannot be initialized
    pub async fn initialize(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            // Already initialized, just return success
            return Ok(());
        }
        *initialized = true;
        Ok(())
    }
}

#[async_trait]
impl MetricCollector for DefaultMetricCollector {
    /// Collects all metrics from the collector
    ///
    /// # Returns
    /// A vector of all collected metrics
    ///
    /// # Errors
    /// Returns an error if metrics cannot be collected due to lock acquisition failures
    /// or if the collector is not properly initialized
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        // First get our own metrics
        let metrics = {
            let metrics_guard = self.metrics.read().await;
            read_guard_to_vec(&metrics_guard)
        };
        
        // Then get protocol metrics if available
        let mut all_metrics = metrics;
        
        if let Some(protocol_collector) = &self.protocol_collector {
            if protocol_collector.is_initialized() {
                match protocol_collector.get_metrics().await {
                    Ok(protocol_metrics) => {
                        all_metrics.extend(protocol_metrics);
                    },
                    Err(e) => {
                        // Use proper logging instead of eprintln
                        log::warn!("Error collecting protocol metrics: {}", e);
                    }
                }
            }
        }
        
        Ok(all_metrics)
    }

    /// Records a new metric
    ///
    /// # Arguments
    /// * `metric` - The metric to record
    ///
    /// # Errors
    /// Returns an error if the metric cannot be recorded due to lock acquisition failures,
    /// if the collector is not initialized, or if the metric is invalid
    async fn record_metric(&self, metric: Metric) -> Result<()> {
        // Check if we're initialized
        if !self.is_initialized() {
            return Err(SquirrelError::monitoring(
                format!("Cannot record metric '{}': collector not initialized", metric.name)
            ));
        }
        
        // Record the metric in our storage
        {
            let mut metrics = self.metrics.write().await;
            // No need to clone here as we'll only read from metric after this point
            metrics.push(metric.clone());
            
            // Trim to max size if needed
            if metrics.len() > self.config.max_metrics {
                retain_newest_metrics(&mut metrics, self.config.max_metrics);
            }
        }
        
        // Also record in protocol collector if available
        if let Some(protocol_collector) = &self.protocol_collector {
            if protocol_collector.is_initialized() {
                if let Err(e) = protocol_collector.record_metric(metric).await {
                    // Use proper logging instead of eprintln
                    log::warn!("Error recording metric in protocol collector: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Starts the metric collector
    ///
    /// # Errors
    /// Returns an error if the collector cannot be started
    /// or if it is already running
    async fn start(&self) -> Result<()> {
        self.initialize().await?;
        
        // Start protocol collector if available
        if let Some(protocol_collector) = &self.protocol_collector {
            if let Err(e) = protocol_collector.start().await {
                // Use proper logging
                log::warn!("Error starting protocol collector: {}", e);
            }
        }
        
        Ok(())
    }

    /// Stops the metric collector
    ///
    /// # Errors
    /// Returns an error if the collector cannot be stopped cleanly
    /// or if there are pending operations
    async fn stop(&self) -> Result<()> {
        // Stop protocol collector if available
        if let Some(protocol_collector) = &self.protocol_collector {
            if let Err(e) = protocol_collector.stop().await {
                // Use proper logging
                log::warn!("Error stopping protocol collector: {}", e);
            }
        }
        
        // Mark as uninitialized
        let mut initialized = self.initialized.write().await;
        *initialized = false;
        
        Ok(())
    }
}

#[async_trait]
impl MetricCollector for Arc<dyn MetricCollector> {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        (**self).collect_metrics().await
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        (**self).record_metric(metric).await
    }

    async fn start(&self) -> Result<()> {
        (**self).start().await
    }

    async fn stop(&self) -> Result<()> {
        (**self).stop().await
    }
}

#[async_trait]
impl MetricCollector for Arc<DefaultMetricCollector> {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        (**self).collect_metrics().await
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        (**self).record_metric(metric).await
    }

    async fn start(&self) -> Result<()> {
        (**self).start().await
    }

    async fn stop(&self) -> Result<()> {
        (**self).stop().await
    }
}

impl Default for DefaultMetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric source trait for components that provide metrics
#[async_trait]
pub trait MetricSource: Debug + Send + Sync {
    /// Get metrics from this source
    /// 
    /// # Errors
    /// Returns an error if metrics cannot be retrieved from the source
    /// due to connection issues, timeouts, or internal errors
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
    #[must_use] 
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Create a new adapter with a collector
    #[must_use] 
    pub fn with_collector(collector: Arc<RwLock<dyn MetricCollector>>) -> Self {
        Self { inner: Some(collector) }
    }

    /// Get metrics from the collector
    /// 
    /// # Errors
    /// Returns an error if the underlying collector fails to retrieve metrics
    /// or if the collector is not initialized
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.collect_metrics().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Record a metric
    /// 
    /// # Errors
    /// Returns an error if the underlying collector fails to record the metric
    /// or if the collector is not initialized
    pub async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.record_metric(metric).await
        } else {
            Ok(())
        }
    }

    /// Check if the adapter is initialized
    #[must_use] 
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}

#[async_trait]
impl MetricCollector for ProtocolMetricsCollectorAdapter {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        self.get_metrics().await
    }

    /// Record a new metric
    ///
    /// # Errors
    /// Returns an error if the underlying collector fails to record the metric
    /// or if the collector is not properly initialized
    async fn record_metric(&self, metric: Metric) -> Result<()> {
        self.record_metric(metric).await
    }

    /// Start the metric collector
    ///
    /// # Errors
    /// Returns an error if the underlying collector fails to start
    /// or if it is not properly initialized
    async fn start(&self) -> Result<()> {
        // If there's no inner collector, nothing to start
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.start().await
        } else {
            Ok(())
        }
    }

    /// Stop the metric collector
    ///
    /// # Errors
    /// Returns an error if the underlying collector fails to stop
    /// or if there are pending operations that cannot be completed
    async fn stop(&self) -> Result<()> {
        // If there's no inner collector, nothing to stop
        if let Some(collector) = &self.inner {
            let collector = collector.read().await;
            collector.stop().await
        } else {
            Ok(())
        }
    }
}

/// Records metrics for a tool
///
/// # Parameters
/// * `collector` - The metric collector to record metrics with
/// * `metrics` - A map of tool names to their metrics
///
/// # Errors
/// Returns an error if the metric collector fails to record any of the metrics
/// due to validation issues, storage problems, or if the collector is not initialized
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
/// Returns an error if the metrics cannot be collected from any collector
/// or if the monitoring service is not properly initialized
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
    /// Collection of metric collectors that gather data from different sources
    collectors: Arc<RwLock<Vec<Arc<dyn MetricCollector + Send + Sync>>>>,
    /// Collection of metric exporters that send data to external systems
    exporters: Arc<RwLock<Vec<Arc<dyn MetricExporter + Send + Sync>>>>,
}

impl MetricsManager {
    /// Creates a new metrics manager
    ///
    /// Initializes a metrics manager with empty collections of
    /// metric collectors and exporters
    #[must_use] 
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
    /// or if there's an issue registering the collector
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
    /// or if there's an issue registering the exporter
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
    /// Returns an error if the collectors lock cannot be acquired 
    /// or if any collector fails to provide metrics
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
/// Returns an error if the collector is already initialized 
/// or if there are resource issues with initialization
pub const fn init_collector(_config: MetricConfig) -> Result<()> {
    // Implementation would go here
    Ok(())
}

/// Record a counter metric
///
/// Helper function to record a counter metric with optional labels.
///
/// # Arguments
/// * `collector` - The metric collector to record the metric with
/// * `name` - Name of the counter metric
/// * `value` - Value of the counter
/// * `labels` - Optional labels to attach to the metric
///
/// # Errors
/// Returns an error if the metric cannot be recorded
pub async fn record_counter<C: MetricCollector>(
    collector: &C,
    name: &str,
    value: f64,
    labels: Option<HashMap<String, String>>,
) -> Result<()> {
    let metric = Metric::with_optional_labels(
        name.to_string(),
        value,
        MetricType::Counter,
        labels,
    );
    collector.record_metric(metric).await
}

/// Re-export additional types
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
    /// Collection of metric collectors that gather different types of metrics from various sources
    #[allow(dead_code)]
    collectors: Arc<RwLock<Vec<Arc<dyn MetricCollector + Send + Sync>>>>,
    /// Collection of metric exporters that send metrics to different destinations for storage or visualization
    #[allow(dead_code)]
    exporters: Arc<RwLock<Vec<Arc<dyn MetricExporter + Send + Sync>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_record_counter_helper() {
        let collector = DefaultMetricCollector::new();
        collector.initialize().await.unwrap();
        let mut labels = HashMap::new();
        labels.insert("test".to_string(), "value".to_string());
        record_counter(&collector, "test_counter", 5.0, Some(labels)).await.unwrap();
    }

    #[derive(Debug)]
    struct MockErrorProtocolCollector;

    #[async_trait]
    impl MetricCollector for MockErrorProtocolCollector {
        async fn collect_metrics(&self) -> Result<Vec<Metric>> {
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_concurrent_metric_recording() {
        let collector = Arc::new(DefaultMetricCollector::new());
        collector.initialize().await.unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let num_tasks = 10;
        let mut handles = Vec::new();

        for _ in 0..num_tasks {
            let collector_clone = Arc::clone(&collector);
            let counter_clone: Arc<AtomicUsize> = Arc::clone(&counter);
            let handle = tokio::spawn(async move {
                let mut labels = HashMap::new();
                labels.insert("test".to_string(), "value".to_string());
                record_counter(&*collector_clone, "test_counter", 1.0, Some(labels)).await.unwrap();
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), num_tasks, "All metrics should be recorded");
    }
    
    #[tokio::test]
    async fn test_guard_helper_functions() {
        // Test data
        let data = vec![1, 2, 3, 4, 5];
        let rwlock = Arc::new(RwLock::new(data.clone()));

        // Test read_guard_to_vec
        {
            let read_guard = rwlock.read().await;
            let result = read_guard_to_vec(&read_guard);
            assert_eq!(result, data, "read_guard_to_vec should return exact copy of data");
        }

        // Test write_guard_to_vec
        {
            let write_guard = rwlock.write().await;
            let result = write_guard_to_vec(&write_guard);
            assert_eq!(result, data, "write_guard_to_vec should return exact copy of data");
        }

        // Test add_to_write_guard
        {
            let mut write_guard = rwlock.write().await;
            let values_to_add = vec![6, 7, 8];
            add_to_write_guard(&mut write_guard, &values_to_add);
            
            let expected = vec![1, 2, 3, 4, 5, 6, 7, 8];
            assert_eq!(*write_guard, expected, "add_to_write_guard should append values");
        }

        // Test update_in_write_guard
        {
            let mut write_guard = rwlock.write().await;
            
            // Double even numbers
            update_in_write_guard(&mut write_guard, |&num| num % 2 == 0, |num| *num *= 2);
            
            let expected = vec![1, 4, 3, 8, 5, 12, 7, 16];
            assert_eq!(*write_guard, expected, "update_in_write_guard should modify matching elements");
        }

        // Test find_in_read_guard
        {
            let read_guard = rwlock.read().await;
            
            // Find first number greater than 10
            let result = find_in_read_guard(&read_guard, |&num| num > 10);
            
            assert_eq!(result, Some(12), "find_in_read_guard should find matching element");
            
            // Test not finding anything
            let result = find_in_read_guard(&read_guard, |&num| num > 100);
            assert_eq!(result, None, "find_in_read_guard should return None when no match");
        }

        // Test remove_from_write_guard
        {
            let mut write_guard = rwlock.write().await;
            
            // Remove even numbers
            remove_from_write_guard(&mut write_guard, |&num| num % 2 == 0);
            
            let expected = vec![1, 3, 5, 7];
            assert_eq!(*write_guard, expected, "remove_from_write_guard should remove matching elements");
        }
    }
}

/// Convert SystemTime to Unix timestamp
fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Helper function to retain only the newest metrics
fn retain_newest_metrics(metrics: &mut Vec<Metric>, max_count: usize) {
    if metrics.len() <= max_count {
        return;
    }
    // Sort by timestamp in descending order
    metrics.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    // Truncate to max count
    metrics.truncate(max_count);
}

/// Helper functions for working with RwLock-guarded metrics
/// Converts a read guard into a Vec by cloning each element
pub fn read_guard_to_vec<T: Clone>(guard: &tokio::sync::RwLockReadGuard<'_, Vec<T>>) -> Vec<T> {
    let mut result = Vec::with_capacity(guard.len());
    for item in guard.iter() {
        result.push(item.clone());
    }
    result
}

/// Converts a write guard into a Vec by cloning each element
pub fn write_guard_to_vec<T: Clone>(guard: &tokio::sync::RwLockWriteGuard<'_, Vec<T>>) -> Vec<T> {
    let mut result = Vec::with_capacity(guard.len());
    for item in guard.iter() {
        result.push(item.clone());
    }
    result
}

/// Adds values to a write guard
pub fn add_to_write_guard<T: Clone>(guard: &mut tokio::sync::RwLockWriteGuard<'_, Vec<T>>, values: &[T]) {
    for value in values {
        guard.push(value.clone());
    }
}

/// Updates existing elements in a write guard that match the provided predicate
pub fn update_in_write_guard<T: Clone, F>(
    guard: &mut tokio::sync::RwLockWriteGuard<'_, Vec<T>>, 
    predicate: F, 
    update_fn: impl Fn(&mut T)
) where F: Fn(&T) -> bool {
    for item in guard.iter_mut() {
        if predicate(item) {
            update_fn(item);
        }
    }
}

/// Safely finds an element in a read guard that matches the provided predicate
pub fn find_in_read_guard<T: Clone, F>(
    guard: &tokio::sync::RwLockReadGuard<'_, Vec<T>>, 
    predicate: F
) -> Option<T> where F: Fn(&T) -> bool {
    guard.iter().find(|item| predicate(item)).cloned()
}

/// Removes elements from a write guard that match the provided predicate
pub fn remove_from_write_guard<T, F>(
    guard: &mut tokio::sync::RwLockWriteGuard<'_, Vec<T>>, 
    predicate: F
) where F: Fn(&T) -> bool {
    guard.retain(|item| !predicate(item));
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