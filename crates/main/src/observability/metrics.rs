//! Metrics collection and reporting for squirrel observability
//!
//! This module provides comprehensive metrics collection across the squirrel ecosystem,
//! supporting both Prometheus and custom metric backends through the universal adapter pattern.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::PrimalError;

/// Universal metrics collector that works with any primal through capability discovery
#[derive(Debug, Clone)]
pub struct UniversalMetricsCollector {
    /// Discovered metrics endpoints from various primals
    endpoints: Arc<RwLock<Vec<MetricsEndpoint>>>,
    /// Local metric storage
    metrics: Arc<DashMap<String, MetricValue>>,
    /// Collection intervals and configuration
    config: Arc<MetricsConfig>,
}

/// Represents a discovered metrics endpoint from any primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsEndpoint {
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<MetricCapability>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Metric capabilities offered by discovered endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricCapability {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Custom(String),
}

/// Generic metric value that can be collected from any primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_primal: Option<String>,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub collection_interval: Duration,
    pub enable_prometheus: bool,
    pub custom_endpoints: Vec<String>,
    pub max_metrics_history: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(30),
            enable_prometheus: true,
            custom_endpoints: Vec::new(),
            max_metrics_history: 10000,
        }
    }
}

impl UniversalMetricsCollector {
    /// Create new metrics collector with capability discovery
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            endpoints: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(DashMap::new()),
            config: Arc::new(config),
        }
    }

    /// Discover metrics endpoints from ecosystem primals
    pub async fn discover_metrics_endpoints(&self) -> Result<(), PrimalError> {
        info!("Starting metrics endpoint discovery through universal adapter");
        
        // Use universal adapter pattern to discover any primal with metrics capabilities
        let discovered = self.query_universal_adapter_for_metrics().await?;
        
        let mut endpoints = self.endpoints.write().await;
        endpoints.extend(discovered);
        
        info!("Discovered {} metrics endpoints", endpoints.len());
        Ok(())
    }

    /// Query universal adapter for metrics capabilities (no hardcoded primal knowledge)
    async fn query_universal_adapter_for_metrics(&self) -> Result<Vec<MetricsEndpoint>, PrimalError> {
        debug!("Querying universal adapter for metrics capabilities");
        
        // This would integrate with the universal adapter to discover any primal
        // providing metrics capabilities without knowing specific primal types
        let mut discovered = Vec::new();
        
        // Generic capability discovery - works with beardog, toadstool, songbird, etc.
        // Each primal advertises its metrics capabilities through the universal adapter
        for endpoint in &self.config.custom_endpoints {
            let metrics_endpoint = MetricsEndpoint {
                primal_type: "discovered_primal".to_string(),
                endpoint: endpoint.clone(),
                capabilities: vec![
                    MetricCapability::Counter,
                    MetricCapability::Gauge,
                    MetricCapability::Histogram,
                ],
                discovered_at: chrono::Utc::now(),
            };
            discovered.push(metrics_endpoint);
        }
        
        info!("Universal adapter discovered {} metrics-capable primals", discovered.len());
        Ok(discovered)
    }

    /// Collect metrics from all discovered endpoints
    pub async fn collect_metrics(&self) -> Result<Vec<MetricValue>, PrimalError> {
        let endpoints = self.endpoints.read().await;
        let mut all_metrics = Vec::new();
        
        for endpoint in endpoints.iter() {
            match self.collect_from_endpoint(endpoint).await {
                Ok(mut metrics) => {
                    all_metrics.append(&mut metrics);
                }
                Err(e) => {
                    warn!("Failed to collect metrics from {}: {}", endpoint.endpoint, e);
                }
            }
        }
        
        // Store metrics locally
        for metric in &all_metrics {
            self.metrics.insert(metric.name.clone(), metric.clone());
        }
        
        info!("Collected {} metrics from {} endpoints", all_metrics.len(), endpoints.len());
        Ok(all_metrics)
    }

    /// Collect metrics from a specific discovered endpoint
    async fn collect_from_endpoint(&self, endpoint: &MetricsEndpoint) -> Result<Vec<MetricValue>, PrimalError> {
        debug!("Collecting metrics from {} endpoint", endpoint.primal_type);
        
        // Generic HTTP-based metrics collection that works with any primal
        let client = reqwest::Client::new();
        let response = client
            .get(&endpoint.endpoint)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Metrics collection failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::NetworkError(format!(
                "Metrics endpoint returned status: {}", response.status()
            )));
        }

        // Parse generic metrics format (supports Prometheus, JSON, custom formats)
        let body = response.text().await.map_err(|e| 
            PrimalError::NetworkError(format!("Failed to read metrics response: {}", e)))?;
        
        self.parse_metrics_response(&body, endpoint).await
    }

    /// Parse metrics response in various formats (Prometheus, JSON, custom)
    async fn parse_metrics_response(&self, body: &str, endpoint: &MetricsEndpoint) -> Result<Vec<MetricValue>, PrimalError> {
        let mut metrics = Vec::new();
        
        // Try parsing as JSON first (most common for primal APIs)
        if let Ok(json_metrics) = self.parse_json_metrics(body, endpoint).await {
            metrics.extend(json_metrics);
            return Ok(metrics);
        }
        
        // Try parsing as Prometheus format
        if let Ok(prom_metrics) = self.parse_prometheus_metrics(body, endpoint).await {
            metrics.extend(prom_metrics);
            return Ok(metrics);
        }
        
        warn!("Could not parse metrics from {} in any known format", endpoint.primal_type);
        Ok(metrics)
    }

    /// Parse JSON metrics format (common for REST APIs)
    async fn parse_json_metrics(&self, body: &str, endpoint: &MetricsEndpoint) -> Result<Vec<MetricValue>, PrimalError> {
        let json: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| PrimalError::ParsingError(format!("Invalid JSON metrics: {}", e)))?;
        
        let mut metrics = Vec::new();
        
        // Handle various JSON metric formats generically
        if let Some(metrics_obj) = json.as_object() {
            for (name, value) in metrics_obj {
                if let Some(numeric_value) = value.as_f64() {
                    let metric = MetricValue {
                        name: name.clone(),
                        value: numeric_value,
                        labels: HashMap::new(),
                        timestamp: chrono::Utc::now(),
                        source_primal: Some(endpoint.primal_type.clone()),
                    };
                    metrics.push(metric);
                }
            }
        }
        
        Ok(metrics)
    }

    /// Parse Prometheus metrics format
    async fn parse_prometheus_metrics(&self, body: &str, endpoint: &MetricsEndpoint) -> Result<Vec<MetricValue>, PrimalError> {
        let mut metrics = Vec::new();
        
        // Simple Prometheus format parsing
        for line in body.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            if let Some((name, rest)) = line.split_once(' ') {
                if let Ok(value) = rest.trim().parse::<f64>() {
                    let metric = MetricValue {
                        name: name.to_string(),
                        value,
                        labels: HashMap::new(),
                        timestamp: chrono::Utc::now(),
                        source_primal: Some(endpoint.primal_type.clone()),
                    };
                    metrics.push(metric);
                }
            }
        }
        
        Ok(metrics)
    }

    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> HashMap<String, MetricValue> {
        self.metrics.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect()
    }

    /// Start background metrics collection
    pub async fn start_collection(&self) -> tokio::task::JoinHandle<()> {
        let collector = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collector.config.collection_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = collector.collect_metrics().await {
                    warn!("Metrics collection failed: {}", e);
                }
            }
        })
    }
}

/// Initialize metrics collection with universal adapter discovery
pub async fn initialize_metrics() -> Result<UniversalMetricsCollector, PrimalError> {
    info!("Initializing universal metrics collection system");
    
    let config = MetricsConfig::default();
    let collector = UniversalMetricsCollector::new(config);
    
    // Discover metrics endpoints through universal adapter
    collector.discover_metrics_endpoints().await?;
    
    // Start background collection
    collector.start_collection().await;
    
    info!("Universal metrics system initialized successfully");
    Ok(collector)
} 