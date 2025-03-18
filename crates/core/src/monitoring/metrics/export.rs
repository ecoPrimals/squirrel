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
use std::sync::OnceLock;
use tokio::sync::RwLock;
use crate::error::{Result, SquirrelError};
use super::Metric;
use serde::{Serialize, Deserialize};
use serde_json;
use prometheus::Registry;

pub mod adapter;
pub use adapter::{MetricExporterAdapter, create_exporter_adapter, create_exporter_adapter_with_exporter};

/// Configuration for metric export functionality.
/// 
/// This struct defines how metrics should be exported, including the format,
/// endpoint, authentication, and batching settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format (e.g., "prometheus", "custom").
    pub format: String,
    /// Export endpoint URL.
    pub endpoint: String,
    /// Export interval in seconds.
    pub interval: u64,
    /// Optional authentication token.
    pub auth_token: Option<String>,
    /// Additional format-specific options.
    pub options: serde_json::Value,
    /// List of enabled exporters.
    pub exporters: Vec<String>,
    /// Maximum number of metrics to batch together.
    pub batch_size: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: "prometheus".to_string(),
            endpoint: "http://localhost:9090/metrics".to_string(),
            interval: 60,
            auth_token: None,
            options: serde_json::Value::Null,
            exporters: vec!["prometheus".to_string()],
            batch_size: 100,
        }
    }
}

/// Configuration for a specific metric exporter.
/// 
/// This struct contains the basic configuration needed for a metric exporter,
/// including the format and endpoint.
#[derive(Debug, Clone)]
pub struct ExporterConfig {
    /// Export format (e.g., "prometheus", "custom").
    pub format: String,
    /// Export endpoint URL.
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

/// Trait for implementing metric exporters.
/// 
/// This trait defines the interface that all metric exporters must implement
/// to support exporting metrics to various destinations.
pub trait MetricExporter: Debug + Send + Sync {
    /// Exports metrics to the configured endpoint.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - List of metrics to export
    /// 
    /// # Returns
    /// 
    /// Returns a `Future` that resolves to a `Result` indicating whether the
    /// export was successful.
    fn export(&self, metrics: Vec<Metric>) -> Box<dyn Future<Output = Result<()>> + Send + '_>;
    
    /// Returns the name of the exporter.
    /// 
    /// This name is used to identify the exporter in configuration and logs.
    fn name(&self) -> &str;
}

/// Default implementation of the MetricExporter trait.
/// 
/// This exporter provides basic functionality for collecting and batching
/// metrics before they are exported.
#[derive(Debug)]
pub struct DefaultMetricExporter {
    /// Export configuration.
    config: ExportConfig,
    /// Currently collected metrics.
    metrics: Arc<RwLock<Vec<Metric>>>,
}

impl DefaultMetricExporter {
    /// Creates a new DefaultMetricExporter with the specified configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Export configuration to use
    #[must_use] pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new DefaultMetricExporter with dependencies
    ///
    /// # Arguments
    ///
    /// * `config` - Export configuration to use
    /// * `metrics` - Optional pre-existing metrics storage
    #[must_use] pub fn with_dependencies(
        config: ExportConfig,
        metrics: Option<Arc<RwLock<Vec<Metric>>>>,
    ) -> Self {
        Self {
            config,
            metrics: metrics.unwrap_or_else(|| Arc::new(RwLock::new(Vec::new()))),
        }
    }

