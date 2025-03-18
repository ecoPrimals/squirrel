use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::app::metrics::{Metric, MetricType, MetricValue};
use thiserror::Error;
use async_trait::async_trait;
use crate::error::{Result as SquirrelResult, SquirrelError};
use super::Metric as SuperMetric;
use std::collections::HashMap;

/// MCP Protocol metrics tracking
#[derive(Debug, Clone)]
pub struct McpMetrics {
    /// Total messages processed
    pub messages_processed: u64,
    /// Average message latency
    pub message_latency: Duration,
    /// Error count
    pub error_count: u64,
    /// Active connections
    pub active_connections: u32,
    /// Message queue depth
    pub queue_depth: u32,
}

impl Default for McpMetrics {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            message_latency: Duration::from_secs(0),
            error_count: 0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

impl McpMetrics {
    pub fn is_active(&self) -> bool {
        self.active_connections > 0
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Metrics error: {0}")]
    Metrics(String),
    #[error("Other error: {0}")]
    Other(String),
}

type Result<T> = std::result::Result<T, ProtocolError>;

/// Protocol metrics collector
#[derive(Debug)]
pub struct ProtocolMetricsCollector {
    metrics: Arc<RwLock<Vec<SuperMetric>>>,
    resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
}

impl ProtocolMetricsCollector {
    /// Create a new protocol metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            resource_collector: None,
        }
    }

    /// Create a new protocol metrics collector with dependencies
    pub fn with_dependencies(
        resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
    ) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            resource_collector,
        }
    }

    /// Collect current protocol metrics
    pub async fn collect_metrics(&self) -> Result<()> {
        let mut metrics = Vec::new();
        
        // Collect basic protocol metrics
        let mcp_metrics = McpMetrics::default(); // Replace with actual metrics collection
        
        // Add protocol-specific metrics
        metrics.push(SuperMetric::new(
            "mcp.messages_processed".to_string(),
            mcp_metrics.messages_processed as f64,
            MetricType::Counter,
            Some(HashMap::new()),
        ));
        
        metrics.push(SuperMetric::new(
            "mcp.message_latency".to_string(),
            mcp_metrics.message_latency.as_secs_f64(),
            MetricType::Gauge,
            Some(HashMap::new()),
        ));
        
        metrics.push(SuperMetric::new(
            "mcp.error_count".to_string(),
            mcp_metrics.error_count as f64,
            MetricType::Counter,
            Some(HashMap::new()),
        ));
        
        metrics.push(SuperMetric::new(
            "mcp.active_connections".to_string(),
            mcp_metrics.active_connections as f64,
            MetricType::Gauge,
            Some(HashMap::new()),
        ));
        
        metrics.push(SuperMetric::new(
            "mcp.queue_depth".to_string(),
            mcp_metrics.queue_depth as f64,
            MetricType::Gauge,
            Some(HashMap::new()),
        ));
        
        // If resource collector is available, collect and add resource metrics
        if let Some(resource_collector) = &self.resource_collector {
            if let Ok(resource_metrics) = resource_collector.get_metrics().await {
                metrics.extend(resource_metrics.into_iter().map(|m| {
                    let mut labels = m.labels.unwrap_or_default();
                    labels.insert("source".to_string(), "resource_collector".to_string());
                    SuperMetric::new(
                        format!("mcp.resource.{}", m.name),
                        m.value,
                        m.metric_type,
                        Some(labels),
                    )
                }));
            }
        }
        
        // Update stored metrics
        let mut current_metrics = self.metrics.write().await;
        *current_metrics = metrics;
        
        Ok(())
    }
}

#[async_trait]
impl super::MetricCollector for ProtocolMetricsCollector {
    async fn get_metrics(&self) -> Result<Vec<SuperMetric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    async fn record_metrics(&self, metrics: &[SuperMetric]) -> Result<()> {
        let mut current_metrics = self.metrics.write().await;
        current_metrics.extend_from_slice(metrics);
        Ok(())
    }
}

impl Clone for ProtocolMetricsCollector {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            resource_collector: self.resource_collector.clone(),
        }
    }
}

/// Configuration for protocol metrics collection
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Whether to enable protocol metrics collection
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval: u64,
    /// Maximum history size
    pub history_size: usize,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            history_size: 100,
        }
    }
}

