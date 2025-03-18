//! Protocol metrics collector module
//!
//! This module provides functionality for collecting and monitoring protocol metrics.
//! It tracks MCP (Machine Context Protocol) usage, performance, and error statistics.

use crate::error::{SquirrelError, SquirrelResult};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use log;

/// Machine Context Protocol metrics
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
            message_latency: Duration::from_millis(0),
            error_count: 0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

impl McpMetrics {
    /// Check if the protocol is active
    #[must_use] pub fn is_active(&self) -> bool {
        self.active_connections > 0
    }
}

/// Protocol metric errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Metrics error: {0}")]
    Metrics(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// Protocol result type
type Result<T> = std::result::Result<T, ProtocolError>;

/// Super metric type for protocol measurements
pub type SuperMetric = crate::monitoring::metrics::Metric;

/// Protocol metrics collector
#[derive(Debug)]
pub struct ProtocolMetricsCollector {
    metrics: Arc<RwLock<Vec<SuperMetric>>>,
    resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
}

impl ProtocolMetricsCollector {
    /// Create a new protocol metrics collector
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            resource_collector: None,
        }
    }

    /// Create a new collector with dependencies
    #[must_use] pub fn with_dependencies(
        resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
    ) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            resource_collector,
        }
    }

    /// Collect metrics from various sources
    pub async fn collect_metrics(&self) -> Result<()> {
        let mut metrics_vec = Vec::new();
        
        // Collect resource metrics if available
        if let Some(resource_collector) = &self.resource_collector {
            match resource_collector.get_team_metrics().await {
                Ok(team_metrics) => {
                    // Process team metrics
                    for (team, metrics) in team_metrics {
                        metrics_vec.push(SuperMetric {
                            name: format!("team.{}.cpu_usage", team),
                            value: metrics.cpu_usage as f64,
                            metric_type: crate::monitoring::metrics::MetricType::Gauge,
                            labels: vec![("team".to_string(), team.clone())],
                            timestamp: chrono::Utc::now(),
                        });
                        
                        metrics_vec.push(SuperMetric {
                            name: format!("team.{}.memory_usage", team),
                            value: metrics.memory_usage as f64,
                            metric_type: crate::monitoring::metrics::MetricType::Gauge,
                            labels: vec![("team".to_string(), team.clone())],
                            timestamp: chrono::Utc::now(),
                        });
                        
                        // Additional team metrics
                        metrics_vec.push(SuperMetric {
                            name: format!("team.{}.disk_usage", team),
                            value: metrics.disk_usage as f64,
                            metric_type: crate::monitoring::metrics::MetricType::Gauge,
                            labels: vec![("team".to_string(), team.clone())],
                            timestamp: chrono::Utc::now(),
                        });
                        
                        // Add network metrics
                        metrics_vec.push(SuperMetric {
                            name: format!("team.{}.network_in", team),
                            value: metrics.network_in as f64,
                            metric_type: crate::monitoring::metrics::MetricType::Counter,
                            labels: vec![("team".to_string(), team.clone())],
                            timestamp: chrono::Utc::now(),
                        });
                        
                        metrics_vec.push(SuperMetric {
                            name: format!("team.{}.network_out", team),
                            value: metrics.network_out as f64,
                            metric_type: crate::monitoring::metrics::MetricType::Counter,
                            labels: vec![("team".to_string(), team.clone())],
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get team metrics: {}", e);
                }
            }
        }
        
        // Update the metrics
        let mut metrics_lock = self.metrics.write().await;
        *metrics_lock = metrics_vec;
        
        Ok(())
    }
}

impl super::MetricCollector for ProtocolMetricsCollector {
    /// Get all protocol metrics
    async fn get_metrics(&self) -> Result<Vec<SuperMetric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// Record a set of metrics
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

/// Protocol metrics configuration
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

/// Factory for creating protocol metrics collectors
#[derive(Debug, Clone)]
pub struct ProtocolMetricsCollectorFactory {
    /// Configuration for creating collectors
    config: ProtocolConfig,
}

impl ProtocolMetricsCollectorFactory {
    /// Create a new factory with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: ProtocolConfig::default(),
        }
    }
    
    /// Create a new factory with specific configuration
    #[must_use] pub const fn with_config(config: ProtocolConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Create a new collector without dependencies
    #[must_use] pub fn create_collector(&self) -> Arc<ProtocolMetricsCollector> {
        Arc::new(ProtocolMetricsCollector::new())
    }
    
    /// Create a new collector with dependencies
    #[must_use] pub fn create_collector_with_dependencies(
        &self,
        resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
    ) -> Arc<ProtocolMetricsCollector> {
        Arc::new(ProtocolMetricsCollector::with_dependencies(resource_collector))
    }
}

impl Default for ProtocolMetricsCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol metrics collector adapter
#[derive(Debug)]
pub struct ProtocolMetricsCollectorAdapter {
    inner: Option<Arc<ProtocolMetricsCollector>>,
}

impl ProtocolMetricsCollectorAdapter {
    /// Create a new adapter
    #[must_use] pub fn new() -> Self {
        Self {
            inner: None,
        }
    }
    
    /// Check if the adapter is valid
    #[must_use] pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }
}

impl super::MetricCollector for ProtocolMetricsCollectorAdapter {
    async fn get_metrics(&self) -> Result<Vec<SuperMetric>> {
        if let Some(collector) = &self.inner {
            collector.get_metrics().await
        } else {
            // Return empty metrics if no collector
            Err(ProtocolError::Other("No protocol metrics collector available".to_string()))
        }
    }
    
