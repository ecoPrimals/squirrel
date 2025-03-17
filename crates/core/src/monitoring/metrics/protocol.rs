use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::app::metrics::{Metric, MetricType, MetricValue};
use thiserror::Error;
use async_trait::async_trait;
use crate::error::{Result as SquirrelResult, SquirrelError};
use super::Metric as SuperMetric;

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
}

impl ProtocolMetricsCollector {
    /// Create a new protocol metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
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
        Arc::new(ProtocolMetricsCollector::new())
    }

    /// Initializes and returns a global protocol metrics collector instance
    ///
    /// # Errors
    /// Returns an error if the collector is already initialized
    pub async fn initialize_global_collector(&self) -> Result<Arc<ProtocolMetricsCollector>> {
        static GLOBAL_COLLECTOR: OnceLock<Arc<ProtocolMetricsCollector>> = OnceLock::new();

        let collector = self.create_collector();
        
        match GLOBAL_COLLECTOR.set(collector.clone()) {
            Ok(()) => Ok(collector),
            Err(_) => {
                // Already initialized, return the existing instance
                Ok(GLOBAL_COLLECTOR.get()
                    .ok_or_else(|| ProtocolError::Other("Failed to get global protocol metrics collector".to_string()))?
                    .clone())
            }
        }
    }

    /// Gets the global protocol metrics collector, initializing it if necessary
    ///
    /// # Errors
    /// Returns an error if the collector cannot be initialized
    pub async fn get_global_collector(&self) -> Result<Arc<ProtocolMetricsCollector>> {
        static GLOBAL_COLLECTOR: OnceLock<Arc<ProtocolMetricsCollector>> = OnceLock::new();

        if let Some(collector) = GLOBAL_COLLECTOR.get() {
            return Ok(collector.clone());
        }

        self.initialize_global_collector().await
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
pub fn get_factory() -> Option<ProtocolMetricsCollectorFactory> {
    FACTORY.get().cloned()
}

/// Get or create the protocol metrics collector factory
#[must_use]
pub fn ensure_factory() -> ProtocolMetricsCollectorFactory {
    FACTORY.get_or_init(ProtocolMetricsCollectorFactory::new).clone()
}

// Module state - keep for backward compatibility
static PROTOCOL_COLLECTOR: tokio::sync::OnceCell<Arc<ProtocolMetricsCollector>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the protocol metrics collector
///
/// # Errors
/// Returns an error if the collector is already initialized
pub async fn initialize(config: Option<ProtocolConfig>) -> Result<Arc<ProtocolMetricsCollector>> {
    let factory = match config {
        Some(cfg) => ProtocolMetricsCollectorFactory::with_config(cfg),
        None => ensure_factory(),
    };
    
    let collector = factory.initialize_global_collector().await?;
    
    // For backward compatibility, also set in the old static
    let _ = PROTOCOL_COLLECTOR.set(collector.clone());
    
    Ok(collector)
}

/// Get protocol metrics collector
pub fn get_collector() -> Option<Arc<ProtocolMetricsCollector>> {
    PROTOCOL_COLLECTOR.get().cloned()
}

/// Get protocol metrics
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