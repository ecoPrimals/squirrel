// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
use serde::{Serialize, Deserialize};

/// Label type for adding dimensions to metrics
pub type Labels = HashMap<String, String>;

/// Snapshot of a metric for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Name of the metric
    pub name: String,
    /// Description of the metric
    pub description: String,
    /// Type of the metric
    pub metric_type: MetricType,
    /// Units for the metric
    pub unit: Option<String>,
    /// Labels/dimensions for the metric
    pub labels: Labels,
    /// Value of the metric
    pub value: MetricValue,
    /// Timestamp when the snapshot was taken
    pub timestamp: std::time::SystemTime,
}

/// Value types for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram {
        buckets: Vec<Bucket>,
        sum: f64,
        count: u64,
    },
}

/// Supported metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Configuration for the metrics registry
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Whether metrics collection is enabled
    pub enabled: bool,
    /// Maximum number of metrics to store
    pub max_metrics: usize,
    /// Prefix for all metric names
    pub prefix: Option<String>,
    /// Namespace for metrics
    pub namespace: Option<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_metrics: 10000,
            prefix: None,
            namespace: None,
        }
    }
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    /// Increment counter with value and labels
    pub fn increment_counter(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> ObservabilityResult<()> {
        let key = self.create_metric_key(name, &labels.as_ref().unwrap_or(&HashMap::new()));
        let counters = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?;
        
        if let Some(counter) = counters.get(&key) {
            counter.inc(value as u64)?;
        } else {
            // Create counter if it doesn't exist
            drop(counters);
            let counter = self.create_counter(name, name, None, labels.unwrap_or_default())?;
            counter.inc(value as u64)?;
        }
        
        Ok(())
    }
    
    /// Set a gauge to a specific value
    pub fn set_gauge(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> ObservabilityResult<()> {
        let gauge = if let Some(g) = self.get_gauge(name)? {
            g
        } else {
            let gauge = self.create_gauge(
                name,
                &format!("Auto-created gauge for {}", name),
                None,
                labels.unwrap_or_default()
            )?;
            gauge
        };
        
        gauge.set(value)
    }

    /// Record a value in a histogram
    pub fn record_histogram(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> ObservabilityResult<()> {
        let histogram = if let Some(h) = self.get_histogram(name)? {
            h
        } else {
            // Default bucket boundaries for auto-created histograms
            let default_buckets = vec![0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0];
            let histogram = self.create_histogram(
                name,
                &format!("Auto-created histogram for {}", name),
                None,
                labels.unwrap_or_default(),
                default_buckets
            )?;
            histogram
        };
        
        histogram.observe(value)
    }

    /// List all metrics in the registry
    pub fn list_metrics(&self) -> ObservabilityResult<Vec<String>> {
        let mut all_metrics = Vec::new();
        
        // Add all counter names
        let counter_names = self.counter_names()?;
        all_metrics.extend(counter_names);
        
        // Add all gauge names
        let gauge_names = self.gauge_names()?;
        all_metrics.extend(gauge_names);
        
        // Add all histogram names
        let histogram_names = self.histogram_names()?;
        all_metrics.extend(histogram_names);
        
        // Sort for consistent ordering
        all_metrics.sort();
        
        Ok(all_metrics)
    }

    /// Get configuration (placeholder for compatibility)
    pub fn get_config(&self) -> ObservabilityResult<MetricsConfig> {
        Ok(MetricsConfig::default())
    }

    /// Set configuration (placeholder for compatibility)
    pub fn set_config(&self, _config: &MetricsConfig) -> ObservabilityResult<()> {
        // Configuration changes would be applied here
        Ok(())
    }

    /// Create counter with namespace (compatibility method)
    pub fn create_counter_with_namespace(
        &self,
        namespace: &str,
        name: impl Into<String>,
        description: impl Into<String>,
        unit: Option<String>,
        labels: Labels,
    ) -> ObservabilityResult<Arc<Counter>> {
        let full_name = format!("{}_{}", namespace, name.into());
        self.create_counter(full_name, description, unit, labels)
    }

    /// Shutdown the metrics registry
    pub fn shutdown(&self) -> ObservabilityResult<()> {
        // Clear all metrics
        {
            let mut counters = self.counters.write().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire counters write lock: {}", e)))?;
            counters.clear();
        }
        
        {
            let mut gauges = self.gauges.write().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire gauges write lock: {}", e)))?;
            gauges.clear();
        }
        
        {
            let mut histograms = self.histograms.write().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire histograms write lock: {}", e)))?;
            histograms.clear();
        }
        
        Ok(())
    }

    /// Export a batch of metrics as snapshots
    pub async fn export_batch(&self, max_batch_size: usize) -> ObservabilityResult<Vec<MetricSnapshot>> {
        let mut snapshots = Vec::new();
        let current_time = std::time::SystemTime::now();

        // Export counters
        {
            let counters = self.counters.read().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?;
            
            for (name, counter) in counters.iter().take(max_batch_size - snapshots.len()) {
                if snapshots.len() >= max_batch_size { break; }
                
                let value = counter.value()?;
                snapshots.push(MetricSnapshot {
                    name: name.clone(),
                    description: counter.metric().description().to_string(),
                    metric_type: MetricType::Counter,
                    unit: counter.metric().unit().map(|s| s.to_string()),
                    labels: counter.metric().labels().clone(),
                    value: MetricValue::Counter(value),
                    timestamp: current_time,
                });
            }
        }

        // Export gauges
        if snapshots.len() < max_batch_size {
            let gauges = self.gauges.read().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?;
            
            for (name, gauge) in gauges.iter().take(max_batch_size - snapshots.len()) {
                if snapshots.len() >= max_batch_size { break; }
                
                let value = gauge.value()?;
                snapshots.push(MetricSnapshot {
                    name: name.clone(),
                    description: gauge.metric().description().to_string(),
                    metric_type: MetricType::Gauge,
                    unit: gauge.metric().unit().map(|s| s.to_string()),
                    labels: gauge.metric().labels().clone(),
                    value: MetricValue::Gauge(value),
                    timestamp: current_time,
                });
            }
        }

        // Export histograms
        if snapshots.len() < max_batch_size {
            let histograms = self.histograms.read().map_err(|e| 
                ObservabilityError::MetricsError(format!("Failed to acquire histograms read lock: {}", e)))?;
            
            for (name, histogram) in histograms.iter().take(max_batch_size - snapshots.len()) {
                if snapshots.len() >= max_batch_size { break; }
                
                let buckets = histogram.buckets()?;
                let sum = histogram.sum()?;
                let count = histogram.count()?;
                
                snapshots.push(MetricSnapshot {
                    name: name.clone(),
                    description: histogram.metric().description().to_string(),
                    metric_type: MetricType::Histogram,
                    unit: histogram.metric().unit().map(|s| s.to_string()),
                    labels: histogram.metric().labels().clone(),
                    value: MetricValue::Histogram { buckets, sum, count },
                    timestamp: current_time,
                });
            }
        }

        Ok(snapshots)
    }

    /// Get total count of metrics
    pub async fn get_metrics_count(&self) -> ObservabilityResult<usize> {
        let counters_count = self.counters.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire counters read lock: {}", e)))?.len();
        let gauges_count = self.gauges.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?.len();
        let histograms_count = self.histograms.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire histograms read lock: {}", e)))?.len();
        
        Ok(counters_count + gauges_count + histograms_count)
    }

    /// Increment a gauge metric
    pub fn increment_gauge(&self, name: &str, labels: Option<HashMap<String, String>>) -> ObservabilityResult<()> {
        let key = self.create_metric_key(name, &labels.unwrap_or_default());
        let gauges = self.gauges.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?;
        
        if let Some(gauge) = gauges.get(&key) {
            gauge.inc(1.0)?;
        }
        
        Ok(())
    }

    /// Decrement a gauge metric
    pub fn decrement_gauge(&self, name: &str, labels: Option<HashMap<String, String>>) -> ObservabilityResult<()> {
        let key = self.create_metric_key(name, &labels.unwrap_or_default());
        let gauges = self.gauges.read().map_err(|e| 
            ObservabilityError::MetricsError(format!("Failed to acquire gauges read lock: {}", e)))?;
        
        if let Some(gauge) = gauges.get(&key) {
            gauge.dec(1.0)?;
        }
        
        Ok(())
    }

    /// Helper to create metric key with labels
    fn create_metric_key(&self, name: &str, labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            name.to_string()
        } else {
            let mut key = name.to_string();
            let mut label_parts: Vec<String> = labels.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            label_parts.sort();
            key.push_str(&format!("{{{}}}", label_parts.join(",")));
            key
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
} 