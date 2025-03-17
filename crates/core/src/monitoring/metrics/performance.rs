//! Performance metrics collection for operation timing
//! 
//! Tracks operation latencies for:
//! - Protocol operations
//! - Context operations
//! - Command execution
//! - Monitoring operations

use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{Result, SquirrelError};
use crate::monitoring::metrics::{Metric, MetricCollector};
use std::fmt;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use prometheus::{Histogram, HistogramOpts};
use std::hash::Hash;
use std::cmp::Eq;
use crate::monitoring::metrics::MetricType;
use async_trait::async_trait;

/// Types of operations that can be monitored for performance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Database read operations
    DatabaseRead,
    /// Database write operations
    DatabaseWrite,
    /// Network requests
    NetworkRequest,
    /// File system operations
    FileSystem,
    /// Cache operations
    Cache,
    /// Custom operation type
    Custom(String),
    /// Unknown operation type
    Unknown,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseRead => write!(f, "Database Read"),
            Self::DatabaseWrite => write!(f, "Database Write"),
            Self::NetworkRequest => write!(f, "Network Request"),
            Self::FileSystem => write!(f, "File System"),
            Self::Cache => write!(f, "Cache"),
            Self::Custom(name) => write!(f, "Custom: {name}"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Performance metrics for an operation type.
/// 
/// This struct tracks various performance metrics for a specific type of operation,
/// including counts, durations, and a histogram of operation timings.
#[derive(Clone)]
pub struct OperationMetrics {
    /// The type of operation being measured.
    pub operation_type: OperationType,
    /// Number of times this operation has been executed.
    pub count: u64,
    /// Total time spent executing this operation.
    pub total_duration: f64,
    /// Shortest duration observed for this operation.
    pub min_duration: f64,
    /// Longest duration observed for this operation.
    pub max_duration: f64,
    /// Histogram of operation durations for statistical analysis.
    pub histogram: Histogram,
}

impl fmt::Debug for OperationMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OperationMetrics")
            .field("operation_type", &self.operation_type)
            .field("count", &self.count)
            .field("total_duration", &self.total_duration)
            .field("min_duration", &self.min_duration)
            .field("max_duration", &self.max_duration)
            .field("histogram", &"<histogram>")
            .finish()
    }
}

/// Performance metrics collector
pub struct PerformanceCollector {
    /// Operation histograms
    histograms: Arc<RwLock<HashMap<OperationType, Histogram>>>,
}

impl PerformanceCollector {
    /// Create a new performance collector
    ///
    /// # Panics
    ///
    /// This function panics if the Histogram cannot be created with the given options
    #[must_use] pub fn new() -> Self {
        Self {
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new performance collector with the given configuration
    ///
    /// # Parameters
    /// * `config` - The performance collector configuration
    ///
    /// # Panics
    ///
    /// This function panics if the Histogram cannot be created with the given options
    #[must_use] pub fn with_config(_config: PerformanceConfig) -> Self {
        // Currently we don't use the config, but we'll keep the method for future extensibility
        Self::new()
    }

    /// Record an operation duration
    /// 
    /// # Parameters
    /// * `op_type` - The type of operation
    /// * `duration` - The duration of the operation
    ///
    /// # Returns
    /// * `Result<()>` - Success or failure
    ///
    /// # Panics
    ///
    /// This function panics if the Histogram cannot be created with the given options
    pub async fn record_operation(
        &self,
        op_type: &OperationType,
        duration: Duration,
    ) -> Result<()> {
        // Convert duration to seconds
        let duration_secs = duration.as_secs_f64();
        
        let mut histograms = self.histograms.write().await;
        let histogram = histograms.entry(op_type.clone()).or_insert_with(|| {
            let name = format!("operation_duration_{op_type:?}");
            let help = format!("Duration of {op_type:?} operations");
            let opts = HistogramOpts::new(name, help);
            Histogram::with_opts(opts).unwrap()
        });
        
        histogram.observe(duration_secs);
        Ok(())
    }

