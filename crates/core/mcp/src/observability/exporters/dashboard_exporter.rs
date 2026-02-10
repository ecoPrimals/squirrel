// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::future::Future;
use chrono::Utc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use tokio::task::JoinHandle;
// Removed: use squirrel_mcp_config::get_service_endpoints;

use crate::observability::tracing::{
    Span, SpanStatus, 
    external::{SpanExporter, ExternalTracingConfig},
};

// Import the interfaces to avoid circular dependencies
use squirrel_interfaces::tracing::{
    TraceData, TraceSpan, TraceEvent, TraceStatus, 
    TraceDataProvider, TraceDataConsumer
};

// Define the types for the dashboard integration
#[derive(Debug, Clone)]
struct SpanData {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
    pub status: String,
    pub service: String,
}

#[derive(Debug, Clone)]
struct SpanEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct InterfaceTraceData {
    pub spans: Vec<InterfaceTraceSpan>,
    pub service: String,
    pub environment: String,
    pub collected_at: u64,
}

#[derive(Debug, Clone)]
struct InterfaceTraceSpan {
    pub id: String,
    pub trace_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<InterfaceTraceEvent>,
    pub status: InterfaceTraceStatus,
    pub service: String,
}

#[derive(Debug, Clone)]
struct InterfaceTraceEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum InterfaceTraceStatus {
    Success,
    Error,
    Running,
    Unknown,
}

// Mock Dashboard Core types - used when the dashboard feature is enabled
#[cfg(feature = "dashboard")]
mod dashboard_core {
    
    
    pub mod data {
        use std::collections::HashMap;
        use chrono::{DateTime, Utc};
        
        #[derive(Debug, Clone)]
        pub struct Alert {
            pub id: String,
            pub name: String,
            pub severity: String,
            pub timestamp: DateTime<Utc>,
            pub description: String,
        }
        
        #[derive(Debug, Clone)]
        pub struct Metrics {
            pub name: String,
            pub values: HashMap<String, f64>,
            pub timestamp: DateTime<Utc>,
        }
        
        #[derive(Debug, Clone)]
        pub struct ProtocolData {
            pub name: String,
            pub data: String,
        }
        
        #[derive(Debug, Clone)]
        pub struct DashboardData {
            pub service: String,
            pub timestamp: DateTime<Utc>,
            pub traces: Vec<String>,
            pub metrics: Vec<Metrics>,
            pub alerts: Vec<Alert>,
            pub protocol_data: Vec<ProtocolData>,
        }
    }
    
    pub mod monitoring {
        use async_trait::async_trait;
        use super::data::DashboardData;
        
        pub struct MonitoringAdapterConfig {
            pub endpoint: String,
            pub service_name: String,
        }
        
        pub trait DashboardDataProvider {
            fn provide_data(&self) -> impl Future<Output = Result<DashboardData, Box<dyn std::error::Error + Send + Sync>>> + Send;
        }
        
        pub trait MonitoringDataAdapter {
            fn send_data(&self, data: DashboardData) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;
        }
    }
}

// This is a fake import path for demonstration; replace with the actual path
// when integrating with the real api-server
#[cfg(feature = "dashboard")]
use self::dashboard_core::data::DashboardData;

/// Configuration for the dashboard exporter
#[derive(Debug, Clone)]
pub struct DashboardExporterConfig {
    /// The URL of the dashboard service
    pub dashboard_url: String,
    /// The maximum batch size for exporting spans
    pub max_batch_size: usize,
    /// The export interval in seconds
    pub export_interval_secs: u64,
    /// The service name to report to the dashboard
    pub service_name: String,
    /// The environment name
    pub environment: String,
    /// Additional connection properties
    pub properties: HashMap<String, String>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Interval for flushing spans to dashboard (in seconds)
    pub flush_interval_seconds: u64,
    /// Maximum size of span buffer before forcing a flush
    pub max_buffer_size: usize,
}

