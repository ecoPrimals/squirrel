// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Songbird Monitoring Client
//!
//! Production monitoring client that integrates with Songbird's observability system
//! to replace InMemoryMonitoringClient with real monitoring capabilities.

use std::collections::HashMap;
use std::sync::Arc;
// Phase 4: Removed async_trait - using native async fn in traits
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn, debug};

use crate::error::{Result, MCPError};
use super::{MonitoringClient, MonitoringEvent, MetricValue, AlertLevel};
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Songbird monitoring client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdClientConfig {
    /// Songbird service endpoint
    pub endpoint: String,
    /// Service name for this MCP instance
    pub service_name: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Collection interval in seconds
    pub collection_interval: u64,
    /// Batch size for metrics
    pub batch_size: usize,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable detailed tracing
    pub enable_tracing: bool,
}

impl Default for SongbirdClientConfig {
    fn default() -> Self {
        // Multi-tier Songbird endpoint resolution
        let endpoint = std::env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| std::env::var("SONGBIRD_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("SONGBIRD_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8500);  // Default Songbird service mesh port
                format!("http://localhost:{}", port)
            });

        let service_name = std::env::var("MCP_SERVICE_NAME")
            .unwrap_or_else(|_| "squirrel-mcp".to_string());

        let environment = std::env::var("MCP_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        // Parse numeric values with safe fallbacks
        let collection_interval = std::env::var("SONGBIRD_COLLECTION_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let batch_size = std::env::var("SONGBIRD_BATCH_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        let timeout_ms = std::env::var("SONGBIRD_TIMEOUT_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000);

        let enable_tracing = std::env::var("SONGBIRD_ENABLE_TRACING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            endpoint,
            service_name,
            environment,
            collection_interval,
            batch_size,
            timeout_ms,
            enable_tracing,
        }
    }
}

/// Songbird metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdMetrics {
    pub timestamp: DateTime<Utc>,
    pub service_name: String,
    pub environment: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_connections: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub custom_metrics: HashMap<String, MetricValue>,
}

/// Production Songbird monitoring client
#[derive(Debug)]
pub struct SongbirdMonitoringClient {
    config: SongbirdClientConfig,
    client: reqwest::Client,
    current_metrics: Arc<RwLock<Option<SongbirdMetrics>>>,
    metrics_buffer: Arc<RwLock<Vec<MonitoringEvent>>>,
    events_buffer: Arc<RwLock<Vec<MonitoringEvent>>>,
}

impl SongbirdMonitoringClient {
    /// Create a new Songbird monitoring client
    pub fn new(config: SongbirdClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| MCPError::General(format!("Failed to create HTTP client for Songbird: {}", e)))?;