    /// Get metrics for an operation type
    pub async fn get_metrics(&self, op_type: &OperationType) -> Result<OperationMetrics> {
        let histograms = self.histograms.read().await;
        histograms.get(op_type).map(|histogram| {
            let count = histogram.get_sample_count();
            let sum = histogram.get_sample_sum();
            
            // Calculate metrics from histogram data
            let _avg = if count > 0 { sum / count as f64 } else { 0.0 };
            
            OperationMetrics {
                operation_type: op_type.clone(),
                count,
                total_duration: sum,
                min_duration: 0.0, // Prometheus doesn't provide min
                max_duration: sum, // Use sum as a conservative estimate
                histogram: histogram.clone(),
            }
        }).ok_or_else(|| "Operation type not found in histograms".into())
    }

    /// Start timing an operation
    #[must_use] pub fn start_operation() -> Instant {
        Instant::now()
    }

    /// Record operation timing with a guard
    pub async fn time_operation<F, T>(
        &self,
        op_type: OperationType,
        f: F,
    ) -> T 
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        let _ = self.record_operation(&op_type, duration).await;
        
        result
    }

    /// Get all metrics
    #[must_use] pub fn get_all_metrics(&self) -> HashMap<OperationType, OperationMetrics> {
        let histograms = self.histograms.blocking_read();
        let mut metrics = HashMap::new();
        for (op_type, histogram) in histograms.iter() {
            let count = histogram.get_sample_count();
            let sum = histogram.get_sample_sum();
            let _avg = if count > 0 { sum / count as f64 } else { 0.0 };
            
            metrics.insert(op_type.clone(), OperationMetrics {
                operation_type: op_type.clone(),
                count,
                total_duration: sum,
                min_duration: 0.0,
                max_duration: sum,
                histogram: histogram.clone(),
            });
        }
        metrics
    }
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating and managing performance collector instances.
/// 
/// This factory provides a more maintainable and testable approach to
/// managing PerformanceCollector instances compared to static variables.
#[derive(Debug, Clone)]
pub struct PerformanceCollectorFactory {
    /// Default configuration for creating collectors
    config: Option<PerformanceConfig>,
}

/// Configuration for performance collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to enable performance tracking
    pub enabled: bool,
    /// The number of histogram buckets to use
    pub histogram_buckets: usize,
    /// The maximum histogram value to track
    pub histogram_max_value: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            histogram_buckets: 20,
            histogram_max_value: 60.0, // 60 seconds
        }
    }
}

impl PerformanceCollectorFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub const fn new() -> Self {
        Self { config: None }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: PerformanceConfig) -> Self {
        Self { config: Some(config) }
    }

    /// Creates a performance collector
    #[must_use]
    pub fn create_collector(&self) -> Arc<PerformanceCollector> {
        let collector = match &self.config {
            Some(config) => PerformanceCollector::with_config(config.clone()),
            None => PerformanceCollector::new(),
        };
        
        Arc::new(collector)
    }

    /// Initialize a global collector instance
    ///
    /// # Errors
    /// Returns an error if the collector is already initialized
    pub async fn initialize_global_collector(&self) -> Result<Arc<PerformanceCollector>> {
        static GLOBAL_COLLECTOR: OnceLock<Arc<PerformanceCollector>> = OnceLock::new();
        
        let collector = self.create_collector();
        GLOBAL_COLLECTOR
            .set(collector.clone())
            .map_err(|_| SquirrelError::metric("Performance collector already initialized"))?;
            
        Ok(collector)
    }
    
    /// Gets the global performance collector if initialized
    ///
    /// # Errors
    /// Returns an error if the collector is not initialized
    pub async fn get_global_collector(&self) -> Result<Arc<PerformanceCollector>> {
        static GLOBAL_COLLECTOR: OnceLock<Arc<PerformanceCollector>> = OnceLock::new();
        
        if let Some(collector) = GLOBAL_COLLECTOR.get() {
            Ok(collector.clone())
        } else {
            Err(SquirrelError::metric("Performance collector not initialized"))
        }
    }
}

impl Default for PerformanceCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory for creating performance collectors
static FACTORY: OnceLock<PerformanceCollectorFactory> = OnceLock::new();

