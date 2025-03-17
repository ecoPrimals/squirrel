//! Performance metrics collection for operation timing
//! 
//! Tracks operation latencies for:
//! - Protocol operations
//! - Context operations
//! - Command execution
//! - Monitoring operations

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use crate::monitoring::metrics::{Metric, MetricCollector};
use std::fmt;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use prometheus::{Histogram, HistogramOpts};
use std::hash::Hash;
use std::cmp::Eq;
use crate::monitoring::metrics::MetricType;
use async_trait::async_trait;

/// Operation type for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum OperationType {
    Request,
    Database,
    FileIO,
    Computation,
    Network,
    Custom(String),
}

impl OperationType {
    pub fn as_str(&self) -> &str {
        match self {
            OperationType::Request => "request",
            OperationType::Database => "database",
            OperationType::FileIO => "fileIO",
            OperationType::Computation => "computation",
            OperationType::Network => "network",
            OperationType::Custom(name) => name.as_str(),
        }
    }
}

/// Performance metrics for an operation type
#[derive(Clone)]
pub struct OperationMetrics {
    pub operation_type: OperationType,
    pub count: u64,
    pub total_duration: f64,
    pub min_duration: f64,
    pub max_duration: f64,
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
    pub fn new() -> Self {
        Self {
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
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
    pub fn start_operation() -> Instant {
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
    pub fn get_all_metrics(&self) -> HashMap<OperationType, OperationMetrics> {
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

// Module initialization
static PERFORMANCE_COLLECTOR: tokio::sync::OnceCell<Arc<PerformanceCollector>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the performance collector
pub async fn initialize() -> Result<()> {
    let collector = Arc::new(PerformanceCollector::new());
    PERFORMANCE_COLLECTOR
        .set(collector)
        .map_err(|_| "Performance collector already initialized")?;
    Ok(())
}

/// Records an operation with the global performance collector
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
/// This function panics if the operation cannot be recorded or if the performance collector is not initialized
pub async fn record_operation(op_type: OperationType, duration: Duration) -> Result<()> {
    if let Some(collector) = PERFORMANCE_COLLECTOR.get() {
        collector.record_operation(&op_type, duration).await
    } else {
        Ok(())
    }
}

/// Time an operation and record its duration
pub async fn time_operation<F, T>(op_type: OperationType, f: F) -> T
where
    F: FnOnce() -> T,
{
    if let Some(collector) = PERFORMANCE_COLLECTOR.get() {
        collector.time_operation(op_type, f).await
    } else {
        f()
    }
}

/// Start timing an operation
pub fn start_operation() -> Instant {
    PerformanceCollector::start_operation()
}

pub struct PerformanceMetrics {
    histogram: Histogram,
}

impl PerformanceMetrics {
    /// Creates a new performance metrics
    ///
    /// # Parameters
    /// * `name` - The name of the metrics
    /// * `help` - A description of the metrics
    ///
    /// # Returns
    /// A new PerformanceMetrics instance
    ///
    /// # Panics
    ///
    /// This function panics if the Histogram cannot be created with the given options
    pub fn new(name: &str, help: &str) -> Self {
        let opts = HistogramOpts::new(name, help);
        Self {
            histogram: Histogram::with_opts(opts).unwrap(),
        }
    }

    pub fn observe(&mut self, duration: Duration) {
        self.histogram.observe(duration.as_secs_f64());
    }

    pub fn get_metrics(&self) -> PerformanceStats {
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

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub operation_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct OperationStats {
    pub count: u64,
    pub total_latency: f64,
    pub min_latency: f64,
    pub max_latency: f64,
}

impl OperationStats {
    pub fn new() -> Self {
        Self {
            count: 0,
            total_latency: 0.0,
            min_latency: f64::MAX,
            max_latency: 0.0,
        }
    }

    pub fn update(&mut self, latency: f64) {
        self.count += 1;
        self.total_latency += latency;
        self.min_latency = self.min_latency.min(latency);
        self.max_latency = self.max_latency.max(latency);
    }

    pub fn average_latency(&self) -> f64 {
        if self.count > 0 {
            self.total_latency / self.count as f64
        } else {
            0.0
        }
    }
}

impl Default for OperationStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct PerformanceMetricsCollector {
    metrics: Arc<RwLock<HashMap<String, OperationStats>>>,
}

impl PerformanceMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_operation(&self, operation: &str, latency: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let stats = metrics.entry(operation.to_string()).or_default();
        stats.update(latency);
        Ok(())
    }

    pub async fn get_operation_stats(&self, operation: &str) -> Result<Option<OperationStats>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(operation).cloned())
    }

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
        let op_type = OperationType::Network;
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