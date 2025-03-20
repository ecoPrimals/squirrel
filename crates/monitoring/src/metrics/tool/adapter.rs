use std::sync::Arc;
use std::collections::HashMap;
use squirrel_core::error::{Result, SquirrelError};
use crate::metrics::{Metric, MetricCollector};
use super::{ToolMetricCollector, ToolMetrics};
use async_trait::async_trait;

/// Adapter for the tool metrics collector to support dependency injection
#[derive(Debug)]
pub struct ToolMetricCollectorAdapter {
    /// The inner tool metrics collector instance
    inner: Option<Arc<ToolMetricCollector>>,
}

impl ToolMetricCollectorAdapter {
    /// Creates a new tool metrics collector adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing collector
    #[must_use]
    pub fn with_collector(collector: Arc<ToolMetricCollector>) -> Self {
        Self {
            inner: Some(collector),
        }
    }

    /// Retrieves metrics for a specific tool
    /// 
    /// # Errors
    /// Returns an error if the tool metrics collector is not available
    pub async fn get_tool_metrics(&self, tool_name: &str) -> Result<Option<ToolMetrics>> {
        if let Some(collector) = &self.inner {
            collector.get_tool_metrics(tool_name).await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }

    /// Retrieves metrics for all tracked tools
    /// 
    /// # Errors
    /// Returns an error if the tool metrics collector is not available
    pub async fn get_all_metrics(&self) -> Result<HashMap<String, ToolMetrics>> {
        if let Some(collector) = &self.inner {
            collector.get_all_metrics().await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }

    /// Records a tool usage event
    /// 
    /// # Errors
    /// Returns an error if the tool metrics collector is not available
    pub async fn record_tool_usage(&self, tool_name: &str, duration: f64, success: bool) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_tool_execution(tool_name, duration, success).await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }
}

#[async_trait]
impl MetricCollector for ToolMetricCollectorAdapter {
    async fn start(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.start().await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.stop().await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }

    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            collector.collect_metrics().await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metric(metric).await
        } else {
            // Return error when no collector is available
            Err(SquirrelError::metric("No tool metrics collector available"))
        }
    }
}

impl Clone for ToolMetricCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for ToolMetricCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new tool metrics collector adapter
#[must_use]
pub fn create_collector_adapter() -> Arc<ToolMetricCollectorAdapter> {
    Arc::new(ToolMetricCollectorAdapter::new())
}

/// Creates a new tool metrics collector adapter with an existing collector
#[must_use]
pub fn create_collector_adapter_with_collector(collector: Arc<ToolMetricCollector>) -> Arc<ToolMetricCollectorAdapter> {
    Arc::new(ToolMetricCollectorAdapter::with_collector(collector))
} 