/// Initialize the performance collector factory
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory(config: Option<PerformanceConfig>) -> Result<()> {
    let factory = match config {
        Some(cfg) => PerformanceCollectorFactory::with_config(cfg),
        None => PerformanceCollectorFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::metric("Performance collector factory already initialized"))?;
    Ok(())
}

/// Get the performance collector factory
#[must_use]
pub fn get_factory() -> Option<PerformanceCollectorFactory> {
    FACTORY.get().cloned()
}

/// Get or create the performance collector factory
#[must_use]
pub fn ensure_factory() -> PerformanceCollectorFactory {
    FACTORY.get_or_init(PerformanceCollectorFactory::new).clone()
}

/// Initialize the performance collector
///
/// # Errors
/// Returns an error if the collector cannot be initialized
pub async fn initialize() -> Result<Arc<PerformanceCollector>> {
    let factory = ensure_factory();
    let collector = factory.initialize_global_collector().await?;
    
    // Also set the collector in the global variable for direct access in performance measurements
    let _ = PERFORMANCE_COLLECTOR.set(collector.clone());
    
    Ok(collector)
}

/// Record an operation's duration
///
/// # Errors
/// Returns an error if:
/// - The performance collector is not initialized
/// - The operation cannot be recorded
pub async fn record_operation(op_type: OperationType, duration: Duration) -> Result<()> {
    let factory = ensure_factory();
    let collector = factory.get_global_collector().await?;
    collector.record_operation(&op_type, duration).await
}

/// Time an operation and record its duration
///
/// # Errors
/// Returns an error if the performance collector is not initialized
pub async fn time_operation<F, T>(op_type: OperationType, f: F) -> Result<T>
where
    F: FnOnce() -> T,
{
    let factory = ensure_factory();
    let collector = factory.get_global_collector().await?;
    Ok(collector.time_operation(op_type, f).await)
}

/// Starts timing an operation.
/// 
/// This function returns an `Instant` that can be used to measure
/// the duration of an operation.
/// 
/// # Returns
/// Returns a `Result` containing the start time `Instant`.
pub fn start_operation() -> Result<Instant> {
    Ok(PerformanceCollector::start_operation())
}

/// Collects and tracks performance metrics for operations.
/// 
/// This struct maintains a histogram of operation durations and provides
/// methods to record new observations and retrieve statistics.
pub struct PerformanceMetrics {
    /// Histogram for tracking operation durations.
    histogram: Histogram,
}

impl PerformanceMetrics {
    /// Creates a new performance metrics instance.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the metrics
    /// * `help` - A description of the metrics
    /// 
    /// # Returns
    /// 
    /// A new `PerformanceMetrics` instance.
    /// 
    /// # Panics
    /// 
    /// This function panics if the Histogram cannot be created with the given options.
    #[must_use] pub fn new(name: &str, help: &str) -> Self {
        let opts = HistogramOpts::new(name, help);
        Self {
            histogram: Histogram::with_opts(opts).unwrap(),
        }
    }

    /// Records a new duration observation in the metrics.
    /// 
    /// # Arguments
    /// 
    /// * `duration` - The duration to record
    pub fn observe(&mut self, duration: Duration) {
        self.histogram.observe(duration.as_secs_f64());
    }

    /// Retrieves current performance statistics.
    /// 
    /// # Returns
    /// 
    /// Returns a `PerformanceStats` instance containing various statistics
    /// calculated from the recorded observations.
    #[must_use] pub fn get_metrics(&self) -> PerformanceStats {
        let count = self.histogram.get_sample_count();
        let sum = self.histogram.get_sample_sum();
        
        PerformanceStats {
            operation_count: count,
            total_duration: Duration::from_secs_f64(sum),
            min_duration: Duration::from_secs_f64(self.histogram.get_sample_sum() / count as f64),
            max_duration: Duration::from_secs_f64(self.histogram.get_sample_sum() / count as f64),
            avg_duration: Duration::from_secs_f64(sum / count as f64),
            p95_duration: Duration::from_secs_f64(sum / count as f64 * 0.95),
            p99_duration: Duration::from_secs_f64(sum / count as f64 * 0.99),
        }
    }
}

