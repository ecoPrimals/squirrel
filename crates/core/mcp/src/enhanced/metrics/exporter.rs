// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics Exporters
//!
//! This module provides exporters for sending metrics to external monitoring
//! and observability systems including Prometheus, InfluxDB, DataDog, and others.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{Result, types::MCPError};
use super::aggregator::AggregatedMetrics;
use super::ExportDestination;

/// Metrics exporter trait
#[async_trait::async_trait]
pub trait MetricsExporter: Send + Sync + std::fmt::Debug {
    /// Export metrics to the destination
    async fn export_metrics(&self, metrics: &AggregatedMetrics) -> Result<ExportResult>;
    
    /// Test connection to the destination
    async fn test_connection(&self) -> Result<bool>;
    
    /// Get exporter name
    fn exporter_name(&self) -> &str;
    
    /// Get exporter type
    fn exporter_type(&self) -> &str;
    
    /// Get exporter capabilities
    fn capabilities(&self) -> Vec<ExporterCapability>;
    
    /// Start the exporter (if it needs background processing)
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    /// Stop the exporter
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    
    /// Get exporter health status
    async fn health_status(&self) -> ExporterHealth;
    
    /// Get export statistics
    async fn get_statistics(&self) -> ExportStatistics;
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Export success status
    pub success: bool,
    
    /// Export message
    pub message: String,
    
    /// Number of metrics exported
    pub metrics_exported: u64,
    
    /// Export duration
    pub duration: Duration,
    
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    
    /// Retry suggested
    pub retry_suggested: bool,
    
    /// Export metadata
    pub metadata: HashMap<String, String>,
}

/// Exporter capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExporterCapability {
    RealTime,
    Batch,
    Compression,
    Authentication,
    Retry,
    BufferedExport,
    MetricFiltering,
    CustomFormatting,
}

/// Exporter health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterHealth {
    /// Health status
    pub status: HealthStatus,
    
    /// Health score (0.0 to 1.0)
    pub score: f64,
    
    /// Last successful export
    pub last_success: Option<DateTime<Utc>>,
    
    /// Last export attempt
    pub last_attempt: Option<DateTime<Utc>>,
    
    /// Recent errors
    pub recent_errors: Vec<String>,
    
    /// Connection status
    pub connection_status: ConnectionStatus,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
    Unknown,
}

/// Export statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatistics {
    /// Total exports attempted
    pub total_exports: u64,
    
    /// Successful exports
    pub successful_exports: u64,
    
    /// Failed exports
    pub failed_exports: u64,
    
    /// Total metrics exported
    pub total_metrics_exported: u64,
    
    /// Average export duration
    pub avg_export_duration: Duration,
    
    /// Last export duration
    pub last_export_duration: Duration,
    
    /// Export rate (exports per minute)
    pub export_rate: f64,
    
    /// Error rate
    pub error_rate: f64,
}

/// Prometheus metrics exporter
#[derive(Debug)]
pub struct PrometheusExporter {
    /// Export destination configuration
    config: ExportDestination,
    
    /// HTTP client
    client: Arc<reqwest::Client>,
    
    /// Export statistics
    statistics: Arc<Mutex<ExportStatistics>>,
    
    /// Exporter state
    state: Arc<RwLock<ExporterState>>,
    
    /// Metric formatter
    formatter: PrometheusFormatter,
}

/// Prometheus metric formatter
#[derive(Debug)]
pub struct PrometheusFormatter {
    /// Metric name prefix
    prefix: String,
    
    /// Default labels
    default_labels: HashMap<String, String>,
}

/// InfluxDB metrics exporter
#[derive(Debug)]
pub struct InfluxDbExporter {
    /// Export destination configuration
    config: ExportDestination,
    
    /// HTTP client
    client: Arc<reqwest::Client>,
    
    /// Export statistics
    statistics: Arc<Mutex<ExportStatistics>>,
    
    /// Exporter state
    state: Arc<RwLock<ExporterState>>,
    
    /// Metric formatter
    formatter: InfluxDbFormatter,
}

