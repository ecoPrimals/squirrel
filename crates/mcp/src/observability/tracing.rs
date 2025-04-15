//! # Distributed Tracing
//! 
//! This module provides distributed tracing capabilities for the MCP,
//! enabling request tracking across system components and services.
//!
//! ## Key Components
//!
//! - **Span**: A unit of work or operation, with start and end times
//! - **Trace**: A collection of spans forming a request flow
//! - **SpanContext**: Context that propagates between services
//! - **Tracer**: Creates and manages spans and traces

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;
use std::cell::RefCell;
use crate::observability::{ObservabilityError, ObservabilityResult};

/// Span represents a single unit of work in a distributed trace
#[derive(Debug, Clone)]
pub struct Span {
    /// Unique identifier for the span
    id: String,
    /// Trace ID this span belongs to
    trace_id: String,
    /// Parent span ID, if any
    parent_id: Option<String>,
    /// Name of the span
    name: String,
    /// When the span was started
    start_time: Instant,
    /// When the span was ended, if complete
    end_time: Option<Instant>,
    /// Key-value attributes for additional context
    attributes: HashMap<String, String>,
    /// Events that occurred during the span
    events: Vec<SpanEvent>,
    /// Status of the span
    status: SpanStatus,
}

/// Status of a span execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    /// Span is still in progress
    Running,
    /// Span completed successfully
    Success,
    /// Span completed with an error
    Error,
}

/// An event that occurs during a span
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Name of the event
    name: String,
    /// When the event occurred
    timestamp: Instant,
    /// Additional event attributes
    attributes: HashMap<String, String>,
}

impl SpanEvent {
    /// Get the name of the event
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the timestamp when the event occurred
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Get the event attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

impl Span {
    /// Create a new span
    pub fn new(
        name: impl Into<String>,
        trace_id: impl Into<String>,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            trace_id: trace_id.into(),
            parent_id,
            name: name.into(),
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Running,
        }
    }

    /// End the span and mark it as successful
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
        self.status = SpanStatus::Success;
    }

    /// End the span and mark it as error
    pub fn end_with_error(&mut self) {
        self.end_time = Some(Instant::now());
        self.status = SpanStatus::Error;
    }

    /// Add an attribute to the span
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: Instant::now(),
            attributes,
        });
    }

    /// Get the span ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the parent span ID
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the span duration
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }

    /// Get the span status
    pub fn status(&self) -> SpanStatus {
        self.status
    }

    /// Check if the span is active
    pub fn is_active(&self) -> bool {
        self.status == SpanStatus::Running
    }

    /// Get all span attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Get all span events
    pub fn events(&self) -> &[SpanEvent] {
        &self.events
    }
}

/// Context for propagating trace information between services
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    trace_id: String,
    /// Span ID
    span_id: String,
    /// Whether this context is sampled (recorded)
    sampled: bool,
    /// Additional context information
    baggage: HashMap<String, String>,
}

impl SpanContext {
    /// Create a new span context
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>, sampled: bool) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            sampled,
            baggage: HashMap::new(),
        }
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the span ID
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// Check if this context is sampled
    pub fn is_sampled(&self) -> bool {
        self.sampled
    }

    /// Add baggage item to the context
    pub fn add_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.baggage.insert(key.into(), value.into());
    }

    /// Get a baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&str> {
        self.baggage.get(key).map(|s| s.as_str())
    }

    /// Get all baggage items
    pub fn baggage(&self) -> &HashMap<String, String> {
        &self.baggage
    }
}

/// The active span
#[derive(Clone)]
pub struct ActiveSpan {
    /// The span being traced
    span: Span,
    /// Whether the span has been ended
    ended: bool,
}

impl ActiveSpan {
    /// Create a new active span
    pub fn new(span: Span) -> Self {
        Self {
            span,
            ended: false,
        }
    }