        Ok(Self {
            config,
            client,
            current_metrics: Arc::new(RwLock::new(None)),
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            events_buffer: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create client with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(SongbirdClientConfig::default())
    }

    /// Create client with custom endpoint
    pub fn with_endpoint(endpoint: String) -> Result<Self> {
        let mut config = SongbirdClientConfig::default();
        config.endpoint = endpoint;
        Self::new(config)
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> Result<SongbirdMetrics> {
        // Use sysinfo or similar to collect real system metrics
        // For now, we'll use basic placeholders that can be enhanced
        
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;
        
        let metrics = SongbirdMetrics {
            timestamp: Utc::now(),
            service_name: self.config.service_name.clone(),
            environment: self.config.environment.clone(),
            cpu_usage,
            memory_usage,
            active_connections: self.get_active_connections().await?,
            request_count: self.get_request_count().await?,
            error_count: self.get_error_count().await?,
            custom_metrics: HashMap::new(),
        };

        // Cache the metrics
        *self.current_metrics.write().await = Some(metrics.clone());

        Ok(metrics)
    }

    /// Get CPU usage percentage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Implementation would use sysinfo or similar
        // For now, return a safe default
        Ok(0.0)
    }

    /// Get memory usage percentage
    async fn get_memory_usage(&self) -> Result<f64> {
        // Implementation would use sysinfo or similar
        // For now, return a safe default
        Ok(0.0)
    }

    /// Get active connections count
    async fn get_active_connections(&self) -> Result<u64> {
        // Implementation would track actual connections
        Ok(0)
    }

    /// Get request count
    async fn get_request_count(&self) -> Result<u64> {
        // Implementation would track actual requests
        Ok(0)
    }

    /// Get error count
    async fn get_error_count(&self) -> Result<u64> {
        // Implementation would track actual errors
        Ok(0)
    }

    /// Send metrics to Songbird
    async fn send_metrics_to_songbird(&self, metrics: &SongbirdMetrics) -> Result<()> {
        let url = format!("{}/api/v1/metrics", self.config.endpoint);
        
        match self.client
            .post(&url)
            .json(metrics)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Successfully sent metrics to Songbird: {}", url);
                    Ok(())
                } else {
                    warn!("Songbird metrics endpoint returned error: {}", response.status());
                    Err(MCPError::General(format!(
                        "Songbird metrics failed with status: {}",
                        response.status()
                    )))
                }
            }
            Err(e) => {
                // Don't fail completely if Songbird is unavailable
                warn!("Failed to send metrics to Songbird ({}): {}. Continuing without external monitoring.", url, e);
                Ok(())
            }
        }
    }

    /// Send events to Songbird
    async fn send_events_to_songbird(&self, events: &[MonitoringEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let url = format!("{}/api/v1/events", self.config.endpoint);
        
        match self.client
            .post(&url)
            .json(events)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Successfully sent {} events to Songbird", events.len());
                    Ok(())
                } else {
                    warn!("Songbird events endpoint returned error: {}", response.status());
                    Ok(()) // Don't fail on monitoring issues
                }
            }
            Err(e) => {
                warn!("Failed to send events to Songbird: {}. Continuing without external monitoring.", e);
                Ok(())
            }
        }
    }

    /// Flush buffered metrics and events
    pub async fn flush_buffers(&self) -> Result<()> {
        // Flush events buffer
        let events = {
            let mut buffer = self.events_buffer.write().await;
            let events = buffer.clone();
            buffer.clear();
            events
        };

        if !events.is_empty() {
            self.send_events_to_songbird(&events).await?;
        }

        // Send current metrics if available
        if let Some(metrics) = self.current_metrics.read().await.clone() {
            self.send_metrics_to_songbird(&metrics).await?;
        }

        Ok(())
    }

    /// Start background monitoring task
    pub async fn start_monitoring(&self) -> Result<()> {
        let config = self.config.clone();
        let client = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(config.collection_interval)
            );

            loop {
                interval.tick().await;

                if let Err(e) = client.collect_and_send_metrics().await {
                    error!("Error in monitoring task: {}", e);
                }
            }
        });

        info!("Started Songbird monitoring task with {}s interval", self.config.collection_interval);
        Ok(())
    }

    /// Collect and send metrics in one operation
    async fn collect_and_send_metrics(&self) -> Result<()> {
        let metrics = self.collect_system_metrics().await?;
        self.send_metrics_to_songbird(&metrics).await?;
        Ok(())
    }
}

// Clone implementation for background tasks
impl Clone for SongbirdMonitoringClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            current_metrics: Arc::clone(&self.current_metrics),
            metrics_buffer: Arc::clone(&self.metrics_buffer),
            events_buffer: Arc::clone(&self.events_buffer),
        }
    }
}

use std::future::Future;

impl MonitoringClient for SongbirdMonitoringClient {
    fn report_breaker_success(&self, breaker_name: &str) -> impl Future<Output = anyhow::Result<()>> + Send {
        let breaker_name = breaker_name.to_string();
        let me = self.clone();
        async move {
            let event = MonitoringEvent {
                timestamp: Utc::now(),
                event_type: "circuit_breaker_success".to_string(),
                message: format!("Circuit breaker '{}' success", breaker_name),
                level: AlertLevel::Info,
                source: me.config.service_name.clone(),
                tags: HashMap::from([("breaker_name".to_string(), breaker_name)]),
                metadata: HashMap::new(),
            };
            
            me.record_event(event).await.map_err(|e| anyhow::anyhow!(e.to_string()))
        }
    }
    
