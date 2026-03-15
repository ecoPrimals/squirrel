// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! # Dashboard Integration
//! 
//! This module provides integration between the MCP tracing system and
//! the api-server for visualizing traces.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use reqwest;
// Phase 4: Removed async_trait - using native async fn in traits
use std::future::Future;
use serde::{Serialize, Deserialize};
use tracing::{debug, info};
use std::collections::HashMap;
use tokio::time::interval;


use squirrel_interfaces::tracing::{
    TraceDataConsumer, TraceConfig
};

use crate::observability::{
    metrics::MetricsRegistry, 
    tracing::Tracer,
    health::HealthChecker,
    alerting::{AlertManager, AlertState},
    ObservabilityResult
};

/// Configuration for dashboard integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIntegrationConfig {
    /// URL of the dashboard service
    pub dashboard_url: String,
    
    /// Authentication token, if needed
    pub auth_token: Option<String>,
    
    /// Interval for sending metrics to dashboard (in seconds)
    pub metrics_interval: u64,
    
    /// Interval for sending trace data to dashboard (in seconds)
    pub traces_interval: u64,
    
    /// Interval for sending health data to dashboard (in seconds)
    pub health_interval: u64,
    
    /// Interval for sending alerts to dashboard (in seconds)
    pub alerts_interval: u64,
    
    /// Service name for identification in the dashboard
    pub service_name: String,
    
    /// Environment (dev, staging, prod)
    pub environment: String,
    
    /// Maximum number of traces to send in one batch
    pub max_traces_per_batch: usize,
    
    /// Maximum number of metrics to send in one batch
    pub max_metrics_per_batch: usize,
}

impl Default for DashboardIntegrationConfig {
    fn default() -> Self {
        // Multi-tier dashboard observability API resolution
        let dashboard_url = std::env::var("DASHBOARD_OBSERVABILITY_URL")
            .or_else(|_| std::env::var("UI_ENDPOINT").map(|e| format!("{}/api/observability", e)))
            .unwrap_or_else(|_| {
                let port = std::env::var("WEB_UI_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8080);  // Default dashboard API port
                format!("http://localhost:{}/api/observability", port)
            });

        Self {
            dashboard_url,
            auth_token: None,
            metrics_interval: 15,
            traces_interval: 10,
            health_interval: 30,
            alerts_interval: 5,
            service_name: "mcp-service".to_string(),
            environment: "development".to_string(),
            max_traces_per_batch: 100,
            max_metrics_per_batch: 250,
        }
    }
}

/// Common trait for dashboard data exporters
pub trait DashboardDataExporter: Send + Sync {
    /// Export metrics to the dashboard
    fn export_metrics(&self, metrics: Vec<MetricData>) -> impl Future<Output = ObservabilityResult<()>> + Send;
    
    /// Export traces to the dashboard
    fn export_traces(&self, traces: Vec<TraceData>) -> impl Future<Output = ObservabilityResult<()>> + Send;
    
    /// Export health data to the dashboard
    fn export_health(&self, health: HealthData) -> impl Future<Output = ObservabilityResult<()>> + Send;
    
    /// Export alerts to the dashboard
    fn export_alerts(&self, alerts: Vec<AlertData>) -> impl Future<Output = ObservabilityResult<()>> + Send;
    
    /// Shutdown the exporter
    fn shutdown(&self) -> impl Future<Output = ObservabilityResult<()>> + Send;
}

/// Data structure for metric information sent to dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    /// Name of the metric
    pub name: String,
    
    /// Type of the metric (counter, gauge, histogram)
    pub metric_type: String,
    
    /// Value of the metric
    pub value: serde_json::Value,
    
    /// Labels/dimensions for the metric
    pub labels: HashMap<String, String>,
    
    /// Timestamp of the metric (milliseconds since epoch)
    pub timestamp: u64,
    
    /// Service name that reported the metric
    pub service: String,
}

/// Data structure for trace information sent to dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    /// Trace ID
    pub trace_id: String,
    
    /// Spans in the trace
    pub spans: Vec<SpanData>,
    
    /// Service name that reported the trace
    pub service: String,
    
    /// Timestamp of the trace (milliseconds since epoch)
    pub timestamp: u64,
}

/// Data structure for span information in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanData {
    /// Span ID
    pub span_id: String,
    
    /// Parent span ID, if any
    pub parent_id: Option<String>,
    
    /// Name of the span
    pub name: String,
    
    /// Start time (milliseconds since epoch)
    pub start_time: u64,
    
    /// End time (milliseconds since epoch), if completed
    pub end_time: Option<u64>,
    
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Status of the span (running, success, error)
    pub status: String,
    
    /// Attributes for the span
    pub attributes: HashMap<String, String>,
}