/// Statistics calculated from performance metrics.
/// 
/// This struct contains various statistical measures calculated from
/// recorded operation durations.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Total number of operations recorded.
    pub operation_count: u64,
    /// Total duration of all recorded operations.
    pub total_duration: Duration,
    /// Shortest recorded operation duration.
    pub min_duration: Duration,
    /// Longest recorded operation duration.
    pub max_duration: Duration,
    /// Average duration of recorded operations.
    pub avg_duration: Duration,
    /// 95th percentile duration.
    pub p95_duration: Duration,
    /// 99th percentile duration.
    pub p99_duration: Duration,
}

/// Statistics for a specific operation type.
/// 
/// This struct tracks basic statistics about operation latencies,
/// including count and various latency measurements.
#[derive(Debug, Clone, Default)]
pub struct OperationStats {
    /// Number of times this operation has been executed.
    pub count: u64,
    /// Total latency across all executions.
    pub total_latency: f64,
    /// Minimum observed latency.
    pub min_latency: f64,
    /// Maximum observed latency.
    pub max_latency: f64,
}

impl OperationStats {
    /// Creates a new OperationStats instance with default values.
    /// 
    /// Initializes all counters to zero and sets min_latency to the maximum
    /// possible value to ensure proper minimum tracking.
    #[must_use] pub const fn new() -> Self {
        Self {
            count: 0,
            total_latency: 0.0,
            min_latency: f64::MAX,
            max_latency: 0.0,
        }
    }

    /// Updates the statistics with a new latency observation.
    /// 
    /// # Arguments
    /// 
    /// * `latency` - The latency value to record
    pub fn update(&mut self, latency: f64) {
        self.count += 1;
        self.total_latency += latency;
        self.min_latency = self.min_latency.min(latency);
        self.max_latency = self.max_latency.max(latency);
    }

    /// Calculates the average latency of all recorded operations.
    /// 
    /// # Returns
    /// 
    /// Returns the average latency, or 0.0 if no operations have been recorded.
    #[must_use] pub fn average_latency(&self) -> f64 {
        if self.count > 0 {
            self.total_latency / self.count as f64
        } else {
            0.0
        }
    }
}

/// Collector for performance metrics across multiple operations.
/// 
/// This struct maintains a collection of operation statistics and provides
/// methods to record and retrieve metrics for different operations.
#[derive(Debug)]
pub struct PerformanceMetricsCollector {
    /// Map of operation names to their statistics.
    metrics: Arc<RwLock<HashMap<String, OperationStats>>>,
}

