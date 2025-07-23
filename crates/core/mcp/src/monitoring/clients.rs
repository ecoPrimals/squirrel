//! Monitoring client interfaces and implementations
//!
//! This module provides the MonitoringClient trait and various implementations
//! for interfacing with monitoring systems and collecting telemetry data.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};
use crate::error::Result;
use std::time::Duration;

/// Trait for monitoring clients that collect and report telemetry data
#[async_trait]
pub trait MonitoringClient: Send + Sync {
    /// Report a circuit breaker success
    async fn report_breaker_success(&self, breaker_name: &str) -> anyhow::Result<()>;
    
    /// Report a circuit breaker failure
    async fn report_breaker_failure(&self, breaker_name: &str) -> anyhow::Result<()>;
    
    /// Report a circuit breaker rejection
    async fn report_breaker_rejection(&self, breaker_name: &str) -> anyhow::Result<()>;
    
    /// Record a monitoring event
    async fn record_event(&self, event: MonitoringEvent) -> Result<()>;
    
    /// Record a metric value
    async fn record_metric(&self, name: &str, value: MetricValue, tags: Option<HashMap<String, String>>) -> Result<()>;
    
    /// Get health status
    async fn get_health_status(&self) -> Result<bool>;
    
    /// Get metrics summary
    async fn get_metrics_summary(&self) -> Result<HashMap<String, MetricValue>>;
}

/// Different types of metric values that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer metric value
    Integer(i64),
    /// Floating point metric value
    Float(f64),
    /// String metric value
    String(String),
    /// Boolean metric value
    Boolean(bool),
}

/// Alert level for monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Critical alert requiring immediate attention
    Critical,
    /// High priority alert
    High,
    /// Medium priority alert
    Medium,
    /// Low priority alert
    Low,
    /// Informational message
    Info,
}

/// Monitoring event structure for tracking significant occurrences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type identifier
    pub event_type: String,
    /// Event message
    pub message: String,
    /// Alert level
    pub level: AlertLevel,
    /// Event source
    pub source: String,
    /// Event tags
    pub tags: HashMap<String, String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Mock implementation of MonitoringClient for testing
pub struct MockMonitoringClient {
    /// Component ID for this client
    component_id: String,
    /// Count of events by type
    event_counts: Mutex<HashMap<String, usize>>,
    /// Recorded metrics
    metrics: Mutex<HashMap<String, MetricValue>>,
    /// Health status
    health_status: Mutex<bool>,
}

impl MockMonitoringClient {
    /// Create a new mock monitoring client
    pub fn new(component_id: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            event_counts: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
            health_status: Mutex::new(true),
        }
    }

    /// Get the count of events recorded for a specific type
    pub fn get_event_count(&self, event_type: &str) -> usize {
        self.event_counts
            .lock()
            .unwrap()
            .get(event_type)
            .copied()
            .unwrap_or(0)
    }

    /// Get all recorded metrics
    pub fn get_recorded_metrics(&self) -> HashMap<String, MetricValue> {
        self.metrics.lock().unwrap().clone()
    }

    /// Set the health status for testing
    pub fn set_health_status(&self, healthy: bool) {
        *self.health_status.lock().unwrap() = healthy;
    }

    /// Clear all recorded data
    pub fn clear(&self) {
        self.event_counts.lock().unwrap().clear();
        self.metrics.lock().unwrap().clear();
        *self.health_status.lock().unwrap() = true;
    }

    /// Get total event count across all types
    pub fn get_total_event_count(&self) -> usize {
        self.event_counts.lock().unwrap().values().sum()
    }

    /// Get component ID
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
}

#[async_trait]
impl MonitoringClient for MockMonitoringClient {
    async fn report_breaker_success(&self, breaker_name: &str) -> anyhow::Result<()> {
        debug!("MockMonitoringClient[{}]: Circuit breaker success for '{}'", 
               self.component_id, breaker_name);
        
        let mut counts = self.event_counts.lock().unwrap();
        *counts.entry(format!("breaker_success_{}", breaker_name)).or_insert(0) += 1;
        
        Ok(())
    }