/// InfluxDB line protocol formatter
#[derive(Debug)]
pub struct InfluxDbFormatter {
    /// Database name
    database: String,
    
    /// Measurement prefix
    measurement_prefix: String,
    
    /// Default tags
    default_tags: HashMap<String, String>,
}

/// DataDog metrics exporter
#[derive(Debug)]
pub struct DatadogExporter {
    /// Export destination configuration
    config: ExportDestination,
    
    /// HTTP client
    client: Arc<reqwest::Client>,
    
    /// Export statistics
    statistics: Arc<Mutex<ExportStatistics>>,
    
    /// Exporter state
    state: Arc<RwLock<ExporterState>>,
    
    /// API key
    api_key: String,
    
    /// Metric formatter
    formatter: DatadogFormatter,
}

/// DataDog metric formatter
#[derive(Debug)]
pub struct DatadogFormatter {
    /// Metric prefix
    prefix: String,
    
    /// Default tags
    default_tags: Vec<String>,
}

/// JSON file exporter
#[derive(Debug)]
pub struct JsonExporter {
    /// Export destination configuration
    config: ExportDestination,
    
    /// Export statistics
    statistics: Arc<Mutex<ExportStatistics>>,
    
    /// Exporter state
    state: Arc<RwLock<ExporterState>>,
    
    /// Output file path
    output_path: std::path::PathBuf,
    
    /// Formatter
    formatter: JsonFormatter,
}

/// JSON formatter
#[derive(Debug)]
pub struct JsonFormatter {
    /// Pretty print JSON
    pretty: bool,
    
    /// Include metadata
    include_metadata: bool,
}

/// Exporter state
#[derive(Debug, Clone)]
pub struct ExporterState {
    /// Exporter status
    pub status: ExporterStatus,
    
    /// Last export attempt
    pub last_export_attempt: Option<DateTime<Utc>>,
    
    /// Last successful export
    pub last_successful_export: Option<DateTime<Utc>>,
    
    /// Recent errors
    pub recent_errors: Vec<ExportError>,
    
    /// Export queue size (if applicable)
    pub queue_size: usize,
}

/// Exporter status
#[derive(Debug, Clone, PartialEq)]
pub enum ExporterStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// Export error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportError {
    /// Error message
    pub message: String,
    
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Error type
    pub error_type: String,
    
    /// Retry attempted
    pub retry_attempted: bool,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub async fn new(config: ExportDestination) -> Result<Self> {
        let client = Arc::new(reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?);
        
        let formatter = PrometheusFormatter {
            prefix: config.parameters.get("prefix").unwrap_or(&"mcp_".to_string()).clone(),
            default_labels: config.parameters.iter()
                .filter_map(|(k, v)| {
                    if k.starts_with("label_") {
                        Some((k.strip_prefix("label_").expect("starts_with checked above").to_string(), v.clone()))
                    } else {
                        None
                    }
                })
                .collect(),
        };
        
        let state = Arc::new(RwLock::new(ExporterState {
            status: ExporterStatus::Stopped,
            last_export_attempt: None,
            last_successful_export: None,
            recent_errors: Vec::new(),
            queue_size: 0,
        }));
        
        Ok(Self {
            config,
            client,
            statistics: Arc::new(Mutex::new(ExportStatistics::default())),
            state,
            formatter,
        })
    }
}

#[async_trait::async_trait]
impl MetricsExporter for PrometheusExporter {
    #[instrument(skip(self, metrics))]
    async fn export_metrics(&self, metrics: &AggregatedMetrics) -> Result<ExportResult> {
        let start_time = Instant::now();
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.last_export_attempt = Some(Utc::now());
        }
        
        // Format metrics for Prometheus
        let prometheus_metrics = self.formatter.format_metrics(metrics)?;
        
        // Get gateway URL
        let gateway_url = self.config.parameters
            .get("gateway_url")
            .ok_or_else(|| MCPError::Configuration("Missing gateway_url parameter".to_string()))?;
        
        let job_name = self.config.parameters
            .get("job_name")
            .unwrap_or(&"mcp_metrics".to_string());
        