/// Data structure for health information sent to dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthData {
    /// Overall system health status
    pub system_status: String,
    
    /// Component health statuses
    pub components: Vec<ComponentHealthData>,
    
    /// Service name that reported the health
    pub service: String,
    
    /// Timestamp of the health check (milliseconds since epoch)
    pub timestamp: u64,
}

/// Data structure for component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthData {
    /// Component name
    pub name: String,
    
    /// Health status
    pub status: String,
    
    /// Message explaining the health status
    pub message: Option<String>,
    
    /// Last check timestamp (milliseconds since epoch)
    pub last_check: u64,
}

/// Data structure for alert information sent to dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    /// Alert ID
    pub alert_id: String,
    
    /// Alert name/type
    pub name: String,
    
    /// Severity level
    pub severity: String,
    
    /// Message explaining the alert
    pub message: String,
    
    /// Component that triggered the alert
    pub component: String,
    
    /// Timestamp when the alert was triggered (milliseconds since epoch)
    pub timestamp: u64,
    
    /// Service name that reported the alert
    pub service: String,
    
    /// Whether the alert is active
    pub active: bool,
}

/// Main adapter for dashboard integration
pub struct DashboardIntegrationAdapter {
    /// Configuration for dashboard integration
    config: DashboardIntegrationConfig,
    
    /// HTTP client for sending data to dashboard
    client: reqwest::Client,
    
    /// Reference to metrics registry
    metrics_registry: Arc<MetricsRegistry>,
    
    /// Reference to tracer
    tracer: Arc<Tracer>,
    
    /// Reference to health checker
    health_checker: Arc<HealthChecker>,
    
    /// Reference to alert manager
    alert_manager: Arc<AlertManager>,
    
    /// Tasks for periodic data sending
    _tasks: RwLock<Vec<tokio::task::JoinHandle<()>>>,
}

impl DashboardIntegrationAdapter {
    /// Create a new dashboard integration adapter
    pub fn new(
        config: DashboardIntegrationConfig,
        metrics_registry: Arc<MetricsRegistry>,
        tracer: Arc<Tracer>,
        health_checker: Arc<HealthChecker>,
        alert_manager: Arc<AlertManager>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
            
        Self {
            config,
            client,
            metrics_registry,
            tracer,
            health_checker,
            alert_manager,
            _tasks: RwLock::new(Vec::new()),
        }
    }
    
    /// Start background tasks for sending data to dashboard
    pub async fn start_background_tasks(&self) -> ObservabilityResult<()> {
        // Start metrics export task
        let metrics_task = self.start_metrics_task()?;
        
        // Start traces export task
        let traces_task = self.start_traces_task()?;
        
        // Start health export task
        let health_task = self.start_health_task()?;
        
        // Start alerts export task
        let alerts_task = self.start_alerts_task()?;
        
        // Store tasks
        let mut tasks = self._tasks.write().await;
        
        tasks.push(metrics_task);
        tasks.push(traces_task);
        tasks.push(health_task);
        tasks.push(alerts_task);
        
        Ok(())
    }
    
    /// Start background task for exporting metrics
    fn start_metrics_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let metrics_registry = self.metrics_registry.clone();
        
