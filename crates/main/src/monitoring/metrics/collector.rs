// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics collector implementation
//!
//! Core metrics collection engine with system monitoring.
//!
//! The public API (`MetricsCollector`, `MetricsSummary`, `HttpMetrics`) is live.
//! Uptime uses [`universal_constants::sys_info::uptime_seconds`]. Other system helpers still use
//! placeholders where noted, with per-method `#[expect(dead_code)]` until wired to `/proc` or HTTP stats.

use chrono::Utc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::PrimalError;
use crate::monitoring::CustomMetricDefinition;

use super::types::{
    AllMetrics, MetricDefinition, MetricInfo, MetricSnapshot, MetricValue, SystemMetrics,
};

/// Comprehensive metrics summary for alerting
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub system: SystemMetrics,
    pub http: HttpMetrics,
}

/// HTTP-related metrics
#[derive(Debug, Clone)]
pub struct HttpMetrics {
    pub total_requests: u64,
    pub error_responses: u64,
    pub avg_response_time_ms: f64,
}

/// Metrics collection engine
pub struct MetricsCollector {
    /// Registered metrics
    pub(crate) metrics: Arc<DashMap<String, MetricDefinition>>,
    /// Metric values storage
    pub(crate) values: Arc<DashMap<String, MetricValue>>,
    /// Component metrics
    pub(crate) component_metrics: Arc<DashMap<String, HashMap<String, f64>>>,
    /// System metrics
    pub(crate) system_metrics: Arc<RwLock<SystemMetrics>>,
    /// Collection history
    pub(crate) history: Arc<RwLock<Vec<MetricSnapshot>>>,
    /// Maximum history size
    pub(crate) max_history_size: usize,
    /// Last time external capability discovery was performed
    last_discovery: Arc<RwLock<std::time::Instant>>,
    /// Minimum interval between external discovery scans
    discovery_interval: std::time::Duration,
    /// Request tracking for rate/latency calculations
    request_tracker: Arc<RequestTracker>,
    /// Live context session count, updated by the JSON-RPC server.
    context_session_count: std::sync::atomic::AtomicU64,
}

/// Tracks request counts and response times for live metrics.
pub struct RequestTracker {
    total_requests: std::sync::atomic::AtomicU64,
    total_errors: std::sync::atomic::AtomicU64,
    /// Cumulative response time in microseconds for average calculation
    total_response_us: std::sync::atomic::AtomicU64,
    /// Timestamp when tracking started
    started_at: std::time::Instant,
}

