//! # Exporters Module
//!
//! This module provides metrics export capabilities for various monitoring systems.

use async_trait::async_trait; // KEEP: MetricsExporter used as trait object (dyn MetricsExporter)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::metrics::AllMetrics;
use crate::error::PrimalError;

/// Trait for metrics exporters

#[async_trait]
pub trait MetricsExporter: Send + Sync {
    /// Export metrics to external system
    async fn export_metrics(&self, metrics: AllMetrics) -> Result<String, PrimalError>;

    /// Get exporter name
    fn name(&self) -> &str;

    /// Get exporter configuration
    fn config(&self) -> &ExporterConfig;
}

/// Exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    /// Exporter name
    pub name: String,
    /// Export endpoint
    pub endpoint: String,
    /// Export interval
    pub interval: std::time::Duration,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    /// Username (for basic auth)
    pub username: Option<String>,
    /// Password (for basic auth)
    pub password: Option<String>,
    /// Token (for bearer auth)
    pub token: Option<String>,
}

/// Authentication type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    /// Exporter configuration
    config: ExporterConfig,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    #[must_use]
    pub fn new(config: ExporterConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl MetricsExporter for PrometheusExporter {
    async fn export_metrics(&self, metrics: AllMetrics) -> Result<String, PrimalError> {
        // Stub implementation - convert metrics to Prometheus format
        let mut prometheus_output = String::new();

        // Add help and type information
        prometheus_output
            .push_str("# HELP squirrel_system_metrics System metrics for Squirrel AI\n");
        prometheus_output.push_str("# TYPE squirrel_system_metrics gauge\n");

        // Export system metrics
        let system_metrics = &metrics.system_metrics;
        prometheus_output.push_str(&format!(
            "squirrel_cpu_usage {}\n",
            system_metrics.cpu_usage
        ));
        prometheus_output.push_str(&format!(
            "squirrel_memory_usage {}\n",
            system_metrics.memory_usage
        ));
        prometheus_output.push_str(&format!(
            "squirrel_memory_percentage {}\n",
            system_metrics.memory_percentage
        ));
        prometheus_output.push_str(&format!(
            "squirrel_disk_usage {}\n",
            system_metrics.disk_usage
        ));
        prometheus_output.push_str(&format!(
            "squirrel_network_bytes_sent {}\n",
            system_metrics.network_bytes_sent
        ));
        prometheus_output.push_str(&format!(
            "squirrel_network_bytes_received {}\n",
            system_metrics.network_bytes_received
        ));
        prometheus_output.push_str(&format!(
            "squirrel_active_connections {}\n",
            system_metrics.active_connections
        ));
        prometheus_output.push_str(&format!(
            "squirrel_request_rate {}\n",
            system_metrics.request_rate
        ));
        prometheus_output.push_str(&format!(
            "squirrel_error_rate {}\n",
            system_metrics.error_rate
        ));
        prometheus_output.push_str(&format!(
            "squirrel_avg_response_time {}\n",
            system_metrics.avg_response_time
        ));
        prometheus_output.push_str(&format!("squirrel_uptime {}\n", system_metrics.uptime));

        // Export component metrics
        for (component, component_metrics) in &metrics.component_metrics {
            for (metric_name, value) in component_metrics {
                prometheus_output.push_str(&format!(
                    "squirrel_component_{metric_name}{{component=\"{component}\"}} {value}\n"
                ));
            }
        }

        // Export custom metrics
        for (metric_name, metric_value) in &metrics.metrics {
            let mut labels = String::new();
            for (label_key, label_value) in &metric_value.labels {
                if !labels.is_empty() {
                    labels.push(',');
                }
                labels.push_str(&format!("{label_key}=\"{label_value}\""));
            }

            if labels.is_empty() {
                prometheus_output.push_str(&format!(
                    "squirrel_custom_{} {}\n",
                    metric_name, metric_value.value
                ));
            } else {
                prometheus_output.push_str(&format!(
                    "squirrel_custom_{}{{{}}} {}\n",
                    metric_name, labels, metric_value.value
                ));
            }
        }

        Ok(prometheus_output)
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &ExporterConfig {
        &self.config
    }
}

/// JSON metrics exporter
pub struct JsonExporter {
    /// Exporter configuration
    config: ExporterConfig,
}

impl JsonExporter {
    /// Create a new JSON exporter
    #[must_use]
    pub fn new(config: ExporterConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl MetricsExporter for JsonExporter {
    async fn export_metrics(&self, metrics: AllMetrics) -> Result<String, PrimalError> {
        // Convert metrics to JSON format
        serde_json::to_string_pretty(&metrics).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to serialize metrics: {e}"))
        })
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &ExporterConfig {
        &self.config
    }
}

impl Default for ExporterConfig {
    fn default() -> Self {
        // Multi-tier metrics endpoint resolution
        // 1. METRICS_EXPORTER_ENDPOINT (full endpoint)
        // 2. METRICS_EXPORTER_PORT (port override)
        // 3. Default: http://localhost:9090/metrics
        let endpoint = std::env::var("METRICS_EXPORTER_ENDPOINT").unwrap_or_else(|_| {
            let port = std::env::var("METRICS_EXPORTER_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(9090); // Default metrics exporter port
            format!("http://localhost:{}/metrics", port)
        });

        Self {
            name: "default".to_string(),
            endpoint,
            interval: std::time::Duration::from_secs(60),
            auth: None,
            headers: HashMap::new(),
        }
    }
}
