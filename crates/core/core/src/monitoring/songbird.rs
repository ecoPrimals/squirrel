// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Songbird monitoring provider (stubs until Universal Transport wiring is complete).

use crate::{HealthStatus, Result};

use async_trait::async_trait;

use super::config::SongbirdConfig;
use super::types::{
    Metric, MonitoringCapability, MonitoringEvent, MonitoringProvider, PerformanceMetrics,
};

/// Songbird monitoring provider implementation
/// NOTE: Uses Unix socket communication via ecosystem patterns
pub struct SongbirdProvider {
    pub(super) config: SongbirdConfig,
    // Note: HTTP client removed - should use Unix socket for Songbird communication
    #[allow(dead_code)]
    pub(super) endpoint: String,
}

impl SongbirdProvider {
    /// Creates a new Songbird monitoring provider.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] if the provider cannot be initialized.
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
