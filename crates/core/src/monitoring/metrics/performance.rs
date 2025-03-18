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

pub mod adapter;
pub use adapter::{PerformanceCollectorAdapter, create_collector_adapter, create_collector_adapter_with_collector};

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

/// Configuration for the performance collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance collection
    pub enabled: bool,
    /// Histogram buckets for operation durations (in seconds)
    pub histogram_buckets: Vec<f64>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            histogram_buckets: vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0],
        }
    }
}

/// Performance metrics collector
#[derive(Debug)]
pub struct PerformanceCollector {
    /// Operation histograms
    histograms: Arc<RwLock<HashMap<OperationType, Histogram>>>,
    /// Configuration
    config: PerformanceConfig,
}

impl PerformanceCollector {
    /// Create a new performance collector with default configuration
    #[must_use] pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create a new performance collector with specific configuration
    #[must_use] pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            histograms: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Record an operation with a specific duration
    pub async fn record_operation(&self, op_type: &OperationType, duration: Duration) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut histograms = self.histograms.write().await;
        let histogram = histograms.entry(op_type.clone()).or_insert_with(|| {
            let opts = HistogramOpts::new(
                format!("operation_{}", op_type),
                format!("Histogram for {} operations", op_type),
            )
            .buckets(self.config.histogram_buckets.clone());
            Histogram::with_opts(opts).expect("Failed to create histogram")
        });

        histogram.observe(duration.as_secs_f64());
        Ok(())
    }

    /// Get metrics for all operations
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let histograms = self.histograms.read().await;
        let mut metrics = Vec::new();

        for (op_type, histogram) in histograms.iter() {
            let count = histogram.get_sample_count();
            let sum = histogram.get_sample_sum();

            metrics.push(Metric::new(
                format!("operation_{}_count", op_type),
                count as f64,
                MetricType::Counter,
                Some(HashMap::from([("operation".to_string(), op_type.to_string())])),
            ));

            metrics.push(Metric::new(
                format!("operation_{}_duration_sum", op_type),
                sum,
                MetricType::Counter,
                Some(HashMap::from([("operation".to_string(), op_type.to_string())])),
            ));

            metrics.push(Metric::new(
                format!("operation_{}_duration_avg", op_type),
                if count > 0 { sum / count as f64 } else { 0.0 },
                MetricType::Gauge,
                Some(HashMap::from([("operation".to_string(), op_type.to_string())])),
            ));
        }

        Ok(metrics)
    }
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricCollector for PerformanceCollector {
    async fn get_metrics(&self) -> Result<Vec<Metric>> {
        self.get_metrics().await
    }

    async fn record_metrics(&self, metrics: &[Metric]) -> Result<()> {
        for metric in metrics {
            if let Some(labels) = &metric.labels {
                if let Some(op_type) = labels.get("operation") {
                    let duration = Duration::from_secs_f64(metric.value);
                    self.record_operation(&OperationType::Custom(op_type.clone()), duration).await?;
                }
            }
        }
        Ok(())
    }
}

/// Factory for creating performance collectors
#[derive(Debug, Clone)]
pub struct PerformanceCollectorFactory {
    /// Configuration for creating collectors
    config: PerformanceConfig,
}

impl PerformanceCollectorFactory {
    /// Create a new factory with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: PerformanceConfig::default(),
        }
    }

    /// Create a new factory with specific configuration
    #[must_use] pub const fn with_config(config: PerformanceConfig) -> Self {
        Self { config }
    }

    /// Create a new collector with the factory's configuration
    #[must_use] pub fn create_collector(&self) -> Arc<PerformanceCollector> {
        Arc::new(PerformanceCollector::with_config(self.config.clone()))
    }

    /// Create a new collector adapter
    #[must_use] pub fn create_collector_adapter(&self) -> Arc<PerformanceCollectorAdapter> {
        let collector = self.create_collector();
        create_collector_adapter_with_collector(collector)
    }
}

impl Default for PerformanceCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new performance collector adapter
#[must_use]
pub fn create_collector_adapter() -> Arc<PerformanceCollectorAdapter> {
    PerformanceCollectorFactory::new().create_collector_adapter()
}

/// Create a new performance collector adapter with a specific collector
#[must_use]
pub fn create_collector_adapter_with_collector(
    collector: Arc<PerformanceCollector>
) -> Arc<PerformanceCollectorAdapter> {
    Arc::new(PerformanceCollectorAdapter::with_collector(collector))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[tokio::test]
    async fn test_performance_collector_basic() {
        let collector = PerformanceCollector::new();

        // Record some operations
        collector.record_operation(&OperationType::DatabaseRead, Duration::from_millis(100)).await.unwrap();
        collector.record_operation(&OperationType::DatabaseWrite, Duration::from_millis(200)).await.unwrap();

        // Get metrics
        let metrics = collector.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_performance_collector_adapter() {
        let collector = Arc::new(PerformanceCollector::new());
        let adapter = PerformanceCollectorAdapter::with_collector(collector);

        // Time an operation
        let result = adapter.time_operation(OperationType::DatabaseRead, || {
            thread::sleep(Duration::from_millis(10));
            42
        }).await;

        assert_eq!(result, 42);

        // Get metrics
        let metrics = adapter.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_performance_collector_factory() {
        let config = PerformanceConfig {
            enabled: true,
            histogram_buckets: vec![0.1, 1.0, 10.0],
        };

        let factory = PerformanceCollectorFactory::with_config(config);
        let adapter = factory.create_collector_adapter();

        // Record an operation
        adapter.record_operation(&OperationType::NetworkRequest, Duration::from_secs(1)).await.unwrap();

        // Get metrics
        let metrics = adapter.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_performance_collector_with_dependencies() {
        let config = PerformanceConfig::default();
        let factory = PerformanceCollectorFactory::with_config(config);
        let collector = factory.create_collector();
        
        // Record some operations
        collector.record_operation(&OperationType::DatabaseRead, Duration::from_millis(100)).await.unwrap();
        collector.record_operation(&OperationType::DatabaseWrite, Duration::from_millis(200)).await.unwrap();
        
        // Get metrics
        let metrics = collector.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }
}