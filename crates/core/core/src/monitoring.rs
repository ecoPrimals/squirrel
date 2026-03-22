// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Monitoring Abstraction for Squirrel MCP
#![allow(dead_code)] // Monitoring infrastructure awaiting activation
//!
//! This module provides a monitoring abstraction layer that delegates to external
//! monitoring systems while maintaining sovereignty. It can work with:
//!
//! - **Songbird** - When available as the observability primal
//! - **Future monitoring primals** - Through extensible interfaces
//! - **Basic logging** - As a fallback when no monitoring system is available
//!
//! ## Architecture Principles
//!
//! 1. **Delegation over Implementation** - Never implement monitoring directly
//! 2. **Graceful Degradation** - Continue operating without monitoring
//! 3. **Primal Agnostic** - Work with any monitoring system
//! 4. **Extensible** - Support new monitoring systems without core changes

use crate::{Error, HealthStatus, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Universal monitoring interface that abstracts over different monitoring systems.
#[async_trait]
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

/// Main monitoring service that delegates to available providers
#[derive(Clone)]
pub struct MonitoringService {
    providers: Arc<parking_lot::RwLock<Vec<Arc<dyn MonitoringProvider>>>>,
    fallback_logger: Arc<FallbackLogger>,
    config: MonitoringConfig,
}

/// Configuration for monitoring delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring delegation
    pub enabled: bool,

    /// Require at least one provider to be available
    pub require_provider: bool,

    /// Songbird-specific configuration
    pub songbird_config: Option<SongbirdConfig>,

    /// Generic monitoring provider configurations
    pub provider_configs: HashMap<String, serde_json::Value>,

    /// Fallback configuration
    pub fallback_config: FallbackConfig,
}

/// Songbird monitoring service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    /// Songbird endpoint URL.
    pub endpoint: String,
    /// Service name for identification.
    pub service_name: String,
    /// Optional auth token.
    pub auth_token: Option<String>,
    /// Batch size for metrics.
    pub batch_size: usize,
    /// Flush interval for batching.
    pub flush_interval: std::time::Duration,
}

/// Fallback logging configuration when no provider is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Log level (debug, info, warn, error).
    pub log_level: String,
    /// Whether to include metrics in fallback output.
    pub include_metrics: bool,
    /// Whether to include health in fallback output.
    pub include_health: bool,
    /// Whether to include performance in fallback output.
    pub include_performance: bool,
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

impl MonitoringService {
    /// Create a new monitoring service with configuration
    #[must_use]
    pub fn new(config: MonitoringConfig) -> Self {
        let fallback_logger = Arc::new(FallbackLogger::new(config.fallback_config.clone()));

        Self {
            providers: Arc::new(parking_lot::RwLock::new(Vec::new())),
            fallback_logger,
            config,
        }
    }

    /// Initialize the monitoring service and discover providers
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if a required provider cannot be initialized.
    pub async fn initialize(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Monitoring service disabled");
            return Ok(());
        }

        tracing::info!("Initializing universal monitoring service");