        // Send to Prometheus Push Gateway
        let response = self.client
            .post(&format!("{}/metrics/job/{}", gateway_url, job_name))
            .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
            .body(prometheus_metrics)
            .send()
            .await;
        
        let duration = start_time.elapsed();
        
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Success
                    let result = ExportResult {
                        success: true,
                        message: "Metrics exported to Prometheus".to_string(),
                        metrics_exported: self.count_metrics(metrics),
                        duration,
                        exported_at: Utc::now(),
                        retry_suggested: false,
                        metadata: HashMap::new(),
                    };
                    
                    // Update state and statistics
                    {
                        let mut state = self.state.write().await;
                        state.last_successful_export = Some(Utc::now());
                    }
                    
                    self.update_statistics_success(duration).await;
                    
                    Ok(result)
                } else {
                    // HTTP error
                    let error_msg = format!("HTTP error: {}", resp.status());
                    self.handle_export_error(&error_msg).await;
                    
                    Ok(ExportResult {
                        success: false,
                        message: error_msg,
                        metrics_exported: 0,
                        duration,
                        exported_at: Utc::now(),
                        retry_suggested: true,
                        metadata: HashMap::new(),
                    })
                }
            }
            Err(e) => {
                // Network/request error
                let error_msg = format!("Request failed: {}", e);
                self.handle_export_error(&error_msg).await;
                
                Ok(ExportResult {
                    success: false,
                    message: error_msg,
                    metrics_exported: 0,
                    duration,
                    exported_at: Utc::now(),
                    retry_suggested: true,
                    metadata: HashMap::new(),
                })
            }
        }
    }
    
    async fn test_connection(&self) -> Result<bool> {
        let gateway_url = self.config.parameters
            .get("gateway_url")
            .ok_or_else(|| MCPError::Configuration("Missing gateway_url parameter".to_string()))?;
        
        match self.client.get(gateway_url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    fn exporter_name(&self) -> &str {
        &self.config.name
    }
    
    fn exporter_type(&self) -> &str {
        "prometheus"
    }
    
    fn capabilities(&self) -> Vec<ExporterCapability> {
        vec![
            ExporterCapability::RealTime,
            ExporterCapability::Compression,
            ExporterCapability::Retry,
            ExporterCapability::MetricFiltering,
        ]
    }
    
    async fn health_status(&self) -> ExporterHealth {
        let state = self.state.read().await;
        let connection_ok = self.test_connection().await.unwrap_or(false);
        
        let health_score = if connection_ok && state.status == ExporterStatus::Running {
            1.0
        } else if state.recent_errors.len() < 3 {
            0.5
        } else {
            0.0
        };
        
        ExporterHealth {
            status: if health_score > 0.8 {
                HealthStatus::Healthy
            } else if health_score > 0.3 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            },
            score: health_score,
            last_success: state.last_successful_export,
            last_attempt: state.last_export_attempt,
            recent_errors: state.recent_errors.iter()
                .take(5)
                .map(|e| e.message.clone())
                .collect(),
            connection_status: if connection_ok {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            },
        }
    }
    
    async fn get_statistics(&self) -> ExportStatistics {
        self.statistics.lock().await.clone()
    }
}

impl PrometheusExporter {
    /// Count metrics in aggregated metrics
    fn count_metrics(&self, metrics: &AggregatedMetrics) -> u64 {
        // Count all metric fields in the aggregated metrics
        let mut count = 0u64;
        
        // Overall performance metrics
        count += 7; // avg_response_time, p95, p99, throughput, error_rate, success_rate, etc.
        
        // Component performance metrics
        count += metrics.component_performance.len() as u64 * 7; // Each component has multiple metrics
        
        // Resource utilization metrics
        count += 10; // CPU, memory, disk, network metrics
        
        // Network statistics
        count += 8; // Connection and message stats
        
        // Error analysis
        count += 5; // Error rates and counts
        
        count
    }
    