        let task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.metrics_interval));
            
            loop {
                interval.tick().await;
                
                // Collect metrics
                if let Err(e) = Self::export_metrics_to_dashboard(&client, &config, &metrics_registry).await {
                    eprintln!("Error exporting metrics to dashboard: {}", e);
                }
            }
        });
        
        Ok(task)
    }
    
    /// Start background task for exporting traces
    fn start_traces_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let tracer = self.tracer.clone();
        
        let task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.traces_interval));
            
            loop {
                interval.tick().await;
                
                // Collect traces
                if let Err(e) = Self::export_traces_to_dashboard(&client, &config, &tracer).await {
                    eprintln!("Error exporting traces to dashboard: {}", e);
                }
            }
        });
        
        Ok(task)
    }
    
    /// Start background task for exporting health data
    fn start_health_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let health_checker = self.health_checker.clone();
        
        let task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.health_interval));
            
            loop {
                interval.tick().await;
                
                // Collect health data
                if let Err(e) = Self::export_health_to_dashboard(&client, &config, &health_checker).await {
                    eprintln!("Error exporting health data to dashboard: {}", e);
                }
            }
        });
        
        Ok(task)
    }
    
    /// Start background task for exporting alerts
    fn start_alerts_task(&self) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let alert_manager = self.alert_manager.clone();
        
        let task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.alerts_interval));
            
            loop {
                interval.tick().await;
                
                // Collect alerts
                if let Err(e) = Self::export_alerts_to_dashboard(&client, &config, &alert_manager).await {
                    eprintln!("Error exporting alerts to dashboard: {}", e);
                }
            }
        });
        
        Ok(task)
    }
    
    /// Export metrics to the dashboard
    async fn export_metrics_to_dashboard(
        client: &reqwest::Client,
        config: &DashboardIntegrationConfig,
        metrics_registry: &MetricsRegistry,
    ) -> ObservabilityResult<()> {
        use crate::observability::metrics::{MetricSnapshot, MetricValue};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Collect all metrics from the registry
        let mut metric_data = Vec::new();
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Collect counters
        if let Ok(counter_names) = metrics_registry.counter_names() {
            for name in counter_names {
                if let Ok(Some(counter)) = metrics_registry.get_counter(&name) {
                    if let Ok(value) = counter.value() {
                        let labels = counter.labels().clone();
                        metric_data.push(MetricData {
                            name: name.clone(),
                            metric_type: "counter".to_string(),
                            value: serde_json::Value::Number(value.into()),
                            labels,
                            timestamp: timestamp_ms,
                            service: config.service_name.clone(),
                        });
                    }
                }
            }
        }
        
        // Collect gauges
        if let Ok(gauge_names) = metrics_registry.gauge_names() {
            for name in gauge_names {
                if let Ok(Some(gauge)) = metrics_registry.get_gauge(&name) {
                    if let Ok(value) = gauge.value() {
                        let labels = gauge.labels().clone();
                        metric_data.push(MetricData {
                            name: name.clone(),
                            metric_type: "gauge".to_string(),
                            value: serde_json::Value::Number(
                                serde_json::Number::from_f64(value).unwrap_or(serde_json::Number::from(0))
                            ),
                            labels,
                            timestamp: timestamp_ms,
                            service: config.service_name.clone(),
                        });
                    }
                }
            }
        }
        
        // Collect histograms
        if let Ok(histogram_names) = metrics_registry.histogram_names() {
            for name in histogram_names {
                if let Ok(Some(histogram)) = metrics_registry.get_histogram(&name) {
                    if let Ok(buckets) = histogram.buckets() {
                        if let Ok(count) = histogram.count() {
                            if let Ok(sum) = histogram.sum() {
                                let labels = histogram.labels().clone();
                                let histogram_value = serde_json::json!({
                                    "buckets": buckets.iter().map(|b| serde_json::json!({
                                        "bound": b.bound,
                                        "count": b.count
                                    })).collect::<Vec<_>>(),
                                    "count": count,
                                    "sum": sum
                                });
                                metric_data.push(MetricData {
                                    name: name.clone(),
                                    metric_type: "histogram".to_string(),
                                    value: histogram_value,
                                    labels,
                                    timestamp: timestamp_ms,
                                    service: config.service_name.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Batch and send metrics
        for chunk in metric_data.chunks(config.max_metrics_per_batch) {
            let url = format!("{}/metrics", config.dashboard_url);
            let mut request = client.post(&url).json(chunk);
            
            if let Some(token) = &config.auth_token {
                request = request.bearer_auth(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Exported {} metrics to dashboard", chunk.len());
                    } else {
                        tracing::warn!("Failed to export metrics: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    tracing::warn!("Error exporting metrics to dashboard: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Export traces to the dashboard
    async fn export_traces_to_dashboard(
        client: &reqwest::Client,
        config: &DashboardIntegrationConfig,
        tracer: &Tracer,
    ) -> ObservabilityResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Export completed spans as snapshots
        let span_snapshots = tracer.export_spans_batch(config.max_traces_per_batch).await?;
        
        if span_snapshots.is_empty() {
            return Ok(());
        }
        
        // Group spans by trace_id
        let mut traces: std::collections::HashMap<String, Vec<SpanData>> = std::collections::HashMap::new();
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        for snapshot in span_snapshots {
            let span_data = SpanData {
                span_id: snapshot.id.clone(),
                parent_id: snapshot.parent_id.clone(),
                name: snapshot.name.clone(),
                start_time: snapshot.start_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                end_time: snapshot.end_time.map(|et| {
                    et.duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                }),
                duration_ms: snapshot.duration_ms,
                status: format!("{:?}", snapshot.status),
                attributes: snapshot.attributes,
            };
            
            traces.entry(snapshot.trace_id.clone())
                .or_insert_with(Vec::new)
                .push(span_data);
        }
        
        // Convert to TraceData format and send
        let mut trace_data_vec = Vec::new();
        for (trace_id, spans) in traces {
            trace_data_vec.push(TraceData {
                trace_id,
                spans,
                service: config.service_name.clone(),
                timestamp: timestamp_ms,
            });
        }
        
        // Send traces in batches
        for chunk in trace_data_vec.chunks(config.max_traces_per_batch) {
            let url = format!("{}/traces", config.dashboard_url);
            let mut request = client.post(&url).json(chunk);
            
            if let Some(token) = &config.auth_token {
                request = request.bearer_auth(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Exported {} traces to dashboard", chunk.len());
                    } else {
                        tracing::warn!("Failed to export traces: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    tracing::warn!("Error exporting traces to dashboard: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Export health data to the dashboard
    async fn export_health_to_dashboard(
        client: &reqwest::Client,
        config: &DashboardIntegrationConfig,
        health_checker: &HealthChecker,
    ) -> ObservabilityResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Get health report
        let health_report = health_checker.get_health_report()?;
        
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Convert component health to ComponentHealthData
        let components: Vec<ComponentHealthData> = health_report.component_statuses
            .iter()
            .map(|(name, component_health)| {
                ComponentHealthData {
                    name: name.clone(),
                    status: format!("{:?}", component_health.status),
                    message: component_health.message.clone(),
                    last_check: component_health.last_check
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() * 1000, // Convert to milliseconds
                }
            })
            .collect();
        
        let health_data = HealthData {
            system_status: format!("{:?}", health_report.overall_status),
            components,
            service: config.service_name.clone(),
            timestamp: timestamp_ms,
        };
        
        // Send health data to dashboard
        let url = format!("{}/health", config.dashboard_url);
        let mut request = client.post(&url).json(&health_data);
        
        if let Some(token) = &config.auth_token {
            request = request.bearer_auth(token);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Exported health data to dashboard");
                } else {
                    tracing::warn!("Failed to export health data: HTTP {}", response.status());
                }
            }
            Err(e) => {
                tracing::warn!("Error exporting health data to dashboard: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Export alerts to the dashboard
    async fn export_alerts_to_dashboard(
        client: &reqwest::Client,
        config: &DashboardIntegrationConfig,
        alert_manager: &AlertManager,
    ) -> ObservabilityResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Get all recent alerts (active and recent resolved)
        let alerts = alert_manager.get_all_recent_alerts().await?;
        
        if alerts.is_empty() {
            return Ok(());
        }
        
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Convert alerts to AlertData format
        let alert_data: Vec<AlertData> = alerts
            .iter()
            .map(|alert| {
                AlertData {
                    alert_id: alert.id().to_string(),
                    name: alert.name().to_string(),
                    severity: format!("{:?}", alert.severity()),
                    message: alert.message().to_string(),
                    component: alert.source().to_string(),
                    timestamp: alert.created_at()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() * 1000, // Convert to milliseconds
                    service: config.service_name.clone(),
                    active: alert.state() == AlertState::Active,
                }
            })
            .collect();
        
        // Send alerts to dashboard
        let url = format!("{}/alerts", config.dashboard_url);
        let mut request = client.post(&url).json(&alert_data);
        
        if let Some(token) = &config.auth_token {
            request = request.bearer_auth(token);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Exported {} alerts to dashboard", alert_data.len());
                } else {
                    tracing::warn!("Failed to export alerts: HTTP {}", response.status());
                }
            }
            Err(e) => {
                tracing::warn!("Error exporting alerts to dashboard: {}", e);
            }
        }
        
        Ok(())
    }
}

/// Create a new dashboard integration with default configuration
pub fn create_default_dashboard_integration() -> ObservabilityResult<Arc<DashboardIntegrationAdapter>> {
    let config = DashboardIntegrationConfig::default();
    let metrics_registry = Arc::new(MetricsRegistry::new());
    let tracer = Arc::new(Tracer::new());
    let health_checker = Arc::new(HealthChecker::new());
    let alert_manager = Arc::new(AlertManager::new());

    let adapter = DashboardIntegrationAdapter::new(
        config,
        metrics_registry,
        tracer,
        health_checker,
        alert_manager,
    );

    Ok(Arc::new(adapter))
}

#[cfg(feature = "dashboard")]
/// Dashboard consumer that forwards traces to api-server
pub struct DashboardCoreConsumer {
    // This would be replaced with the actual api-server client
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
impl TraceDataConsumer for DashboardCoreConsumer {
    fn consume_trace_data(&self, trace_data: squirrel_interfaces::tracing::TraceData) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send {
        async move {
            // In a real implementation, this would convert the trace data
            // to the api-server format and send it to the dashboard
            
            info!("Would send trace data to api-server: {} spans in trace", 
                trace_data.spans.len());
                
            // For testing, just log the trace data
            for span in &trace_data.spans {
                debug!("Span: {} - {} (parent: {:?})", 
                    span.name, span.id, span.parent_id);
            }
            
            Ok(())
        }
    }
}

#[cfg(feature = "dashboard")]
/// Create a dashboard consumer that forwards traces to api-server
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