    /// Get a reference to the underlying span
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Get a mutable reference to the underlying span
    pub fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }

    /// End the span and mark as successful
    pub fn end(mut self) {
        if !self.ended {
            self.span.end();
            self.ended = true;
        }
    }

    /// End the span and mark as error
    pub fn end_with_error(mut self) {
        if !self.ended {
            self.span.end_with_error();
            self.ended = true;
        }
    }

    /// Add an attribute to the span
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.span.add_attribute(key, value);
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.span.add_event(name, attributes);
    }

    /// Check if the span has been ended
    pub fn is_ended(&self) -> bool {
        self.ended
    }
}

impl Drop for ActiveSpan {
    fn drop(&mut self) {
        if !self.ended {
            self.span.end();
            self.ended = true;
        }
    }
}

/// Configuration for the tracer
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// Whether tracing is enabled
    pub enabled: bool,
    /// Sampling rate (0.0-1.0)
    pub sampling_rate: f64,
    /// Maximum number of spans to keep in memory
    pub max_spans: usize,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1.0, // Sample all traces by default
            max_spans: 10000,
        }
    }
}

/// The tracer creates and manages spans
#[derive(Debug)]
pub struct Tracer {
    /// Tracer configuration
    config: RwLock<TracerConfig>,
    /// All spans created by this tracer
    spans: Mutex<HashMap<String, Span>>,
}

thread_local! {
    /// Current active span for this thread
    static ACTIVE_SPAN: RefCell<Option<Arc<Mutex<ActiveSpan>>>> = RefCell::new(None);
}

impl Tracer {
    /// Create a new tracer
    pub fn new() -> Self {
        Self {
            config: RwLock::new(TracerConfig::default()),
            spans: Mutex::new(HashMap::new()),
        }
    }

