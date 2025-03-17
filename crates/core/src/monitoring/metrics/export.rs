//! Metric export functionality for monitoring system
//! 
//! Supports exporting metrics to:
//! - Prometheus
//! - Custom formats via trait implementation

use std::fmt::Debug;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json;
use tokio::sync::RwLock;
use prometheus::Registry;

use crate::error::Result;
use crate::monitoring::metrics::Metric;

/// Export configuration for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format (e.g., "prometheus", "custom")
    pub format: String,
    /// Export endpoint URL
    pub endpoint: String,
    /// Export interval in seconds
    pub interval: u64,
    /// Optional authentication token
    pub auth_token: Option<String>,
    /// Additional format-specific options
    pub options: serde_json::Value,
    pub exporters: Vec<String>,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct ExporterConfig {
    pub format: String,
    pub endpoint: String,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            format: "prometheus".to_string(),
            endpoint: "http://localhost:9090".to_string(),
        }
    }
}

pub trait MetricExporter: Debug + Send + Sync {
    /// Export metrics to the configured endpoint
    fn export(&self, metrics: Vec<Metric>) -> Box<dyn Future<Output = Result<()>> + Send + '_>;
    
    /// Get the name of the exporter
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct DefaultMetricExporter {
    config: ExportConfig,
    metrics: Arc<RwLock<Vec<Metric>>>,
}

impl DefaultMetricExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn export_metrics(&self, metrics: Vec<Metric>) -> Result<()> {
        let mut current_metrics = self.metrics.write().await;
        current_metrics.extend(metrics);
        
        // Enforce the maximum metrics limit from config
        if current_metrics.len() > self.config.batch_size {
            let overflow = current_metrics.len() - self.config.batch_size;
            // Remove oldest metrics
            current_metrics.drain(0..overflow);
        }
        
        Ok(())
    }

    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// Get the configured export endpoint
    pub fn get_endpoint(&self) -> &str {
        &self.config.endpoint
    }
    
    /// Get the configured export format
    pub fn get_format(&self) -> &str {
        &self.config.format
    }
}

impl MetricExporter for DefaultMetricExporter {
    fn export(&self, metrics: Vec<Metric>) -> Box<dyn Future<Output = Result<()>> + Send + '_> {
        Box::new(async move {
            tracing::info!("Exporting {} metrics to {}", metrics.len(), self.config.endpoint);
            self.export_metrics(metrics).await
        })
    }

    fn name(&self) -> &'static str {
        "default"
    }
}

pub fn create_exporter(config: &ExporterConfig) -> Result<Arc<dyn MetricExporter + Send + Sync>> {
    let exporter: Arc<dyn MetricExporter + Send + Sync> = match config.format.as_str() {
        "prometheus" => Arc::new(PrometheusExporter::new(ExportConfig {
            format: config.format.clone(),
            endpoint: config.endpoint.clone(),
            interval: 60,
            auth_token: None,
            options: serde_json::json!({}),
            exporters: vec![],
            batch_size: 100,
        })),
        _ => Arc::new(DefaultMetricExporter::new(ExportConfig {
            format: config.format.clone(),
            endpoint: config.endpoint.clone(),
            interval: 60,
            auth_token: None,
            options: serde_json::json!({}),
            exporters: vec![],
            batch_size: 100,
        })),
    };
    
    Ok(exporter)
}

/// Default exporter name
#[derive(Debug)]
pub struct PrometheusExporter {
    #[allow(dead_code)]
    name: String,
    config: ExportConfig,
    #[allow(dead_code)]
    registry: Registry,
    #[allow(dead_code)]
    metrics: Arc<RwLock<HashMap<String, String>>>,
}

impl PrometheusExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self {
            name: "prometheus".to_string(),
            config,
            registry: Registry::new(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get the configured endpoint
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }
}

