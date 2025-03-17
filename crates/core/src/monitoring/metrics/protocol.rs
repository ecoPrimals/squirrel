use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::app::metrics::{Metric, MetricType, MetricValue};
use thiserror::Error;
use async_trait::async_trait;
use crate::error::Result;
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

// Module state - keep for backward compatibility
static PROTOCOL_COLLECTOR: tokio::sync::OnceCell<Arc<ProtocolMetricsCollector>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the protocol metrics collector
pub async fn initialize() -> Result<Arc<ProtocolMetricsCollector>> {
    let collector = Arc::new(ProtocolMetricsCollector::new());
    PROTOCOL_COLLECTOR.set(collector.clone())
        .map_err(|_| ProtocolError::Other("Protocol collector already initialized".to_string()))?;
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
        None
    }
} 