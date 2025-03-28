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
    #[must_use]
    pub fn new_counter(name: impl Into<String>, value: u64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(value),
            last_updated: Utc::now(),
        }
    }

    /// Create a new gauge metric
    #[must_use]
    pub fn new_gauge(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            last_updated: Utc::now(),
        }
    }

    /// Create a new histogram metric
    #[must_use]
    pub fn new_histogram(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Histogram,
            value: MetricValue::Histogram(values),
            last_updated: Utc::now(),
        }
    }

    /// Create a new timer metric
    #[must_use]
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
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            collector: None,
        }
    }

    /// Create a new timer with a collector
    #[must_use]
    pub fn with_collector(name: impl Into<String>, collector: Arc<MetricsCollector>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            collector: Some(collector),
        }
    }

    /// Get the elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and return the elapsed time
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic unless there is a critical system failure.
    pub fn increment_counter(&self, name: &str) {
        match self.counters.write() {
            Ok(mut counters_guard) => {
                let counter = counters_guard
                    .entry(name.to_string())
                    .or_insert_with(|| AtomicU64::new(0));
                counter.fetch_add(1, Ordering::SeqCst);
            },
            Err(e) => {
                // Log the error but don't propagate it since metrics shouldn't affect core functionality
                log::error!("Failed to increment counter '{}': RwLock poisoned: {}", name, e);
            }
        }
    }

    /// Set a gauge
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    pub fn set_gauge(&self, name: &str, value: i64) {
        match self.gauges.write() {
            Ok(mut gauges) => {
                gauges.insert(name.to_string(), value);
            },
            Err(e) => {
                log::error!("Failed to set gauge '{}': RwLock poisoned: {}", name, e);
            }
        }
    }

    /// Record a histogram value
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    pub fn record_histogram(&self, name: &str, value: Duration) {
        // Convert to milliseconds as f64 without precision loss
        let millis = value.as_secs() as f64 * 1000.0 + value.subsec_millis() as f64;
        
        match self.histograms.write() {
            Ok(mut histograms_guard) => {
                let values = histograms_guard
                    .entry(name.to_string())
                    .or_insert_with(Vec::new);
                
                if values.len() >= self.max_histogram_size {
                    values.remove(0); // Remove oldest value if at capacity
                }
                
                values.push(millis);
            },
            Err(e) => {
                log::error!("Failed to record histogram '{}': RwLock poisoned: {}", name, e);
            }
        }
    }

    /// Start a timer
    #[must_use]
    pub fn start_timer(&self, name: &str) -> MetricsTimer {
        MetricsTimer::with_collector(name, Arc::new(self.clone()))
    }

    /// Get all metrics
    /// 
    /// # Errors
    ///
    /// This method logs errors if any of the underlying RwLocks are poisoned but continues operation with available data.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    #[must_use]
    pub fn get_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Add counters
        if let Ok(counters) = self.counters.read() {
            for (name, counter) in counters.iter() {
                metrics.push(Metric::new_counter(
                    name.clone(),
                    counter.load(Ordering::SeqCst),
                ));
            }
        } else {
            log::error!("Failed to read counters: RwLock poisoned");
        }
        
        // Add gauges
        if let Ok(gauges) = self.gauges.read() {
            for (name, value) in gauges.iter() {
                metrics.push(Metric::new_gauge(name.clone(), *value));
            }
        } else {
            log::error!("Failed to read gauges: RwLock poisoned");
        }
        
        // Add histograms
        if let Ok(histograms) = self.histograms.read() {
            for (name, values) in histograms.iter() {
                metrics.push(Metric::new_histogram(name.clone(), values.clone()));
            }
        } else {
            log::error!("Failed to read histograms: RwLock poisoned");
        }
        
        // Add timers
        if let Ok(timers) = self.timers.read() {
            for (name, values) in timers.iter() {
                metrics.push(Metric::new_timer(name.clone(), values.clone()));
            }
        } else {
            log::error!("Failed to read timers: RwLock poisoned");
        }
        
        metrics
    }

    /// Get a counter value
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    #[must_use]
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        match self.counters.read() {
            Ok(counters) => counters.get(name).map(|c| c.load(Ordering::SeqCst)),
            Err(e) => {
                log::error!("Failed to read counter '{}': RwLock poisoned: {}", name, e);
                None
            }
        }
    }

    /// Get a gauge value
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    #[must_use]
    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        match self.gauges.read() {
            Ok(gauges) => gauges.get(name).copied(),
            Err(e) => {
                log::error!("Failed to read gauge '{}': RwLock poisoned: {}", name, e);
                None
            }
        }
    }

    /// Get histogram values
    /// 
    /// # Errors
    ///
    /// This method logs an error if the underlying RwLock is poisoned but continues operation.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    #[must_use]
    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        match self.histograms.read() {
            Ok(histograms) => histograms.get(name).cloned(),
            Err(e) => {
                log::error!("Failed to read histogram '{}': RwLock poisoned: {}", name, e);
                None
            }
        }
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        // Create a new collector
        let new_collector = Self::new();
        
        // Copy counter values
        if let (Ok(counters), Ok(mut new_counters)) = (self.counters.read(), new_collector.counters.write()) {
            for (name, counter) in counters.iter() {
                new_counters.insert(
                    name.clone(),
                    AtomicU64::new(counter.load(Ordering::SeqCst)),
                );
            }
        } else {
            log::error!("Failed to clone counters: RwLock poisoned");
        }
        
        // Copy gauge values
        if let (Ok(gauges), Ok(mut new_gauges)) = (self.gauges.read(), new_collector.gauges.write()) {
            for (name, value) in gauges.iter() {
                new_gauges.insert(name.clone(), *value);
            }
        } else {
            log::error!("Failed to clone gauges: RwLock poisoned");
        }
        
        // Copy histogram values
        if let (Ok(histograms), Ok(mut new_histograms)) = (self.histograms.read(), new_collector.histograms.write()) {
            for (name, values) in histograms.iter() {
                new_histograms.insert(name.clone(), values.clone());
            }
        } else {
            log::error!("Failed to clone histograms: RwLock poisoned");
        }
        
        // Copy timer values
        if let (Ok(timers), Ok(mut new_timers)) = (self.timers.read(), new_collector.timers.write()) {
            for (name, values) in timers.iter() {
                new_timers.insert(name.clone(), values.clone());
            }
        } else {
            log::error!("Failed to clone timers: RwLock poisoned");
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