    async fn report_breaker_failure(&self, breaker_name: &str) -> anyhow::Result<()> {
        debug!("MockMonitoringClient[{}]: Circuit breaker failure for '{}'", 
               self.component_id, breaker_name);
        
        let mut counts = self.event_counts.lock().unwrap();
        *counts.entry(format!("breaker_failure_{}", breaker_name)).or_insert(0) += 1;
        
        Ok(())
    }

    async fn report_breaker_rejection(&self, breaker_name: &str) -> anyhow::Result<()> {
        debug!("MockMonitoringClient[{}]: Circuit breaker rejection for '{}'", 
               self.component_id, breaker_name);
        
        let mut counts = self.event_counts.lock().unwrap();
        *counts.entry(format!("breaker_rejection_{}", breaker_name)).or_insert(0) += 1;
        
        Ok(())
    }

    async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        debug!("MockMonitoringClient[{}]: Recording event '{}' with level {:?}", 
               self.component_id, event.event_type, event.level);
        
        let mut counts = self.event_counts.lock().unwrap();
        *counts.entry(event.event_type.clone()).or_insert(0) += 1;
        
        // Also count by alert level
        let level_key = format!("level_{:?}", event.level).to_lowercase();
        *counts.entry(level_key).or_insert(0) += 1;
        
        Ok(())
    }

    async fn record_metric(&self, name: &str, value: MetricValue, _tags: Option<HashMap<String, String>>) -> Result<()> {
        debug!("MockMonitoringClient[{}]: Recording metric '{}' = {:?}", 
               self.component_id, name, value);
        
        let mut metrics = self.metrics.lock().map_err(|e| {
            crate::error::types::MCPError::ResourceContention(format!("Failed to acquire metrics lock: {}", e))
        })?;
        metrics.insert(name.to_string(), value);
        
        // Count metric recordings
        let mut counts = self.event_counts.lock().map_err(|e| {
            crate::error::types::MCPError::ResourceContention(format!("Failed to acquire event counts lock: {}", e))
        })?;
        *counts.entry("metric_recorded".to_string()).or_insert(0) += 1;
        
        Ok(())
    }

    async fn get_health_status(&self) -> Result<bool> {
        let healthy = *self.health_status.lock().unwrap();
        debug!("MockMonitoringClient[{}]: Health status = {}", self.component_id, healthy);
        Ok(healthy)
    }

    async fn get_metrics_summary(&self) -> Result<HashMap<String, MetricValue>> {
        let metrics = self.metrics.lock().unwrap().clone();
        
        // Add some summary metrics
        let mut summary = metrics;
        let total_events = self.get_total_event_count();
        summary.insert("total_events".to_string(), MetricValue::Integer(total_events as i64));
        summary.insert("component_id".to_string(), MetricValue::String(self.component_id.clone()));
        
        debug!("MockMonitoringClient[{}]: Returning metrics summary with {} entries", 
               self.component_id, summary.len());
        
        Ok(summary)
    }
}

/// Production monitoring client that integrates with external monitoring systems
pub struct ProductionMonitoringClient {
    /// Component ID for this client
    component_id: String,
    /// Configuration for the monitoring client
    config: MonitoringClientConfig,
    /// HTTP client for external API calls
    http_client: reqwest::Client,
    /// Internal metrics storage
    metrics: Mutex<HashMap<String, MetricValue>>,
}

/// Configuration for production monitoring clients
#[derive(Debug, Clone)]
pub struct MonitoringClientConfig {
    /// Endpoint URL for the monitoring service
    pub endpoint: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Timeout for HTTP requests in milliseconds
    pub timeout_ms: u64,
    /// Whether to enable SSL verification
    pub ssl_verify: bool,
    /// Batch size for sending events
    pub batch_size: usize,
    /// Flush interval for batched events
    pub flush_interval_ms: u64,
}

