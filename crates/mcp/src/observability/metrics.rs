//! # Metrics Collection
//! 
//! This module provides a comprehensive metrics collection system for the MCP, enabling
//! quantitative monitoring of system performance, resource usage, and business metrics.
//!
//! ## Metric Types
//!
//! - **Counter**: Monotonically increasing value (e.g., request count)
//! - **Gauge**: Value that can go up and down (e.g., active connections)
//! - **Histogram**: Distribution of values (e.g., request duration)
//! - **Summary**: Similar to histogram but with configurable quantiles

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::observability::{ObservabilityError, ObservabilityResult};

/// Label type for adding dimensions to metrics
pub type Labels = HashMap<String, String>;

/// Supported metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Counter metrics can only increase
    Counter,
    /// Gauge metrics can increase and decrease
    Gauge,
    /// Histogram metrics track value distributions
    Histogram,
    /// Summary metrics track value distributions with quantiles
    Summary,
}

/// A metric with name, description, type, and value
#[derive(Debug, Clone)]
pub struct Metric {
    /// Name of the metric
    name: String,
    /// Description of what the metric measures
    description: String,
    /// Type of the metric
    metric_type: MetricType,
    /// Units for the metric (e.g., "seconds", "bytes")
    unit: Option<String>,
    /// Dimensions for the metric
    labels: Labels,
    /// Last update timestamp
    last_updated: Instant,
}

impl Metric {
    /// Create a new metric
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        metric_type: MetricType,
        unit: Option<String>,
        labels: Labels,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type,
            unit,
            labels,
            last_updated: Instant::now(),
        }
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the metric description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the metric type
    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }

    /// Get the metric unit
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    /// Get the metric labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Get the time since the metric was last updated
    pub fn time_since_update(&self) -> Duration {
        self.last_updated.elapsed()
    }
}

/// A counter metric that can only increase
#[derive(Debug)]
pub struct Counter {
    /// Base metric information
    metric: Metric,
    /// Current value of the counter
    value: RwLock<u64>,
}

impl Counter {
    /// Create a new counter metric
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
    ) -> Self {
        Self {
            metric: Metric::new(name, description, MetricType::Counter, unit, labels),
            value: RwLock::new(0),
        }
    }

    /// Increment the counter by the given amount
    pub fn inc(&self, amount: u64) -> ObservabilityResult<()> {
        let mut value = self.value.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire write lock: {}", e)))?;
        *value += amount;
        Ok(())
    }

    /// Increment the counter by 1
    pub fn inc_one(&self) -> ObservabilityResult<()> {
        self.inc(1)
    }

    /// Get the current value of the counter
    pub fn value(&self) -> ObservabilityResult<u64> {
        let value = self.value.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire read lock: {}", e)))?;
        Ok(*value)
    }

    /// Get the base metric
    pub fn metric(&self) -> &Metric {
        &self.metric
    }
}

/// A gauge metric that can increase and decrease
#[derive(Debug)]
pub struct Gauge {
    /// Base metric information
    metric: Metric,
    /// Current value of the gauge
    value: RwLock<f64>,
}

impl Gauge {
    /// Create a new gauge metric
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
    ) -> Self {
        Self {
            metric: Metric::new(name, description, MetricType::Gauge, unit, labels),
            value: RwLock::new(0.0),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: f64) -> ObservabilityResult<()> {
        let mut current = self.value.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire write lock: {}", e)))?;
        *current = value;
        Ok(())
    }

    /// Increment the gauge by the given amount
    pub fn inc(&self, amount: f64) -> ObservabilityResult<()> {
        let mut current = self.value.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire write lock: {}", e)))?;
        *current += amount;
        Ok(())
    }

    /// Decrement the gauge by the given amount
    pub fn dec(&self, amount: f64) -> ObservabilityResult<()> {
        let mut current = self.value.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire write lock: {}", e)))?;
        *current -= amount;
        Ok(())
    }

    /// Get the current value of the gauge
    pub fn value(&self) -> ObservabilityResult<f64> {
        let value = self.value.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire read lock: {}", e)))?;
        Ok(*value)
    }

    /// Get the base metric
    pub fn metric(&self) -> &Metric {
        &self.metric
    }
}

/// Bucket definition for histograms
#[derive(Debug, Clone)]
pub struct Bucket {
    /// Upper bound of the bucket
    pub upper_bound: f64,
    /// Count of values in this bucket
    pub count: u64,
}

/// A histogram metric that tracks value distributions
#[derive(Debug)]
pub struct Histogram {
    /// Base metric information
    metric: Metric,
    /// Bucket definitions with counts
    buckets: RwLock<Vec<Bucket>>,
    /// Sum of all observed values
    sum: RwLock<f64>,
    /// Count of all observed values
    count: RwLock<u64>,
}