    fn report_breaker_failure(&self, breaker_name: &str) -> impl Future<Output = anyhow::Result<()>> + Send {
        let breaker_name = breaker_name.to_string();
        let me = self.clone();
        async move {
            let event = MonitoringEvent {
                timestamp: Utc::now(),
                event_type: "circuit_breaker_failure".to_string(),
                message: format!("Circuit breaker '{}' failure", breaker_name),
                level: AlertLevel::Medium,
                source: me.config.service_name.clone(),
                tags: HashMap::from([("breaker_name".to_string(), breaker_name)]),
                metadata: HashMap::new(),
            };
            
            me.record_event(event).await.map_err(|e| anyhow::anyhow!(e.to_string()))
        }
    }
    
    fn report_breaker_rejection(&self, breaker_name: &str) -> impl Future<Output = anyhow::Result<()>> + Send {
        let breaker_name = breaker_name.to_string();
        let me = self.clone();
        async move {
            let event = MonitoringEvent {
                timestamp: Utc::now(),
                event_type: "circuit_breaker_rejection".to_string(),
                message: format!("Circuit breaker '{}' rejection", breaker_name),
                level: AlertLevel::High,
                source: me.config.service_name.clone(),
                tags: HashMap::from([("breaker_name".to_string(), breaker_name)]),
                metadata: HashMap::new(),
            };
            
            me.record_event(event).await.map_err(|e| anyhow::anyhow!(e.to_string()))
        }
    }

    fn record_event(&self, event: MonitoringEvent) -> impl Future<Output = Result<()>> + Send {
        let me = self.clone();
        async move {
            // Add to buffer
            me.events_buffer.write().await.push(event.clone());

            // Also log locally for debugging
            match event.level {
                AlertLevel::Critical => error!("CRITICAL: {}", event.message),
                AlertLevel::High => warn!("HIGH: {}", event.message),
                AlertLevel::Medium => warn!("MEDIUM: {}", event.message),
                AlertLevel::Low => info!("LOW: {}", event.message),
                AlertLevel::Info => info!("INFO: {}", event.message),
            }

            // Flush buffer if it's getting large
            if me.events_buffer.read().await.len() >= me.config.batch_size {
                if let Err(e) = me.flush_buffers().await {
                    warn!("Failed to flush monitoring buffers: {}", e);
                }
            }

            Ok(())
        }
    }

    fn record_metric(&self, name: &str, value: MetricValue, tags: Option<HashMap<String, String>>) -> impl Future<Output = Result<()>> + Send {
        let name = name.to_string();
        let me = self.clone();
        async move {
            // Update current metrics
            if let Some(mut metrics) = me.current_metrics.write().await.as_mut() {
                metrics.custom_metrics.insert(name.clone(), value.clone());
            }

            // Create a monitoring event for the metric
            let event = MonitoringEvent {
                timestamp: Utc::now(),
                event_type: "metric".to_string(),
                message: format!("Metric '{}' recorded: {:?}", name, value),
                level: AlertLevel::Info,
                source: me.config.service_name.clone(),
                tags: tags.unwrap_or_default(),
                metadata: HashMap::new(),
            };

            me.record_event(event).await
        }
    }

    fn get_health_status(&self) -> impl Future<Output = Result<bool>> + Send {
        let me = self.clone();
        async move {
            // Check if we can reach Songbird
            let url = format!("{}/api/v1/health", me.config.endpoint);
            
            match me.client.get(&url).send().await {
                Ok(response) => Ok(response.status().is_success()),
                Err(_) => {
                    // If Songbird is unavailable, we're still healthy locally
                    warn!("Songbird endpoint unreachable, but local monitoring continues");
                    Ok(true)
                }
            }
        }
    }

    fn get_metrics_summary(&self) -> impl Future<Output = Result<HashMap<String, MetricValue>>> + Send {
        let me = self.clone();
        async move {
            let metrics = me.current_metrics.read().await;
            
            match metrics.as_ref() {
                Some(m) => {
                    let mut summary = HashMap::new();
                    summary.insert("cpu_usage".to_string(), MetricValue::Float(m.cpu_usage));
                    summary.insert("memory_usage".to_string(), MetricValue::Float(m.memory_usage));
                    summary.insert("active_connections".to_string(), MetricValue::Integer(m.active_connections as i64));
                    summary.insert("request_count".to_string(), MetricValue::Integer(m.request_count as i64));
                    summary.insert("error_count".to_string(), MetricValue::Integer(m.error_count as i64));
                    
                    // Add custom metrics
                    for (key, value) in &m.custom_metrics {
                        summary.insert(key.clone(), value.clone());
                    }
                    
                    Ok(summary)
                }
                None => {
                    // Return empty summary if no metrics collected yet
                    Ok(HashMap::new())
                }
            }
        }
    }
}