/// Factory for creating and managing protocol metrics collector instances
#[derive(Debug, Clone)]
pub struct ProtocolMetricsCollectorFactory {
    /// Configuration for creating collectors
    config: ProtocolConfig,
}

impl ProtocolMetricsCollectorFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProtocolConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Creates a protocol metrics collector
    #[must_use]
    pub fn create_collector(&self) -> Arc<ProtocolMetricsCollector> {
        self.create_collector_with_dependencies(None)
    }

    /// Creates a new collector instance with dependency injection
    ///
    /// # Arguments
    /// * `resource_collector` - Optional resource metrics collector adapter
    #[must_use]
    pub fn create_collector_with_dependencies(
        &self,
        resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
    ) -> Arc<ProtocolMetricsCollector> {
        let collector = ProtocolMetricsCollector {
            metrics: Arc::new(RwLock::new(Vec::new())),
            resource_collector,
        };
        Arc::new(collector)
    }

    /// Gets the global collector instance, initializing it if necessary
    pub async fn get_global_collector(&self) -> Result<Arc<ProtocolMetricsCollector>> {
        if let Some(collector) = PROTOCOL_COLLECTOR.get() {
            Ok(collector.clone())
        } else {
            // Create resource collector adapter
            let resource_collector = match crate::monitoring::metrics::resource::create_collector_adapter().await {
                Ok(adapter) => Some(adapter),
                Err(_) => None,
            };

            // Create collector with adapter
            let collector = self.create_collector_with_dependencies(resource_collector);
            
            // Initialize the collector
            match PROTOCOL_COLLECTOR.set(collector.clone()) {
                Ok(_) => Ok(collector),
                Err(_) => Err(ProtocolError::Other("Failed to set global protocol collector".to_string())),
            }
        }
    }
}

impl Default for ProtocolMetricsCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory for creating protocol metrics collectors
static FACTORY: OnceLock<ProtocolMetricsCollectorFactory> = OnceLock::new();

/// Initialize the protocol metrics collector factory
///
/// # Errors
/// Returns an error if the factory is already initialized
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory::new() or ProtocolMetricsCollectorFactory::with_config() instead"
)]
pub fn initialize_factory(config: Option<ProtocolConfig>) -> SquirrelResult<()> {
    let factory = match config {
        Some(cfg) => ProtocolMetricsCollectorFactory::with_config(cfg),
        None => ProtocolMetricsCollectorFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::metric("Protocol metrics collector factory already initialized"))?;
    Ok(())
}

/// Get the protocol metrics collector factory
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory::new() or ProtocolMetricsCollectorFactory::with_config() instead"
)]
pub fn get_factory() -> Option<ProtocolMetricsCollectorFactory> {
    FACTORY.get().cloned()
}

/// Get or create the protocol metrics collector factory
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory::new() or ProtocolMetricsCollectorFactory::with_config() instead"
)]
pub fn ensure_factory() -> ProtocolMetricsCollectorFactory {
    FACTORY.get_or_init(ProtocolMetricsCollectorFactory::new).clone()
}

// Module state - keep for backward compatibility
static PROTOCOL_COLLECTOR: tokio::sync::OnceCell<Arc<ProtocolMetricsCollector>> = 
    tokio::sync::OnceCell::const_new();

/// Check if the protocol metrics collector is initialized
#[must_use]
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory instead of relying on global state"
)]
pub fn is_initialized() -> bool {
    PROTOCOL_COLLECTOR.get().is_some()
}