impl Histogram {
    /// Create a new histogram metric with the given bucket boundaries
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
        bucket_boundaries: Vec<f64>,
    ) -> Self {
        // Create buckets from boundaries
        let buckets = bucket_boundaries
            .into_iter()
            .map(|upper_bound| Bucket { upper_bound, count: 0 })
            .collect();

        Self {
            metric: Metric::new(name, description, MetricType::Histogram, unit, labels),
            buckets: RwLock::new(buckets),
            sum: RwLock::new(0.0),
            count: RwLock::new(0),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) -> ObservabilityResult<()> {
        // Update sum and count
        {
            let mut sum = self.sum.write().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire sum write lock: {}", e)))?;
            *sum += value;

            let mut count = self.count.write().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire count write lock: {}", e)))?;
            *count += 1;
        }

        // Update buckets
        let mut buckets = self.buckets.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire buckets write lock: {}", e)))?;
        
        for bucket in buckets.iter_mut() {
            if value <= bucket.upper_bound {
                bucket.count += 1;
            }
        }

        Ok(())
    }

    /// Get the current bucket counts
    pub fn buckets(&self) -> ObservabilityResult<Vec<Bucket>> {
        let buckets = self.buckets.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire buckets read lock: {}", e)))?;
        Ok(buckets.clone())
    }

    /// Get the total count of observations
    pub fn count(&self) -> ObservabilityResult<u64> {
        let count = self.count.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire count read lock: {}", e)))?;
        Ok(*count)
    }

    /// Get the sum of all observations
    pub fn sum(&self) -> ObservabilityResult<f64> {
        let sum = self.sum.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire sum read lock: {}", e)))?;
        Ok(*sum)
    }

    /// Get the base metric
    pub fn metric(&self) -> &Metric {
        &self.metric
    }
}

/// The central registry for all metrics
#[derive(Debug)]
pub struct MetricsRegistry {
    /// All registered counters
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    /// All registered gauges
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    /// All registered histograms
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize the metrics registry
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Register internal metrics for the registry itself
        self.create_counter(
            "mcp_metrics_registry_counters",
            "Number of counters registered in the metrics registry",
            Some("count".to_string()),
            HashMap::new(),
        )?;

        self.create_counter(
            "mcp_metrics_registry_gauges",
            "Number of gauges registered in the metrics registry",
            Some("count".to_string()),
            HashMap::new(),
        )?;

        self.create_counter(
            "mcp_metrics_registry_histograms",
            "Number of histograms registered in the metrics registry",
            Some("count".to_string()),
            HashMap::new(),
        )?;

        Ok(())
    }

    /// Create and register a new counter
    pub fn create_counter(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
    ) -> ObservabilityResult<Arc<Counter>> {
        let name_str = name.into();
        let counter = Arc::new(Counter::new(name_str.clone(), description, unit, labels));

        let mut counters = self.counters.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters write lock: {}", e)))?;
        
        counters.insert(name_str, counter.clone());
        
        // Update internal metrics counter if it exists
        if let Some(internal_counter) = counters.get("mcp_metrics_registry_counters") {
            let _ = internal_counter.inc_one();
        }
        
        Ok(counter)
    }

    /// Create and register a new gauge
    pub fn create_gauge(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
    ) -> ObservabilityResult<Arc<Gauge>> {
        let name_str = name.into();
        let gauge = Arc::new(Gauge::new(name_str.clone(), description, unit, labels));

        let mut gauges = self.gauges.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges write lock: {}", e)))?;
        
        gauges.insert(name_str, gauge.clone());
        
        // Update internal metrics counter if it exists
        if let Some(internal_counter) = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?.get("mcp_metrics_registry_gauges") {
            let _ = internal_counter.inc_one();
        }
        
        Ok(gauge)
    }

    /// Create and register a new histogram
    pub fn create_histogram(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
        bucket_boundaries: Vec<f64>,
    ) -> ObservabilityResult<Arc<Histogram>> {
        let name_str = name.into();
        let histogram = Arc::new(Histogram::new(name_str.clone(), description, unit, labels, bucket_boundaries));

        let mut histograms = self.histograms.write().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire histograms write lock: {}", e)))?;
        
        histograms.insert(name_str, histogram.clone());
        
        // Update internal metrics counter if it exists
        if let Some(internal_counter) = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?.get("mcp_metrics_registry_histograms") {
            let _ = internal_counter.inc_one();
        }
        
        Ok(histogram)
    }

    /// Get a counter by name
    pub fn get_counter(&self, name: &str) -> ObservabilityResult<Option<Arc<Counter>>> {
        let counters = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?;
        
        Ok(counters.get(name).cloned())
    }

    /// Get a gauge by name
    pub fn get_gauge(&self, name: &str) -> ObservabilityResult<Option<Arc<Gauge>>> {
        let gauges = self.gauges.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?;
        
        Ok(gauges.get(name).cloned())
    }

    /// Get a histogram by name
    pub fn get_histogram(&self, name: &str) -> ObservabilityResult<Option<Arc<Histogram>>> {
        let histograms = self.histograms.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire histograms read lock: {}", e)))?;
        
        Ok(histograms.get(name).cloned())
    }

    /// Get all counter names
    pub fn counter_names(&self) -> ObservabilityResult<Vec<String>> {
        let counters = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?;
        
        Ok(counters.keys().cloned().collect())
    }

    /// Get all gauge names
    pub fn gauge_names(&self) -> ObservabilityResult<Vec<String>> {
        let gauges = self.gauges.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?;
        
        Ok(gauges.keys().cloned().collect())
    }

    /// Get all histogram names
    pub fn histogram_names(&self) -> ObservabilityResult<Vec<String>> {
        let histograms = self.histograms.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire histograms read lock: {}", e)))?;
        
        Ok(histograms.keys().cloned().collect())
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
} 