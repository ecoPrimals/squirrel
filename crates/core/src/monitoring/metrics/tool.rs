// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![warn(clippy::missing_errors_doc)] // Enable warnings for missing error documentation
#![allow(clippy::unused_async)] // Allow unused async functions
#![allow(clippy::redundant_closure_for_method_calls)] // Allow redundant closures

//! Tool metrics collection and monitoring
//!
//! This module provides functionality for tracking and analyzing tool usage metrics.
//! It collects data on tool execution frequency, success rates, and performance characteristics.
//! The metrics can be used to identify bottlenecks, optimize workflows, and monitor system health.

use crate::error::Result;
use crate::monitoring::metrics::{Metric, MetricCollector, MetricType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait;
use crate::error::SquirrelError;

pub mod adapter;
pub use adapter::{ToolMetricsCollectorAdapter, create_collector_adapter, create_collector_adapter_with_collector};

/// Tool execution metrics
#[derive(Debug, Clone, Default)]
pub struct ToolMetrics {
    /// Name of the tool being monitored
    pub name: String,
    /// Total number of times the tool has been used
    pub usage_count: u64,
    /// Number of successful tool executions
    pub success_count: u64,
    /// Number of failed tool executions
    pub failure_count: u64,
    /// Average duration of tool executions in milliseconds
    pub average_duration: f64,
}

impl ToolMetrics {
    /// Creates a new tool metrics tracker for the specified tool
    ///
    /// # Arguments
    /// * `name` - The name of the tool to track
    ///
    /// # Returns
    /// A new ToolMetrics instance initialized with zero usage
    #[must_use] pub const fn new(name: String) -> Self {
        Self {
            name,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            average_duration: 0.0,
        }
    }

    /// Records a tool usage event with execution duration and outcome
    ///
    /// # Arguments
    /// * `duration` - The execution duration in milliseconds
    /// * `success` - Whether the execution was successful
    pub fn record_usage(&mut self, duration: f64, success: bool) {
        self.usage_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        // Update average duration using running average formula
        self.average_duration = self.average_duration.mul_add((self.usage_count - 1) as f64, duration) / self.usage_count as f64;
    }
    
    /// Calculates the success rate of tool executions
    ///
    /// # Returns
    /// A float between 0.0 and 1.0 representing the percentage of successful executions
    #[must_use] pub fn success_rate(&self) -> f64 {
        if self.usage_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.usage_count as f64
    }
}

/// Configuration for tool metrics collection
#[derive(Debug, Clone)]
pub struct ToolMetricsConfig {
    /// Whether to enable tool metrics collection
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval: u64,
    /// Maximum history size
    pub history_size: usize,
}

impl Default for ToolMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            history_size: 100,
        }
    }
}

/// Tool metrics collector
#[derive(Debug)]
pub struct ToolMetricsCollector {
    /// Storage for tool metrics by tool name
    metrics: Arc<RwLock<HashMap<String, ToolMetrics>>>,
    /// Configuration for the collector
    config: ToolMetricsConfig,
    /// Performance collector adapter for additional metrics
    performance_collector: Option<Arc<crate::monitoring::metrics::performance::PerformanceCollectorAdapter>>,
}