    /// Handle export error
    async fn handle_export_error(&self, error_msg: &str) {
        let mut state = self.state.write().await;
        
        let export_error = ExportError {
            message: error_msg.to_string(),
            timestamp: Utc::now(),
            error_type: "export_failure".to_string(),
            retry_attempted: false,
        };
        
        state.recent_errors.push(export_error);
        
        // Keep only last 10 errors
        if state.recent_errors.len() > 10 {
            state.recent_errors.remove(0);
        }
        
        self.update_statistics_failure().await;
    }
    
    /// Update statistics for successful export
    async fn update_statistics_success(&self, duration: Duration) {
        let mut stats = self.statistics.lock().await;
        stats.total_exports += 1;
        stats.successful_exports += 1;
        stats.last_export_duration = duration;
        
        // Update average duration
        if stats.total_exports == 1 {
            stats.avg_export_duration = duration;
        } else {
            let total_duration = stats.avg_export_duration.as_millis() as u64 * (stats.total_exports - 1) + duration.as_millis() as u64;
            stats.avg_export_duration = Duration::from_millis(total_duration / stats.total_exports);
        }
        
        // Update error rate
        stats.error_rate = stats.failed_exports as f64 / stats.total_exports as f64;
    }
    
    /// Update statistics for failed export
    async fn update_statistics_failure(&self) {
        let mut stats = self.statistics.lock().await;
        stats.total_exports += 1;
        stats.failed_exports += 1;
        
        // Update error rate
        stats.error_rate = stats.failed_exports as f64 / stats.total_exports as f64;
    }
}

impl PrometheusFormatter {
    /// Format aggregated metrics for Prometheus
    fn format_metrics(&self, metrics: &AggregatedMetrics) -> Result<String> {
        let mut output = String::new();
        
        // Format overall performance metrics
        self.add_metric(&mut output, "avg_response_time_ms", metrics.overall_performance.avg_response_time.as_millis() as f64, &HashMap::new())?;
        self.add_metric(&mut output, "p95_response_time_ms", metrics.overall_performance.p95_response_time.as_millis() as f64, &HashMap::new())?;
        self.add_metric(&mut output, "p99_response_time_ms", metrics.overall_performance.p99_response_time.as_millis() as f64, &HashMap::new())?;
        self.add_metric(&mut output, "throughput_per_second", metrics.overall_performance.throughput_per_second, &HashMap::new())?;
        self.add_metric(&mut output, "error_rate", metrics.overall_performance.error_rate, &HashMap::new())?;
        self.add_metric(&mut output, "success_rate", metrics.overall_performance.success_rate, &HashMap::new())?;
        self.add_metric(&mut output, "active_connections", metrics.overall_performance.active_connections as f64, &HashMap::new())?;
        self.add_metric(&mut output, "health_score", metrics.overall_performance.health_score, &HashMap::new())?;
        
        // Format component performance metrics
        for (component, perf) in &metrics.component_performance {
            let mut labels = HashMap::new();
            labels.insert("component".to_string(), component.clone());
            
            self.add_metric(&mut output, "component_avg_execution_time_ms", perf.avg_execution_time.as_millis() as f64, &labels)?;
            self.add_metric(&mut output, "component_throughput", perf.throughput, &labels)?;
            self.add_metric(&mut output, "component_success_rate", perf.success_rate, &labels)?;
            self.add_metric(&mut output, "component_error_rate", perf.error_rate, &labels)?;
            self.add_metric(&mut output, "component_resource_usage", perf.resource_usage, &labels)?;
            self.add_metric(&mut output, "component_health_score", perf.health_score, &labels)?;
        }
        
        // Format resource utilization metrics
        self.add_metric(&mut output, "cpu_usage_percent", metrics.resource_utilization.avg_cpu_usage_percent, &HashMap::new())?;
        self.add_metric(&mut output, "memory_usage_bytes", metrics.resource_utilization.avg_memory_usage_bytes as f64, &HashMap::new())?;
        self.add_metric(&mut output, "memory_usage_percent", metrics.resource_utilization.memory_usage_percent, &HashMap::new())?;
        
        // Format network statistics
        self.add_metric(&mut output, "total_connections", metrics.network_statistics.connection_stats.total_connections as f64, &HashMap::new())?;
        self.add_metric(&mut output, "active_connections_net", metrics.network_statistics.connection_stats.active_connections as f64, &HashMap::new())?;
        self.add_metric(&mut output, "messages_per_second", metrics.network_statistics.message_stats.messages_per_second, &HashMap::new())?;
        
        // Format error analysis
        self.add_metric(&mut output, "overall_error_rate", metrics.error_analysis.overall_error_rate, &HashMap::new())?;
        self.add_metric(&mut output, "critical_errors", metrics.error_analysis.critical_errors as f64, &HashMap::new())?;
        
        Ok(output)
    }
    
