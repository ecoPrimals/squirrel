//! Observability configuration types for Squirrel MCP
//!
//! This module defines observability-related configuration including
//! logging, metrics, tracing, and health check settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Observability configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub tracing: TracingConfig,
    pub health_checks: HealthCheckConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub destination: String,
    pub file_path: Option<PathBuf>,
    pub rotation: Option<String>,
    pub max_size: Option<String>,
    pub max_files: Option<u32>,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub port: u16,
    pub collection_interval: Duration,
    pub retention_period: Duration,
}

/// Tracing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub jaeger_endpoint: Option<String>,
    pub service_name: String,
}

/// Health check configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                destination: "stdout".to_string(),
                file_path: None,
                rotation: None,
                max_size: None,
                max_files: None,
            },
            metrics: MetricsConfig {
                enabled: true,
                endpoint: "/metrics".to_string(),
                port: 9090,
                collection_interval: Duration::from_secs(15),
                retention_period: Duration::from_secs(86400),
            },
            tracing: TracingConfig {
                enabled: true,
                sampling_rate: 1.0,
                jaeger_endpoint: None,
                service_name: "squirrel-mcp".to_string(),
            },
            health_checks: HealthCheckConfig {
                enabled: true,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                failure_threshold: 3,
                recovery_threshold: 2,
            },
        }
    }
}