impl Default for RequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            total_errors: std::sync::atomic::AtomicU64::new(0),
            total_response_us: std::sync::atomic::AtomicU64::new(0),
            started_at: std::time::Instant::now(),
        }
    }

    /// Record a completed request with its latency.
    pub fn record_request(&self, response_time: std::time::Duration, is_error: bool) {
        self.total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if is_error {
            self.total_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "microseconds fit u64 for realistic durations"
        )]
        let us = response_time.as_micros() as u64;
        self.total_response_us
            .fetch_add(us, std::sync::atomic::Ordering::Relaxed);
    }

    fn request_rate(&self) -> f64 {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        if elapsed < 0.001 {
            return 0.0;
        }
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        total as f64 / elapsed
    }

    fn error_rate(&self) -> f64 {
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let errors = self.total_errors.load(std::sync::atomic::Ordering::Relaxed);
        errors as f64 / total as f64
    }

    /// Total requests recorded since tracking started.
    pub fn total_requests(&self) -> u64 {
        self.total_requests
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Total errors recorded since tracking started.
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn avg_response_time_ms(&self) -> f64 {
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let us = self
            .total_response_us
            .load(std::sync::atomic::Ordering::Relaxed);
        (us as f64 / total as f64) / 1000.0
    }
}
impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            values: Arc::new(DashMap::new()),
            component_metrics: Arc::new(DashMap::new()),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
            last_discovery: Arc::new(RwLock::new(std::time::Instant::now())),
            discovery_interval: std::time::Duration::from_secs(60),
            request_tracker: Arc::new(RequestTracker::new()),
            context_session_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Access the request tracker for recording request metrics.
    #[must_use]
    pub const fn request_tracker(&self) -> &Arc<RequestTracker> {
        &self.request_tracker
    }

    /// Update the live context session count (called by the JSON-RPC server).
    pub fn set_context_session_count(&self, count: u64) {
        self.context_session_count
            .store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Current context session count.
    #[must_use]
    pub fn context_session_count(&self) -> u64 {
        self.context_session_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Total number of recorded metric values (proxy for operation count).
    #[must_use]
    pub fn total_operations(&self) -> usize {
        self.values.len()
    }

    /// Get comprehensive metrics summary
    pub async fn get_summary(&self) -> Result<MetricsSummary, PrimalError> {
        let system_metrics = self.system_metrics.read().await.clone();

        Ok(MetricsSummary {
            system: system_metrics,
            http: HttpMetrics {
                total_requests: self.request_tracker.total_requests(),
                error_responses: self.request_tracker.total_errors(),
                avg_response_time_ms: self.request_tracker.avg_response_time_ms(),
            },
        })
    }

    /// Register a custom metric
    pub async fn register_custom_metric(
        &self,
        definition: CustomMetricDefinition,
    ) -> Result<(), PrimalError> {
        let name = definition.name;
        let metric_def = MetricDefinition {
            name: name.clone(),
            metric_type: definition.metric_type,
            description: definition.description,
            labels: definition.labels,
            unit: definition.unit,
            source: definition.source,
        };

        info!("Registered custom metric: {}", metric_def.name);
        self.metrics.insert(name, metric_def);

        Ok(())
    }

    /// Record a metric value
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), PrimalError> {
        if let Some(metric_def) = self.metrics.get(name) {
            let metric_value = MetricValue {
                value,
                labels,
                timestamp: Utc::now(),
                metric_type: metric_def.metric_type,
            };

            self.values.insert(name.to_string(), metric_value);
            debug!("Recorded metric: {} = {}", name, value);

            Ok(())
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Metric '{name}' not registered"
            )))
        }
    }

    /// Get metrics for a specific component
    pub async fn get_component_metrics(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        if let Some(metrics) = self.component_metrics.get(component) {
            Ok(metrics.value().clone())
        } else {
            Ok(HashMap::new())
        }
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Result<AllMetrics, PrimalError> {
        let values: HashMap<String, MetricValue> = self
            .values
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        let component_metrics: HashMap<String, HashMap<String, f64>> = self
            .component_metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        let system_metrics = self.system_metrics.read().await;

        Ok(AllMetrics {
            metrics: values,
            component_metrics,
            system_metrics: system_metrics.clone(),
        })
    }

    /// Get metric definition and metadata
    pub async fn get_metric_info(&self, metric_name: &str) -> Result<MetricInfo, PrimalError> {
        let Some(definition) = self.metrics.get(metric_name) else {
            return Err(PrimalError::NotFoundError(format!(
                "Metric '{metric_name}' not found"
            )));
        };
        Ok(definition.to_metric_info())
    }

    /// List all registered metrics with their metadata
    pub async fn list_metric_definitions(&self) -> Result<Vec<MetricInfo>, PrimalError> {
        Ok(self
            .metrics
            .iter()
            .map(|entry| entry.value().to_metric_info())
            .collect())
    }

    /// Search metrics by source
    pub async fn get_metrics_by_source(
        &self,
        source: &str,
    ) -> Result<Vec<MetricInfo>, PrimalError> {
        Ok(self
            .metrics
            .iter()
            .filter(|entry| entry.value().source == source)
            .map(|entry| entry.value().to_metric_info())
            .collect())
    }

    /// Get metrics by unit type (e.g., "bytes", "seconds", "count")
    pub async fn get_metrics_by_unit(&self, unit: &str) -> Result<Vec<MetricInfo>, PrimalError> {
        Ok(self
            .metrics
            .iter()
            .filter(|entry| entry.value().unit == unit)
            .map(|entry| entry.value().to_metric_info())
            .collect())
    }

    /// Collect all metrics from various sources
    pub async fn collect_metrics(&self) -> Result<(), PrimalError> {
        debug!("Collecting metrics from all sources");

        // Collect system metrics
        self.collect_system_metrics().await?;

        // Collect component metrics
        self.collect_component_metrics().await?;

        // Create snapshot
        self.create_snapshot().await?;

        debug!("Metrics collection completed");
        Ok(())
    }

    /// Collect system-wide metrics
    async fn collect_system_metrics(&self) -> Result<(), PrimalError> {
        let mut system_metrics = self.system_metrics.write().await;

        #[cfg(feature = "system-metrics")]
        {
            system_metrics.cpu_usage = self.get_cpu_usage().await?;
            system_metrics.memory_usage = self.get_memory_usage().await?;
            system_metrics.memory_percentage = self.get_memory_percentage().await?;
        }

        system_metrics.disk_usage = self.get_disk_usage().await?;
        system_metrics.network_bytes_sent = self.get_network_bytes_sent().await?;
        system_metrics.network_bytes_received = self.get_network_bytes_received().await?;
        system_metrics.active_connections = self.get_active_connections().await?;
        system_metrics.request_rate = self.get_request_rate().await?;
        system_metrics.error_rate = self.get_error_rate().await?;
        system_metrics.avg_response_time = self.get_avg_response_time().await?;
        system_metrics.uptime = self.get_uptime().await?;

        Ok(())
    }

    /// Collect component-specific metrics
    async fn collect_component_metrics(&self) -> Result<(), PrimalError> {
        // TRUE PRIMAL: Discover components by capability, not hardcoded primal names
        // Internal components (always present)
        let internal_components = vec![
            "ai_intelligence",
            "mcp_integration",
            "context_state",
            "agent_deployment",
        ];

        for component in internal_components {
            let metrics = self.collect_component_specific_metrics(component).await?;
            self.component_metrics
                .insert(component.to_string(), metrics);
        }

        // Discover external components by capability domain (rate-limited to avoid slow socket scans)
        let should_discover = {
            let last = self.last_discovery.read().await;
            last.elapsed() >= self.discovery_interval
        };

        if should_discover {
            let capability_domains = vec![
                "network",  // Service mesh / orchestration capability
                "compute",  // Compute / GPU capability
                "storage",  // Persistence capability
                "security", // Security / auth capability
            ];

            for domain in capability_domains {
                if let Ok(_service) =
                    crate::capabilities::discovery::discover_capability(domain).await
                {
                    debug!(
                        "Discovered {} capability provider, collecting metrics",
                        domain
                    );
                    let metrics = self.collect_component_specific_metrics(domain).await?;
                    self.component_metrics
                        .insert(format!("capability.{domain}"), metrics);
                } else {
                    debug!("No provider found for capability: {}", domain);
                }
            }

            *self.last_discovery.write().await = std::time::Instant::now();
        }

        Ok(())
    }

    /// Collect metrics for a specific component
    async fn collect_component_specific_metrics(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        let mut metrics = HashMap::new();

        match component {
            "ai_intelligence" => {
                use crate::monitoring::metric_names::ai_intelligence::{
                    AVG_PROCESSING_TIME, MEMORY_USAGE, REQUESTS_PROCESSED, SUCCESS_RATE,
                };
                let tracker = self.request_tracker();
                let total = tracker.total_requests();
                let errors = tracker.total_errors();
                metrics.insert(REQUESTS_PROCESSED.to_string(), total as f64);
                metrics.insert(
                    AVG_PROCESSING_TIME.to_string(),
                    tracker.avg_response_time_ms(),
                );
                let success_rate = if total > 0 {
                    ((total - errors) as f64 / total as f64) * 100.0
                } else {
                    100.0
                };
                metrics.insert(SUCCESS_RATE.to_string(), success_rate);
                metrics.insert(
                    MEMORY_USAGE.to_string(),
                    universal_constants::sys_info::process_rss_mb().unwrap_or(0.0),
                );
            }
            "mcp_integration" => {
                use crate::monitoring::metric_names::mcp_integration::{
                    CONNECTION_COUNT, MESSAGES_RECEIVED, MESSAGES_SENT, PROTOCOL_ERRORS,
                };
                let tracker = self.request_tracker();
                let total = tracker.total_requests() as f64;
                let errors = tracker.total_errors() as f64;
                metrics.insert(MESSAGES_SENT.to_string(), total);
                metrics.insert(MESSAGES_RECEIVED.to_string(), total);
                metrics.insert(CONNECTION_COUNT.to_string(), 1.0);
                metrics.insert(PROTOCOL_ERRORS.to_string(), errors);
            }
            "context_state" => {
                use crate::monitoring::metric_names::context_state::{
                    ACTIVE_SESSIONS, CACHE_HIT_RATE, CONTEXT_SIZE, PERSISTENCE_LATENCY,
                };
                metrics.insert(
                    ACTIVE_SESSIONS.to_string(),
                    self.context_session_count() as f64,
                );
                metrics.insert(CONTEXT_SIZE.to_string(), 0.0);
                metrics.insert(CACHE_HIT_RATE.to_string(), 0.0);
                metrics.insert(PERSISTENCE_LATENCY.to_string(), 0.0);
            }
            "agent_deployment" => {
                use crate::monitoring::metric_names::agent_deployment::{
                    DEPLOYED_AGENTS, DEPLOYMENT_TIME, FAILED_DEPLOYMENTS, RUNNING_AGENTS,
                };
                metrics.insert(DEPLOYED_AGENTS.to_string(), 0.0);
                metrics.insert(RUNNING_AGENTS.to_string(), 0.0);
                metrics.insert(FAILED_DEPLOYMENTS.to_string(), 0.0);
                metrics.insert(DEPLOYMENT_TIME.to_string(), 0.0);
            }
            // External capability domains — squirrel does not own these metrics.
            // Return empty; the owning primal (songBird/toadStool/nestGate/bearDog)
            // should be queried via its own system.metrics endpoint at runtime.
            "network" | "compute" | "storage" | "security" => {}
            _ => {
                use crate::monitoring::metric_names::default::{STATUS, UPTIME};
                metrics.insert(STATUS.to_string(), 1.0);
                #[expect(clippy::cast_precision_loss, reason = "uptime seconds fits f64")]
                let uptime = universal_constants::sys_info::uptime_seconds().unwrap_or(0) as f64;
                metrics.insert(UPTIME.to_string(), uptime);
            }
        }

        Ok(metrics)
    }

    /// Create a snapshot of current metrics
    async fn create_snapshot(&self) -> Result<(), PrimalError> {
        let values: HashMap<String, MetricValue> = self
            .values
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        let system_metrics = self.system_metrics.read().await;

        let snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            metrics: values,
            system_metrics: system_metrics.clone(),
        };

        let mut history = self.history.write().await;
        history.push(snapshot);

        // Limit history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        Ok(())
    }

    #[cfg(feature = "system-metrics")]
    async fn get_cpu_usage(&self) -> Result<f64, PrimalError> {
        let cpu_usage = universal_constants::sys_info::system_cpu_usage_percent().unwrap_or(0.0);
        debug!("Current CPU usage: {:.2}%", cpu_usage);
        Ok(cpu_usage)
    }

    #[cfg(feature = "system-metrics")]
    async fn get_memory_usage(&self) -> Result<u64, PrimalError> {
        let used_memory = universal_constants::sys_info::memory_info()
            .map(|m| m.used)
            .unwrap_or(0);
        debug!("Current memory usage: {} bytes", used_memory);
        Ok(used_memory)
    }

    #[cfg(feature = "system-metrics")]
    async fn get_memory_percentage(&self) -> Result<f64, PrimalError> {
        let mem = universal_constants::sys_info::memory_info().unwrap_or_default();
        if mem.total == 0 {
            return Ok(0.0);
        }
        let percentage = (mem.used as f64 / mem.total as f64) * 100.0;
        debug!("Current memory percentage: {:.2}%", percentage);
        Ok(percentage)
    }

    async fn get_disk_usage(&self) -> Result<f64, PrimalError> {
        universal_constants::sys_info::disk_usage_percent("/")
            .map_err(|e| PrimalError::Internal(format!("disk_usage: {e}")))
    }

    async fn get_network_bytes_sent(&self) -> Result<f64, PrimalError> {
        let net = universal_constants::sys_info::network_bytes()
            .map_err(|e| PrimalError::Internal(format!("network_bytes: {e}")))?;
        #[expect(clippy::cast_precision_loss, reason = "byte counter display")]
        Ok(net.tx_bytes as f64)
    }

    async fn get_network_bytes_received(&self) -> Result<f64, PrimalError> {
        let net = universal_constants::sys_info::network_bytes()
            .map_err(|e| PrimalError::Internal(format!("network_bytes: {e}")))?;
        #[expect(clippy::cast_precision_loss, reason = "byte counter display")]
        Ok(net.rx_bytes as f64)
    }

    async fn get_active_connections(&self) -> Result<u32, PrimalError> {
        // Count established TCP connections from /proc/net/tcp
        let content = std::fs::read_to_string("/proc/net/tcp").unwrap_or_default();
        // State 01 = ESTABLISHED in /proc/net/tcp
        let count = content
            .lines()
            .skip(1)
            .filter(|line| {
                line.split_whitespace()
                    .nth(3)
                    .is_some_and(|state| state == "01")
            })
            .count();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TCP connections won't exceed u32"
        )]
        Ok(count as u32)
    }

    async fn get_request_rate(&self) -> Result<f64, PrimalError> {
        Ok(self.request_tracker.request_rate())
    }

    async fn get_error_rate(&self) -> Result<f64, PrimalError> {
        Ok(self.request_tracker.error_rate())
    }

    async fn get_avg_response_time(&self) -> Result<f64, PrimalError> {
        Ok(self.request_tracker.avg_response_time_ms())
    }

    /// Host uptime in seconds from [`universal_constants::sys_info::uptime_seconds`] (`/proc/uptime` on Linux).
    async fn get_uptime(&self) -> Result<u64, PrimalError> {
        Ok(universal_constants::sys_info::uptime_seconds().unwrap_or(0))
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_percentage: 0.0,
            disk_usage: 0.0,
            network_bytes_sent: 0.0,
            network_bytes_received: 0.0,
            active_connections: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            avg_response_time: 0.0,
            uptime: 0,
        }
    }
}
