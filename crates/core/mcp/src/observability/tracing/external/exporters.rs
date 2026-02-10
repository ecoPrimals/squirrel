// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! External Span Exporters
//!
//! This module provides concrete implementations of span exporters for
//! different external tracing systems.

use std::sync::{Arc, Mutex};
// Phase 4: Removed async_trait - using native async fn in traits
use std::future::Future;
use tokio;

use crate::observability::{ObservabilityError, ObservabilityResult};
use crate::observability::tracing::types::Span;
use crate::monitoring::MetricsCollector;
use super::config::ExternalTracingConfig;
use super::traits::SpanExporter;
use super::converters::{convert_to_otlp_format, convert_to_zipkin_format};

/// OpenTelemetry exporter for OTLP protocol
#[derive(Debug, Clone)]
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
        let timeout = std::time::Duration::from_secs(
            std::env::var("OPENTELEMETRY_EXPORT_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30)
        );
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
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
                    let mut buffer_guard = buffer.lock().expect("OpenTelemetryExporter: buffer lock poisoned");
                    if buffer_guard.is_empty() {
                        continue;
                    }
                    
                    tracing::debug!("Flushing {} spans to OpenTelemetry", buffer_guard.len());
                    std::mem::take(&mut *buffer_guard)
                };
                
                // Export the spans
                if let Err(e) = Self::do_export_spans(&client, &config, &spans).await {
                    tracing::error!("Failed to export spans to OpenTelemetry: {}", e);
                    
                    // Return spans to buffer
                    let mut buffer_guard = buffer.lock().expect("OpenTelemetryExporter: buffer lock poisoned");
                    buffer_guard.extend(spans);
                    
                    // Update metrics
                    if let Some(metrics) = &metrics {
                        metrics.increment_counter("tracing.otlp.export.failures");
                    }
                } else {
                    // Update metrics
                    if let Some(metrics) = &metrics {
                        metrics.increment_counter("tracing.otlp.export.success");
                    }
                }
            }
        });
        
        Ok(handle)
    }

    /// Export spans to OpenTelemetry
    async fn do_export_spans(
        client: &reqwest::Client,
        config: &ExternalTracingConfig,
        spans: &[Span]
    ) -> ObservabilityResult<()> {
        // Convert Span to OTLP format
        let otlp_data = convert_to_otlp_format(spans, config);
        
        // Build request
        let mut request = client
            .post(&config.endpoint_url)
            .json(&otlp_data);
        
        // Add auth token if provided
        if let Some(token) = &config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        // Send to OpenTelemetry
        let response = request
            .send()
            .await
            .map_err(|e| ObservabilityError::External(format!("Failed to send spans to OpenTelemetry: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ObservabilityError::External(
                format!("Failed to export spans to OpenTelemetry: HTTP {}", response.status())
            ));
        }
        
        Ok(())
    }
}

impl SpanExporter for OpenTelemetryExporter {
    fn export_spans(&self, spans: Vec<Span>) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let buffer = self.buffer.clone();
        let config = self.config.clone();
        let client = self.client.clone();
        async move {
            // Add spans to buffer
            {
                let mut buffer_guard = buffer.lock().expect("OpenTelemetryExporter: buffer lock poisoned");
                buffer_guard.extend(spans.clone());
                
                // Check if we need to flush
                if buffer_guard.len() >= config.max_buffer_size {
                    tracing::debug!("Buffer size exceeded, flushing {} spans", buffer_guard.len());
                    
                    // Take all spans from buffer
                    let spans_to_flush = std::mem::take(&mut *buffer_guard);
                    
                    // Clone the values we need inside the tokio::spawn
                    let client_clone = client.clone();
                    let config_clone = config.clone();
                    
                    // Export the spans in the background
                    tokio::spawn(async move {
                        if let Err(e) = OpenTelemetryExporter::do_export_spans(&client_clone, &config_clone, &spans_to_flush).await {
                            tracing::error!("Failed to export spans: {}", e);
                        }
                    });
                }
            }
            
            Ok(())
        }
    }

    fn shutdown(&self) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let buffer = self.buffer.clone();
        let client = self.client.clone();
        let config = self.config.clone();
        async move {
            // Flush any remaining spans
            let spans = {
                let mut buffer_guard = buffer.lock().expect("OpenTelemetryExporter: buffer lock poisoned");
                std::mem::take(&mut *buffer_guard)
            };
            
            if !spans.is_empty() {
                tracing::debug!("Flushing {} spans to OpenTelemetry during shutdown", spans.len());
                OpenTelemetryExporter::do_export_spans(&client, &config, &spans).await?;
            }
            
            Ok(())
        }
    }
}

/// Jaeger exporter using OpenTelemetry protocol
#[derive(Debug)]
pub struct JaegerExporter {
    /// Configuration for the exporter
    config: ExternalTracingConfig,
    