impl ToolMetricsCollector {
    /// Creates a new tool metrics collector
    ///
    /// # Returns
    /// A new collector instance with an empty metrics collection
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config: ToolMetricsConfig::default(),
            performance_collector: None,
        }
    }

    /// Creates a new tool metrics collector with configuration
    ///
    /// # Returns
    /// A new collector instance with the specified configuration
    #[must_use] pub fn with_config(config: ToolMetricsConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            performance_collector: None,
        }
    }

    /// Creates a new tool metrics collector with dependencies
    ///
    /// # Returns
    /// A new collector instance with the specified dependencies
    #[must_use] pub fn with_dependencies(
        config: ToolMetricsConfig,
        performance_collector: Option<Arc<crate::monitoring::metrics::performance::PerformanceCollectorAdapter>>,
    ) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            performance_collector,
        }
    }

    /// Retrieves metrics for a specific tool
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool to get metrics for
    ///
    /// # Returns
    /// The metrics for the specified tool, or None if no metrics exist
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    pub async fn get_tool_metrics(&self, tool_name: &str) -> Result<Option<ToolMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(tool_name).cloned())
    }

    /// Retrieves metrics for all tracked tools
    ///
    /// # Returns
    /// A hashmap containing tool names mapped to their metric data
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    pub async fn get_all_metrics(&self) -> Result<HashMap<String, ToolMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Records a tool usage event
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool being used
    /// * `duration` - The execution duration in milliseconds
    /// * `success` - Whether the tool execution was successful
    ///
    /// # Returns
    /// A Result indicating success or failure of recording the metrics
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    pub async fn record_tool_usage(&self, tool_name: &str, duration: f64, success: bool) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let tool_metrics = metrics.entry(tool_name.to_string()).or_insert_with(|| ToolMetrics::new(tool_name.to_string()));
        tool_metrics.record_usage(duration, success);

        // Record performance metrics if available
        if let Some(perf_collector) = &self.performance_collector {
            perf_collector.record_operation_duration(tool_name, duration).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
/// Implementation of the MetricCollector trait for tool metrics
impl MetricCollector for ToolMetricsCollector {
    /// Starts the tool metrics collector
    ///
    /// # Returns
    /// A Result indicating success of the start operation
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Stops the tool metrics collector
    ///
    /// # Returns
    /// A Result indicating success of the stop operation
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    /// Collects all tool metrics and converts them to the standard Metric format
    ///
    /// This method generates multiple metrics for each tool:
    /// - Usage count
    /// - Success count
    /// - Failure count
    /// - Average duration
    /// - Success rate
    ///
    /// # Returns
    /// A vector of standardized metrics with tool-specific labels
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        let mut result = Vec::new();

        for (tool_name, tool_metrics) in metrics.iter() {
            // Tool usage count
            let mut labels = HashMap::new();
            labels.insert("tool".to_string(), tool_name.clone());
            
            result.push(Metric::new(
                "tool.usage_count".to_string(),
                tool_metrics.usage_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool success count
            result.push(Metric::new(
                "tool.success_count".to_string(),
                tool_metrics.success_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool failure count
            result.push(Metric::new(
                "tool.failure_count".to_string(),
                tool_metrics.failure_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool average duration
            result.push(Metric::new(
                "tool.average_duration".to_string(),
                tool_metrics.average_duration,
                MetricType::Gauge,
                Some(labels.clone()),
            ));
            
            // Tool success rate
            if tool_metrics.usage_count > 0 {
                result.push(Metric::new(
                    "tool.success_rate".to_string(),
                    tool_metrics.success_rate(),
                    MetricType::Gauge,
                    Some(labels),
                ));
            }
        }
        
        Ok(result)
    }

    /// Records a specific metric in the system
    ///
    /// This implementation doesn't use the provided metric as tool metrics
    /// are recorded through the dedicated methods.
    ///
    /// # Arguments
    /// * `_metric` - The metric to record (unused in this implementation)
    ///
    /// # Returns
    /// A Result indicating success of the operation
    ///
    /// # Errors
    /// This function doesn't produce errors currently, but returns a Result for API consistency
    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        // Not implemented for tool metrics collector
        Ok(())
    }
}

/// Default implementation for ToolMetricsCollector
///
/// Creates a new collector with an empty metrics collection
impl Default for ToolMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating and managing tool metrics collector instances
#[derive(Debug, Clone)]
pub struct ToolMetricsCollectorFactory {
    /// Configuration for creating collectors
    config: ToolMetricsConfig,
}

impl ToolMetricsCollectorFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ToolMetricsConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: ToolMetricsConfig) -> Self {
        Self { config }
    }

    /// Creates a tool metrics collector
    #[must_use]
    pub fn create_collector(&self) -> Arc<ToolMetricsCollector> {
        Arc::new(ToolMetricsCollector::with_config(self.config.clone()))
    }

    /// Creates a new collector instance with dependency injection
    ///
    /// # Arguments
    /// * `performance_collector` - Optional performance metrics collector adapter
    #[must_use]
    pub fn create_collector_with_dependencies(
        &self,
        performance_collector: Option<Arc<crate::monitoring::metrics::performance::PerformanceCollectorAdapter>>,
    ) -> Arc<ToolMetricsCollector> {
        Arc::new(ToolMetricsCollector::with_dependencies(
            self.config.clone(),
            performance_collector,
        ))
    }
}

impl Default for ToolMetricsCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metrics_collector() {
        let collector = ToolMetricsCollector::new();
        
        // Record tool usage
        collector.record_tool_usage("test_tool", 0.5, true).await.unwrap();
        collector.record_tool_usage("test_tool", 1.5, true).await.unwrap();
        collector.record_tool_usage("test_tool", 0.8, false).await.unwrap();
        
        // Get tool metrics
        let metrics = collector.get_tool_metrics("test_tool").await.unwrap().unwrap();
        
        // Verify metrics
        assert_eq!(metrics.name, "test_tool");
        assert_eq!(metrics.usage_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.failure_count, 1);
        
        // Get all metrics
        let all_metrics = collector.get_all_metrics().await.unwrap();
        assert_eq!(all_metrics.len(), 1);
        
        // Collect standardized metrics
        let standard_metrics = collector.collect_metrics().await.unwrap();
        assert!(!standard_metrics.is_empty());
    }
    
    #[tokio::test]
    async fn test_tool_metrics_factory() {
        let factory = ToolMetricsCollectorFactory::new();
        let collector = factory.create_collector();
        
        // Record usage
        collector.record_tool_usage("factory_test", 1.0, true).await.unwrap();
        
        // Verify metrics
        let metrics = collector.get_tool_metrics("factory_test").await.unwrap().unwrap();
        assert_eq!(metrics.name, "factory_test");
        assert_eq!(metrics.usage_count, 1);
        
        // Test adapter
        let adapter = create_collector_adapter_with_collector(collector.clone());
        let adapter_metrics = adapter.get_tool_metrics("factory_test").await.unwrap().unwrap();
        assert_eq!(adapter_metrics.name, "factory_test");
    }
} 