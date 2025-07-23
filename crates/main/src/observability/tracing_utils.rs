//! Tracing utilities for enhanced observability across the squirrel ecosystem
//!
//! This module provides advanced tracing capabilities that work seamlessly with
//! the universal adapter pattern, enabling comprehensive observability without
//! hardcoded primal dependencies.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, span, Instrument, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use uuid::Uuid;

use crate::error::PrimalError;

/// Universal tracing coordinator that discovers tracing endpoints through universal adapter
#[derive(Debug, Clone)]
pub struct UniversalTracingCoordinator {
    /// Discovered tracing endpoints from various primals
    endpoints: Arc<tokio::sync::RwLock<Vec<TracingEndpoint>>>,
    /// Active spans being tracked
    active_spans: Arc<dashmap::DashMap<Uuid, TracedOperation>>,
    /// Configuration for tracing behavior
    config: Arc<TracingConfig>,
}

/// Represents a discovered tracing endpoint from any primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingEndpoint {
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<TracingCapability>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Tracing capabilities offered by discovered endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingCapability {
    SpanCollection,
    EventStreaming,
    MetricsIntegration,
    CustomAttributes,
    DistributedTracing,
}

/// Represents a traced operation that can span multiple primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracedOperation {
    pub operation_id: Uuid,
    pub operation_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub attributes: HashMap<String, String>,
    pub source_primal: Option<String>,
    pub related_operations: Vec<Uuid>,
}

/// Configuration for universal tracing
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub enable_distributed_tracing: bool,
    pub max_span_duration: std::time::Duration,
    pub custom_attributes: HashMap<String, String>,
    pub trace_sampling_ratio: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enable_distributed_tracing: true,
            max_span_duration: std::time::Duration::from_secs(300), // 5 minutes
            custom_attributes: HashMap::new(),
            trace_sampling_ratio: 1.0, // Trace everything in development
        }
    }
}

impl UniversalTracingCoordinator {
    /// Create new tracing coordinator with universal adapter discovery
    pub fn new(config: TracingConfig) -> Self {
        Self {
            endpoints: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            active_spans: Arc::new(dashmap::DashMap::new()),
            config: Arc::new(config),
        }
    }

    /// Discover tracing endpoints through universal adapter (no hardcoded primals)
    pub async fn discover_tracing_endpoints(&self) -> Result<(), PrimalError> {
        info!("Discovering tracing endpoints through universal adapter");
        
        // Use universal adapter to discover any primal with tracing capabilities
        let discovered = self.query_universal_adapter_for_tracing().await?;
        
        let mut endpoints = self.endpoints.write().await;
        endpoints.extend(discovered);
        
        info!("Discovered {} tracing endpoints", endpoints.len());
        Ok(())
    }

    /// Query universal adapter for tracing capabilities
    async fn query_universal_adapter_for_tracing(&self) -> Result<Vec<TracingEndpoint>, PrimalError> {
        debug!("Querying universal adapter for tracing capabilities");
        
        let mut discovered = Vec::new();
        
        // Generic capability discovery - works with any primal providing tracing
        // This could discover beardog security tracing, toadstool compute tracing, etc.
        
        // For now, create a generic endpoint that represents discovered capabilities
        let generic_endpoint = TracingEndpoint {
            primal_type: "discovered_primal".to_string(),
            endpoint: "http://localhost:8080/tracing".to_string(),
            capabilities: vec![
                TracingCapability::SpanCollection,
                TracingCapability::EventStreaming,
                TracingCapability::DistributedTracing,
            ],
            discovered_at: chrono::Utc::now(),
        };
        discovered.push(generic_endpoint);
        
        info!("Universal adapter discovered {} tracing-capable primals", discovered.len());
        Ok(discovered)
    }

    /// Start a new traced operation with universal context
    pub async fn start_traced_operation(&self, operation_name: String, attributes: HashMap<String, String>) -> Result<Uuid, PrimalError> {
        let operation_id = Uuid::new_v4();
        
        let traced_operation = TracedOperation {
            operation_id,
            operation_name: operation_name.clone(),
            start_time: chrono::Utc::now(),
            attributes,
            source_primal: Some("squirrel".to_string()),
            related_operations: Vec::new(),
        };
        
        self.active_spans.insert(operation_id, traced_operation);
        
        // Propagate trace context to discovered endpoints
        self.propagate_trace_context(operation_id, &operation_name).await?;
        
        info!("Started traced operation: {} ({})", operation_name, operation_id);
        Ok(operation_id)
    }

    /// Propagate trace context to all discovered tracing endpoints
    async fn propagate_trace_context(&self, operation_id: Uuid, operation_name: &str) -> Result<(), PrimalError> {
        let endpoints = self.endpoints.read().await;
        
        for endpoint in endpoints.iter() {
            if let Err(e) = self.send_trace_context(endpoint, operation_id, operation_name).await {
                debug!("Failed to propagate trace context to {}: {}", endpoint.primal_type, e);
                // Don't fail the entire operation if one endpoint is unreachable
            }
        }
        
        Ok(())
    }

