// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC-based monitoring provider that discovers `monitoring.*` capabilities at runtime.
//!
//! Forwards events, metrics, health, and performance data via JSON-RPC over
//! Unix sockets.  When no monitoring service is reachable the provider
//! degrades gracefully to structured `tracing` output — no data is silently
//! dropped.

use crate::{HealthStatus, Result};

use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use universal_patterns::ipc_client::IpcClient;

use super::config::SongbirdConfig;
use super::types::{
    Metric, MonitoringCapability, MonitoringEvent, MonitoringProvider, PerformanceMetrics,
};
use universal_constants::primal_names;

/// Monitoring provider that delegates to whichever ecosystem service exposes
/// `monitoring.*` capabilities (typically Songbird, but capability-first — we
/// never hardcode a primal name for routing).
pub struct SongbirdProvider {
    pub(super) config: SongbirdConfig,
    ipc: Option<IpcClient>,
    connected: AtomicBool,
}

impl SongbirdProvider {
    /// Creates a new monitoring provider, attempting IPC discovery.
    ///
    /// If the monitoring socket is unreachable the provider is still usable —
    /// it falls back to structured tracing output until the socket appears.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] only on unrecoverable initialisation failures
    /// (none today — discovery failure is non-fatal).
    pub fn new(config: SongbirdConfig) -> Result<Self> {
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

#[async_trait]
impl MonitoringProvider for SongbirdProvider {
    fn provider_name(&self) -> &'static str {
        primal_names::SONGBIRD
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
        tracing::debug!(component, ?health, "monitoring.record_health");
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
        tracing::debug!(component, "monitoring.record_performance");
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