    /// Initialize the tracer
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Any initialization tasks would go here
        Ok(())
    }

    /// Set the tracer configuration
    pub fn set_config(&self, config: TracerConfig) -> ObservabilityResult<()> {
        let mut current_config = self.config.write().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire config write lock: {}", e)))?;
        *current_config = config;
        Ok(())
    }

    /// Start a new span
    pub fn start_span(&self, name: impl Into<String>) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        self.start_span_with_parent(name, None)
    }

    /// Start a new span with a parent
    pub fn start_span_with_parent(
        &self,
        name: impl Into<String>,
        parent_span: Option<Arc<Mutex<ActiveSpan>>>,
    ) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        let config = self.config.read().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire config read lock: {}", e)))?;
        
        if !config.enabled {
            return Err(ObservabilityError::TracingError("Tracing is disabled".to_string()));
        }

        // Determine if we should sample this span
        let should_sample = rand::random::<f64>() <= config.sampling_rate;
        if !should_sample {
            return Err(ObservabilityError::TracingError("Span not sampled".to_string()));
        }

        // Get parent span info if available
        let (trace_id, parent_id) = if let Some(parent) = parent_span {
            let parent = parent.lock().map_err(|e| 
                ObservabilityError::TracingError(format!("Failed to acquire parent span lock: {}", e)))?;
            
            (parent.span().trace_id().to_string(), Some(parent.span().id().to_string()))
        } else {
            (Uuid::new_v4().to_string(), None)
        };

        // Create the span
        let span = Span::new(name, trace_id, parent_id);
        let span_id = span.id().to_string();
        
        // Store the span
        let active_span = Arc::new(Mutex::new(ActiveSpan::new(span.clone())));
        
        // Set as current active span for this thread
        ACTIVE_SPAN.with(|current| {
            *current.borrow_mut() = Some(active_span.clone());
        });
        
        // Store in spans collection
        let mut spans = self.spans.lock().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire spans lock: {}", e)))?;
        
        // Check if we need to evict old spans
        if spans.len() >= config.max_spans {
            // Simple eviction: remove oldest span
            // In a real implementation, this would be more sophisticated
            if let Some((oldest_id, _)) = spans.iter().next() {
                let oldest_id = oldest_id.clone();
                spans.remove(&oldest_id);
            }
        }
        
        spans.insert(span_id, span);
        
        Ok(active_span)
    }

    /// Get the current active span
    pub fn current_span(&self) -> ObservabilityResult<Option<Arc<Mutex<ActiveSpan>>>> {
        let result = ACTIVE_SPAN.with(|current| {
            current.borrow().clone()
        });
        
        Ok(result)
    }

    /// Get a span by ID
    pub fn get_span(&self, span_id: &str) -> ObservabilityResult<Option<Span>> {
        let spans = self.spans.lock().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire spans lock: {}", e)))?;
        
        Ok(spans.get(span_id).cloned())
    }

    /// Get all spans for a trace
    pub fn get_trace_spans(&self, trace_id: &str) -> ObservabilityResult<Vec<Span>> {
        let spans = self.spans.lock().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire spans lock: {}", e)))?;
        
        Ok(spans.values()
            .filter(|span| span.trace_id() == trace_id)
            .cloned()
            .collect())
    }

    /// Clear all spans
    pub fn clear_spans(&self) -> ObservabilityResult<()> {
        let mut spans = self.spans.lock().map_err(|e| 
            ObservabilityError::TracingError(format!("Failed to acquire spans lock: {}", e)))?;
        
        spans.clear();
        Ok(())
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

/// External tracing system support
pub mod external {
    use super::*;
    use crate::observability::{ObservabilityError, ObservabilityResult};
    use crate::monitoring::metrics::{MetricsCollector, Metric, MetricType, MetricValue};
    use async_trait::async_trait;
    use std::sync::Arc;
    use tracing::{debug, error};

    /// Trait for exporting spans to external tracing systems
    #[async_trait]
    pub trait SpanExporter: Send + Sync {
        /// Export a batch of spans to an external system
        async fn export_spans(&self, spans: Vec<Span>) -> ObservabilityResult<()>;
        
        /// Shutdown the exporter
        async fn shutdown(&self) -> ObservabilityResult<()>;
    }

    /// Configuration for external tracing integration
    #[derive(Debug, Clone)]
    pub struct ExternalTracingConfig {
        /// Endpoint URL for the tracing system
        pub endpoint_url: String,
        
        /// Authentication token, if needed
        pub auth_token: Option<String>,
        
        /// How often to flush spans to the external system (in seconds)
        pub flush_interval_seconds: u64,
        
        /// Maximum number of spans to buffer before flushing
        pub max_buffer_size: usize,
        
        /// Whether to add certain standard attributes to all spans
        pub add_standard_attributes: bool,
        
        /// Name of the service for tracing
        pub service_name: String,
        
        /// Environment name (dev, staging, prod)
        pub environment: String,
    }

    impl Default for ExternalTracingConfig {
        fn default() -> Self {
            Self {
                endpoint_url: "http://localhost:4318/v1/traces".to_string(),
                auth_token: None,
                flush_interval_seconds: 5,
                max_buffer_size: 100,
                add_standard_attributes: true,
                service_name: "squirrel-mcp".to_string(),
                environment: "development".to_string(),
            }
        }
    }

    /// External tracing adapter for OpenTelemetry
    pub struct OpenTelemetryExporter {
        /// Configuration for the exporter
        config: ExternalTracingConfig,
        
        /// HTTP client for sending spans
        client: reqwest::Client,
        
        /// Buffer of spans to be exported
        buffer: Arc<Mutex<Vec<Span>>>,
        
        /// Metrics collector for reporting exporter status
        metrics: Option<Arc<MetricsCollector>>,
    }

    impl OpenTelemetryExporter {
        /// Create a new OpenTelemetry exporter
        pub fn new(config: ExternalTracingConfig) -> Self {
            let client_builder = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10));
            
            let client = if let Some(token) = &config.auth_token {
                client_builder
                    .default_headers(
                        std::iter::once((
                            reqwest::header::AUTHORIZATION,
                            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static(""))
                        ))
                        .collect()
                    )
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            } else {
                client_builder
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            };
            
            Self {
                config,
                client,
                buffer: Arc::new(Mutex::new(Vec::new())),
                metrics: None,
            }
        }
        
        /// Add metrics collector for telemetry
        pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
            self.metrics = Some(metrics);
            self
        }
        
        /// Start the background flush task
        pub fn start_flush_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
            let config = self.config.clone();
            let buffer = self.buffer.clone();
            let client = self.client.clone();
            let metrics = self.metrics.clone();
            
            let handle = tokio::spawn(async move {
                let interval_duration = std::time::Duration::from_secs(config.flush_interval_seconds);
                let mut interval = tokio::time::interval(interval_duration);
                
                loop {
                    interval.tick().await;
                    
                    // Take spans from buffer
                    let spans = {
                        let mut buffer_guard = buffer.lock().unwrap();
                        if buffer_guard.is_empty() {
                            continue;
                        }
                        
                        debug!("Flushing {} spans to external tracing system", buffer_guard.len());
                        std::mem::take(&mut *buffer_guard)
                    };
                    
                    // Export the spans
                    if let Err(e) = Self::do_export_spans(&client, &config, &spans).await {
                        error!("Failed to export spans: {}", e);
                        
                        // Return spans to buffer
                        let mut buffer_guard = buffer.lock().unwrap();
                        buffer_guard.extend(spans);
                        
                        // Update metrics
                        if let Some(metrics) = &metrics {
                            metrics.increment_counter("tracing.export.failures");
                        }
                    } else {
                        // Update metrics
                        if let Some(metrics) = &metrics {
                            metrics.increment_counter("tracing.export.success");
                            metrics.register_metric(Metric::new(
                                "tracing.export.spans",
                                "Number of spans exported",
                                MetricType::Gauge,
                                MetricValue::Float(spans.len() as f64)
                            ));
                        }
                    }
                }
            });
            
            Ok(handle)
        }
        
        /// Export spans to OpenTelemetry collector
        async fn do_export_spans(
            client: &reqwest::Client,
            config: &ExternalTracingConfig,
            spans: &[Span]
        ) -> ObservabilityResult<()> {
            // Convert Span to OpenTelemetry format
            let otlp_spans = convert_to_otlp_format(spans, config);
            
            // Send to OpenTelemetry collector
            let response = client
                .post(&config.endpoint_url)
                .json(&otlp_spans)
                .send()
                .await
                .map_err(|e| ObservabilityError::External(format!("Failed to send spans: {}", e)))?;
            
            if !response.status().is_success() {
                return Err(ObservabilityError::External(
                    format!("Failed to export spans: HTTP {}", response.status())
                ));
            }
            
            Ok(())
        }
    }

    #[async_trait]
    impl SpanExporter for OpenTelemetryExporter {
        async fn export_spans(&self, spans: Vec<Span>) -> ObservabilityResult<()> {
            // Add spans to buffer
            {
                let mut buffer = self.buffer.lock().unwrap();
                buffer.extend(spans.clone());
                
                // Check if we need to flush
                if buffer.len() >= self.config.max_buffer_size {
                    debug!("Buffer size exceeded, flushing {} spans", buffer.len());
                    
                    // Take all spans from buffer
                    let spans_to_flush = std::mem::take(&mut *buffer);
                    
                    // Clone the values we need inside the tokio::spawn
                    let client = self.client.clone();
                    let config = self.config.clone();
                    
                    // Export the spans in the background
                    tokio::spawn(async move {
                        if let Err(e) = Self::do_export_spans(&client, &config, &spans_to_flush).await {
                            error!("Failed to export spans: {}", e);
                        }
                    });
                }
            }
            
            Ok(())
        }
        
        async fn shutdown(&self) -> ObservabilityResult<()> {
            // Flush any remaining spans
            let spans = {
                let mut buffer = self.buffer.lock().unwrap();
                std::mem::take(&mut *buffer)
            };
            
            if !spans.is_empty() {
                debug!("Flushing {} spans during shutdown", spans.len());
                Self::do_export_spans(&self.client, &self.config, &spans).await?;
            }
            
            Ok(())
        }
    }

    /// Jaeger exporter for tracing
    pub struct JaegerExporter {
        /// Configuration for the exporter
        config: ExternalTracingConfig,
        
        /// OpenTelemetry exporter
        otlp_exporter: OpenTelemetryExporter,
    }

    impl JaegerExporter {
        /// Create a new Jaeger exporter
        pub fn new(mut config: ExternalTracingConfig) -> Self {
            // Default to Jaeger endpoint if not specified
            if config.endpoint_url == "http://localhost:4318/v1/traces" {
                config.endpoint_url = "http://localhost:14268/api/traces".to_string();
            }
            
            Self {
                config: config.clone(),
                otlp_exporter: OpenTelemetryExporter::new(config),
            }
        }
        
        /// Add metrics collector for telemetry
        pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
            self.otlp_exporter = self.otlp_exporter.with_metrics(metrics);
            self
        }
        
        /// Start the background flush task
        pub fn start_flush_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
            self.otlp_exporter.start_flush_task()
        }
    }

    #[async_trait]
    impl SpanExporter for JaegerExporter {
        async fn export_spans(&self, spans: Vec<Span>) -> ObservabilityResult<()> {
            self.otlp_exporter.export_spans(spans).await
        }
        
        async fn shutdown(&self) -> ObservabilityResult<()> {
            self.otlp_exporter.shutdown().await
        }
    }

    /// Zipkin exporter for tracing
    pub struct ZipkinExporter {
        /// Configuration for the exporter
        config: ExternalTracingConfig,
        
        /// HTTP client for sending spans
        client: reqwest::Client,
        
        /// Buffer of spans to be exported
        buffer: Arc<Mutex<Vec<Span>>>,
        
        /// Metrics collector for reporting exporter status
        metrics: Option<Arc<MetricsCollector>>,
    }

    impl ZipkinExporter {
        /// Create a new Zipkin exporter
        pub fn new(mut config: ExternalTracingConfig) -> Self {
            // Default to Zipkin endpoint if not specified
            if config.endpoint_url == "http://localhost:4318/v1/traces" {
                config.endpoint_url = "http://localhost:9411/api/v2/spans".to_string();
            }
            
            let client_builder = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10));
            
            let client = if let Some(token) = &config.auth_token {
                client_builder
                    .default_headers(
                        std::iter::once((
                            reqwest::header::AUTHORIZATION,
                            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static(""))
                        ))
                        .collect()
                    )
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            } else {
                client_builder
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            };
            
            Self {
                config,
                client,
                buffer: Arc::new(Mutex::new(Vec::new())),
                metrics: None,
            }
        }
        
        /// Add metrics collector for telemetry
        pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
            self.metrics = Some(metrics);
            self
        }
        
        /// Start the background flush task
        pub fn start_flush_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
            let config = self.config.clone();
            let buffer = self.buffer.clone();
            let client = self.client.clone();
            let metrics = self.metrics.clone();
            
            let handle = tokio::spawn(async move {
                let interval_duration = std::time::Duration::from_secs(config.flush_interval_seconds);
                let mut interval = tokio::time::interval(interval_duration);
                
                loop {
                    interval.tick().await;
                    
                    // Take spans from buffer
                    let spans = {
                        let mut buffer_guard = buffer.lock().unwrap();
                        if buffer_guard.is_empty() {
                            continue;
                        }
                        
                        debug!("Flushing {} spans to Zipkin", buffer_guard.len());
                        std::mem::take(&mut *buffer_guard)
                    };
                    
                    // Export the spans
                    if let Err(e) = Self::do_export_spans(&client, &config, &spans).await {
                        error!("Failed to export spans to Zipkin: {}", e);
                        
                        // Return spans to buffer
                        let mut buffer_guard = buffer.lock().unwrap();
                        buffer_guard.extend(spans);
                        
                        // Update metrics
                        if let Some(metrics) = &metrics {
                            metrics.increment_counter("tracing.zipkin.export.failures");
                        }
                    } else {
                        // Update metrics
                        if let Some(metrics) = &metrics {
                            metrics.increment_counter("tracing.zipkin.export.success");
                            metrics.register_metric(Metric::new(
                                "tracing.zipkin.export.spans",
                                "Number of spans exported to Zipkin",
                                MetricType::Gauge,
                                MetricValue::Float(spans.len() as f64)
                            ));
                        }
                    }
                }
            });
            
            Ok(handle)
        }
        
        /// Export spans to Zipkin
        async fn do_export_spans(
            client: &reqwest::Client,
            config: &ExternalTracingConfig,
            spans: &[Span]
        ) -> ObservabilityResult<()> {
            // Convert Span to Zipkin format
            let zipkin_spans = convert_to_zipkin_format(spans, config);
            
            // Send to Zipkin
            let response = client
                .post(&config.endpoint_url)
                .json(&zipkin_spans)
                .send()
                .await
                .map_err(|e| ObservabilityError::External(format!("Failed to send spans to Zipkin: {}", e)))?;
            
            if !response.status().is_success() {
                return Err(ObservabilityError::External(
                    format!("Failed to export spans to Zipkin: HTTP {}", response.status())
                ));
            }
            
            Ok(())
        }
    }

    #[async_trait]
    impl SpanExporter for ZipkinExporter {
        async fn export_spans(&self, spans: Vec<Span>) -> ObservabilityResult<()> {
            // Add spans to buffer
            {
                let mut buffer = self.buffer.lock().unwrap();
                buffer.extend(spans.clone());
                
                // Check if we need to flush
                if buffer.len() >= self.config.max_buffer_size {
                    debug!("Buffer size exceeded, flushing {} spans", buffer.len());
                    
                    // Take all spans from buffer
                    let spans_to_flush = std::mem::take(&mut *buffer);
                    
                    // Clone the values we need inside the tokio::spawn
                    let client = self.client.clone();
                    let config = self.config.clone();
                    
                    // Export the spans in the background
                    tokio::spawn(async move {
                        if let Err(e) = Self::do_export_spans(&client, &config, &spans_to_flush).await {
                            error!("Failed to export spans: {}", e);
                        }
                    });
                }
            }
            
            Ok(())
        }
        
        async fn shutdown(&self) -> ObservabilityResult<()> {
            // Flush any remaining spans
            let spans = {
                let mut buffer = self.buffer.lock().unwrap();
                std::mem::take(&mut *buffer)
            };
            
            if !spans.is_empty() {
                debug!("Flushing {} spans to Zipkin during shutdown", spans.len());
                Self::do_export_spans(&self.client, &self.config, &spans).await?;
            }
            
            Ok(())
        }
    }

    /// Enhanced tracer with external system integration
    pub struct ExternalTracer<E: SpanExporter> {
        /// Inner tracer for span management
        inner: Tracer,
        
        /// Exporter for sending spans to external systems
        pub exporter: E,
        
        /// Handle to the background flush task
        _flush_task: Option<tokio::task::JoinHandle<()>>,
        
        /// Whether this tracer has been initialized
        initialized: std::sync::atomic::AtomicBool,
    }

    impl<E: SpanExporter> ExternalTracer<E> {
        /// Create a new external tracer
        pub fn new(exporter: E) -> Self {
            Self {
                inner: Tracer::new(),
                exporter,
                _flush_task: None,
                initialized: std::sync::atomic::AtomicBool::new(false),
            }
        }
        
        /// Initialize the tracer
        pub async fn initialize(&mut self) -> ObservabilityResult<()> {
            // Initialize inner tracer
            self.inner.initialize()?;
            
            // Set initialized flag
            self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            
            Ok(())
        }
        
        /// Start a new span
        pub fn start_span(&self, name: impl Into<String>) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
            self.inner.start_span(name)
        }
        
        /// Start a new span with parent
        pub fn start_span_with_parent(
            &self,
            name: impl Into<String>,
            parent_span: Option<Arc<Mutex<ActiveSpan>>>,
        ) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
            self.inner.start_span_with_parent(name, parent_span)
        }
        
        /// Get the current span
        pub fn current_span(&self) -> ObservabilityResult<Option<Arc<Mutex<ActiveSpan>>>> {
            self.inner.current_span()
        }
        
        /// Export completed spans to external system
        pub async fn export_completed_spans(&self) -> ObservabilityResult<usize> {
            // Get all spans
            let spans = {
                let spans_lock = self.inner.spans.lock().unwrap();
                spans_lock.values()
                    .filter(|span| span.end_time.is_some()) // Only completed spans
                    .cloned()
                    .collect::<Vec<_>>()
            };
            
            // Export spans
            if !spans.is_empty() {
                self.exporter.export_spans(spans.clone()).await?;
            }
            
            Ok(spans.len())
        }
        
        /// Shutdown the tracer
        pub async fn shutdown(&self) -> ObservabilityResult<()> {
            // Export any completed spans
            let _ = self.export_completed_spans().await;
            
            // Shutdown the exporter
            self.exporter.shutdown().await
        }
    }
}

