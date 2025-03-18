use std::sync::Arc;
use crate::error::Result;
use crate::monitoring::metrics::{Metric, MetricCollector};
use super::{ResourceMetricsCollector, ensure_factory};
use async_trait::async_trait;

/// Adapter for the resource metrics collector to support dependency injection
#[derive(Debug)]
pub struct ResourceMetricsCollectorAdapter {
    inner: Option<Arc<ResourceMetricsCollector>>,
}

impl ResourceMetricsCollectorAdapter {
    /// Creates a new resource metrics collector adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing collector
    #[must_use]
    pub fn with_collector(collector: Arc<ResourceMetricsCollector>) -> Self {
        Self {
            inner: Some(collector),
        }
    }

    /// Gets the team metrics for a specific team
    pub async fn get_team_metrics(&self, team_name: &str) -> Option<super::TeamResourceMetrics> {
        if let Some(collector) = &self.inner {
            collector.get_team_metrics(team_name).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.get_team_metrics(team_name).await,
                Err(_) => None,
            }
        }
    }

    /// Registers a new team for resource tracking
    pub async fn register_team(&self, team_name: String, workspace_path: std::path::PathBuf) {
        if let Some(collector) = &self.inner {
            collector.register_team(team_name, workspace_path).await;
        } else {
            // Try to initialize on-demand
            if let Ok(collector) = ensure_factory().get_global_collector().await {
                collector.register_team(team_name, workspace_path).await;
            }
        }
    }
}

#[async_trait]
impl MetricCollector for ResourceMetricsCollectorAdapter {
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
}

impl Clone for ResourceMetricsCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for ResourceMetricsCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new resource metrics collector adapter
#[must_use]
pub fn create_collector_adapter() -> ResourceMetricsCollectorAdapter {
    ResourceMetricsCollectorAdapter::new()
}

/// Creates a new resource metrics collector adapter with an existing collector
#[must_use]
pub fn create_collector_adapter_with_collector(collector: Arc<ResourceMetricsCollector>) -> ResourceMetricsCollectorAdapter {
    ResourceMetricsCollectorAdapter::with_collector(collector)
} 