impl MetricExporter for PrometheusExporter {
    fn export(&self, metrics: Vec<Metric>) -> Box<dyn Future<Output = Result<()>> + Send + '_> {
        Box::new(async move {
            tracing::info!("Exporting {} metrics via prometheus to {}", 
                metrics.len(), self.config.endpoint);
            
            // In a real implementation, we would format metrics for Prometheus
            // and send them to the configured endpoint
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "prometheus"
    }
}

// Module state
static EXPORTER: tokio::sync::OnceCell<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> = tokio::sync::OnceCell::const_new();

/// Initialize the metric export system
pub async fn initialize_exporter(config: ExportConfig) -> Result<()> {
    let exporter = create_exporter(&ExporterConfig {
        format: config.format.clone(),
        endpoint: config.endpoint.clone(),
    })?;
    EXPORTER.get_or_init(|| async move {
        Arc::new(RwLock::new(exporter))
    }).await;
    Ok(())
}

/// Export metrics using the configured exporter
pub async fn export_metrics(exporter: &dyn MetricExporter, metrics: &[Metric]) -> Result<()> {
    let export_future = exporter.export(metrics.to_vec());
    Pin::from(export_future).await
}

pub async fn initialize_exporters(config: ExportConfig) -> Result<()> {
    for exporter_type in config.exporters.clone() {
        match exporter_type.as_str() {
            "prometheus" => {
                let port = config.options.get("port")
                    .and_then(serde_json::Value::as_u64)
                    .map_or(9090, |v| u16::try_from(v).unwrap_or(9090));
                let _exporter = PrometheusExporter::new(ExportConfig {
                    format: "prometheus".to_string(),
                    endpoint: format!("http://localhost:{port}"),
                    interval: config.interval,
                    auth_token: config.auth_token.clone(),
                    options: config.options.clone(),
                    exporters: vec![],
                    batch_size: config.batch_size,
                });
                initialize_exporter(ExportConfig {
                    format: "prometheus".to_string(),
                    endpoint: format!("http://localhost:{port}"),
                    interval: config.interval,
                    auth_token: config.auth_token.clone(),
                    options: config.options.clone(),
                    exporters: vec![],
                    batch_size: config.batch_size,
                }).await?;
            }
            _ => return Err(crate::error::SquirrelError::metric(
                &format!("Unknown exporter type: {exporter_type}")
            )),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::{Metric, MetricType};

    #[derive(Debug, Clone)]
    struct JsonExportConfig {
        path: String,
        pretty: bool,
    }

    impl Default for JsonExportConfig {
        fn default() -> Self {
            Self {
                path: "metrics.json".to_string(),
                pretty: true,
            }
        }
    }

    #[derive(Debug)]
    struct JsonMetricExporter {
        config: JsonExportConfig,
        name: String,
    }

    impl JsonMetricExporter {
        fn new(config: JsonExportConfig) -> Self {
            Self {
                config,
                name: "json".to_string(),
            }
        }

        async fn export(&self, metrics: Vec<Metric>) -> Result<()> {
            // In a real implementation, we would write metrics to a file
            // For testing purposes, we just log the configuration
            tracing::info!(
                "Exporting {} metrics to file: {}, pretty: {}", 
                metrics.len(), 
                self.config.path, 
                self.config.pretty
            );
            Ok(())
        }
        
        fn get_config(&self) -> &JsonExportConfig {
            &self.config
        }
    }

    impl MetricExporter for JsonMetricExporter {
        fn export(&self, metrics: Vec<Metric>) -> Box<dyn Future<Output = Result<()>> + Send + '_> {
            Box::new(Box::pin(self.export(metrics)))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_json_exporter() {
        let config = JsonExportConfig {
            path: "test_metrics.json".to_string(),
            pretty: true,
        };
        
        let exporter = JsonMetricExporter::new(config);
        let metrics = vec![
            Metric::new("test_metric".to_string(), 1.0, MetricType::Gauge, None),
            Metric::new("test_counter".to_string(), 10.0, MetricType::Counter, None),
        ];
        
        // Export metrics using the exporter trait method
        exporter.export(metrics).await.unwrap();
        
        // Verify exporter name
        assert_eq!(exporter.name(), "json", "Exporter name should match");
        
        // Clean up test file if it exists (our mock implementation doesn't actually create it)
        std::fs::remove_file("test_metrics.json").ok();
    }
} 