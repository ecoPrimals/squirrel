// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration for observability framework
//!
//! This module provides configuration structures and defaults
//! for the observability system.

use tracing::Level;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Configuration for the observability framework
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    // Core Flags
    /// Enable dashboard integration
    pub enable_dashboard: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Enable alerting
    pub enable_alerting: bool,
    
    // Logging and Tracing Configuration
    /// Default log level
    pub default_log_level: Level,
    /// Include trace context in logs
    pub include_trace_context_in_logs: bool,
    /// Enable tracing
    pub enable_tracing: bool,
    /// Trace sampling rate (0.0-1.0)
    pub trace_sampling_rate: f64,
    /// Maximum trace spans to keep
    pub max_trace_spans: usize,
    
    // Health Check Configuration
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Health check timeout in seconds
    pub health_check_timeout: u64,
    /// Maximum number of health check subscribers
    pub max_health_subscribers: usize,
    /// Connect health checks to alerting system
    pub connect_health_to_alerting: bool,
    
    // Alerting Configuration
    /// Alert retention period in seconds
    pub alert_retention_secs: u64,
    /// Maximum number of alerts to retain
    pub max_alerts: usize,
    /// Alert notification buffer size
    pub alert_notification_buffer: usize,
    
    // Dashboard Configuration
    /// Enable dashboard integration
    pub enable_dashboard_integration: bool,
    /// Dashboard URL
    pub dashboard_url: String,
    /// Dashboard authentication token
    pub dashboard_auth_token: Option<String>,
    /// Dashboard metrics update interval in seconds
    pub dashboard_metrics_interval: u64,
    /// Dashboard traces update interval in seconds
    pub dashboard_traces_interval: u64,
    /// Dashboard health update interval in seconds
    pub dashboard_health_interval: u64,
    /// Dashboard alerts update interval in seconds
    pub dashboard_alerts_interval: u64,
    /// Maximum traces per batch to send to dashboard
    pub dashboard_max_traces_per_batch: usize,
    /// Maximum metrics per batch to send to dashboard
    pub dashboard_max_metrics_per_batch: usize,
    
    // External Tracing Configuration
    /// Enable external tracing exporters
    pub enable_external_tracing: bool,
    /// Trace export interval in seconds
    pub trace_export_interval: u64,
    /// Trace export buffer size
    pub trace_export_buffer_size: usize,
    /// Enable OTLP exporter
    pub enable_otlp_exporter: bool,
    /// OTLP endpoint
    pub otlp_endpoint: String,
    /// OTLP authentication token
    pub otlp_auth_token: Option<String>,
    /// Enable Jaeger exporter
    pub enable_jaeger_exporter: bool,
    /// Jaeger endpoint
    pub jaeger_endpoint: String,
    /// Jaeger authentication token
    pub jaeger_auth_token: Option<String>,
    
    // Service information
    /// Service name for metrics and traces
    pub service_name: String,
    /// Environment name (e.g., "development", "production")
    pub environment: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            // Core flags
            enable_dashboard: false,
            enable_metrics: true,
            enable_health_checks: true,
            enable_alerting: true,
            
            // Logging and tracing
            default_log_level: Level::Info,
            include_trace_context_in_logs: true,
            enable_tracing: true,
            trace_sampling_rate: 1.0,
            max_trace_spans: 10000,
            
            // Health checks
            health_check_interval: 30,
            health_check_timeout: 5,
            max_health_subscribers: 100,
            connect_health_to_alerting: true,
            
            // Alerting
            alert_retention_secs: 86400, // 24 hours
            max_alerts: 1000,
            alert_notification_buffer: 100,
            
            // Dashboard
            enable_dashboard_integration: false,
            dashboard_url: std::env::var("UI_ENDPOINT").unwrap_or_else(|_| {
                // Multi-tier dashboard URL resolution
                let port = std::env::var("WEB_UI_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(3000);  // Default Web UI port
                format!("http://localhost:{}", port)
            }),
            dashboard_auth_token: None,
            dashboard_metrics_interval: 60,
            dashboard_traces_interval: 30,
            dashboard_health_interval: 60,
            dashboard_alerts_interval: 10,
            dashboard_max_traces_per_batch: 100,
            dashboard_max_metrics_per_batch: 50,
            
            // External tracing
            enable_external_tracing: false,
            trace_export_interval: 30,
            trace_export_buffer_size: 1000,
            enable_otlp_exporter: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            otlp_auth_token: None,
            enable_jaeger_exporter: false,
            jaeger_endpoint: "http://localhost:14268".to_string(),
            jaeger_auth_token: None,
            
            // Service info
            service_name: "mcp-service".to_string(),
            environment: "development".to_string(),
        }
    }
}

impl ObservabilityConfig {
    /// Create a new configuration with development defaults
    pub fn development() -> Self {
        Self {
            default_log_level: Level::Debug,
            trace_sampling_rate: 1.0,
            enable_dashboard_integration: false,
            enable_external_tracing: false,
            environment: "development".to_string(),
            ..Default::default()
        }
    }

    /// Create a new configuration with production defaults
    pub fn production() -> Self {
        Self {
            default_log_level: Level::Info,
            trace_sampling_rate: 0.1, // Sample 10% of traces
            enable_dashboard_integration: true,
            enable_external_tracing: true,
            enable_otlp_exporter: true,
            alert_retention_secs: 604800, // 7 days
            max_alerts: 10000,
            environment: "production".to_string(),
            ..Default::default()
        }
    }

    /// Create a new configuration with testing defaults
    pub fn testing() -> Self {
        Self {
            default_log_level: Level::Warn,
            enable_dashboard_integration: false,
            enable_external_tracing: false,
            enable_alerting: false,
            health_check_interval: 5, // Faster health checks for testing
            environment: "testing".to_string(),
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.trace_sampling_rate < 0.0 || self.trace_sampling_rate > 1.0 {
            return Err("Trace sampling rate must be between 0.0 and 1.0".to_string());
        }

        if self.health_check_interval == 0 {
            return Err("Health check interval must be greater than 0".to_string());
        }

        if self.health_check_timeout == 0 {
            return Err("Health check timeout must be greater than 0".to_string());
        }

        if self.max_trace_spans == 0 {
            return Err("Max trace spans must be greater than 0".to_string());
        }

        if self.max_alerts == 0 {
            return Err("Max alerts must be greater than 0".to_string());
        }

        if self.dashboard_max_traces_per_batch == 0 {
            return Err("Dashboard max traces per batch must be greater than 0".to_string());
        }

        if self.dashboard_max_metrics_per_batch == 0 {
            return Err("Dashboard max metrics per batch must be greater than 0".to_string());
        }

        if self.enable_dashboard_integration && self.dashboard_url.is_empty() {
            return Err("Dashboard URL must be provided when dashboard integration is enabled".to_string());
        }

        if self.enable_otlp_exporter && self.otlp_endpoint.is_empty() {
            return Err("OTLP endpoint must be provided when OTLP exporter is enabled".to_string());
        }

        if self.enable_jaeger_exporter && self.jaeger_endpoint.is_empty() {
            return Err("Jaeger endpoint must be provided when Jaeger exporter is enabled".to_string());
        }

        Ok(())
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "ObservabilityConfig {{ service: {}, env: {}, dashboard: {}, tracing: {}, sampling: {:.1}% }}",
            self.service_name,
            self.environment,
            self.enable_dashboard_integration,
            self.enable_tracing,
            self.trace_sampling_rate * 100.0
        )
    }
} 