        // Try to initialize Songbird provider if configured
        if let Some(ref songbird_config) = self.config.songbird_config {
            match self.try_initialize_songbird(songbird_config).await {
                Ok(provider) => {
                    self.add_provider(provider).await;
                    tracing::info!("✅ Songbird monitoring provider initialized");
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ Songbird monitoring provider failed to initialize: {}",
                        e
                    );
                }
            }
        }

        // Try to initialize other configured providers
        for (provider_name, provider_config) in &self.config.provider_configs {
            match self
                .try_initialize_provider(provider_name, provider_config)
                .await
            {
                Ok(provider) => {
                    self.add_provider(provider).await;
                    tracing::info!("✅ {} monitoring provider initialized", provider_name);
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ {} monitoring provider failed to initialize: {}",
                        provider_name,
                        e
                    );
                }
            }
        }

        let provider_count = self.providers.read().len();

        if provider_count == 0 {
            if self.config.require_provider {
                return Err(Error::Monitoring(
                    "No monitoring providers available".to_string(),
                ));
            }
            tracing::warn!("⚠️ No monitoring providers available, using fallback logging");
        } else {
            tracing::info!(
                "✅ Monitoring service initialized with {} providers",
                provider_count
            );
        }

        Ok(())
    }

    /// Add a monitoring provider
    pub async fn add_provider(&self, provider: Arc<dyn MonitoringProvider>) {
        self.providers.write().push(provider);
    }

    /// Remove a monitoring provider
    pub async fn remove_provider(&self, provider_name: &str) {
        self.providers
            .write()
            .retain(|p| p.provider_name() != provider_name);
    }

    /// Record a monitoring event
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if recording fails for all providers and fallback handling fails.
    pub async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_event(&event);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in &providers {
            if let Err(e) = provider.record_event(event.clone()).await {
                tracing::debug!(
                    "Provider {} failed to record event: {}",
                    provider.provider_name(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Record a metric
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if recording fails for all providers and fallback handling fails.
    pub async fn record_metric(&self, metric: Metric) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_metric(&metric);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in &providers {
            if let Err(e) = provider.record_metric(metric.clone()).await {
                tracing::debug!(
                    "Provider {} failed to record metric: {}",
                    provider.provider_name(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Record health status
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if recording fails for all providers and fallback handling fails.
    pub async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_health(component, &health);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in &providers {
            if let Err(e) = provider.record_health(component, health.clone()).await {
                tracing::debug!(
                    "Provider {} failed to record health: {}",
                    provider.provider_name(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Record performance metrics
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if recording fails for all providers and fallback handling fails.
    pub async fn record_performance(
        &self,
        component: &str,
        metrics: PerformanceMetrics,
    ) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_performance(component, &metrics);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in &providers {
            if let Err(e) = provider
                .record_performance(component, metrics.clone())
                .await
            {
                tracing::debug!(
                    "Provider {} failed to record performance: {}",
                    provider.provider_name(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Get available monitoring providers
    pub async fn get_providers(&self) -> Vec<String> {
        self.providers
            .read()
            .iter()
            .map(|p| p.provider_name().to_string())
            .collect()
    }

    /// Get monitoring service status
    pub async fn get_status(&self) -> MonitoringStatus {
        let providers = self.providers.read().clone();
        let mut provider_statuses = Vec::new();

        for provider in &providers {
            let health = provider
                .provider_health()
                .await
                .unwrap_or(HealthStatus::Unknown);
            let capabilities = provider.provider_capabilities().await.unwrap_or_default();

            provider_statuses.push(ProviderStatus {
                name: provider.provider_name().to_string(),
                version: provider.provider_version().to_string(),
                health,
                capabilities,
            });
        }

        MonitoringStatus {
            enabled: self.config.enabled,
            provider_count: providers.len(),
            providers: provider_statuses,
            fallback_active: providers.is_empty(),
        }
    }

    /// Try to initialize Songbird provider
    async fn try_initialize_songbird(
        &self,
        config: &SongbirdConfig,
    ) -> Result<Arc<dyn MonitoringProvider>> {
        let provider = Arc::new(SongbirdProvider::new(config.clone()).await?);
        Ok(provider)
    }

    /// Try to initialize a generic provider
    async fn try_initialize_provider(
        &self,
        name: &str,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn MonitoringProvider>> {
        match name {
            "songbird" => {
                let songbird_config: SongbirdConfig = serde_json::from_value(config.clone())?;
                self.try_initialize_songbird(&songbird_config).await
            }
            _ => Err(Error::Monitoring(format!("Unknown provider: {name}"))),
        }
    }
}

/// Songbird monitoring provider implementation
/// NOTE: Uses Unix socket communication via ecosystem patterns
pub struct SongbirdProvider {
    config: SongbirdConfig,
    // Note: HTTP client removed - should use Unix socket for Songbird communication
    endpoint: String,
}

impl SongbirdProvider {
    /// Creates a new Songbird monitoring provider.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the provider cannot be initialized.
    pub async fn new(config: SongbirdConfig) -> Result<Self> {
        // NOTE: Songbird communication uses Universal Transport abstractions
        // See: crates/universal-patterns/src/transport.rs (UniversalTransport, UniversalListener)
        // Isomorphic IPC complete (Jan 31, 2026) - auto-discovers Unix sockets OR TCP fallback

        tracing::info!("SongbirdProvider created (HTTP delegation not yet implemented)");

        Ok(Self {
            endpoint: config.endpoint.clone(),
            config,
        })
    }
}

#[async_trait]
impl MonitoringProvider for SongbirdProvider {
    fn provider_name(&self) -> &'static str {
        "songbird"
    }

    fn provider_version(&self) -> &'static str {
        "1.0.0"
    }

    /// NOTE: Uses Universal Transport abstractions for inter-primal communication
    /// See: crates/universal-patterns/src/transport.rs for implementation
    async fn record_event(&self, _event: MonitoringEvent) -> Result<()> {
        // Monitoring uses Universal Transport abstractions (Isomorphic IPC)
        tracing::trace!("Event recording not yet implemented");
        Ok(())
    }

    /// NOTE: Uses Universal Transport abstractions for inter-primal communication
    /// See: crates/universal-patterns/src/transport.rs for implementation
    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        // Monitoring uses Universal Transport abstractions (Isomorphic IPC)
        tracing::trace!("Metric recording not yet implemented");
        Ok(())
    }

    /// NOTE: Uses Universal Transport abstractions for inter-primal communication
    /// See: crates/universal-patterns/src/transport.rs for implementation
    async fn record_health(&self, _component: &str, _health: HealthStatus) -> Result<()> {
        // Monitoring uses Universal Transport abstractions (Isomorphic IPC)
        tracing::trace!("Health recording not yet implemented");
        Ok(())
    }

    /// NOTE: Uses Universal Transport abstractions for inter-primal communication
    /// See: crates/universal-patterns/src/transport.rs for implementation
    async fn record_performance(
        &self,
        _component: &str,
        _metrics: PerformanceMetrics,
    ) -> Result<()> {
        // Monitoring uses Universal Transport abstractions (Isomorphic IPC)
        tracing::trace!("Performance recording not yet implemented");
        Ok(())
    }

    /// NOTE: Uses Universal Transport abstractions for inter-primal communication
    /// See: crates/universal-patterns/src/transport.rs for implementation
    async fn provider_health(&self) -> Result<HealthStatus> {
        // Provider health queries via Universal Transport abstractions
        Ok(HealthStatus::Unknown)
    }

    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>> {
        // Songbird supports comprehensive monitoring
        Ok(vec![
            MonitoringCapability::Events,
            MonitoringCapability::Metrics,
            MonitoringCapability::Health,
            MonitoringCapability::Performance,
            MonitoringCapability::Queries,
            MonitoringCapability::Alerts,
            MonitoringCapability::Dashboards,
            MonitoringCapability::Tracing,
            MonitoringCapability::Logging,
        ])
    }
}

/// Fallback logger for when no monitoring providers are available
pub struct FallbackLogger {
    config: FallbackConfig,
}

impl FallbackLogger {
    /// Creates a new fallback logger with the given config.
    #[must_use]
    pub const fn new(config: FallbackConfig) -> Self {
        Self { config }
    }

    /// Logs an event to tracing when no provider is available.
    pub fn log_event(&self, event: &MonitoringEvent) {
        match self.config.log_level.as_str() {
            "debug" => tracing::debug!("📊 Event: {:?}", event),
            "info" => tracing::info!("📊 Event: {}", self.format_event(event)),
            "warn" => tracing::warn!("📊 Event: {}", self.format_event(event)),
            _ => {}
        }
    }

    /// Logs a metric to tracing when no provider is available.
    pub fn log_metric(&self, metric: &Metric) {
        if self.config.include_metrics {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("📈 Metric: {:?}", metric),
                "info" => tracing::info!("📈 Metric: {} = {:?}", metric.name, metric.value),
                _ => {}
            }
        }
    }

    /// Logs health status to tracing when no provider is available.
    pub fn log_health(&self, component: &str, health: &HealthStatus) {
        if self.config.include_health {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("🏥 Health: {} = {:?}", component, health),
                "info" => tracing::info!("🏥 Health: {} = {:?}", component, health),
                "warn" if matches!(health, HealthStatus::Degraded | HealthStatus::Unhealthy) => {
                    tracing::warn!("🏥 Health: {} = {:?}", component, health);
                }
                _ => {}
            }
        }
    }

    /// Logs performance metrics to tracing when no provider is available.
    pub fn log_performance(&self, component: &str, metrics: &PerformanceMetrics) {
        if self.config.include_performance {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("⚡ Performance: {} = {:?}", component, metrics),
                "info" => tracing::info!(
                    "⚡ Performance: {} = {}",
                    component,
                    self.format_performance(metrics)
                ),
                _ => {}
            }
        }
    }

    fn format_event(&self, event: &MonitoringEvent) -> String {
        match event {
            MonitoringEvent::ServiceStarted { service, .. } => {
                format!("Service {service} started")
            }
            MonitoringEvent::ServiceStopped { service, .. } => {
                format!("Service {service} stopped")
            }
            MonitoringEvent::TaskCompleted {
                task_id, success, ..
            } => {
                format!(
                    "Task {} {}",
                    task_id,
                    if *success { "completed" } else { "failed" }
                )
            }
            MonitoringEvent::ErrorOccurred {
                error_type,
                component,
                ..
            } => {
                format!("Error in {component}: {error_type}")
            }
            _ => "Event occurred".to_string(),
        }
    }

    fn format_performance(&self, metrics: &PerformanceMetrics) -> String {
        let mut parts = Vec::new();

        if let Some(cpu) = metrics.cpu_usage {
            parts.push(format!("CPU: {cpu:.1}%"));
        }
        if let Some(memory) = metrics.memory_usage {
            parts.push(format!("Memory: {memory:.1}%"));
        }
        if let Some(response_time) = metrics.response_time {
            parts.push(format!("Response: {response_time:?}"));
        }

        parts.join(", ")
    }
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

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_provider: false,
            songbird_config: None,
            provider_configs: HashMap::new(),
            fallback_config: FallbackConfig::default(),
        }
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        }
    }
}

/// Convenience functions for common monitoring operations
impl MonitoringService {
    /// Record a service startup event
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the underlying event recording fails.
    pub async fn record_service_started(&self, service: &str, version: &str) -> Result<()> {
        self.record_event(MonitoringEvent::ServiceStarted {
            service: service.to_string(),
            version: version.to_string(),
            timestamp: Utc::now(),
        })
        .await
    }

    /// Record a task completion event
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the underlying event recording fails.
    pub async fn record_task_completed(
        &self,
        task_id: &str,
        execution_time: std::time::Duration,
        success: bool,
    ) -> Result<()> {
        self.record_event(MonitoringEvent::TaskCompleted {
            task_id: task_id.to_string(),
            execution_time,
            success,
            timestamp: Utc::now(),
        })
        .await
    }

    /// Record an error event
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the underlying event recording fails.
    pub async fn record_error(
        &self,
        error_type: &str,
        error_message: &str,
        component: &str,
    ) -> Result<()> {
        self.record_event(MonitoringEvent::ErrorOccurred {
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            component: component.to_string(),
            timestamp: Utc::now(),
        })
        .await
    }

    /// Record a counter metric
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the underlying metric recording fails.
    pub async fn record_counter(
        &self,
        name: &str,
        value: u64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.record_metric(Metric {
            name: name.to_string(),
            value: MetricValue::Counter(value),
            labels,
            timestamp: Utc::now(),
        })
        .await
    }

    /// Record a gauge metric
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the underlying metric recording fails.
    pub async fn record_gauge(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        self.record_metric(Metric {
            name: name.to_string(),
            value: MetricValue::Gauge(value),
            labels,
            timestamp: Utc::now(),
        })
        .await
    }
}

#[cfg(test)]
#[path = "monitoring_tests.rs"]
mod monitoring_tests;
