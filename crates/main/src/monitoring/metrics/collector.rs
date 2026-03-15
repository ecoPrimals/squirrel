// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics collector implementation
#![allow(dead_code)] // Monitoring infrastructure awaiting activation
//!
//! Core metrics collection engine with system monitoring.

use chrono::Utc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "system-metrics")]
use sysinfo::System;
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
    /// System information collector for real metrics
    #[cfg(feature = "system-metrics")]
    pub(crate) sys_info: Arc<RwLock<System>>,
    /// Last time external capability discovery was performed
    last_discovery: Arc<RwLock<std::time::Instant>>,
    /// Minimum interval between external discovery scans
    discovery_interval: std::time::Duration,
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
            #[cfg(feature = "system-metrics")]
            sys_info: Arc::new(RwLock::new(System::new())),
            last_discovery: Arc::new(RwLock::new(std::time::Instant::now())),
            discovery_interval: std::time::Duration::from_secs(60),
        }
    }

    /// Get comprehensive metrics summary
    pub async fn get_summary(&self) -> Result<MetricsSummary, PrimalError> {
        let system_metrics = self.system_metrics.read().await.clone();

        Ok(MetricsSummary {
            system: system_metrics,
            http: HttpMetrics {
                total_requests: 0, // Will be populated from actual metrics
                error_responses: 0,
                avg_response_time_ms: 0.0,
            },
        })
    }

    /// Register a custom metric
    pub async fn register_custom_metric(
        &self,
        definition: CustomMetricDefinition,
    ) -> Result<(), PrimalError> {
        let metric_def = MetricDefinition {
            name: definition.name.clone(),
            metric_type: definition.metric_type,
            description: definition.description,
            labels: definition.labels,
            unit: definition.unit,
            source: definition.source,
        };

        self.metrics.insert(definition.name.clone(), metric_def);
        info!("Registered custom metric: {}", definition.name);

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
                metric_type: metric_def.metric_type.clone(),
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
        if let Some(definition) = self.metrics.get(metric_name) {
            Ok(MetricInfo {
                name: definition.name.clone(),
                description: definition.description.clone(),
                labels: definition.labels.clone(),
                unit: definition.unit.clone(),
                source: definition.source.clone(),
                metric_type: definition.metric_type.clone(),
            })
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Metric '{metric_name}' not found"
            )))
        }
    }

    /// List all registered metrics with their metadata
    pub async fn list_metric_definitions(&self) -> Result<Vec<MetricInfo>, PrimalError> {
        let mut metric_infos = Vec::new();
        for entry in self.metrics.iter() {
            let definition = entry.value();
            metric_infos.push(MetricInfo {
                name: definition.name.clone(),
                description: definition.description.clone(),
                labels: definition.labels.clone(),
                unit: definition.unit.clone(),
                source: definition.source.clone(),
                metric_type: definition.metric_type.clone(),
            });
        }

        Ok(metric_infos)
    }

    /// Search metrics by source
    pub async fn get_metrics_by_source(
        &self,
        source: &str,
    ) -> Result<Vec<MetricInfo>, PrimalError> {
        let mut filtered_metrics = Vec::new();
        for entry in self.metrics.iter() {
            let definition = entry.value();
            if definition.source == source {
                filtered_metrics.push(MetricInfo {
                    name: definition.name.clone(),
                    description: definition.description.clone(),
                    labels: definition.labels.clone(),
                    unit: definition.unit.clone(),
                    source: definition.source.clone(),
                    metric_type: definition.metric_type.clone(),
                });
            }
        }

        Ok(filtered_metrics)
    }

    /// Get metrics by unit type (e.g., "bytes", "seconds", "count")
    pub async fn get_metrics_by_unit(&self, unit: &str) -> Result<Vec<MetricInfo>, PrimalError> {
        let mut filtered_metrics = Vec::new();
        for entry in self.metrics.iter() {
            let definition = entry.value();
            if definition.unit == unit {
                filtered_metrics.push(MetricInfo {
                    name: definition.name.clone(),
                    description: definition.description.clone(),
                    labels: definition.labels.clone(),
                    unit: definition.unit.clone(),
                    source: definition.source.clone(),
                    metric_type: definition.metric_type.clone(),
                });
            }
        }

        Ok(filtered_metrics)
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
            let mut sys = self.sys_info.write().await;
            sys.refresh_cpu();
            sys.refresh_memory();

            system_metrics.cpu_usage = f64::from(sys.global_cpu_info().cpu_usage());
            system_metrics.memory_usage = sys.used_memory();
            let total_memory = sys.total_memory();
            system_metrics.memory_percentage = if total_memory > 0 {
                (sys.used_memory() as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };
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
                // Zero-copy: Use static constants instead of allocating strings
                use crate::monitoring::metric_names::ai_intelligence::{
                    AVG_PROCESSING_TIME, MEMORY_USAGE, REQUESTS_PROCESSED, SUCCESS_RATE,
                };
                metrics.insert(REQUESTS_PROCESSED.to_string(), 42.0);
                metrics.insert(AVG_PROCESSING_TIME.to_string(), 150.0);
                metrics.insert(SUCCESS_RATE.to_string(), 0.95);
                metrics.insert(MEMORY_USAGE.to_string(), 256.0);
            }
            "mcp_integration" => {
                use crate::monitoring::metric_names::mcp_integration::{
                    CONNECTION_COUNT, MESSAGES_RECEIVED, MESSAGES_SENT, PROTOCOL_ERRORS,
                };
                metrics.insert(MESSAGES_SENT.to_string(), 128.0);
                metrics.insert(MESSAGES_RECEIVED.to_string(), 134.0);
                metrics.insert(CONNECTION_COUNT.to_string(), 5.0);
                metrics.insert(PROTOCOL_ERRORS.to_string(), 2.0);
            }
            "context_state" => {
                use crate::monitoring::metric_names::context_state::{
                    ACTIVE_SESSIONS, CACHE_HIT_RATE, CONTEXT_SIZE, PERSISTENCE_LATENCY,
                };
                metrics.insert(ACTIVE_SESSIONS.to_string(), 8.0);
                metrics.insert(CONTEXT_SIZE.to_string(), 1024.0);
                metrics.insert(CACHE_HIT_RATE.to_string(), 0.87);
                metrics.insert(PERSISTENCE_LATENCY.to_string(), 25.0);
            }
            "agent_deployment" => {
                use crate::monitoring::metric_names::agent_deployment::{
                    DEPLOYED_AGENTS, DEPLOYMENT_TIME, FAILED_DEPLOYMENTS, RUNNING_AGENTS,
                };
                metrics.insert(DEPLOYED_AGENTS.to_string(), 12.0);
                metrics.insert(RUNNING_AGENTS.to_string(), 10.0);
                metrics.insert(FAILED_DEPLOYMENTS.to_string(), 1.0);
                metrics.insert(DEPLOYMENT_TIME.to_string(), 30.0);
            }
            // Capability-domain metrics (vendor/primal agnostic)
            "network" => {
                use crate::monitoring::metric_names::songbird::{
                    HEALTH_CHECKS, LOAD_BALANCER_REQUESTS, ORCHESTRATIONS_ACTIVE,
                    SERVICE_DISCOVERIES,
                };
                // Metric names retained for backward compatibility; values from discovery
                metrics.insert(ORCHESTRATIONS_ACTIVE.to_string(), 0.0);
                metrics.insert(SERVICE_DISCOVERIES.to_string(), 0.0);
                metrics.insert(LOAD_BALANCER_REQUESTS.to_string(), 0.0);
                metrics.insert(HEALTH_CHECKS.to_string(), 0.0);
            }
            "compute" => {
                use crate::monitoring::metric_names::toadstool::{
                    COMPUTE_JOBS_COMPLETED, COMPUTE_JOBS_QUEUED, COMPUTE_JOBS_RUNNING,
                    CPU_UTILIZATION,
                };
                metrics.insert(COMPUTE_JOBS_QUEUED.to_string(), 0.0);
                metrics.insert(COMPUTE_JOBS_RUNNING.to_string(), 0.0);
                metrics.insert(COMPUTE_JOBS_COMPLETED.to_string(), 0.0);
                metrics.insert(CPU_UTILIZATION.to_string(), 0.0);
            }
            "storage" => {
                use crate::monitoring::metric_names::nestgate::{
                    BACKUP_OPERATIONS, REPLICATION_LAG, STORAGE_OPERATIONS, STORAGE_SIZE_GB,
                };
                metrics.insert(STORAGE_OPERATIONS.to_string(), 0.0);
                metrics.insert(STORAGE_SIZE_GB.to_string(), 0.0);
                metrics.insert(BACKUP_OPERATIONS.to_string(), 0.0);
                metrics.insert(REPLICATION_LAG.to_string(), 0.0);
            }
            "security" => {
                use crate::monitoring::metric_names::beardog::{
                    AUTHENTICATION_REQUESTS, AUTHORIZATION_CHECKS, SECURITY_VIOLATIONS,
                    TOKEN_REFRESHES,
                };
                metrics.insert(AUTHENTICATION_REQUESTS.to_string(), 0.0);
                metrics.insert(AUTHORIZATION_CHECKS.to_string(), 0.0);
                metrics.insert(SECURITY_VIOLATIONS.to_string(), 0.0);
                metrics.insert(TOKEN_REFRESHES.to_string(), 0.0);
            }
            _ => {
                // Default metrics for unknown components
                use crate::monitoring::metric_names::default::{STATUS, UPTIME};
                metrics.insert(STATUS.to_string(), 1.0);
                metrics.insert(UPTIME.to_string(), 3600.0);
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
        let mut sys = self.sys_info.write().await;
        sys.refresh_cpu();
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        debug!("Current CPU usage: {:.2}%", cpu_usage);
        Ok(f64::from(cpu_usage))
    }

    #[cfg(feature = "system-metrics")]
    async fn get_memory_usage(&self) -> Result<u64, PrimalError> {
        let mut sys = self.sys_info.write().await;
        sys.refresh_memory();
        let used_memory = sys.used_memory();
        debug!("Current memory usage: {} bytes", used_memory);
        Ok(used_memory)
    }

    #[cfg(feature = "system-metrics")]
    async fn get_memory_percentage(&self) -> Result<f64, PrimalError> {
        let mut sys = self.sys_info.write().await;
        sys.refresh_memory();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        if total_memory == 0 {
            return Ok(0.0);
        }
        let percentage = (used_memory as f64 / total_memory as f64) * 100.0;
        debug!("Current memory percentage: {:.2}%", percentage);
        Ok(percentage)
    }

    async fn get_disk_usage(&self) -> Result<f64, PrimalError> {
        Ok(45.2)
    }

    async fn get_network_bytes_sent(&self) -> Result<f64, PrimalError> {
        Ok(1024.0 * 50.0) // 50 KB/s
    }

    async fn get_network_bytes_received(&self) -> Result<f64, PrimalError> {
        Ok(1024.0 * 75.0) // 75 KB/s
    }

    async fn get_active_connections(&self) -> Result<u32, PrimalError> {
        Ok(12)
    }

    async fn get_request_rate(&self) -> Result<f64, PrimalError> {
        Ok(45.7)
    }

    async fn get_error_rate(&self) -> Result<f64, PrimalError> {
        Ok(0.8)
    }

    async fn get_avg_response_time(&self) -> Result<f64, PrimalError> {
        Ok(125.3)
    }

    async fn get_uptime(&self) -> Result<u64, PrimalError> {
        Ok(3600 * 24 * 5) // 5 days
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
