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

/// Tool metrics collector
#[derive(Debug)]
pub struct ToolMetricsCollector {
    /// Storage for tool metrics by tool name
    metrics: Arc<RwLock<HashMap<String, ToolMetrics>>>,
}

impl ToolMetricsCollector {
    /// Creates a new tool metrics collector
    ///
    /// # Returns
    /// A new collector instance with an empty metrics collection
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
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

// Static collector instance
#[allow(dead_code)]
static TOOL_COLLECTOR: tokio::sync::OnceCell<Arc<ToolMetricsCollector>> = tokio::sync::OnceCell::const_new();

/// Initializes the tool metrics system
///
/// This function sets up the global tool metrics collector if it hasn't been initialized yet.
///
/// # Returns
/// A Result indicating success or failure of the initialization
///
/// # Errors
/// This function doesn't produce errors currently, but returns a Result for API consistency
pub async fn initialize() -> Result<()> {
    // No initialization needed yet, but this function exists for API consistency
    Ok(())
}

/// Retrieves metrics for a specific tool from the global collector
///
/// # Arguments
/// * `tool_name` - The name of the tool to get metrics for
///
/// # Returns
/// The metrics for the specified tool, or None if no metrics exist or the collector is not initialized
pub async fn get_tool_metrics(_tool_name: &str) -> Option<ToolMetrics> {
    // Implementation would retrieve from global state
    None
}

/// Retrieves metrics for all tracked tools from the global collector
///
/// # Returns
/// A hashmap containing tool names mapped to their metric data, or None if the collector is not initialized
pub async fn get_all_metrics() -> Option<HashMap<String, ToolMetrics>> {
    // Implementation would retrieve from global state
    None
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
        
        assert_eq!(metrics.name, "test_tool");
        assert_eq!(metrics.usage_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.failure_count, 1);
        assert!(metrics.average_duration > 0.0);
        
        // Collect metrics
        let collected = collector.collect_metrics().await.unwrap();
        assert!(!collected.is_empty());
    }
} 