// Helper functions for conversion

/// Convert spans to OpenTelemetry format
fn convert_to_otlp_format(spans: &[Span], config: &external::ExternalTracingConfig) -> serde_json::Value {
    // Implementation note: this is a simplified version that focuses on the structure
    // For a complete implementation, you'd need to follow the OpenTelemetry OTLP format exactly
    
    let span_values: Vec<serde_json::Value> = spans.iter().map(|span| {
        let mut attributes = serde_json::Map::new();
        
        // Add standard attributes if configured
        if config.add_standard_attributes {
            attributes.insert("service.name".to_string(), serde_json::Value::String(config.service_name.clone()));
            attributes.insert("environment".to_string(), serde_json::Value::String(config.environment.clone()));
        }
        
        // Add span attributes
        for (k, v) in span.attributes() {
            attributes.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        
        // Calculate duration
        let duration_nanos = span.duration()
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        
        // Create span representation
        serde_json::json!({
            "name": span.name(),
            "trace_id": span.trace_id(),
            "span_id": span.id(),
            "parent_span_id": span.parent_id(),
            "start_time_unix_nano": 0, // Would need real timestamp conversion in production
            "end_time_unix_nano": duration_nanos,
            "attributes": attributes,
            "status": {
                "code": match span.status() {
                    SpanStatus::Success => "OK",
                    SpanStatus::Error => "ERROR",
                    SpanStatus::Running => "UNSET",
                }
            },
            "events": span.events().iter().map(|event| {
                serde_json::json!({
                    "name": event.name,
                    "timestamp": 0, // Would need real timestamp conversion in production
                    "attributes": event.attributes
                })
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    serde_json::json!({
        "resourceSpans": [
            {
                "resource": {
                    "attributes": {
                        "service.name": config.service_name
                    }
                },
                "scopeSpans": [
                    {
                        "scope": {
                            "name": "squirrel-mcp"
                        },
                        "spans": span_values
                    }
                ]
            }
        ]
    })
}

/// Convert spans to Zipkin format
fn convert_to_zipkin_format(spans: &[Span], config: &external::ExternalTracingConfig) -> serde_json::Value {
    // Implementation note: this is a simplified version that focuses on the structure
    // For a complete implementation, you'd need to follow the Zipkin format exactly
    
    let span_values: Vec<serde_json::Value> = spans.iter().map(|span| {
        let mut tags = serde_json::Map::new();
        
        // Add standard tags if configured
        if config.add_standard_attributes {
            tags.insert("service.name".to_string(), serde_json::Value::String(config.service_name.clone()));
            tags.insert("environment".to_string(), serde_json::Value::String(config.environment.clone()));
        }
        
        // Add span attributes as tags
        for (k, v) in span.attributes() {
            tags.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        
        // Calculate duration
        let duration_micros = span.duration()
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        
        // Create Zipkin span representation
        serde_json::json!({
            "name": span.name(),
            "traceId": span.trace_id(),
            "id": span.id(),
            "parentId": span.parent_id(),
            "timestamp": 0, // Would need real timestamp conversion in production
            "duration": duration_micros,
            "localEndpoint": {
                "serviceName": config.service_name
            },
            "tags": tags,
            "annotations": span.events().iter().map(|event| {
                serde_json::json!({
                    "value": event.name,
                    "timestamp": 0 // Would need real timestamp conversion in production
                })
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    serde_json::Value::Array(span_values)
} 