    /// Exports a batch of metrics.
    /// 
    /// This method adds the provided metrics to the current batch and enforces
    /// the configured batch size limit.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - List of metrics to export
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` indicating whether the export was successful.
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

    /// Retrieves all currently collected metrics.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of all collected metrics.
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// Returns the configured export endpoint URL.
    #[must_use] pub fn get_endpoint(&self) -> &str {
        &self.config.endpoint
    }
    
    /// Returns the configured export format.
    #[must_use] pub fn get_format(&self) -> &str {
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

/// Creates a new metric exporter based on the provided configuration.
/// 
/// # Arguments
/// 
/// * `config` - Configuration for the exporter
/// 
/// # Returns
/// 
/// Returns a `Result` containing the created exporter wrapped in an Arc.
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

/// Prometheus-specific metric exporter implementation.
/// 
/// This exporter formats metrics in Prometheus format and exports them
/// to a Prometheus-compatible endpoint.
#[derive(Debug)]
pub struct PrometheusExporter {
    /// Name of the exporter.
    #[allow(dead_code)]
    name: String,
    /// Export configuration.
    config: ExportConfig,
    /// Prometheus registry for metric registration.
    #[allow(dead_code)]
    registry: Registry,
    /// Currently collected metrics in Prometheus format.
    #[allow(dead_code)]
    metrics: Arc<RwLock<HashMap<String, String>>>,
}

impl PrometheusExporter {
    /// Creates a new PrometheusExporter with the specified configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Export configuration to use
    #[must_use] pub fn new(config: ExportConfig) -> Self {
        Self {
            name: "prometheus".to_string(),
            config,
            registry: Registry::new(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new PrometheusExporter with dependencies
    ///
    /// # Arguments
    ///
    /// * `config` - Export configuration to use
    /// * `registry` - Optional pre-existing Prometheus registry
    /// * `metrics` - Optional pre-existing metrics storage
    #[must_use] pub fn with_dependencies(
        config: ExportConfig,
        registry: Option<Registry>,
        metrics: Option<Arc<RwLock<HashMap<String, String>>>>,
    ) -> Self {
        Self {
            name: "prometheus".to_string(),
            config,
            registry: registry.unwrap_or_else(Registry::new),
            metrics: metrics.unwrap_or_else(|| Arc::new(RwLock::new(HashMap::new()))),
        }
    }

    /// Returns the configured export endpoint URL.
    #[must_use] pub fn endpoint(&self) -> &str {
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

/// Factory for creating and managing metric exporter instances
#[derive(Debug, Clone)]
pub struct MetricExporterFactory {
    /// Configuration for creating exporters
    config: ExportConfig,
}

impl MetricExporterFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ExportConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: ExportConfig) -> Self {
        Self { config }
    }

    /// Creates a metric exporter
    ///
    /// # Errors
    /// Returns an error if exporter creation fails
    pub fn create_exporter(&self) -> Result<Arc<dyn MetricExporter + Send + Sync>> {
        create_exporter(&ExporterConfig {
            format: self.config.format.clone(),
            endpoint: self.config.endpoint.clone(),
        })
    }

    /// Creates a metric exporter with dependencies
    ///
    /// # Arguments
    ///
    /// * `registry` - Optional pre-existing Prometheus registry
    /// * `metrics` - Optional pre-existing metrics storage
    ///
    /// # Errors
    /// Returns an error if exporter creation fails
    pub fn create_exporter_with_dependencies(
        &self,
        registry: Option<Registry>,
        metrics: Option<Arc<RwLock<HashMap<String, String>>>>,
    ) -> Result<Arc<dyn MetricExporter + Send + Sync>> {
        let exporter: Arc<dyn MetricExporter + Send + Sync> = match self.config.format.as_str() {
            "prometheus" => Arc::new(PrometheusExporter::with_dependencies(
                self.config.clone(),
                registry,
                metrics,
            )),
            _ => Arc::new(DefaultMetricExporter::with_dependencies(
                self.config.clone(),
                None,
            )),
        };
        Ok(exporter)
    }

    /// Creates a metric exporter adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<MetricExporterAdapter> {
        create_exporter_adapter()
    }

    /// Creates a metric exporter adapter with an existing exporter
    #[must_use]
    pub fn create_adapter_with_exporter(
        &self,
        exporter: Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>,
    ) -> Arc<MetricExporterAdapter> {
        create_exporter_adapter_with_exporter(exporter)
    }

    /// Initializes and returns a global metric exporter instance
    ///
    /// # Errors
    /// Returns an error if the exporter is already initialized or creation fails
    pub async fn initialize_global_exporter(&self) -> Result<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> {
        static GLOBAL_EXPORTER: OnceLock<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> = OnceLock::new();

        let exporter = self.create_exporter()?;
        let exporter_lock = Arc::new(RwLock::new(exporter));
        
        match GLOBAL_EXPORTER.set(exporter_lock.clone()) {
            Ok(()) => Ok(exporter_lock),
            Err(_) => {
                // Already initialized, return the existing instance
                Ok(GLOBAL_EXPORTER.get()
                    .ok_or_else(|| SquirrelError::metric("Failed to get global metric exporter"))?
                    .clone())
            }
        }
    }

    /// Gets the global metric exporter, initializing it if necessary
    ///
    /// # Errors
    /// Returns an error if the exporter cannot be initialized
    pub async fn get_global_exporter(&self) -> Result<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> {
        static GLOBAL_EXPORTER: OnceLock<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> = OnceLock::new();

        if let Some(exporter) = GLOBAL_EXPORTER.get() {
            return Ok(exporter.clone());
        }

        self.initialize_global_exporter().await
    }
}

impl Default for MetricExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory for creating metric exporters
static FACTORY: OnceLock<MetricExporterFactory> = OnceLock::new();

/// Initialize the metric exporter factory
///
/// # Errors
/// Returns an error if the factory is already initialized
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::new() or MetricExporterFactory::with_config() instead"
)]
pub fn initialize_factory(config: Option<ExportConfig>) -> Result<()> {
    let factory = match config {
        Some(cfg) => MetricExporterFactory::with_config(cfg),
        None => MetricExporterFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::metric("Metric exporter factory already initialized"))?;
    Ok(())
}

/// Get the metric exporter factory
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::new() or MetricExporterFactory::with_config() instead"
)]
pub fn get_factory() -> Option<MetricExporterFactory> {
    FACTORY.get().cloned()
}

/// Get or create the metric exporter factory
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::new() or MetricExporterFactory::with_config() instead"
)]
pub fn ensure_factory() -> MetricExporterFactory {
    FACTORY.get_or_init(MetricExporterFactory::new).clone()
}

// Module state
static EXPORTER: tokio::sync::OnceCell<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> = tokio::sync::OnceCell::const_new();

/// Initializes a single metric exporter.
/// 
/// # Arguments
/// 
/// * `config` - Export configuration to use
/// 
/// # Returns
/// 
/// Returns a `Result` indicating whether initialization was successful.
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::create_adapter() instead"
)]
pub async fn initialize_exporter(config: ExportConfig) -> Result<()> {
    let factory = MetricExporterFactory::with_config(config.clone());
    let exporter_lock = factory.initialize_global_exporter().await?;
    
    // For backward compatibility, also set in the old static
    EXPORTER.get_or_init(|| async move { exporter_lock }).await;
    
    Ok(())
}

/// Initializes multiple metric exporters.
/// 
/// # Arguments
/// 
/// * `config` - Export configuration to use
/// 
/// # Returns
/// 
/// Returns a `Result` indicating whether initialization was successful.
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::create_adapter() instead"
)]
pub async fn initialize_exporters(config: ExportConfig) -> Result<()> {
    // For now, we only support one exporter at a time.
    // This can be extended in the future to support multiple exporters.
    initialize_exporter(config).await
}

/// Check if the exporter is initialized
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory instead of relying on global state"
)]
pub fn is_initialized() -> bool {
    EXPORTER.get().is_some()
}

/// Get the global exporter
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterFactory::create_adapter() instead"
)]
pub async fn get_exporter() -> Option<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> {
    EXPORTER.get().cloned()
}

/// Exports metrics using the specified exporter.
/// 
/// # Arguments
/// 
/// * `exporter` - The exporter to use
/// * `metrics` - List of metrics to export
/// 
/// # Returns
/// 
/// Returns a `Result` indicating whether the export was successful.
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with MetricExporterAdapter::export_metrics() instead"
)]
pub async fn export_metrics(exporter: &dyn MetricExporter, metrics: &[Metric]) -> Result<()> {
    let export_future = exporter.export(metrics.to_vec());
    Pin::from(export_future).await
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
    async fn test_metric_exporter_factory() {
        let factory = MetricExporterFactory::new();
        let exporter = factory.create_exporter().unwrap();
        
        // Test exporter creation
        assert!(Arc::strong_count(&exporter) > 0);
        
        // Test with dependencies
        let registry = Some(Registry::new());
        let metrics = Some(Arc::new(RwLock::new(HashMap::new())));
        let exporter_with_deps = factory.create_exporter_with_dependencies(registry, metrics).unwrap();
        
        assert!(Arc::strong_count(&exporter_with_deps) > 0);
    }

    #[tokio::test]
    async fn test_metric_exporter_adapter() {
        let factory = MetricExporterFactory::new();
        let adapter = factory.create_adapter();
        
        // Test adapter creation
        assert!(Arc::strong_count(&adapter) > 0);
        
        // Test with existing exporter
        let exporter = factory.create_exporter().unwrap();
        let exporter_lock = Arc::new(RwLock::new(exporter));
        let adapter_with_exporter = factory.create_adapter_with_exporter(exporter_lock);
        
        assert!(Arc::strong_count(&adapter_with_exporter) > 0);
    }
} 