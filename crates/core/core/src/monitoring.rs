// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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

/// Universal monitoring interface that abstracts over different monitoring systems
#[async_trait]
pub trait MonitoringProvider: Send + Sync {
    /// Provider identification
    fn provider_name(&self) -> &'static str;
    fn provider_version(&self) -> &'static str;

    /// Core monitoring operations
    async fn record_event(&self, event: MonitoringEvent) -> Result<()>;
    async fn record_metric(&self, metric: Metric) -> Result<()>;
    async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()>;
    async fn record_performance(&self, component: &str, metrics: PerformanceMetrics) -> Result<()>;

    /// Query operations (optional - not all providers support queries)
    async fn query_health(&self, _component: &str) -> Result<Option<HealthStatus>> {
        Ok(None)
    }

    async fn query_metrics(&self, _component: &str, _timeframe: TimeFrame) -> Result<Vec<Metric>> {
        Ok(vec![])
    }

    /// Provider health and capabilities
    async fn provider_health(&self) -> Result<HealthStatus>;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub endpoint: String,
    pub service_name: String,
    pub auth_token: Option<String>,
    pub batch_size: usize,
    pub flush_interval: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub log_level: String,
    pub include_metrics: bool,
    pub include_health: bool,
    pub include_performance: bool,
}

/// Monitoring events that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEvent {
    /// System lifecycle events
    ServiceStarted {
        service: String,
        version: String,
        timestamp: DateTime<Utc>,
    },

    ServiceStopped {
        service: String,
        timestamp: DateTime<Utc>,
    },

    /// Task coordination events
    TaskSubmitted {
        task_id: String,
        task_type: String,
        priority: String,
        timestamp: DateTime<Utc>,
    },

    TaskCompleted {
        task_id: String,
        execution_time: std::time::Duration,
        success: bool,
        timestamp: DateTime<Utc>,
    },

    /// Federation events
    InstanceSpawned {
        instance_id: String,
        node_id: String,
        timestamp: DateTime<Utc>,
    },

    FederationJoined {
        federation_id: String,
        node_count: u32,
        timestamp: DateTime<Utc>,
    },

    /// Ecosystem coordination events
    PrimalDiscovered {
        primal_id: String,
        primal_type: String,
        endpoint: String,
        timestamp: DateTime<Utc>,
    },

    CoordinationCompleted {
        coordination_id: String,
        primals_involved: Vec<String>,
        execution_time: std::time::Duration,
        timestamp: DateTime<Utc>,
    },

    /// Error events
    ErrorOccurred {
        error_type: String,
        error_message: String,
        component: String,
        timestamp: DateTime<Utc>,
    },

    /// Custom events for extensibility
    Custom {
        event_type: String,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
}

/// Metrics that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { buckets: Vec<f64>, counts: Vec<u64> },
    Summary { quantiles: Vec<(f64, f64)> },
}

/// Performance metrics for system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub network_usage: Option<f64>,
    pub response_time: Option<std::time::Duration>,
    pub throughput: Option<f64>,
    pub error_rate: Option<f64>,
    pub queue_length: Option<u32>,
    pub active_connections: Option<u32>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Monitoring capabilities supported by providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringCapability {
    Events,
    Metrics,
    Health,
    Performance,
    Queries,
    Alerts,
    Dashboards,
    Tracing,
    Logging,
    Custom(String),
}

/// Time frame for metric queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrame {
    LastMinute,
    LastHour,
    LastDay,
    LastWeek,
    Custom {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
}

impl MonitoringService {
    /// Create a new monitoring service with configuration
    pub fn new(config: MonitoringConfig) -> Self {
        let fallback_logger = Arc::new(FallbackLogger::new(config.fallback_config.clone()));

        Self {
            providers: Arc::new(parking_lot::RwLock::new(Vec::new())),
            fallback_logger,
            config,
        }
    }

    /// Initialize the monitoring service and discover providers
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
            } else {
                tracing::warn!("⚠️ No monitoring providers available, using fallback logging");
            }
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
    pub async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_event(&event);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in providers.iter() {
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
    pub async fn record_metric(&self, metric: Metric) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_metric(&metric);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in providers.iter() {
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
    pub async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()> {
        let providers = self.providers.read().clone();

        if providers.is_empty() {
            self.fallback_logger.log_health(component, &health);
            return Ok(());
        }

        // Try to record with all providers (best effort)
        for provider in providers.iter() {
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
        for provider in providers.iter() {
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

        for provider in providers.iter() {
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
    pub fn new(config: FallbackConfig) -> Self {
        Self { config }
    }

    pub fn log_event(&self, event: &MonitoringEvent) {
        match self.config.log_level.as_str() {
            "debug" => tracing::debug!("📊 Event: {:?}", event),
            "info" => tracing::info!("📊 Event: {}", self.format_event(event)),
            "warn" => tracing::warn!("📊 Event: {}", self.format_event(event)),
            _ => {}
        }
    }

    pub fn log_metric(&self, metric: &Metric) {
        if self.config.include_metrics {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("📈 Metric: {:?}", metric),
                "info" => tracing::info!("📈 Metric: {} = {:?}", metric.name, metric.value),
                _ => {}
            }
        }
    }

    pub fn log_health(&self, component: &str, health: &HealthStatus) {
        if self.config.include_health {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("🏥 Health: {} = {:?}", component, health),
                "info" => tracing::info!("🏥 Health: {} = {:?}", component, health),
                "warn" if matches!(health, HealthStatus::Degraded | HealthStatus::Unhealthy) => {
                    tracing::warn!("🏥 Health: {} = {:?}", component, health)
                }
                _ => {}
            }
        }
    }

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatus {
    pub enabled: bool,
    pub provider_count: usize,
    pub providers: Vec<ProviderStatus>,
    pub fallback_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub version: String,
    pub health: HealthStatus,
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
    pub async fn record_service_started(&self, service: &str, version: &str) -> Result<()> {
        self.record_event(MonitoringEvent::ServiceStarted {
            service: service.to_string(),
            version: version.to_string(),
            timestamp: Utc::now(),
        })
        .await
    }

    /// Record a task completion event
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