/// Initialize the protocol metrics collector
///
/// # Arguments
/// * `config` - Optional configuration for the collector
///
/// # Errors
/// Returns an error if the collector cannot be initialized
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory::create_collector() or create_collector_di() instead"
)]
pub async fn initialize(config: Option<ProtocolConfig>) -> Result<Arc<ProtocolMetricsCollector>> {
    // Initialize factory with config
    initialize_factory(config.clone())?;

    // Create resource collector adapter
    let resource_collector = match crate::monitoring::metrics::resource::create_collector_adapter().await {
        Ok(adapter) => {
            log::debug!("Successfully created resource collector adapter");
            Some(Arc::new(adapter))
        }
        Err(e) => {
            log::warn!("Failed to create resource collector adapter: {}", e);
            None
        }
    };

    // Get factory and create collector with dependencies
    let factory = ensure_factory();
    let collector = factory.create_collector_with_dependencies(resource_collector);

    // Initialize global collector
    match PROTOCOL_COLLECTOR.set(collector.clone()) {
        Ok(_) => {
            log::info!("Successfully initialized protocol metrics collector");
            
            // Start any necessary background tasks or initialization
            if let Some(config) = config {
                if config.enabled {
                    // Initialize metrics collection
                    let metrics = collector.clone();
                    tokio::spawn(async move {
                        let interval = tokio::time::Duration::from_secs(config.interval);
                        let mut ticker = tokio::time::interval(interval);
                        
                        loop {
                            ticker.tick().await;
                            if let Err(e) = metrics.collect_metrics().await {
                                log::error!("Failed to collect protocol metrics: {}", e);
                            }
                        }
                    });
                }
            }
            
            Ok(collector)
        }
        Err(_) => {
            log::error!("Failed to set global protocol collector");
            Err(ProtocolError::Other("Failed to set global protocol collector".to_string()))
        }
    }
}

/// Get protocol metrics collector
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorFactory::create_collector() or create_collector_di() instead"
)]
pub fn get_collector() -> Option<Arc<ProtocolMetricsCollector>> {
    PROTOCOL_COLLECTOR.get().cloned()
}

/// Get protocol metrics
#[deprecated(
    since = "0.2.0",
    note = "Use DI pattern with ProtocolMetricsCollectorAdapter::get_metrics() instead"
)]
pub async fn get_metrics() -> Option<Vec<SuperMetric>> {
    if let Some(collector) = PROTOCOL_COLLECTOR.get() {
        match collector.get_metrics().await {
            Ok(metrics) => Some(metrics),
            Err(_) => None,
        }
    } else {
        // Try to initialize on-demand
        match ensure_factory().get_global_collector().await {
            Ok(collector) => match collector.get_metrics().await {
                Ok(metrics) => Some(metrics),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
}

/// Adapter for protocol metrics collector to support
/// transition from singleton to dependency injection
#[derive(Debug)]
pub struct ProtocolMetricsCollectorAdapter {
    inner: Option<Arc<ProtocolMetricsCollector>>,
}

impl ProtocolMetricsCollectorAdapter {
    /// Create a new adapter that uses the global singleton
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: get_collector(),
        }
    }
    
    /// Check if adapter has a valid inner collector
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }
}

#[async_trait]
impl super::MetricCollector for ProtocolMetricsCollectorAdapter {
    async fn get_metrics(&self) -> Result<Vec<SuperMetric>> {
        if let Some(collector) = &self.inner {
            collector.get_metrics().await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.get_metrics().await,
                Err(_) => Err(ProtocolError::Other("Protocol metrics collector not initialized".to_string())),
            }
        }
    }

    async fn record_metrics(&self, metrics: &[SuperMetric]) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metrics(metrics).await
        } else {
            // Try to initialize on-demand
            match ensure_factory().get_global_collector().await {
                Ok(collector) => collector.record_metrics(metrics).await,
                Err(_) => Err(ProtocolError::Other("Protocol metrics collector not initialized".to_string())),
            }
        }
    }
}

impl Clone for ProtocolMetricsCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Create a protocol metrics collector using dependency injection
/// 
/// This function creates a new collector without relying on the global singleton.
/// Use this for components that have been updated to use dependency injection.
/// 
/// # Arguments
/// 
/// * `config` - Optional configuration for the collector
/// * `resource_collector` - Optional resource metrics collector adapter
#[must_use]
pub fn create_collector_di(
    config: Option<ProtocolConfig>,
    resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
) -> Arc<ProtocolMetricsCollector> {
    let factory = match config {
        Some(cfg) => ProtocolMetricsCollectorFactory::with_config(cfg),
        None => ProtocolMetricsCollectorFactory::new(),
    };
    
    factory.create_collector_with_dependencies(resource_collector)
}