    /// Add a single metric to the output
    fn add_metric(&self, output: &mut String, name: &str, value: f64, labels: &HashMap<String, String>) -> Result<()> {
        let full_name = format!("{}{}", self.prefix, name);
        
        // Combine default labels with provided labels
        let mut all_labels = self.default_labels.clone();
        all_labels.extend(labels.clone());
        
        if all_labels.is_empty() {
            output.push_str(&format!("{} {}\n", full_name, value));
        } else {
            let label_string = all_labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(",");
            
            output.push_str(&format!("{}{{{}} {}\n", full_name, label_string, value));
        }
        
        Ok(())
    }
}

// Similar implementations for other exporters...

impl InfluxDbExporter {
    pub async fn new(config: ExportDestination) -> Result<Self> {
        let client = Arc::new(reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?);
        
        let formatter = InfluxDbFormatter {
            database: config.parameters.get("database").unwrap_or(&"mcp_metrics".to_string()).clone(),
            measurement_prefix: config.parameters.get("measurement_prefix").unwrap_or(&"mcp_".to_string()).clone(),
            default_tags: HashMap::new(),
        };
        
        let state = Arc::new(RwLock::new(ExporterState {
            status: ExporterStatus::Stopped,
            last_export_attempt: None,
            last_successful_export: None,
            recent_errors: Vec::new(),
            queue_size: 0,
        }));
        
        Ok(Self {
            config,
            client,
            statistics: Arc::new(Mutex::new(ExportStatistics::default())),
            state,
            formatter,
        })
    }
}

impl DatadogExporter {
    pub async fn new(config: ExportDestination) -> Result<Self> {
        let api_key = config.parameters
            .get("api_key")
            .ok_or_else(|| MCPError::Configuration("Missing api_key parameter".to_string()))?
            .clone();
        
        let client = Arc::new(reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?);
        
        let formatter = DatadogFormatter {
            prefix: config.parameters.get("prefix").unwrap_or(&"mcp.".to_string()).clone(),
            default_tags: vec!["service:mcp".to_string()],
        };
        
        let state = Arc::new(RwLock::new(ExporterState {
            status: ExporterStatus::Stopped,
            last_export_attempt: None,
            last_successful_export: None,
            recent_errors: Vec::new(),
            queue_size: 0,
        }));
        
        Ok(Self {
            config,
            client,
            statistics: Arc::new(Mutex::new(ExportStatistics::default())),
            state,
            api_key,
            formatter,
        })
    }
}

impl JsonExporter {
    pub async fn new(config: ExportDestination) -> Result<Self> {
        let output_path = config.parameters
            .get("output_path")
            .ok_or_else(|| MCPError::Configuration("Missing output_path parameter".to_string()))?;
        
        let formatter = JsonFormatter {
            pretty: config.parameters
                .get("pretty")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            include_metadata: config.parameters
                .get("include_metadata")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
        };
        
        let state = Arc::new(RwLock::new(ExporterState {
            status: ExporterStatus::Stopped,
            last_export_attempt: None,
            last_successful_export: None,
            recent_errors: Vec::new(),
            queue_size: 0,
        }));
        
        Ok(Self {
            config,
            statistics: Arc::new(Mutex::new(ExportStatistics::default())),
            state,
            output_path: std::path::PathBuf::from(output_path),
            formatter,
        })
    }
}

