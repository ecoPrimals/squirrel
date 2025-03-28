//! Metrics module for MCP
//!
//! This module provides metrics collection functionality for MCP components.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (only increases)
    Counter,
    /// Gauge metric (can go up and down)
    Gauge,
    /// Histogram metric (distribution of values)
    Histogram,
    /// Timer metric (special case of histogram)
    Timer,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(i64),
    /// Histogram values
    Histogram(Vec<f64>),
    /// Timer values (in milliseconds)
    Timer(Vec<f64>),
}

/// Metric datapoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Metric {
    /// Create a new counter metric
    pub fn new_counter(name: impl Into<String>, value: u64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(value),
            last_updated: Utc::now(),
        }
    }

    /// Create a new gauge metric
    pub fn new_gauge(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            last_updated: Utc::now(),
        }
    }

    /// Create a new histogram metric
    pub fn new_histogram(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Histogram,
            value: MetricValue::Histogram(values),
            last_updated: Utc::now(),
        }
    }

    /// Create a new timer metric
    pub fn new_timer(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Timer,
            value: MetricValue::Timer(values),
            last_updated: Utc::now(),
        }
    }
}

/// Timer for measuring durations
#[derive(Debug)]
pub struct MetricsTimer {
    /// Timer name
    name: String,
    /// Start time
    start: Instant,
    /// Optional collector for automatic recording
    collector: Option<Arc<MetricsCollector>>,
}

impl MetricsTimer {
    /// Create a new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            collector: None,
        }
    }

    /// Create a new timer with a collector
    pub fn with_collector(name: impl Into<String>, collector: Arc<MetricsCollector>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            collector: Some(collector),
        }
    }

    /// Get the elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and return the elapsed time
    pub fn stop(self) -> Duration {
        let elapsed = self.elapsed();
        if let Some(collector) = self.collector {
            collector.record_histogram(&self.name, elapsed);
        }
        elapsed
    }
}

/// Metrics collector for MCP components
#[derive(Debug)]
pub struct MetricsCollector {
    /// Counter metrics
    counters: RwLock<HashMap<String, AtomicU64>>,
    /// Gauge metrics
    gauges: RwLock<HashMap<String, i64>>,
    /// Histogram metrics
    histograms: RwLock<HashMap<String, Vec<f64>>>,
    /// Timer metrics
    timers: RwLock<HashMap<String, Vec<f64>>>,
    /// Maximum number of values to store in histograms/timers
    max_histogram_size: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            timers: RwLock::new(HashMap::new()),
            max_histogram_size: 1000,
        }
    }

    /// Create a test metrics collector
    pub fn new_test() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            timers: RwLock::new(HashMap::new()),
            max_histogram_size: 100,
        }
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.write().unwrap();
        let counter = counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Set a gauge
    pub fn set_gauge(&self, name: &str, value: i64) {
        let mut gauges = self.gauges.write().unwrap();
        gauges.insert(name.to_string(), value);
    }

    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: Duration) {
        let millis = value.as_millis() as f64;
        let mut histograms = self.histograms.write().unwrap();
        let values = histograms.entry(name.to_string()).or_insert_with(Vec::new);
        
        if values.len() >= self.max_histogram_size {
            values.remove(0); // Remove oldest value if at capacity
        }
        
        values.push(millis);
    }

    /// Start a timer
    pub fn start_timer(&self, name: &str) -> MetricsTimer {
        MetricsTimer::with_collector(name, Arc::new(self.clone()))
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Add counters
        let counters = self.counters.read().unwrap();
        for (name, counter) in counters.iter() {
            metrics.push(Metric::new_counter(
                name.clone(),
                counter.load(Ordering::SeqCst),
            ));
        }
        
        // Add gauges
        let gauges = self.gauges.read().unwrap();
        for (name, value) in gauges.iter() {
            metrics.push(Metric::new_gauge(name.clone(), *value));
        }
        
        // Add histograms
        let histograms = self.histograms.read().unwrap();
        for (name, values) in histograms.iter() {
            metrics.push(Metric::new_histogram(name.clone(), values.clone()));
        }
        
        // Add timers
        let timers = self.timers.read().unwrap();
        for (name, values) in timers.iter() {
            metrics.push(Metric::new_timer(name.clone(), values.clone()));
        }
        
        metrics
    }

    /// Get a counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read().unwrap();
        counters.get(name).map(|c| c.load(Ordering::SeqCst))
    }

    /// Get a gauge value
    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        let gauges = self.gauges.read().unwrap();
        gauges.get(name).copied()
    }

    /// Get histogram values
    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        let histograms = self.histograms.read().unwrap();
        histograms.get(name).cloned()
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        // Create a new collector
        let new_collector = Self::new();
        
        // Copy counter values
        {
            let counters = self.counters.read().unwrap();
            let mut new_counters = new_collector.counters.write().unwrap();
            for (name, counter) in counters.iter() {
                new_counters.insert(
                    name.clone(),
                    AtomicU64::new(counter.load(Ordering::SeqCst)),
                );
            }
        }
        
        // Copy gauge values
        {
            let gauges = self.gauges.read().unwrap();
            let mut new_gauges = new_collector.gauges.write().unwrap();
            for (name, value) in gauges.iter() {
                new_gauges.insert(name.clone(), *value);
            }
        }
        
        // Copy histogram values
        {
            let histograms = self.histograms.read().unwrap();
            let mut new_histograms = new_collector.histograms.write().unwrap();
            for (name, values) in histograms.iter() {
                new_histograms.insert(name.clone(), values.clone());
            }
        }
        
        // Copy timer values
        {
            let timers = self.timers.read().unwrap();
            let mut new_timers = new_collector.timers.write().unwrap();
            for (name, values) in timers.iter() {
                new_timers.insert(name.clone(), values.clone());
            }
        }
        
        new_collector
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_counter_metrics() {
        let collector = MetricsCollector::new_test();
        
        collector.increment_counter("test_counter");
        collector.increment_counter("test_counter");
        
        assert_eq!(collector.get_counter("test_counter"), Some(2));
        assert_eq!(collector.get_counter("non_existent_counter"), None);
    }
    
    #[test]
    fn test_gauge_metrics() {
        let collector = MetricsCollector::new_test();
        
        collector.set_gauge("test_gauge", 42);
        
        assert_eq!(collector.get_gauge("test_gauge"), Some(42));
        assert_eq!(collector.get_gauge("non_existent_gauge"), None);
    }
    
    #[test]
    fn test_timer_metrics() {
        let collector = MetricsCollector::new_test();
        
        let timer = collector.start_timer("test_timer");
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.stop();
        
        assert!(elapsed.as_millis() >= 10);
        
        let histogram = collector.get_histogram("test_timer");
        assert!(histogram.is_some());
        assert!(!histogram.unwrap().is_empty());
    }
    
    #[test]
    fn test_get_metrics() {
        let collector = MetricsCollector::new_test();
        
        collector.increment_counter("test_counter");
        collector.set_gauge("test_gauge", 42);
        
        let metrics = collector.get_metrics();
        assert_eq!(metrics.len(), 2);
    }
    
    #[test]
    fn test_clone() {
        let collector = MetricsCollector::new_test();
        
        collector.increment_counter("test_counter");
        collector.set_gauge("test_gauge", 42);
        
        let cloned = collector.clone();
        
        assert_eq!(cloned.get_counter("test_counter"), Some(1));
        assert_eq!(cloned.get_gauge("test_gauge"), Some(42));
    }
} 