impl PerformanceMetricsCollector {
    /// Creates a new PerformanceMetricsCollector instance.
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records a latency observation for a specific operation.
    /// 
    /// # Arguments
    /// 
    /// * `operation` - The name of the operation
    /// * `latency` - The latency to record
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` indicating whether the operation was recorded successfully.
    pub async fn record_operation(&self, operation: &str, latency: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let stats = metrics.entry(operation.to_string()).or_default();
        stats.update(latency);
        Ok(())
    }

    /// Retrieves statistics for a specific operation.
    /// 
    /// # Arguments
    /// 
    /// * `operation` - The name of the operation
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing an `Option` with the operation's statistics
    /// if they exist.
    pub async fn get_operation_stats(&self, operation: &str) -> Result<Option<OperationStats>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(operation).cloned())
    }

    /// Retrieves all operation statistics.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a `HashMap` with all operation statistics.
    pub async fn get_all_stats(&self) -> Result<HashMap<String, OperationStats>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
}

impl Default for PerformanceMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricCollector for PerformanceMetricsCollector {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let mut metrics = Vec::new();
        let perf_metrics = self.metrics.read().await;

        for (operation, stats) in perf_metrics.iter() {
            let mut labels = HashMap::new();
            labels.insert("operation".to_string(), operation.clone());
            
            // Add count metric
            metrics.push(Metric::new(
                "performance.operation.count".to_string(),
                stats.count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Add total latency metric
            metrics.push(Metric::new(
                "performance.operation.total_latency".to_string(),
                stats.total_latency,
                MetricType::Gauge,
                Some(labels.clone()),
            ));
            
            // Add min latency metric
            metrics.push(Metric::new(
                "performance.operation.min_latency".to_string(),
                stats.min_latency,
                MetricType::Gauge,
                Some(labels.clone()),
            ));
            
            // Add max latency metric
            metrics.push(Metric::new(
                "performance.operation.max_latency".to_string(),
                stats.max_latency,
                MetricType::Gauge,
                Some(labels.clone()),
            ));
            
            // Add average latency metric
            if stats.count > 0 {
                metrics.push(Metric::new(
                    "performance.operation.avg_latency".to_string(),
                    stats.total_latency / stats.count as f64,
                    MetricType::Gauge,
                    Some(labels.clone()),
                ));
            }
        }
        
        Ok(metrics)
    }

    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        // Not implemented for performance collector
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_collector() {
        let collector = PerformanceCollector::new();
        let op_type = OperationType::NetworkRequest;
        let duration = Duration::from_millis(100);

        collector.record_operation(&op_type, duration).await.unwrap();

        let metrics = collector.get_metrics(&op_type).await.unwrap();
        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.total_duration, duration.as_secs_f64());
        assert!(metrics.min_duration <= duration.as_secs_f64());
    }

    #[tokio::test]
    async fn test_performance_metrics_collector() {
        let collector = PerformanceMetricsCollector::new();
        
        // Record some test operations
        collector.record_operation("test_op", 0.5).await.unwrap();
        collector.record_operation("test_op", 1.0).await.unwrap();
        collector.record_operation("test_op", 0.75).await.unwrap();

        // Get stats for specific operation
        let stats = collector.get_operation_stats("test_op").await.unwrap().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min_latency, 0.5);
        assert_eq!(stats.max_latency, 1.0);
        assert!((stats.average_latency() - 0.75).abs() < f64::EPSILON);

        // Collect all metrics
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }
}

/// Measures the duration of an operation and records it.
/// 
/// # Type Parameters
/// 
/// * `F` - A function type that returns `T`
/// * `T` - The return type of the operation
/// 
/// # Arguments
/// 
/// * `operation` - The name of the operation to measure
/// * `f` - The operation to measure
/// 
/// # Returns
/// 
/// Returns a `Result` containing the operation's return value.
pub async fn measure_operation<F, T>(operation: &str, f: F) -> Result<T>
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    
    // Try to get a global collector using the non-async static variable
    if let Some(collector) = PERFORMANCE_COLLECTOR.get() {
        let _ = collector.record_operation(&OperationType::Custom(operation.to_string()), duration).await;
    } else {
        // Only initialize collector if needed and missing
        let factory = ensure_factory();
        let collector = factory.create_collector();
        let _ = collector.record_operation(&OperationType::Custom(operation.to_string()), duration).await;
    }
    
    Ok(result)
}

/// Measures an operation and records it only if it exceeds a duration threshold.
/// 
/// # Type Parameters
/// 
/// * `F` - A function type that returns `T`
/// * `T` - The return type of the operation
/// 
/// # Arguments
/// 
/// * `operation` - The name of the operation to measure
/// * `duration` - The minimum duration threshold for recording
/// * `f` - The operation to measure
/// 
/// # Returns
/// 
/// Returns a `Result` containing the operation's return value.
pub async fn measure_operation_with_duration<F, T>(operation: &str, duration: Duration, f: F) -> Result<T>
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let actual_duration = start.elapsed();
    
    // Record operation duration if it exceeds expected duration
    if actual_duration > duration {
        // Try to get a global collector using the non-async static variable
        if let Some(collector) = PERFORMANCE_COLLECTOR.get() {
            let _ = collector.record_operation(&OperationType::Custom(operation.to_string()), actual_duration).await;
        } else {
            // Only initialize collector if needed and missing
            let factory = ensure_factory();
            let collector = factory.create_collector();
            let _ = collector.record_operation(&OperationType::Custom(operation.to_string()), actual_duration).await;
        }
    }
    
    Ok(result)
}

/// Global performance collector singleton
static PERFORMANCE_COLLECTOR: OnceLock<Arc<PerformanceCollector>> = OnceLock::new();