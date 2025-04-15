//! # Dashboard Integration
//! 
//! This module provides integration between the MCP tracing system and
//! the dashboard-core for visualizing traces.

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, error, info};

use crate::observability::tracing::external::ExternalTracingConfig;
use crate::observability::exporters::dashboard_exporter::DashboardExporter;
use crate::observability::ObservabilityError;

use squirrel_interfaces::tracing::{
    TraceData, TraceDataProvider, TraceDataConsumer, TraceConfig
};

/// Configuration for dashboard integration
#[derive(Debug, Clone)]
pub struct DashboardIntegrationConfig {
    /// Tracing configuration
    pub tracing_config: ExternalTracingConfig,
    /// Dashboard endpoint
    pub dashboard_endpoint: String,
    /// Whether to enable real-time updates
    pub enable_realtime: bool,
    /// Maximum number of traces to keep in history
    pub max_trace_history: usize,
}

impl Default for DashboardIntegrationConfig {
    fn default() -> Self {
        Self {
            tracing_config: ExternalTracingConfig::default(),
            dashboard_endpoint: "http://localhost:3000".to_string(),
            enable_realtime: true,
            max_trace_history: 100,
        }
    }
}

/// Integration adapter between MCP tracing and dashboard
pub struct DashboardIntegrationAdapter {
    /// The dashboard exporter
    exporter: Arc<DashboardExporter>,
    /// Configuration
    config: DashboardIntegrationConfig,
    /// Dashboard consumers to forward traces to
    consumers: Arc<tokio::sync::RwLock<Vec<Arc<dyn TraceDataConsumer>>>>,
    /// Background task handle
    task_handle: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl DashboardIntegrationAdapter {
    /// Create a new dashboard integration adapter
    pub fn new(config: DashboardIntegrationConfig) -> Self {
        let exporter = Arc::new(DashboardExporter::new(config.tracing_config.clone()));
        
        Self {
            exporter,
            config,
            consumers: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            task_handle: tokio::sync::Mutex::new(None),
        }
    }
    
    /// Add a trace data consumer
    pub async fn add_consumer(&self, consumer: Arc<dyn TraceDataConsumer>) -> Result<(), ObservabilityError> {
        let mut consumers = self.consumers.write().await;
        consumers.push(consumer);
        info!("Added trace data consumer, total consumers: {}", consumers.len());
        Ok(())
    }
    
    /// Get the underlying dashboard exporter
    pub fn exporter(&self) -> Arc<DashboardExporter> {
        self.exporter.clone()
    }
    
    /// Start the adapter
    pub async fn start(&self) -> Result<(), ObservabilityError> {
        // Start a background task to periodically check for new traces
        // and forward them to dashboard consumers
        if self.config.enable_realtime {
            let exporter = self.exporter.clone();
            let consumers = self.consumers.clone();
            let update_interval = std::time::Duration::from_secs(5);
            
            let mut task_handle_guard = self.task_handle.lock().await;
            if task_handle_guard.is_some() {
                return Err(ObservabilityError::TracingError(
                    "Dashboard integration already started".to_string()
                ));
            }
            
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(update_interval);
                
                loop {
                    interval.tick().await;
                    
                    // Get trace data from the provider
                    match exporter.get_trace_data().await {
                        Ok(traces) => {
                            if !traces.is_empty() {
                                debug!("Forwarding {} traces to {} consumers", 
                                    traces.len(), consumers.read().await.len());
                                
                                // Get a read lock on the consumers
                                let consumers_guard = consumers.read().await;
                                
                                // For each consumer, send all traces
                                for consumer in consumers_guard.iter() {
                                    for trace in &traces {
                                        if let Err(e) = consumer.consume_trace_data(trace.clone()).await {
                                            error!("Error sending trace to consumer: {}", e);
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            error!("Error getting trace data: {}", e);
                        }
                    }
                }
            });
            
            *task_handle_guard = Some(handle);
        }
        
        Ok(())
    }
    
    /// Stop the adapter
    pub async fn stop(&self) -> Result<(), ObservabilityError> {
        let mut task_handle_guard = self.task_handle.lock().await;
        if let Some(handle) = task_handle_guard.take() {
            handle.abort();
            info!("Stopped dashboard integration background task");
        }
        Ok(())
    }
}

#[cfg(feature = "dashboard")]
/// Dashboard consumer that forwards traces to dashboard-core
pub struct DashboardCoreConsumer {
    // This would be replaced with the actual dashboard-core client
    // when integrated with the real dashboard
    #[allow(dead_code)]
    config: TraceConfig,
}

#[cfg(feature = "dashboard")]
impl DashboardCoreConsumer {
    /// Create a new dashboard core consumer
    pub fn new(config: TraceConfig) -> Self {
        Self {
            config,
        }
    }
}

#[cfg(feature = "dashboard")]
#[async_trait]
impl TraceDataConsumer for DashboardCoreConsumer {
    async fn consume_trace_data(&self, trace_data: TraceData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would convert the trace data
        // to the dashboard-core format and send it to the dashboard
        
        info!("Would send trace data to dashboard-core: {} spans in trace", 
            trace_data.spans.len());
            
        // For testing, just log the trace data
        for span in &trace_data.spans {
            debug!("Span: {} - {} (parent: {:?})", 
                span.name, span.id, span.parent_id);
            
            for event in &span.events {
                debug!("  Event: {}", event.name);
            }
        }
        
        Ok(())
    }
}

/// Create a new dashboard integration with default configuration
pub fn create_default_dashboard_integration() -> Arc<DashboardIntegrationAdapter> {
    let config = DashboardIntegrationConfig::default();
    Arc::new(DashboardIntegrationAdapter::new(config))
}

#[cfg(feature = "dashboard")]
/// Create a dashboard consumer that forwards traces to dashboard-core
pub fn create_dashboard_core_consumer(service_name: &str, environment: &str) -> Arc<dyn TraceDataConsumer> {
    let config = TraceConfig {
        service_name: service_name.to_string(),
        environment: environment.to_string(),
        include_standard_attributes: true,
        max_events_per_span: 100,
        max_spans: 1000,
    };
    
    Arc::new(DashboardCoreConsumer::new(config))
} 