    /// OpenTelemetry exporter
    otlp_exporter: OpenTelemetryExporter,
}

impl JaegerExporter {
    /// Create a new Jaeger exporter
    pub fn new(mut config: ExternalTracingConfig) -> Self {
        // Multi-tier Jaeger endpoint resolution
        if config.endpoint_url == ExternalTracingConfig::default().endpoint_url {
            config.endpoint_url = std::env::var("JAEGER_ENDPOINT")
                .or_else(|_| std::env::var("TRACING_ENDPOINT"))
                .unwrap_or_else(|_| {
                    let port = std::env::var("JAEGER_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(14268);  // Default Jaeger collector port
                    format!("http://localhost:{}/api/traces", port)
                });
        }
        
        let otlp_exporter = OpenTelemetryExporter::new(config.clone());
        
        Self {
            config,
            otlp_exporter,
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

impl SpanExporter for JaegerExporter {
    fn export_spans(&self, spans: Vec<Span>) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let otlp_exporter = self.otlp_exporter.clone();
        async move {
            otlp_exporter.export_spans(spans).await
        }
    }

    fn shutdown(&self) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let otlp_exporter = self.otlp_exporter.clone();
        async move {
            otlp_exporter.shutdown().await
        }
    }
}

/// Zipkin exporter for Zipkin tracing system
#[derive(Debug)]
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
        // Multi-tier Zipkin endpoint resolution
        if config.endpoint_url == ExternalTracingConfig::default().endpoint_url {
            config.endpoint_url = std::env::var("ZIPKIN_ENDPOINT")
                .or_else(|_| std::env::var("TRACING_ENDPOINT"))
                .unwrap_or_else(|_| {
                    let port = std::env::var("ZIPKIN_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(9411);  // Default Zipkin collector port
                    format!("http://localhost:{}/api/v2/spans", port)
                });
        }
        
        let timeout = std::time::Duration::from_secs(
            std::env::var("ZIPKIN_EXPORT_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30)
        );
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
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
                    let mut buffer_guard = buffer.lock().expect("ZipkinExporter: buffer lock poisoned");
                    if buffer_guard.is_empty() {
                        continue;
                    }
                    
                    tracing::debug!("Flushing {} spans to Zipkin", buffer_guard.len());
                    std::mem::take(&mut *buffer_guard)
                };
                
                // Export the spans
                if let Err(e) = Self::do_export_spans(&client, &config, &spans).await {
                    tracing::error!("Failed to export spans to Zipkin: {}", e);
                    
                    // Return spans to buffer
                    let mut buffer_guard = buffer.lock().expect("ZipkinExporter: buffer lock poisoned");
                    buffer_guard.extend(spans);
                    
                    // Update metrics
                    if let Some(metrics) = &metrics {
                        metrics.increment_counter("tracing.zipkin.export.failures");
                    }
                } else {
                    // Update metrics
                    if let Some(metrics) = &metrics {
                        metrics.increment_counter("tracing.zipkin.export.success");
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

impl SpanExporter for ZipkinExporter {
    fn export_spans(&self, spans: Vec<Span>) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let buffer = self.buffer.clone();
        let config = self.config.clone();
        let client = self.client.clone();
        async move {
            // Add spans to buffer
            {
                let mut buffer_guard = buffer.lock().expect("ZipkinExporter: buffer lock poisoned");
                buffer_guard.extend(spans.clone());
                
                // Check if we need to flush
                if buffer_guard.len() >= config.max_buffer_size {
                    tracing::debug!("Buffer size exceeded, flushing {} spans", buffer_guard.len());
                    
                    // Take all spans from buffer
                    let spans_to_flush = std::mem::take(&mut *buffer_guard);
                    
                    // Clone the values we need inside the tokio::spawn
                    let client_clone = client.clone();
                    let config_clone = config.clone();
                    
                    // Export the spans in the background
                    tokio::spawn(async move {
                        if let Err(e) = ZipkinExporter::do_export_spans(&client_clone, &config_clone, &spans_to_flush).await {
                            tracing::error!("Failed to export spans: {}", e);
                        }
                    });
                }
            }
            
            Ok(())
        }
    }

    fn shutdown(&self) -> impl Future<Output = ObservabilityResult<()>> + Send {
        let buffer = self.buffer.clone();
        let client = self.client.clone();
        let config = self.config.clone();
        async move {
            // Flush any remaining spans
            let spans = {
                let mut buffer_guard = buffer.lock().expect("ZipkinExporter: buffer lock poisoned");
                std::mem::take(&mut *buffer_guard)
            };
            
            if !spans.is_empty() {
                tracing::debug!("Flushing {} spans to Zipkin during shutdown", spans.len());
                ZipkinExporter::do_export_spans(&client, &config, &spans).await?;
            }
            
            Ok(())
        }
    }
} 