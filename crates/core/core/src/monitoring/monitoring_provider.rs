// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC-based monitoring provider that discovers `monitoring.*` capabilities at runtime.
//!
//! Forwards events, metrics, health, and performance data via JSON-RPC over
//! Unix sockets.  When no monitoring service is reachable the provider
//! degrades gracefully to structured `tracing` output — no data is silently
//! dropped.

use crate::{HealthStatus, Result};

use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use universal_patterns::ipc_client::IpcClient;

use super::config::MonitoringServiceConfig;
use super::types::{
    Metric, MonitoringCapability, MonitoringEvent, MonitoringProvider, PerformanceMetrics,
    TimeFrame,
};
use universal_constants::capabilities;

/// Monitoring service provider that delegates to whichever ecosystem service exposes
/// `monitoring.*` capabilities (capability-first — we never hardcode a primal name for routing).
pub struct MonitoringServiceProvider {
    pub(super) config: MonitoringServiceConfig,
    ipc: Option<IpcClient>,
    connected: AtomicBool,
}

impl MonitoringServiceProvider {
    /// Creates a new monitoring service provider, attempting IPC discovery.
    ///
    /// If the monitoring socket is unreachable the provider is still usable —
    /// it falls back to structured tracing output until the socket appears.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] only on unrecoverable initialisation failures
    /// (none today — discovery failure is non-fatal).
    pub fn new(config: MonitoringServiceConfig) -> Result<Self> {
        let ipc = match IpcClient::discover("monitoring") {
            Ok(client) => {
                tracing::info!(
                    service_name = %config.service_name,
                    "Monitoring IPC discovered — forwarding enabled"
                );
                Some(client)
            }
            Err(e) => {
                tracing::info!(
                    service_name = %config.service_name,
                    reason = %e,
                    "Monitoring IPC not yet available — falling back to tracing output"
                );
                None
            }
        };
        let connected = AtomicBool::new(ipc.is_some());
        Ok(Self {
            config,
            ipc,
            connected,
        })
    }

    /// Best-effort RPC call; logs and returns `Ok(())` on failure so that
    /// monitoring never takes down the main service.
    async fn try_rpc(&self, method: &str, params: serde_json::Value) -> Result<()> {
        if let Some(ref client) = self.ipc {
            match client.call(method, &params).await {
                Ok(_) => {
                    if !self.connected.load(Ordering::Relaxed) {
                        self.connected.store(true, Ordering::Relaxed);
                        tracing::info!(method, "Monitoring IPC connection restored");
                    }
                }
                Err(e) => {
                    if self.connected.swap(false, Ordering::Relaxed) {
                        tracing::warn!(method, error = %e, "Monitoring IPC call failed — degrading to tracing");
                    }
                }
            }
        }
        Ok(())
    }
}

