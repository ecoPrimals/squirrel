use std::sync::Arc;
use std::collections::HashMap;
use crate::error::Result;
use crate::monitoring::metrics::{Metric, MetricCollector};
use super::{ToolMetricsCollector, ToolMetrics, ensure_factory};
use async_trait::async_trait;

/// Adapter for the tool metrics collector to support dependency injection
#[derive(Debug)]
pub struct ToolMetricsCollectorAdapter {
    inner: Option<Arc<ToolMetricsCollector>>,
}

impl ToolMetricsCollectorAdapter {
    /// Creates a new tool metrics collector adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing collector
    #[must_use]
    pub fn with_collector(collector: Arc<ToolMetricsCollector>) -> Self {
        Self {
            inner: Some(collector),
        }
    }

    /// Retrieves metrics for a specific tool
    pub async fn get_tool_metrics(&self, tool_name: &str) -> Result<Option<ToolMetrics>> {
        if let Some(collector) = &self.inner {
            collector.get_tool_metrics(tool_name).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.get_tool_metrics(tool_name).await,
                Err(e) => Err(e),
            }
        }
    }

    /// Retrieves metrics for all tracked tools
    pub async fn get_all_metrics(&self) -> Result<HashMap<String, ToolMetrics>> {
        if let Some(collector) = &self.inner {
            collector.get_all_metrics().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.get_all_metrics().await,
                Err(e) => Err(e),
            }
        }
    }

    /// Records a tool usage event
    pub async fn record_tool_usage(&self, tool_name: &str, duration: f64, success: bool) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_tool_usage(tool_name, duration, success).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.record_tool_usage(tool_name, duration, success).await,
                Err(e) => Err(e),
            }
        }
    }
}

#[async_trait]
impl MetricCollector for ToolMetricsCollectorAdapter {
    async fn start(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.start().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.start().await,
                Err(e) => Err(e),
            }
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.stop().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.stop().await,
                Err(e) => Err(e),
            }
        }
    }

    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            collector.collect_metrics().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.collect_metrics().await,
                Err(e) => Err(e),
            }
        }
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metric(metric).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.record_metric(metric).await,
                Err(e) => Err(e),
            }
        }
    }
}

impl Clone for ToolMetricsCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for ToolMetricsCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new tool metrics collector adapter
#[must_use]
pub fn create_collector_adapter() -> Arc<ToolMetricsCollectorAdapter> {
    Arc::new(ToolMetricsCollectorAdapter::new())
}

/// Creates a new tool metrics collector adapter with an existing collector
#[must_use]
pub fn create_collector_adapter_with_collector(collector: Arc<ToolMetricsCollector>) -> Arc<ToolMetricsCollectorAdapter> {
    Arc::new(ToolMetricsCollectorAdapter::with_collector(collector))
} 