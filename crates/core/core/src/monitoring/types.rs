// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring domain types, events, and the provider trait.

use crate::{HealthStatus, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal monitoring interface that abstracts over different monitoring systems.
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait MonitoringProvider: Send + Sync {
    /// Returns the provider name.
    fn provider_name(&self) -> &'static str;
    /// Returns the provider version.
    fn provider_version(&self) -> &'static str;

    /// Records an event.
    async fn record_event(&self, event: MonitoringEvent) -> Result<()>;
    /// Records a metric.
    async fn record_metric(&self, metric: Metric) -> Result<()>;
    /// Records health status for a component.
    async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()>;
    /// Records performance metrics for a component.
    async fn record_performance(&self, component: &str, metrics: PerformanceMetrics) -> Result<()>;

    /// Queries health for a component (optional; returns None if unsupported).
    async fn query_health(&self, _component: &str) -> Result<Option<HealthStatus>> {
        Ok(None)
    }

    /// Queries metrics for a component (optional; returns empty if unsupported).
    async fn query_metrics(&self, _component: &str, _timeframe: TimeFrame) -> Result<Vec<Metric>> {
        Ok(vec![])
    }

    /// Returns the provider's own health status.
    async fn provider_health(&self) -> Result<HealthStatus>;
    /// Returns capabilities the provider supports.
    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>>;
}

/// Monitoring events that can be recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEvent {
    /// Service has started.
    ServiceStarted {
        /// Service name.
        service: String,
        /// Service version.
        version: String,
        /// When the event occurred.
        timestamp: DateTime<Utc>,
    },

    /// Service has stopped.
    ServiceStopped {
        /// Service name.
        service: String,
        /// When the event occurred.
        timestamp: DateTime<Utc>,
    },

    /// Task was submitted for execution.
    TaskSubmitted {
        /// Task identifier.
        task_id: String,
        /// Task type.
        task_type: String,
        /// Task priority.
        priority: String,
        /// When submitted.
        timestamp: DateTime<Utc>,
    },

    /// Task execution completed.
    TaskCompleted {
        /// Task identifier.
        task_id: String,
        /// How long execution took.
        execution_time: std::time::Duration,
        /// Whether execution succeeded.
        success: bool,
        /// When completed.
        timestamp: DateTime<Utc>,
    },

    /// Instance was spawned.
    InstanceSpawned {
        /// Instance identifier.
        instance_id: String,
        /// Node identifier.
        node_id: String,
        /// When spawned.
        timestamp: DateTime<Utc>,
    },

    /// Node joined a federation.
    FederationJoined {
        /// Federation identifier.
        federation_id: String,
        /// Number of nodes in federation.
        node_count: u32,
        /// When joined.
        timestamp: DateTime<Utc>,
    },

    /// Primal was discovered.
    PrimalDiscovered {
        /// Primal identifier.
        primal_id: String,
        /// Primal type.
        primal_type: String,
        /// Primal endpoint.
        endpoint: String,
        /// When discovered.
        timestamp: DateTime<Utc>,
    },

    /// Coordination completed.
    CoordinationCompleted {
        /// Coordination identifier.
        coordination_id: String,
        /// Primals involved.
        primals_involved: Vec<String>,
        /// Execution duration.
        execution_time: std::time::Duration,
        /// When completed.
        timestamp: DateTime<Utc>,
    },

    /// Error occurred.
    ErrorOccurred {
        /// Error type/category.
        error_type: String,
        /// Error message.
        error_message: String,
        /// Component where error occurred.
        component: String,
        /// When it occurred.
        timestamp: DateTime<Utc>,
    },

    /// Custom event for extensibility.
    Custom {
        /// Event type name.
        event_type: String,
        /// Event payload.
        data: serde_json::Value,
        /// When it occurred.
        timestamp: DateTime<Utc>,
    },
}

/// A recorded metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Metric value.
    pub value: MetricValue,
    /// Labels for dimensional data.
    pub labels: HashMap<String, String>,
    /// When recorded.
    pub timestamp: DateTime<Utc>,
}

/// Value type for a metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Monotonically increasing counter.
    Counter(u64),
    /// Point-in-time gauge value.
    Gauge(f64),
    /// Distribution histogram with bucket boundaries and counts.
    Histogram {
        /// Bucket boundaries.
        buckets: Vec<f64>,
        /// Count per bucket.
        counts: Vec<u64>,
    },
    /// Summary with quantile values.
    Summary {
        /// Quantile (0-1) and value pairs.
        quantiles: Vec<(f64, f64)>,
    },
}

/// Performance metrics for system components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage (0-1).
    pub cpu_usage: Option<f64>,
    /// Memory usage (0-1).
    pub memory_usage: Option<f64>,
    /// Network usage.
    pub network_usage: Option<f64>,
    /// Response time.
    pub response_time: Option<std::time::Duration>,
    /// Throughput (ops/sec).
    pub throughput: Option<f64>,
    /// Error rate (0-1).
    pub error_rate: Option<f64>,
    /// Queue length.
    pub queue_length: Option<u32>,
    /// Active connections.
    pub active_connections: Option<u32>,
    /// Additional custom metrics.
    pub custom_metrics: HashMap<String, f64>,
}

/// Monitoring capabilities supported by providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringCapability {
    /// Event recording.
    Events,
    /// Metric recording.
    Metrics,
    /// Health checks.
    Health,
    /// Performance metrics.
    Performance,
    /// Metric queries.
    Queries,
    /// Alerting.
    Alerts,
    /// Dashboards.
    Dashboards,
    /// Distributed tracing.
    Tracing,
    /// Log aggregation.
    Logging,
    /// Custom capability.
    Custom(String),
}

/// Time frame for metric queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrame {
    /// Last minute.
    LastMinute,
    /// Last hour.
    LastHour,
    /// Last day.
    LastDay,
    /// Last week.
    LastWeek,
    /// Custom time range.
    Custom {
        /// Start of range.
        from: DateTime<Utc>,
        /// End of range.
        to: DateTime<Utc>,
    },
}

/// Monitoring service status
/// Status of the monitoring service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatus {
    /// Whether monitoring is enabled.
    pub enabled: bool,
    /// Number of registered providers.
    pub provider_count: usize,
    /// Status of each provider.
    pub providers: Vec<ProviderStatus>,
    /// Whether fallback logger is active (no providers).
    pub fallback_active: bool,
}

/// Status of a monitoring provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Provider name.
    pub name: String,
    /// Provider version.
    pub version: String,
    /// Provider health.
    pub health: HealthStatus,
    /// Capabilities the provider supports.
    pub capabilities: Vec<MonitoringCapability>,
}