impl ProductionMonitoringClient {
    /// Create a new production monitoring client
    pub fn new(component_id: &str, config: MonitoringClientConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            component_id: component_id.to_string(),
            config,
            http_client,
            metrics: Mutex::new(HashMap::new()),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &MonitoringClientConfig {
        &self.config
    }

    /// Send a request to the monitoring service with comprehensive resilience
    async fn send_request(&self, path: &str, body: serde_json::Value) -> anyhow::Result<()> {
        use std::time::Instant;
        
        let url = format!("{}/{}", self.config.endpoint, path);
        
        // Resilience configuration for monitoring requests
        let max_retries = 3;
        let per_attempt_timeout = Duration::from_millis(self.config.timeout_ms / max_retries as u64);
        let base_delay = Duration::from_millis(200);
        
        let mut last_error = None;
        let start_time = Instant::now();
        
        for attempt in 1..=max_retries {
            tracing::debug!("Monitoring request attempt {}/{} to {} (timeout: {:?})", 
                attempt, max_retries, url, per_attempt_timeout);
                
            // Create request with timeout
            let mut request = self.http_client
                .post(&url)
                .json(&body)
                .timeout(per_attempt_timeout);

            if let Some(api_key) = &self.config.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        tracing::debug!("Monitoring request succeeded on attempt {} in {:?}", 
                            attempt, start_time.elapsed());
                        return Ok(());
                    } else {
                        // Try to get error body for debugging
                        let status = response.status();
                        let error_body = match tokio::time::timeout(
                            Duration::from_secs(2), 
                            response.text()
                        ).await {
                            Ok(Ok(text)) => text,
                            _ => "Unable to read error body".to_string(),
                        };
                        
                        let error_msg = format!("HTTP {} - {}", status, error_body);
                        last_error = Some(error_msg.clone());
                        tracing::warn!("Monitoring request attempt {} failed with HTTP error: {}", 
                            attempt, error_msg);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Network/timeout error: {}", e);
                    last_error = Some(error_msg.clone());
                    tracing::warn!("Monitoring request attempt {} failed: {}", attempt, error_msg);
                }
            }
            
            // Exponential backoff between retries (except on last attempt)
            if attempt < max_retries {
                let delay = base_delay * (2_u32.pow((attempt - 1).min(4))); // Cap at 2^4 = 16x
                let jitter = Duration::from_millis(rand::random::<u64>() % 100); // Small jitter
                let total_delay = delay + jitter;
                
                tracing::debug!("Retrying monitoring request after {:?} delay", total_delay);
                tokio::time::sleep(total_delay).await;
            }
        }
        
        let final_error = last_error.unwrap_or_else(|| "All monitoring request attempts failed".to_string());
        tracing::error!("Monitoring request to {} failed after {} attempts in {:?}: {}", 
            url, max_retries, start_time.elapsed(), final_error);
            
        Err(anyhow::anyhow!("Monitoring service request failed: {}", final_error))
    }
}

#[async_trait]
impl MonitoringClient for ProductionMonitoringClient {
    async fn report_breaker_success(&self, breaker_name: &str) -> anyhow::Result<()> {
        let event = serde_json::json!({
            "component_id": self.component_id,
            "event_type": "circuit_breaker_success",
            "breaker_name": breaker_name,
            "timestamp": Utc::now()
        });

        self.send_request("events/breaker", event).await.map_err(|e| {
            error!("Failed to report breaker success: {}", e);
            e
        })
    }

    async fn report_breaker_failure(&self, breaker_name: &str) -> anyhow::Result<()> {
        let event = serde_json::json!({
            "component_id": self.component_id,
            "event_type": "circuit_breaker_failure",
            "breaker_name": breaker_name,
            "timestamp": Utc::now()
        });

        self.send_request("events/breaker", event).await.map_err(|e| {
            error!("Failed to report breaker failure: {}", e);
            e
        })
    }