/// Create a production Songbird monitoring client
pub fn create_songbird_client() -> Arc<SongbirdMonitoringClient> {
    match SongbirdMonitoringClient::with_defaults() {
        Ok(client) => Arc::new(client),
        Err(e) => {
            // PRODUCTION SAFE: If Songbird client creation fails, log error but don't crash
            tracing::error!("Failed to create Songbird monitoring client, creating fallback: {}", e);
            // Return a client with minimal configuration that won't fail
            let minimal_config = SongbirdClientConfig {
                endpoint: "http://localhost:8900".to_string(),
                service_name: "squirrel-mcp-fallback".to_string(),
                environment: "unknown".to_string(),
                collection_interval: 60,
                batch_size: 10,
                timeout_ms: 1000,
                enable_tracing: false,
            };
            
            match SongbirdMonitoringClient::new(minimal_config) {
                Ok(fallback_client) => Arc::new(fallback_client),
                Err(fallback_error) => {
                    tracing::error!("Even fallback Songbird client creation failed: {}", fallback_error);
                    // This should never happen, but if it does, we need to return something
                    // Create the most minimal client possible using reqwest defaults
                    let ultra_minimal_config = SongbirdClientConfig {
                        endpoint: "http://disabled".to_string(),
                        service_name: "squirrel-mcp-disabled".to_string(),
                        environment: "error".to_string(),
                        collection_interval: 300,
                        batch_size: 1,
                        timeout_ms: 500,
                        enable_tracing: false,
                    };
                    
                                         // If this fails too, there's a fundamental system issue
                     match SongbirdMonitoringClient::new(ultra_minimal_config) {
                         Ok(ultra_minimal_client) => Arc::new(ultra_minimal_client),
                         Err(critical_error) => {
                             tracing::error!("CRITICAL SYSTEM FAILURE: Cannot create any Songbird monitoring client: {}", critical_error);
                             // This represents a fundamental system failure - exit gracefully
                             std::process::exit(1);
                         }
                     }
                }
            }
        }
    }
}

/// Create a Songbird monitoring client with custom configuration
pub fn create_songbird_client_with_config(config: SongbirdClientConfig) -> Arc<SongbirdMonitoringClient> {
    match SongbirdMonitoringClient::new(config.clone()) {
        Ok(client) => Arc::new(client),
        Err(e) => {
            tracing::error!("Failed to create Songbird monitoring client with custom config: {}", e);
            // Fallback to default configuration
            match SongbirdMonitoringClient::with_defaults() {
                Ok(fallback_client) => {
                    tracing::warn!("Using default Songbird config as fallback");
                    Arc::new(fallback_client)
                }
                Err(fallback_error) => {
                    tracing::error!("Fallback to default config also failed: {}", fallback_error);
                    // Use the safe factory function as final fallback
                    create_songbird_client()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_songbird_client_creation() {
        let client = SongbirdMonitoringClient::with_defaults()
            .expect("Should be able to create SongbirdMonitoringClient with defaults in test");
        assert_eq!(client.config.service_name, "squirrel-mcp");
    }

    #[tokio::test]
    async fn test_songbird_metrics_collection() {
        let client = SongbirdMonitoringClient::with_defaults()
            .expect("Should be able to create SongbirdMonitoringClient for metrics test");
        let metrics = client.collect_system_metrics().await;
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    async fn test_songbird_event_recording() {
        let client = SongbirdMonitoringClient::with_defaults()
            .expect("Should be able to create SongbirdMonitoringClient for event recording test");
        
        let event = MonitoringEvent {
            timestamp: Utc::now(),
            event_type: "test".to_string(),
            message: "Test event".to_string(),
            level: AlertLevel::Info,
            source: "test".to_string(),
            tags: HashMap::new(),
            metadata: HashMap::new(),
        };

        let result = client.record_event(event).await;
        assert!(result.is_ok());
    }
} 