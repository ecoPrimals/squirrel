use std::sync::Arc;
use async_trait::async_trait;
use squirrel_core::error::Result;
use crate::metrics::{Metric, MetricCollector};
use super::{PerformanceCollector, OperationType};
use std::time::{Duration, Instant};

/// Adapter for the Performance Collector to support dependency injection
#[derive(Debug)]
pub struct PerformanceCollectorAdapter {
    /// The inner collector instance
    inner: Option<Arc<PerformanceCollector>>,
}

impl Default for PerformanceCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceCollectorAdapter {
    /// Create a new adapter with no inner collector
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }

    /// Create a new adapter with a specific collector
    #[must_use] pub fn with_collector(collector: Arc<PerformanceCollector>) -> Self {
        Self {
            inner: Some(collector),
        }
    }

    /// Check if the adapter has a valid inner collector
    #[must_use] pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// Time an operation using the collector
    pub async fn time_operation<F, T>(&self, op_type: OperationType, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        if let Some(collector) = &self.inner {
            let _ = collector.record_operation(&op_type, duration).await;
        }

        result
    }

    /// Record an operation with a specific duration
    pub async fn record_operation(&self, op_type: &OperationType, duration: Duration) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_operation(op_type, duration).await
        } else {
            Ok(()) // Silently ignore if no collector is configured
        }
    }

    /// Get metrics for all operations
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            collector.get_metrics().await
        } else {
            Ok(Vec::new()) // Return empty metrics if no collector is configured
        }
    }
}

impl Clone for PerformanceCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
impl MetricCollector for PerformanceCollectorAdapter {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        self.get_metrics().await
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metric(metric).await
        } else {
            Ok(()) // Silently ignore if no collector is configured
        }
    }

    async fn start(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.start().await
        } else {
            Ok(()) // Silently ignore if no collector is configured
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.stop().await
        } else {
            Ok(()) // Silently ignore if no collector is configured
        }
    }
}

/// Create a new performance collector adapter
#[must_use] pub fn create_collector_adapter() -> Arc<PerformanceCollectorAdapter> {
    Arc::new(PerformanceCollectorAdapter::new())
}

/// Create a new performance collector adapter with a specific collector
#[must_use] pub fn create_collector_adapter_with_collector(collector: Arc<PerformanceCollector>) -> Arc<PerformanceCollectorAdapter> {
    Arc::new(PerformanceCollectorAdapter::with_collector(collector))
} 