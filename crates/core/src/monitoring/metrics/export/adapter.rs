use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use crate::error::Result;
use crate::monitoring::metrics::Metric;
use super::MetricExporter;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Adapter for the metric exporter to support dependency injection
#[derive(Debug)]
pub struct MetricExporterAdapter {
    inner: Option<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>>,
}

impl MetricExporterAdapter {
    /// Creates a new metric exporter adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing exporter
    #[must_use]
    pub fn with_exporter(exporter: Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>) -> Self {
        Self {
            inner: Some(exporter),
        }
    }

    /// Exports metrics using the underlying exporter
    ///
    /// # Errors
    /// Returns an error if the exporter is not initialized via dependency injection
    pub async fn export_metrics(&self, metrics: Vec<Metric>) -> Result<()> {
        if let Some(exporter_lock) = &self.inner {
            let exporter = exporter_lock.read().await;
            Pin::from(exporter.export(metrics)).await
        } else {
            Err(format!("Metric exporter not initialized via dependency injection").into())
        }
    }

    /// Gets the name of the underlying exporter
    ///
    /// # Errors
    /// Returns an error if the exporter is not initialized via dependency injection
    pub async fn get_name(&self) -> Result<String> {
        if let Some(exporter_lock) = &self.inner {
            let exporter = exporter_lock.read().await;
            Ok(exporter.name().to_string())
        } else {
            Err(format!("Metric exporter not initialized via dependency injection").into())
        }
    }
}

impl Clone for MetricExporterAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for MetricExporterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new metric exporter adapter
#[must_use]
pub fn create_exporter_adapter() -> Arc<MetricExporterAdapter> {
    Arc::new(MetricExporterAdapter::new())
}

/// Creates a new metric exporter adapter with an existing exporter
#[must_use]
pub fn create_exporter_adapter_with_exporter(
    exporter: Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>
) -> Arc<MetricExporterAdapter> {
    Arc::new(MetricExporterAdapter::with_exporter(exporter))
} 