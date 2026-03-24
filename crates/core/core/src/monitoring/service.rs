// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! [`MonitoringService`] — delegation to [`crate::monitoring::MonitoringProvider`]s with fallback.

use crate::{Error, HealthStatus, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use universal_constants::primal_names;

use super::config::{MonitoringConfig, SongbirdConfig};
use super::fallback::FallbackLogger;
use super::songbird::SongbirdProvider;
use super::types::{
    Metric, MetricValue, MonitoringEvent, MonitoringProvider, MonitoringStatus, PerformanceMetrics,
    ProviderStatus,
};

/// Main monitoring service that delegates to available providers
#[derive(Clone)]
pub struct MonitoringService {
    providers: Arc<parking_lot::RwLock<Vec<Arc<dyn MonitoringProvider>>>>,
    fallback_logger: Arc<FallbackLogger>,
    config: MonitoringConfig,
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
    pub fn initialize(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Monitoring service disabled");
            return Ok(());
        }

        tracing::info!("Initializing universal monitoring service");

        // Try to initialize Songbird provider if configured
        if let Some(ref songbird_config) = self.config.songbird_config {
            match Self::try_initialize_songbird(songbird_config) {
                Ok(provider) => {
                    self.add_provider(provider);
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
            match Self::try_initialize_provider(provider_name, provider_config) {
                Ok(provider) => {
                    self.add_provider(provider);
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
    pub fn add_provider(&self, provider: Arc<dyn MonitoringProvider>) {
        self.providers.write().push(provider);
    }

    /// Remove a monitoring provider
    pub fn remove_provider(&self, provider_name: &str) {
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
            if let Err(e) = provider.record_health(component, health).await {
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
        let providers: Vec<_> = self.providers.read().iter().map(Arc::clone).collect();

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
    #[must_use]
    pub fn get_providers(&self) -> Vec<String> {
        self.providers
            .read()
            .iter()
            .map(|p| p.provider_name().to_string())
            .collect()
    }

    /// Get monitoring service status
    pub async fn get_status(&self) -> MonitoringStatus {
        let providers: Vec<_> = self.providers.read().iter().map(Arc::clone).collect();
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
    fn try_initialize_songbird(config: &SongbirdConfig) -> Result<Arc<dyn MonitoringProvider>> {
        let provider = Arc::new(SongbirdProvider::new(config.clone())?);
        Ok(provider)
    }

    /// Try to initialize a generic provider
    fn try_initialize_provider(
        name: &str,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn MonitoringProvider>> {
        match name {
            primal_names::SONGBIRD => {
                let songbird_config: SongbirdConfig = serde_json::from_value(config.clone())?;
                Self::try_initialize_songbird(&songbird_config)
            }
            _ => Err(Error::Monitoring(format!("Unknown provider: {name}"))),
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