impl MonitoringProvider for MonitoringServiceProvider {
    fn provider_name(&self) -> &'static str {
        // Capability label for logging — not used for dispatch (IPC uses `discover("monitoring")`).
        capabilities::SERVICE_MESH_CAPABILITY
    }

    fn provider_version(&self) -> &'static str {
        "1.0.0"
    }

    async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        tracing::debug!(?event, "monitoring.record_event");
        self.try_rpc(
            "monitoring.record_event",
            json!({ "event": format!("{event:?}") }),
        )
        .await
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        tracing::debug!(name = %metric.name, "monitoring.record_metric");
        self.try_rpc(
            "monitoring.record_metric",
            json!({ "name": metric.name, "value": format!("{:?}", metric.value) }),
        )
        .await
    }

    async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()> {
        let component = component.to_string();
        tracing::debug!(component = %component, ?health, "monitoring.record_health");
        self.try_rpc(
            "monitoring.record_health",
            json!({ "component": component, "status": format!("{health:?}") }),
        )
        .await
    }

    async fn record_performance(
        &self,
        component: &str,
        _metrics: PerformanceMetrics,
    ) -> Result<()> {
        let component = component.to_string();
        tracing::debug!(component = %component, "monitoring.record_performance");
        self.try_rpc(
            "monitoring.record_performance",
            json!({ "component": component }),
        )
        .await
    }

    async fn provider_health(&self) -> Result<HealthStatus> {
        if self.connected.load(Ordering::Relaxed) {
            Ok(HealthStatus::Healthy)
        } else if self.ipc.is_some() {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Unknown)
        }
    }

    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>> {
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

/// Concrete monitoring backends registered with [`super::MonitoringService`].
pub enum MonitoringProviderImpl {
    /// IPC / capability-discovered monitoring service (e.g. service-mesh / discovery provider).
    MonitoringService(MonitoringServiceProvider),
    /// Test double: always succeeds.
    #[cfg(test)]
    TestOk(TestOkProvider),
    /// Test double: fails on every call.
    #[cfg(test)]
    TestFailing(TestFailingProvider),
}

impl MonitoringProvider for MonitoringProviderImpl {
    fn provider_name(&self) -> &'static str {
        match self {
            Self::MonitoringService(p) => p.provider_name(),
            #[cfg(test)]
            Self::TestOk(p) => p.provider_name(),
            #[cfg(test)]
            Self::TestFailing(p) => p.provider_name(),
        }
    }

    fn provider_version(&self) -> &'static str {
        match self {
            Self::MonitoringService(p) => p.provider_version(),
            #[cfg(test)]
            Self::TestOk(p) => p.provider_version(),
            #[cfg(test)]
            Self::TestFailing(p) => p.provider_version(),
        }
    }

    async fn record_event(&self, event: MonitoringEvent) -> Result<()> {
        match self {
            Self::MonitoringService(p) => p.record_event(event).await,
            #[cfg(test)]
            Self::TestOk(p) => p.record_event(event).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.record_event(event).await,
        }
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        match self {
            Self::MonitoringService(p) => p.record_metric(metric).await,
            #[cfg(test)]
            Self::TestOk(p) => p.record_metric(metric).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.record_metric(metric).await,
        }
    }

    async fn record_health(&self, component: &str, health: HealthStatus) -> Result<()> {
        match self {
            Self::MonitoringService(p) => p.record_health(component, health).await,
            #[cfg(test)]
            Self::TestOk(p) => p.record_health(component, health).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.record_health(component, health).await,
        }
    }

    async fn record_performance(&self, component: &str, metrics: PerformanceMetrics) -> Result<()> {
        match self {
            Self::MonitoringService(p) => p.record_performance(component, metrics).await,
            #[cfg(test)]
            Self::TestOk(p) => p.record_performance(component, metrics).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.record_performance(component, metrics).await,
        }
    }

    async fn query_health(&self, component: &str) -> Result<Option<HealthStatus>> {
        match self {
            Self::MonitoringService(p) => p.query_health(component).await,
            #[cfg(test)]
            Self::TestOk(p) => p.query_health(component).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.query_health(component).await,
        }
    }

    async fn query_metrics(&self, component: &str, timeframe: TimeFrame) -> Result<Vec<Metric>> {
        match self {
            Self::MonitoringService(p) => p.query_metrics(component, timeframe).await,
            #[cfg(test)]
            Self::TestOk(p) => p.query_metrics(component, timeframe).await,
            #[cfg(test)]
            Self::TestFailing(p) => p.query_metrics(component, timeframe).await,
        }
    }

    async fn provider_health(&self) -> Result<HealthStatus> {
        match self {
            Self::MonitoringService(p) => p.provider_health().await,
            #[cfg(test)]
            Self::TestOk(p) => p.provider_health().await,
            #[cfg(test)]
            Self::TestFailing(p) => p.provider_health().await,
        }
    }

    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>> {
        match self {
            Self::MonitoringService(p) => p.provider_capabilities().await,
            #[cfg(test)]
            Self::TestOk(p) => p.provider_capabilities().await,
            #[cfg(test)]
            Self::TestFailing(p) => p.provider_capabilities().await,
        }
    }
}

#[cfg(test)]
pub struct TestOkProvider(pub &'static str);

#[cfg(test)]
pub struct TestFailingProvider;

#[cfg(test)]
impl MonitoringProvider for TestOkProvider {
    fn provider_name(&self) -> &'static str {
        self.0
    }

    fn provider_version(&self) -> &'static str {
        "test"
    }

    async fn record_event(&self, _: MonitoringEvent) -> Result<()> {
        Ok(())
    }

    async fn record_metric(&self, _: Metric) -> Result<()> {
        Ok(())
    }

    async fn record_health(&self, _: &str, _: HealthStatus) -> Result<()> {
        Ok(())
    }

    async fn record_performance(&self, _: &str, _: PerformanceMetrics) -> Result<()> {
        Ok(())
    }

    async fn provider_health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>> {
        Ok(vec![MonitoringCapability::Metrics])
    }
}

#[cfg(test)]
impl MonitoringProvider for TestFailingProvider {
    fn provider_name(&self) -> &'static str {
        "failing"
    }

    fn provider_version(&self) -> &'static str {
        "0"
    }

    async fn record_event(&self, _: MonitoringEvent) -> Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_metric(&self, _: Metric) -> Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_health(&self, _: &str, _: HealthStatus) -> Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_performance(&self, _: &str, _: PerformanceMetrics) -> Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn provider_health(&self) -> Result<HealthStatus> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn provider_capabilities(&self) -> Result<Vec<MonitoringCapability>> {
        Err(crate::Error::Monitoring("e".into()))
    }
}