    async fn report_breaker_rejection(&self, breaker_name: &str) -> anyhow::Result<()> {
        let event = serde_json::json!({
            "component_id": self.component_id,
            "event_type": "circuit_breaker_rejection",
            "breaker_name": breaker_name,
            "timestamp": Utc::now()
        });

        self.send_request("events/breaker", event).await.map_err(|e| {
            error!("Failed to report breaker rejection: {}", e);
            e
        })
    }

    async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        let payload = serde_json::json!({
            "component_id": self.component_id,
            "event": event
        });

        self.send_request("events", payload).await.map_err(|e| {
            error!("Failed to record event: {}", e);
            crate::error::MCPError::MonitoringError(e.to_string())
        })
    }

    async fn record_metric(&self, name: &str, value: MetricValue, tags: Option<HashMap<String, String>>) -> Result<()> {
        // Store locally
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.insert(name.to_string(), value.clone());
        }

        // Send to external service
        let payload = serde_json::json!({
            "component_id": self.component_id,
            "metric_name": name,
            "metric_value": value,
            "tags": tags.unwrap_or_default(),
            "timestamp": Utc::now()
        });

        self.send_request("metrics", payload).await.map_err(|e| {
            error!("Failed to record metric: {}", e);
            crate::error::MCPError::MonitoringError(e.to_string())
        })
    }

    async fn get_health_status(&self) -> Result<bool> {
        // Try to ping the monitoring service
        match self.send_request("health", serde_json::json!({})).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_metrics_summary(&self) -> Result<HashMap<String, MetricValue>> {
        let local_metrics = self.metrics.lock().unwrap().clone();
        
        // Try to get remote metrics as well
        match self.http_client.get(&format!("{}/metrics/summary", self.config.endpoint)).send().await {
            Ok(response) if response.status().is_success() => {
                if let Ok(remote_metrics) = response.json::<HashMap<String, MetricValue>>().await {
                    let mut combined = local_metrics;
                    combined.extend(remote_metrics);
                    Ok(combined)
                } else {
                    Ok(local_metrics)
                }
            }
            _ => Ok(local_metrics)
        }
    }
}

impl Default for MonitoringClientConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("MONITORING_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("MONITORING_API_KEY").ok(),
            timeout_ms: std::env::var("MONITORING_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            ssl_verify: std::env::var("MONITORING_SSL_VERIFY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            batch_size: std::env::var("MONITORING_BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            flush_interval_ms: std::env::var("MONITORING_FLUSH_INTERVAL_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
        }
    }
}

impl MonitoringClientConfig {
    /// Create a new monitoring client configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the endpoint URL
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the timeout
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set SSL verification
    pub fn with_ssl_verify(mut self, ssl_verify: bool) -> Self {
        self.ssl_verify = ssl_verify;
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set flush interval
    pub fn with_flush_interval_ms(mut self, flush_interval_ms: u64) -> Self {
        self.flush_interval_ms = flush_interval_ms;
        self
    }
}

impl MonitoringEvent {
    /// Create a new monitoring event
    pub fn new(event_type: &str, message: &str, level: AlertLevel, source: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            message: message.to_string(),
            level,
            source: source.to_string(),
            tags: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a tag to the event
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Integer(i) => write!(f, "{}", i),
            MetricValue::Float(fl) => write!(f, "{}", fl),
            MetricValue::String(s) => write!(f, "{}", s),
            MetricValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Critical => write!(f, "CRITICAL"),
            AlertLevel::High => write!(f, "HIGH"),
            AlertLevel::Medium => write!(f, "MEDIUM"),
            AlertLevel::Low => write!(f, "LOW"),
            AlertLevel::Info => write!(f, "INFO"),
        }
    }
} 