    /// Send trace context to a specific discovered endpoint
    async fn send_trace_context(&self, endpoint: &TracingEndpoint, operation_id: Uuid, operation_name: &str) -> Result<(), PrimalError> {
        let trace_payload = serde_json::json!({
            "operation_id": operation_id,
            "operation_name": operation_name,
            "timestamp": chrono::Utc::now(),
            "source_primal": "squirrel",
            "trace_context": {
                "sampling_ratio": self.config.trace_sampling_ratio,
                "distributed_tracing": self.config.enable_distributed_tracing
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint.endpoint)
            .json(&trace_payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to send trace context: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::NetworkError(format!(
                "Trace endpoint returned error: {}", response.status()
            )));
        }

        debug!("Successfully propagated trace context to {}", endpoint.primal_type);
        Ok(())
    }

    /// Finish a traced operation and collect results
    pub async fn finish_traced_operation(&self, operation_id: Uuid) -> Result<TracedOperation, PrimalError> {
        if let Some((_, mut operation)) = self.active_spans.remove(&operation_id) {
            // Calculate operation duration
            let duration = chrono::Utc::now() - operation.start_time;
            operation.attributes.insert("duration_ms".to_string(), duration.num_milliseconds().to_string());
            
            // Collect trace data from discovered endpoints
            self.collect_trace_data(operation_id).await?;
            
            info!("Finished traced operation: {} (duration: {}ms)", 
                operation.operation_name, duration.num_milliseconds());
            
            Ok(operation)
        } else {
            Err(PrimalError::InvalidOperation(format!("Operation {} not found", operation_id)))
        }
    }

    /// Collect trace data from all discovered endpoints
    async fn collect_trace_data(&self, operation_id: Uuid) -> Result<(), PrimalError> {
        let endpoints = self.endpoints.read().await;
        
        for endpoint in endpoints.iter() {
            if let Ok(trace_data) = self.collect_from_endpoint(endpoint, operation_id).await {
                debug!("Collected trace data from {}: {} events", endpoint.primal_type, trace_data.len());
                // Process and store trace data as needed
            }
        }
        
        Ok(())
    }

    /// Collect trace data from a specific endpoint
    async fn collect_from_endpoint(&self, endpoint: &TracingEndpoint, operation_id: Uuid) -> Result<Vec<serde_json::Value>, PrimalError> {
        let client = reqwest::Client::new();
        let url = format!("{}/traces/{}", endpoint.endpoint, operation_id);
        
        let response = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to collect trace data: {}", e)))?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // Return empty if not found
        }

        let trace_data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| PrimalError::ParsingError(format!("Failed to parse trace data: {}", e)))?;

        Ok(trace_data)
    }

    /// Get active operations snapshot
    pub async fn get_active_operations(&self) -> Vec<TracedOperation> {
        self.active_spans.iter().map(|entry| entry.value().clone()).collect()
    }
}

/// Create a traced span that automatically propagates across discovered primals
pub async fn create_universal_span(name: &str, attributes: HashMap<String, String>) -> Span {
    let span = span!(Level::INFO, "universal_operation", operation_name = name);
    
    // Add custom attributes to the span
    for (key, value) in attributes {
        span.record(key.as_str(), &value.as_str());
    }
    
    span
}

/// Instrument a future with universal tracing context
pub async fn trace_operation<F, T>(operation_name: &str, future: F) -> Result<T, PrimalError>
where
    F: std::future::Future<Output = Result<T, PrimalError>>,
{
    let attributes = HashMap::new();
    let span = create_universal_span(operation_name, attributes).await;
    
    async move {
        info!("Starting traced operation: {}", operation_name);
        let result = future.await;
        
        match &result {
            Ok(_) => info!("Traced operation completed successfully: {}", operation_name),
            Err(e) => tracing::error!("Traced operation failed: {}: {}", operation_name, e),
        }
        
        result
    }
    .instrument(span)
    .await
}

/// Initialize universal tracing system
pub async fn initialize_tracing() -> Result<UniversalTracingCoordinator, PrimalError> {
    info!("Initializing universal tracing system");
    
    // Set up tracing subscriber
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer());
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| PrimalError::ConfigurationError(format!("Failed to set tracing subscriber: {}", e)))?;
    
    let config = TracingConfig::default();
    let coordinator = UniversalTracingCoordinator::new(config);
    
    // Discover tracing endpoints through universal adapter
    coordinator.discover_tracing_endpoints().await?;
    
    info!("Universal tracing system initialized successfully");
    Ok(coordinator)
} 