    async fn record_metrics(&self, metrics: &[SuperMetric]) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metrics(metrics).await
        } else {
            // Can't record metrics
            Err(ProtocolError::Other("No protocol metrics collector available".to_string()))
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

impl Default for ProtocolMetricsCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a protocol metrics collector with dependencies
#[must_use] pub fn create_collector_di(
    config: Option<ProtocolConfig>,
    resource_collector: Option<Arc<crate::monitoring::metrics::resource::ResourceMetricsCollectorAdapter>>,
) -> Arc<ProtocolMetricsCollector> {
    let factory = match config {
        Some(cfg) => ProtocolMetricsCollectorFactory::with_config(cfg),
        None => ProtocolMetricsCollectorFactory::new(),
    };
    
    factory.create_collector_with_dependencies(resource_collector)
}

/// Create a protocol metrics collector adapter
#[must_use] pub fn create_collector_adapter() -> Arc<ProtocolMetricsCollectorAdapter> {
    Arc::new(ProtocolMetricsCollectorAdapter::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::resource::{ResourceMetricsCollectorAdapter, create_collector_adapter as create_resource_adapter};
    
    async fn create_test_resource_adapter() -> Arc<ResourceMetricsCollectorAdapter> {
        let adapter = create_resource_adapter();
        // Register a team for testing
        let _ = adapter.register_team("test_team").await;
        adapter
    }

    #[tokio::test]
    async fn test_protocol_collector_di() {
        // Create with dependencies
        let resource_adapter = create_test_resource_adapter().await;
        let collector = ProtocolMetricsCollector::with_dependencies(Some(resource_adapter));
        
        // Test metrics collection
        collector.collect_metrics().await.unwrap();
        
        // Test metrics retrieval
        let metrics = collector.get_metrics().await.unwrap();
        assert!(!metrics.is_empty(), "Metrics should not be empty");
        
        // Check metrics for team
        let team_metrics = metrics.iter().filter(|m| m.name.starts_with("team.")).collect::<Vec<_>>();
        assert!(!team_metrics.is_empty(), "Team metrics should exist");
    }
    
    #[tokio::test]
    async fn test_protocol_collector_adapter() {
        // Create collector
        let collector = Arc::new(ProtocolMetricsCollector::new());
        
        // Create test metrics
        let test_metrics = vec![
            SuperMetric {
                name: "test.metric".to_string(),
                value: 42.0,
                metric_type: crate::monitoring::metrics::MetricType::Gauge,
                labels: vec![("test".to_string(), "value".to_string())],
                timestamp: chrono::Utc::now(),
            }
        ];
        
        // Record metrics
        collector.record_metrics(&test_metrics).await.unwrap();
        
        // Create adapter
        let adapter = ProtocolMetricsCollectorAdapter {
            inner: Some(collector),
        };
        
        // Get metrics through adapter
        let metrics = adapter.get_metrics().await.unwrap();
        assert_eq!(metrics.len(), 1, "Should have one metric");
        assert_eq!(metrics[0].name, "test.metric", "Metric name should match");
        assert_eq!(metrics[0].value, 42.0, "Metric value should match");
    }
    
    #[tokio::test]
    async fn test_protocol_collector_factory() {
        // Create factory
        let factory = ProtocolMetricsCollectorFactory::new();
        
        // Create collector
        let collector = factory.create_collector();
        assert!(Arc::strong_count(&collector) > 0, "Collector should have references");
        
        // Create with dependencies
        let resource_adapter = create_test_resource_adapter().await;
        let collector_with_deps = factory.create_collector_with_dependencies(Some(resource_adapter));
        assert!(Arc::strong_count(&collector_with_deps) > 0, "Collector should have references");
    }
    
    #[tokio::test]
    async fn test_background_collection() {
        // Create collector with dependencies
        let resource_adapter = create_test_resource_adapter().await;
        let collector = Arc::new(ProtocolMetricsCollector::with_dependencies(Some(resource_adapter)));
        
        // Run collection in the background
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..3 {
                collector_clone.collect_metrics().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        
        // Wait for collection to complete
        handle.await.unwrap();
        
        // Verify metrics were collected
        let metrics = collector.get_metrics().await.unwrap();
        assert!(!metrics.is_empty(), "Metrics should not be empty after collection");
    }
    
    #[tokio::test]
    async fn test_resource_metrics_integration() {
        // Create with resource metrics
        let resource_adapter = create_test_resource_adapter().await;
        let collector = Arc::new(ProtocolMetricsCollector::with_dependencies(Some(resource_adapter.clone())));
        
        // Register a team in resource metrics
        let _ = resource_adapter.register_team("test_integration").await;
        
        // Collect metrics (which should include resource metrics)
        collector.collect_metrics().await.unwrap();
        
        // Check that we have team metrics
        let metrics = collector.get_metrics().await.unwrap();
        let team_metrics = metrics.iter()
            .filter(|m| m.name.contains("test_integration"))
            .collect::<Vec<_>>();
            
        assert!(!team_metrics.is_empty(), "Should have metrics for the test team");
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // Create adapter with no collector
        let adapter = ProtocolMetricsCollectorAdapter::new();
        
        // Getting metrics should error
        let result = adapter.get_metrics().await;
        assert!(result.is_err(), "Getting metrics should fail when no collector is set");
        
        // Recording metrics should error
        let test_metrics = vec![
            SuperMetric {
                name: "test.error".to_string(),
                value: 1.0,
                metric_type: crate::monitoring::metrics::MetricType::Gauge,
                labels: vec![],
                timestamp: chrono::Utc::now(),
            }
        ];
        
        let result = adapter.record_metrics(&test_metrics).await;
        assert!(result.is_err(), "Recording metrics should fail when no collector is set");
    }
} 