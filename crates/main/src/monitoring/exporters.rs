// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Metrics export mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! # Exporters Module
//!
//! This module provides metrics export capabilities for various monitoring systems.

use async_trait::async_trait; // KEEP: MetricsExporter used as trait object (dyn MetricsExporter)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use universal_constants::network::{DEFAULT_LOCALHOST, get_service_port, http_url};

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
    /// No authentication.
    None,
    /// Basic HTTP authentication.
    Basic,
    /// Bearer token authentication.
    Bearer,
    /// API key authentication.
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
        // Prometheus exposition format (text/plain; version=0.0.4)
        // See: https://prometheus.io/docs/instrumenting/exposition_formats/
        let mut prometheus_output = String::new();

        // Exposition format header (informational)
        prometheus_output.push_str("# Prometheus exposition format (text/plain; version=0.0.4)\n");

        // Helper to emit a metric with HELP and TYPE (Prometheus exposition format)
        let emit_gauge = |out: &mut String, name: &str, help: &str, value: f64| {
            out.push_str(&format!("# HELP {name} {help}\n"));
            out.push_str(&format!("# TYPE {name} gauge\n"));
            out.push_str(&format!("{name} {value}\n"));
        };
        let emit_counter = |out: &mut String, name: &str, help: &str, value: f64| {
            out.push_str(&format!("# HELP {name} {help}\n"));
            out.push_str(&format!("# TYPE {name} counter\n"));
            out.push_str(&format!("{name} {value}\n"));
        };

        // Export system metrics - each with proper HELP and TYPE
        let system_metrics = &metrics.system_metrics;
        emit_gauge(
            &mut prometheus_output,
            "squirrel_cpu_usage",
            "CPU usage percentage",
            system_metrics.cpu_usage,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_memory_usage",
            "Memory usage in bytes",
            system_metrics.memory_usage as f64,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_memory_percentage",
            "Memory usage as percentage of total",
            system_metrics.memory_percentage,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_disk_usage",
            "Disk usage percentage",
            system_metrics.disk_usage,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_network_bytes_sent",
            "Network bytes sent",
            system_metrics.network_bytes_sent,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_network_bytes_received",
            "Network bytes received",
            system_metrics.network_bytes_received,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_active_connections",
            "Number of active connections",
            system_metrics.active_connections as f64,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_request_rate",
            "Requests per second",
            system_metrics.request_rate,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_error_rate",
            "Error rate percentage",
            system_metrics.error_rate,
        );
        emit_gauge(
            &mut prometheus_output,
            "squirrel_avg_response_time",
            "Average response time in milliseconds",
            system_metrics.avg_response_time,
        );
        emit_counter(
            &mut prometheus_output,
            "squirrel_uptime",
            "Process uptime in seconds",
            system_metrics.uptime as f64,
        );

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
        // 2. METRICS_EXPORTER_PORT or METRICS_PORT (port override)
        // 3. Default: use get_service_port("metrics") for discovery
        let endpoint = std::env::var("METRICS_EXPORTER_ENDPOINT").unwrap_or_else(|_| {
            let port = std::env::var("METRICS_EXPORTER_PORT")
                .or_else(|_| std::env::var("METRICS_PORT"))
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| get_service_port("metrics"));
            http_url(DEFAULT_LOCALHOST, port, "/metrics")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::MetricType;
    use crate::monitoring::metrics::{MetricValue, SystemMetrics};
    use chrono::Utc;

    fn create_test_config(name: &str) -> ExporterConfig {
        ExporterConfig {
            name: name.to_string(),
            endpoint: "http://localhost:9090/metrics".to_string(),
            interval: std::time::Duration::from_secs(30),
            auth: None,
            headers: HashMap::new(),
        }
    }

    fn create_test_metrics() -> AllMetrics {
        let mut metrics = HashMap::new();
        metrics.insert(
            "requests_total".to_string(),
            MetricValue {
                value: 42.0,
                labels: HashMap::from([("service".to_string(), "squirrel".to_string())]),
                timestamp: Utc::now(),
                metric_type: MetricType::Counter,
            },
        );

        let mut component_metrics = HashMap::new();
        component_metrics.insert(
            "ai_router".to_string(),
            HashMap::from([("latency_ms".to_string(), 15.5)]),
        );

        AllMetrics {
            metrics,
            component_metrics,
            system_metrics: SystemMetrics {
                cpu_usage: 45.5,
                memory_usage: 1024 * 1024 * 512,
                memory_percentage: 50.0,
                disk_usage: 60.0,
                network_bytes_sent: 1000.0,
                network_bytes_received: 2000.0,
                active_connections: 10,
                request_rate: 100.5,
                error_rate: 0.5,
                avg_response_time: 25.0,
                uptime: 3600,
            },
        }
    }

    // --- ExporterConfig tests ---

    #[test]
    fn test_exporter_config_default() {
        let config = ExporterConfig::default();
        assert_eq!(config.name, "default");
        assert!(config.endpoint.contains("metrics"));
        assert_eq!(config.interval, std::time::Duration::from_secs(60));
        assert!(config.auth.is_none());
        assert!(config.headers.is_empty());
    }

    #[test]
    fn test_exporter_config_serde() {
        let config = create_test_config("prometheus");
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExporterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "prometheus");
        assert_eq!(deserialized.endpoint, "http://localhost:9090/metrics");
    }

    // --- AuthConfig tests ---

    #[test]
    fn test_auth_config_none() {
        let auth = AuthConfig {
            auth_type: AuthType::None,
            username: None,
            password: None,
            token: None,
        };
        let json = serde_json::to_string(&auth).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.username.is_none());
    }

    #[test]
    fn test_auth_config_basic() {
        let auth = AuthConfig {
            auth_type: AuthType::Basic,
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
            token: None,
        };
        let json = serde_json::to_string(&auth).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username.as_deref(), Some("admin"));
    }

    #[test]
    fn test_auth_config_bearer() {
        let auth = AuthConfig {
            auth_type: AuthType::Bearer,
            username: None,
            password: None,
            token: Some("eyJ0eXAi...".to_string()),
        };
        let json = serde_json::to_string(&auth).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.token.is_some());
    }

    #[test]
    fn test_auth_type_serde() {
        let types = vec![
            AuthType::None,
            AuthType::Basic,
            AuthType::Bearer,
            AuthType::ApiKey,
        ];
        for auth_type in types {
            let json = serde_json::to_string(&auth_type).unwrap();
            let deserialized: AuthType = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    // --- PrometheusExporter tests ---

    #[test]
    fn test_prometheus_exporter_creation() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);
        assert_eq!(exporter.name(), "prometheus");
    }

    #[test]
    fn test_prometheus_exporter_config() {
        let config = create_test_config("prom");
        let exporter = PrometheusExporter::new(config);
        let cfg = exporter.config();
        assert_eq!(cfg.name, "prom");
        assert_eq!(cfg.endpoint, "http://localhost:9090/metrics");
    }

    #[tokio::test]
    async fn test_prometheus_export_metrics() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);
        let metrics = create_test_metrics();

        let result = exporter.export_metrics(metrics).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("squirrel_cpu_usage 45.5"));
        assert!(output.contains("squirrel_memory_percentage 50"));
        assert!(output.contains("squirrel_disk_usage 60"));
        assert!(output.contains("squirrel_network_bytes_sent 1000"));
        assert!(output.contains("squirrel_active_connections 10"));
        assert!(output.contains("squirrel_request_rate 100.5"));
        assert!(output.contains("squirrel_error_rate 0.5"));
        assert!(output.contains("squirrel_avg_response_time 25"));
        assert!(output.contains("squirrel_uptime 3600"));
    }

    #[tokio::test]
    async fn test_prometheus_export_component_metrics() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);
        let metrics = create_test_metrics();

        let output = exporter.export_metrics(metrics).await.unwrap();
        assert!(output.contains("squirrel_component_latency_ms{component=\"ai_router\"}"));
    }

    #[tokio::test]
    async fn test_prometheus_export_custom_metrics_with_labels() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);
        let metrics = create_test_metrics();

        let output = exporter.export_metrics(metrics).await.unwrap();
        assert!(output.contains("squirrel_custom_requests_total"));
        assert!(output.contains("service=\"squirrel\""));
    }

    #[tokio::test]
    async fn test_prometheus_export_custom_metrics_without_labels() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);

        let mut metrics = create_test_metrics();
        metrics.metrics.clear();
        metrics.metrics.insert(
            "simple_counter".to_string(),
            MetricValue {
                value: 100.0,
                labels: HashMap::new(),
                timestamp: Utc::now(),
                metric_type: MetricType::Counter,
            },
        );

        let output = exporter.export_metrics(metrics).await.unwrap();
        assert!(output.contains("squirrel_custom_simple_counter 100"));
    }

    #[tokio::test]
    async fn test_prometheus_export_help_header() {
        let config = create_test_config("prometheus");
        let exporter = PrometheusExporter::new(config);
        let metrics = create_test_metrics();

        let output = exporter.export_metrics(metrics).await.unwrap();
        assert!(output.contains("# HELP squirrel_cpu_usage"));
        assert!(output.contains("# TYPE squirrel_cpu_usage gauge"));
    }

    // --- JsonExporter tests ---

    #[test]
    fn test_json_exporter_creation() {
        let config = create_test_config("json");
        let exporter = JsonExporter::new(config);
        assert_eq!(exporter.name(), "json");
    }

    #[tokio::test]
    async fn test_json_export_metrics() {
        let config = create_test_config("json");
        let exporter = JsonExporter::new(config);
        let metrics = create_test_metrics();

        let result = exporter.export_metrics(metrics).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("system_metrics").is_some());
        assert!(parsed.get("component_metrics").is_some());
        assert!(parsed.get("metrics").is_some());
    }

    #[tokio::test]
    async fn test_json_export_empty_metrics() {
        let config = create_test_config("json");
        let exporter = JsonExporter::new(config);

        let metrics = AllMetrics {
            metrics: HashMap::new(),
            component_metrics: HashMap::new(),
            system_metrics: SystemMetrics::default(),
        };

        let result = exporter.export_metrics(metrics).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["metrics"].as_object().unwrap().is_empty());
    }

    // --- ExporterConfig with auth ---

    #[test]
    fn test_exporter_config_with_auth() {
        let config = ExporterConfig {
            name: "secure_exporter".to_string(),
            endpoint: "https://metrics.example.com".to_string(),
            interval: std::time::Duration::from_secs(120),
            auth: Some(AuthConfig {
                auth_type: AuthType::Bearer,
                username: None,
                password: None,
                token: Some("secret-token".to_string()),
            }),
            headers: HashMap::from([("X-Custom".to_string(), "value".to_string())]),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExporterConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.auth.is_some());
        assert_eq!(deserialized.headers.get("X-Custom").unwrap(), "value");
    }

    #[test]
    fn test_exporter_config_default_env_override() {
        temp_env::with_var(
            "METRICS_EXPORTER_ENDPOINT",
            Some("http://custom:9999/metrics"),
            || {
                let config = ExporterConfig::default();
                assert_eq!(config.endpoint, "http://custom:9999/metrics");
            },
        );
        temp_env::with_var("METRICS_EXPORTER_PORT", Some("1234"), || {
            let config = ExporterConfig::default();
            assert_eq!(config.endpoint, "http://localhost:1234/metrics");
        });
    }

    #[test]
    fn test_auth_config_api_key() {
        let auth = AuthConfig {
            auth_type: AuthType::ApiKey,
            username: None,
            password: None,
            token: Some("api-key-123".to_string()),
        };
        let json = serde_json::to_string(&auth).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized.auth_type, AuthType::ApiKey));
    }

    #[test]
    fn test_json_exporter_config() {
        let config = create_test_config("json_exporter");
        let exporter = JsonExporter::new(config);
        let cfg = exporter.config();
        assert_eq!(cfg.name, "json_exporter");
    }
}
