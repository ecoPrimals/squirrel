// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Platform configuration and state types for Enhanced MCP.
//!
//! Centralized type definitions for platform settings, metrics, and health.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::service_composition::HealthStatus;

/// Platform settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Platform name
    pub name: String,

    /// Platform version
    pub version: String,

    /// Platform description
    pub description: String,

    /// Platform metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Performance settings
    pub performance: PerformanceSettings,

    /// Security settings
    pub security: SecuritySettings,

    /// Monitoring settings
    pub monitoring: MonitoringSettings,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable performance monitoring
    pub monitoring_enabled: bool,

    /// Performance metrics interval
    pub metrics_interval: Duration,

    /// Performance optimization enabled
    pub optimization_enabled: bool,

    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable security monitoring
    pub monitoring_enabled: bool,

    /// Security audit enabled
    pub audit_enabled: bool,

    /// Encryption enabled
    pub encryption_enabled: bool,

    /// Authentication required
    pub auth_required: bool,
}

/// Monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    /// Enable monitoring
    pub enabled: bool,

    /// Monitoring interval
    pub interval: Duration,

    /// Metrics collection enabled
    pub metrics_enabled: bool,

    /// Alerting enabled
    pub alerting_enabled: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,

    /// Maximum CPU usage (percentage)
    pub max_cpu: f64,

    /// Maximum network bandwidth (bytes/sec)
    pub max_network: u64,

    /// Maximum disk I/O (bytes/sec)
    pub max_disk_io: u64,
}

/// Platform state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformState {
    /// Platform status
    pub status: PlatformStatus,

    /// Platform start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Platform uptime
    pub uptime: Duration,

    /// Platform metrics
    pub metrics: PlatformMetrics,

    /// Platform health
    pub health: PlatformHealth,
}

/// Platform status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlatformStatus {
    /// Platform is initializing
    Initializing,

    /// Platform is starting
    Starting,

    /// Platform is healthy
    Healthy,

    /// Platform is degraded
    Degraded,

    /// Platform is unhealthy
    Unhealthy,

    /// Platform is shutting down
    ShuttingDown,

    /// Platform is stopped
    Stopped,
}

/// Platform metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMetrics {
    /// Total AI requests processed
    pub total_requests: u64,

    /// Active AI requests
    pub active_requests: u64,

    /// Total services registered
    pub total_services: u64,

    /// Active services
    pub active_services: u64,

    /// Total workflows executed
    pub total_workflows: u64,

    /// Active workflows
    pub active_workflows: u64,

    /// Average response time
    pub avg_response_time: Duration,

    /// Success rate
    pub success_rate: f64,

    /// Error rate
    pub error_rate: f64,
}

/// Platform health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformHealth {
    /// Overall health status
    pub status: HealthStatus,

    /// Health score (0.0 to 1.0)
    pub score: f64,

    /// Component health
    pub components: HashMap<String, ComponentHealth>,

    /// Health checks
    pub checks: Vec<HealthCheck>,

    /// Last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Component status
    pub status: HealthStatus,

    /// Component score
    pub score: f64,

    /// Component metrics
    pub metrics: HashMap<String, serde_json::Value>,

    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,

    /// Check status
    pub status: HealthStatus,

    /// Check message
    pub message: String,

    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Check duration
    pub duration: Duration,
}

impl Default for PlatformState {
    fn default() -> Self {
        Self {
            status: PlatformStatus::Initializing,
            start_time: chrono::Utc::now(),
            uptime: Duration::from_secs(0),
            metrics: PlatformMetrics {
                total_requests: 0,
                active_requests: 0,
                total_services: 0,
                active_services: 0,
                total_workflows: 0,
                active_workflows: 0,
                avg_response_time: Duration::from_secs(0),
                success_rate: 0.0,
                error_rate: 0.0,
            },
            health: PlatformHealth {
                status: HealthStatus::Unknown,
                score: 0.0,
                components: HashMap::new(),
                checks: Vec::new(),
                last_check: chrono::Utc::now(),
            },
        }
    }
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            name: "Enhanced MCP Platform".to_string(),
            version: "1.0.0".to_string(),
            description: "Universal AI Integration Platform with Service Composition and Workflow Management".to_string(),
            metadata: HashMap::new(),
            performance: PerformanceSettings {
                monitoring_enabled: true,
                metrics_interval: Duration::from_secs(30),
                optimization_enabled: true,
                resource_limits: ResourceLimits {
                    max_memory: 8 * 1024 * 1024 * 1024, // 8GB
                    max_cpu: 80.0,
                    max_network: 1024 * 1024 * 100, // 100MB/s
                    max_disk_io: 1024 * 1024 * 100, // 100MB/s
                },
            },
            security: SecuritySettings {
                monitoring_enabled: true,
                audit_enabled: true,
                encryption_enabled: true,
                auth_required: true,
            },
            monitoring: MonitoringSettings {
                enabled: true,
                interval: Duration::from_secs(30),
                metrics_enabled: true,
                alerting_enabled: true,
            },
        }
    }
}