impl Default for DashboardExporterConfig {
    fn default() -> Self {
        // Multi-tier dashboard URL resolution
        let dashboard_url = std::env::var("UI_ENDPOINT").unwrap_or_else(|_| {
            let port = std::env::var("WEB_UI_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(3000);  // Default Web UI port
            format!("http://localhost:{}", port)
        });

        Self {
            dashboard_url,
            max_batch_size: 100,
            export_interval_secs: 5,
            service_name: "unknown".to_string(),
            environment: "development".to_string(),
            properties: HashMap::new(),
            auth_token: None,
            flush_interval_seconds: 15,
            max_buffer_size: 100,
        }
    }
}

/// Exporter for sending trace data to the dashboard
pub struct DashboardExporter {
    /// Configuration for the exporter
    config: DashboardExporterConfig,
    /// Spans waiting to be exported
    spans: Arc<Mutex<Vec<Span>>>,
    /// Last export time
    last_export: Arc<Mutex<Instant>>,
    /// Whether an export is currently in progress
    exporting: Arc<Mutex<bool>>,
    /// Trace data cache for the TraceDataProvider
    trace_cache: Arc<Mutex<HashMap<String, TraceData>>>,
    /// HTTP client for sending spans
    client: reqwest::Client,
    /// Buffer of spans to be exported
    buffer: Arc<Mutex<Vec<Span>>>,
    /// Consumers that will receive the trace data
    consumers: Arc<Mutex<Vec<Arc<dyn TraceDataConsumer>>>>,
}

impl DashboardExporter {
    /// Create a new dashboard exporter
    pub fn new(config: ExternalTracingConfig) -> Self {
        let dashboard_config = DashboardExporterConfig {
            dashboard_url: config.endpoint_url,
            max_batch_size: config.max_buffer_size,
            export_interval_secs: config.flush_interval_seconds,
            service_name: config.service_name,
            environment: config.environment,
            properties: HashMap::new(),
            auth_token: config.auth_token,
            flush_interval_seconds: config.flush_interval_seconds,
            max_buffer_size: config.max_buffer_size,
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            config: dashboard_config,
            spans: Arc::new(Mutex::new(Vec::new())),
            last_export: Arc::new(Mutex::new(Instant::now())),
            exporting: Arc::new(Mutex::new(false)),
            trace_cache: Arc::new(Mutex::new(HashMap::new())),
            client,
            buffer: Arc::new(Mutex::new(Vec::new())),
            consumers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Convert internal spans to TraceData format
    fn convert_spans_to_trace_data(&self, spans: &[Span]) -> TraceData {
        // Group spans by trace ID
        let mut trace_spans = Vec::new();
        
        for span in spans {
            trace_spans.push(convert_span_to_trace_span(span, &self.config.service_name));
        }
        
        TraceData {
            spans: trace_spans,
            service: self.config.service_name.clone(),
            environment: self.config.environment.clone(),
            collected_at: Utc::now().timestamp() as u64,
        }
    }

    /// Start the export background task
    pub fn start(&self) -> Result<(), crate::observability::ObservabilityError> {
        let spans = self.spans.clone();
        let last_export = self.last_export.clone();
        let exporting = self.exporting.clone();
        let config = self.config.clone();

        // Start background task for periodic exports
        tokio::spawn(async move {
            let export_interval = Duration::from_secs(config.export_interval_secs);
            loop {
                tokio::time::sleep(export_interval).await;

                // Check if export is needed
                let should_export = {
                    let spans_guard = spans.lock().await;
                    let last_export_guard = last_export.lock().await;
                    let exporting_guard = exporting.lock().await;

                    !*exporting_guard && 
                    (!spans_guard.is_empty() || last_export_guard.elapsed() >= export_interval)
                };

                if should_export {
                    // Mark as exporting
                    {
                        let mut exporting_guard = exporting.lock().await;
                        *exporting_guard = true;
                    }

                    // Take spans to export
                    let export_spans = {
                        let mut spans_guard = spans.lock().await;
                        if spans_guard.len() > config.max_batch_size {
                            spans_guard.drain(0..config.max_batch_size).collect()
                        } else {
                            std::mem::take(&mut *spans_guard)
                        }
                    };

                    // Only export if we have spans
                    if !export_spans.is_empty() {
                        let spans_count = export_spans.len();
                        match Self::export_spans(&export_spans, &config).await {
                            Ok(_) => {
                                debug!("Successfully exported {} spans to dashboard", spans_count);
                                // Update last export time
                                let mut last_export_guard = last_export.lock().await;
                                *last_export_guard = Instant::now();
                            }
                            Err(e) => {
                                error!("Failed to export spans to dashboard: {}", e);
                                // Put spans back
                                let mut spans_guard = spans.lock().await;
                                for span in export_spans {
                                    spans_guard.push(span);
                                }
                            }
                        }
                    }

                    // Mark as not exporting
                    {
                        let mut exporting_guard = exporting.lock().await;
                        *exporting_guard = false;
                    }
                }
            }
        });

        Ok(())
    }

    /// Export spans to the dashboard
    #[cfg(feature = "dashboard")]
    async fn export_spans(spans: &[Span], config: &DashboardExporterConfig) -> Result<(), crate::observability::ObservabilityError> {
        
        

        // In a real implementation, we would create a proper dashboard client
        // For now, we'll just log the spans and create a proto format
        info!("Exporting {} spans to dashboard at {}", spans.len(), config.dashboard_url);

        for span in spans {
            // Use accessor methods instead of direct field access
            debug!("Span: [{:?}] {}", 
                span.trace_id(),
                span.name()
            );

            for (key, value) in span.attributes() {
                debug!("  Attr: {} = {}", key, value);
            }

            for event in span.events() {
                debug!("  Event: {}", event.name());
                for (key, value) in event.attributes() {
                    debug!("    Attr: {} = {}", key, value);
                }
            }
        }

        // Convert to dashboard trace format
        // In a real implementation, this would integrate with the dashboard API
        let _trace_data = create_dashboard_trace_data(spans, config);

        // Here we would send the data to the dashboard
        // For example:
        // dashboard_client.send_traces(trace_data).await?;

        Ok(())
    }

    /// Export spans to the dashboard (non-dashboard build)
    #[cfg(not(feature = "dashboard"))]
    async fn export_spans(spans: &[Span], config: &DashboardExporterConfig) -> Result<(), crate::observability::ObservabilityError> {
        // In builds without dashboard support, just log the spans
        info!("Dashboard export not enabled. Would export {} spans to {}", 
            spans.len(), config.dashboard_url);
        
        for span in spans {
            debug!("Span: {} ({})",
                span.name(),
                span.id()
            );
        }
        
        Ok(())
    }

    /// Add a trace data consumer that will receive exported traces
    pub async fn add_consumer(&self, consumer: Arc<dyn TraceDataConsumer>) -> Result<(), crate::observability::ObservabilityError> {
        let mut consumers = self.consumers.lock().await;
        consumers.push(consumer);
        Ok(())
    }

    /// Start the flush task to periodically export spans
    pub fn start_flush_task(&self) -> Result<JoinHandle<()>, crate::observability::ObservabilityError> {
        let buffer = self.buffer.clone();
        let config = self.config.clone();
        let exporting = self.exporting.clone();
        let last_export = self.last_export.clone();
        let trace_cache = self.trace_cache.clone();
        let consumers = self.consumers.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(config.flush_interval_seconds)
            );
            
            loop {
                interval.tick().await;
                
                // Check if we should export spans
                let should_export = {
                    let buffer_guard = buffer.lock().await;
                    !buffer_guard.is_empty()
                };
                
                if should_export {
                    // Only one export at a time
                    let mut is_exporting = exporting.lock().await;
                    if *is_exporting {
                        debug!("Skipping export, already in progress");
                        continue;
                    }
                    
                    *is_exporting = true;
                    
                    // Update last export time
                    let mut last_time = last_export.lock().await;
                    *last_time = Instant::now();
                    
                    // Take spans from buffer
                    let spans_to_export = {
                        let mut buffer_guard = buffer.lock().await;
                        std::mem::take(&mut *buffer_guard)
                    };
                    
                    if !spans_to_export.is_empty() {
                        debug!("Flushing {} spans to dashboard", spans_to_export.len());
                        
                        // Create a reference to self for calling the method
                        let trace_data = match Self::create_dashboard_trace_data(&spans_to_export, &config) {
                            Ok(data) => data,
                            Err(e) => {
                                error!("Error creating trace data: {}", e);
                                *is_exporting = false;
                                continue;
                            }
                        };
                        
                        // Store in trace cache using trace ID as key
                        if !spans_to_export.is_empty() {
                            let trace_id = spans_to_export[0].trace_id().to_string();
                            let mut trace_cache_guard = trace_cache.lock().await;
                            trace_cache_guard.insert(trace_id, trace_data.clone());
                        }
                        
                        // Notify consumers
                        let consumers_guard = consumers.lock().await;
                        for consumer in consumers_guard.iter() {
                            if let Err(e) = consumer.consume_trace_data(trace_data.clone()).await {
                                error!("Error notifying consumer: {}", e);
                            }
                        }
                        
                        // Export spans
                        if let Err(e) = Self::export_spans_to_dashboard(&spans_to_export, &config).await {
                            error!("Error exporting spans to dashboard: {}", e);
                        }
                    }
                    
                    *is_exporting = false;
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Export spans to the dashboard server
    async fn export_spans_to_dashboard(
        spans: &[Span], 
        config: &DashboardExporterConfig
    ) -> Result<(), crate::observability::ObservabilityError> {
        info!("Exporting {} spans to dashboard at {}", spans.len(), config.dashboard_url);

        for span in spans {
            debug!("Span: [{:?}] {}", 
                span.trace_id(),
                span.name()
            );

            for (key, value) in span.attributes() {
                debug!("  Attr: {} = {}", key, value);
            }

            for event in span.events() {
                debug!("  Event: {}", event.name());
                for (key, value) in event.attributes() {
                    debug!("    Attr: {} = {}", key, value);
                }
            }
        }

        // Convert to dashboard trace format
        let _trace_data = create_dashboard_trace_data(spans, config);

        Ok(())
    }

    /// Create dashboard trace data from spans
    fn create_dashboard_trace_data(
        spans: &[Span], 
        config: &DashboardExporterConfig
    ) -> Result<TraceData, crate::observability::ObservabilityError> {
        if spans.is_empty() {
            return Err(crate::observability::ObservabilityError::TracingError(
                "Cannot create trace data from empty span list".to_string(),
            ));
        }

        let mut trace_spans = Vec::new();
        
        for span in spans {
            let trace_span = TraceSpan {
                id: span.id().to_string(),
                trace_id: span.trace_id().to_string(),
                parent_id: span.parent_id().map(|s| s.to_string()),
                name: span.name().to_string(),
                start_time: 0, // We'll need to implement proper time conversion
                end_time: if span.status() != SpanStatus::Running { Some(0) } else { None },
                attributes: span.attributes().clone(),
                events: span.events().iter().map(|e| TraceEvent {
                    name: e.name().to_string(),
                    timestamp: 0, // Will need proper conversion
                    attributes: e.attributes().clone(),
                }).collect(),
                status: match span.status() {
                    SpanStatus::Success => TraceStatus::Success,
                    SpanStatus::Error => TraceStatus::Error,
                    SpanStatus::Running => TraceStatus::Running,
                },
                service: config.service_name.clone(),
            };
            
            trace_spans.push(trace_span);
        }
        
        Ok(TraceData {
            spans: trace_spans,
            service: config.service_name.clone(),
            environment: config.environment.clone(),
            collected_at: Utc::now().timestamp() as u64,
        })
    }
}

impl SpanExporter for DashboardExporter {
    fn export_spans(&self, spans: Vec<Span>) -> impl Future<Output = Result<(), crate::observability::ObservabilityError>> + Send {
        let buffer = self.buffer.clone();
        let max_buffer_size = self.config.max_buffer_size;
        
        async move {
            info!("Exporting {} spans to dashboard", spans.len());
            
            // Add to buffer
            let mut buffer = buffer.lock().await;
            buffer.extend(spans);
            
            // If buffer exceeds max size, remove oldest entries
            if buffer.len() > max_buffer_size {
                let overflow = buffer.len() - max_buffer_size;
                buffer.drain(0..overflow);
            }
            
            Ok(())
        }
    }

    fn shutdown(&self) -> impl Future<Output = Result<(), crate::observability::ObservabilityError>> + Send {
        async move {
            // No special shutdown needed for this exporter
            Ok(())
        }
    }
}

/// Create dashboard trace data from spans
#[cfg(feature = "dashboard")]
fn create_dashboard_trace_data(spans: &[Span], config: &DashboardExporterConfig) -> DashboardData {
    use self::dashboard_core::data::DashboardData;
    
    let mut traces = Vec::new();
    
    for span in spans {
        // Create a simple string representation of the span
        let trace_str = format!(
            "Trace: {} - Span: {} ({}) - Status: {:?}",
            span.trace_id(),
            span.name(),
            span.id(),
            span.status()
        );
        
        traces.push(trace_str);
    }
    
    DashboardData {
        service: config.service_name.clone(),
        timestamp: Utc::now(),
        traces,
        metrics: Vec::new(),
        alerts: Vec::new(),
        protocol_data: Vec::new(),
    }
}

/// Create dashboard trace data (non-dashboard build)
#[cfg(not(feature = "dashboard"))]
fn create_dashboard_trace_data(spans: &[Span], _config: &DashboardExporterConfig) -> String {
    format!("Would export {} spans", spans.len())
}

/// Helper function to create a dashboard exporter
pub fn create_dashboard_exporter(config: ExternalTracingConfig) -> Box<dyn SpanExporter> {
    Box::new(DashboardExporter::new(config))
}

/// Implementation of TraceDataProvider for DashboardExporter
impl TraceDataProvider for DashboardExporter {
    fn get_trace_data(&self) -> impl Future<Output = Result<Vec<TraceData>, Box<dyn std::error::Error + Send + Sync>>> + Send {
        let trace_cache = self.trace_cache.clone();
        
        async move {
            let trace_cache = trace_cache.lock().await;
            let traces = trace_cache.values().cloned().collect();
            Ok(traces)
        }
    }
    
    fn get_trace_by_id(&self, trace_id: &str) -> impl Future<Output = Result<Option<TraceData>, Box<dyn std::error::Error + Send + Sync>>> + Send {
        let trace_cache = self.trace_cache.clone();
        let trace_id = trace_id.to_string();
        
        async move {
            let trace_cache = trace_cache.lock().await;
            Ok(trace_cache.get(&trace_id).cloned())
        }
    }
}

/// Convert a span to the TraceSpan format
fn convert_span_to_trace_span(span: &Span, service_name: &str) -> TraceSpan {
    let mut trace_span = TraceSpan {
        id: span.id().to_string(),
        trace_id: span.trace_id().to_string(),
        parent_id: span.parent_id().map(|s| s.to_string()),
        name: span.name().to_string(),
        // We need to implement proper time conversion
        start_time: 0,
        end_time: if span.status() != SpanStatus::Running { Some(0) } else { None },
        attributes: span.attributes().clone(),
        events: Vec::new(),
        status: match span.status() {
            SpanStatus::Success => TraceStatus::Success,
            SpanStatus::Error => TraceStatus::Error,
            SpanStatus::Running => TraceStatus::Running,
        },
        service: service_name.to_string(),
    };
    
    // Convert events
    for event in span.events() {
        trace_span.events.push(TraceEvent {
            name: event.name().to_string(),
            timestamp: 0, // Will need proper time conversion
            attributes: event.attributes().clone(),
        });
    }
    
    trace_span
} 