#[async_trait::async_trait]
impl MetricsExporter for JsonExporter {
    async fn export_metrics(&self, metrics: &AggregatedMetrics) -> Result<ExportResult> {
        let start_time = Instant::now();
        
        // Format metrics as JSON
        let json_data = if self.formatter.pretty {
            serde_json::to_string_pretty(metrics)?
        } else {
            serde_json::to_string(metrics)?
        };
        
        // Write to file
        match tokio::fs::write(&self.output_path, json_data).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                self.update_statistics_success(duration).await;
                
                Ok(ExportResult {
                    success: true,
                    message: format!("Metrics exported to {}", self.output_path.display()),
                    metrics_exported: 1, // JSON exports the entire metrics object as one
                    duration,
                    exported_at: Utc::now(),
                    retry_suggested: false,
                    metadata: HashMap::new(),
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to write JSON file: {}", e);
                self.handle_export_error(&error_msg).await;
                
                Ok(ExportResult {
                    success: false,
                    message: error_msg,
                    metrics_exported: 0,
                    duration: start_time.elapsed(),
                    exported_at: Utc::now(),
                    retry_suggested: true,
                    metadata: HashMap::new(),
                })
            }
        }
    }
    
    async fn test_connection(&self) -> Result<bool> {
        // Test if we can write to the output directory
        let parent_dir = self.output_path.parent().unwrap_or(std::path::Path::new("."));
        Ok(parent_dir.exists() && parent_dir.is_dir())
    }
    
    fn exporter_name(&self) -> &str {
        &self.config.name
    }
    
    fn exporter_type(&self) -> &str {
        "json"
    }
    
    fn capabilities(&self) -> Vec<ExporterCapability> {
        vec![
            ExporterCapability::Batch,
            ExporterCapability::CustomFormatting,
        ]
    }
    
    async fn health_status(&self) -> ExporterHealth {
        let state = self.state.read().await;
        let can_write = self.test_connection().await.unwrap_or(false);
        
        ExporterHealth {
            status: if can_write { HealthStatus::Healthy } else { HealthStatus::Critical },
            score: if can_write { 1.0 } else { 0.0 },
            last_success: state.last_successful_export,
            last_attempt: state.last_export_attempt,
            recent_errors: state.recent_errors.iter()
                .take(5)
                .map(|e| e.message.clone())
                .collect(),
            connection_status: if can_write { ConnectionStatus::Connected } else { ConnectionStatus::Error },
        }
    }
    
    async fn get_statistics(&self) -> ExportStatistics {
        self.statistics.lock().await.clone()
    }
}

impl JsonExporter {
    async fn update_statistics_success(&self, duration: Duration) {
        let mut stats = self.statistics.lock().await;
        stats.total_exports += 1;
        stats.successful_exports += 1;
        stats.last_export_duration = duration;
        
        if stats.total_exports == 1 {
            stats.avg_export_duration = duration;
        } else {
            let total_duration = stats.avg_export_duration.as_millis() as u64 * (stats.total_exports - 1) + duration.as_millis() as u64;
            stats.avg_export_duration = Duration::from_millis(total_duration / stats.total_exports);
        }
        
        stats.error_rate = stats.failed_exports as f64 / stats.total_exports as f64;
    }
    
    async fn handle_export_error(&self, error_msg: &str) {
        let mut state = self.state.write().await;
        
        state.recent_errors.push(ExportError {
            message: error_msg.to_string(),
            timestamp: Utc::now(),
            error_type: "file_write_error".to_string(),
            retry_attempted: false,
        });
        
        if state.recent_errors.len() > 10 {
            state.recent_errors.remove(0);
        }
        
        let mut stats = self.statistics.lock().await;
        stats.total_exports += 1;
        stats.failed_exports += 1;
        stats.error_rate = stats.failed_exports as f64 / stats.total_exports as f64;
    }
}

impl Default for ExportStatistics {
    fn default() -> Self {
        Self {
            total_exports: 0,
            successful_exports: 0,
            failed_exports: 0,
            total_metrics_exported: 0,
            avg_export_duration: Duration::from_millis(0),
            last_export_duration: Duration::from_millis(0),
            export_rate: 0.0,
            error_rate: 0.0,
        }
    }
} 