/// Create an adapter for the protocol metrics collector
/// 
/// This function creates an adapter that implements the MetricCollector trait
/// but delegates to the global singleton internally. Use this for components
/// that are transitioning to dependency injection but still need to work with
/// code that uses the singleton pattern.
#[must_use]
pub fn create_collector_adapter() -> Arc<ProtocolMetricsCollectorAdapter> {
    Arc::new(ProtocolMetricsCollectorAdapter::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter;
    use std::time::Duration;

    /// Helper function to create a test resource collector adapter
    async fn create_test_resource_adapter() -> Arc<ResourceMetricsCollectorAdapter> {
        match crate::monitoring::metrics::resource::create_collector_adapter().await {
            Ok(adapter) => Arc::new(adapter),
            Err(_) => panic!("Failed to create resource collector adapter for test"),
        }
    }

    #[tokio::test]
    async fn test_protocol_collector_di() {
        // Test direct DI constructor
        let resource_adapter = create_test_resource_adapter().await;
        let collector = ProtocolMetricsCollector::with_dependencies(Some(resource_adapter));
        
        // Verify collector is properly initialized
        assert!(collector.resource_collector.is_some());
        
        // Test metrics collection
        collector.collect_metrics().await.expect("Failed to collect metrics");
        
        // Verify metrics were collected
        let metrics = collector.get_metrics().await.expect("Failed to get metrics");
        assert!(!metrics.is_empty());
        
        // Verify metric types
        let metric_names: Vec<String> = metrics.iter().map(|m| m.name.clone()).collect();
        assert!(metric_names.contains(&"mcp.messages_processed".to_string()));
        assert!(metric_names.contains(&"mcp.message_latency".to_string()));
        assert!(metric_names.contains(&"mcp.error_count".to_string()));
    }

    #[tokio::test]
    async fn test_protocol_collector_adapter() {
        // Initialize the global collector first
        let config = ProtocolConfig {
            enabled: true,
            interval: 1,
            history_size: 10,
        };
        initialize(Some(config.clone())).await.expect("Failed to initialize collector");
        
        // Create adapter
        let adapter = ProtocolMetricsCollectorAdapter::new();
        assert!(adapter.is_valid());
        
        // Test metrics collection through adapter
        let test_metrics = vec![
            SuperMetric::new(
                "test.metric".to_string(),
                42.0,
                MetricType::Counter,
                None,
            ),
        ];
        
        adapter.record_metrics(&test_metrics).await.expect("Failed to record metrics");
        
        // Verify metrics were recorded
        let metrics = adapter.get_metrics().await.expect("Failed to get metrics");
        assert!(!metrics.is_empty());
        
        // Verify test metric was included
        let test_metric = metrics.iter().find(|m| m.name == "test.metric");
        assert!(test_metric.is_some());
        assert_eq!(test_metric.unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_protocol_collector_factory() {
        // Test factory with dependencies
        let resource_adapter = create_test_resource_adapter().await;
        let factory = ProtocolMetricsCollectorFactory::new();
        let collector = factory.create_collector_with_dependencies(Some(resource_adapter));
        
        // Verify collector was created with dependencies
        assert!(collector.resource_collector.is_some());
        
        // Test collector functionality
        collector.collect_metrics().await.expect("Failed to collect metrics");
        let metrics = collector.get_metrics().await.expect("Failed to get metrics");
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_background_collection() {
        // Create collector with short interval
        let config = ProtocolConfig {
            enabled: true,
            interval: 1,
            history_size: 10,
        };
        
        let collector = initialize(Some(config)).await.expect("Failed to initialize collector");
        
        // Wait for background collection
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Verify metrics were collected
        let metrics = collector.get_metrics().await.expect("Failed to get metrics");
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_resource_metrics_integration() {
        // Create collector with resource adapter
        let resource_adapter = create_test_resource_adapter().await;
        let collector = ProtocolMetricsCollector::with_dependencies(Some(resource_adapter));
        
        // Collect metrics
        collector.collect_metrics().await.expect("Failed to collect metrics");
        
        // Verify metrics include resource metrics
        let metrics = collector.get_metrics().await.expect("Failed to get metrics");
        let resource_metrics = metrics.iter().filter(|m| m.name.contains("mcp.resource"));
        assert!(resource_metrics.count() > 0);
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test initialization with invalid config
        let result = initialize(None).await;
        assert!(result.is_ok());
        
        // Test adapter with uninitialized collector
        let adapter = ProtocolMetricsCollectorAdapter::new();
        if !adapter.is_valid() {
            let result = adapter.get_metrics().await;
            assert!(result.is_err());
        }
    }
} 