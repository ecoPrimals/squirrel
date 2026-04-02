// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`EcosystemService`] construction, lifecycle, background loops, and public accessors.

use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;

use super::ecosystem_state::{CoordinationStats, EcosystemState, ServiceStatus, metric_u64_as_f64};
use crate::{
    EcosystemConfig, EcosystemMode, HealthStatus, MonitoringService, PerformanceMetrics,
    PrimalCoordinator, PrimalEndpoint, Result, SQUIRREL_MCP_VERSION,
};

/// Main ecosystem coordination service for Squirrel MCP
///
/// This service implements the sovereign primal pattern where Squirrel MCP:
/// - Operates completely independently when needed
/// - Coordinates with other primals when available
/// - Provides multi-MCP task routing and coordination
/// - Supports federation and scaling across nodes
#[derive(Clone)]
pub struct EcosystemService {
    pub(crate) config: EcosystemConfig,
    pub(crate) state: Arc<EcosystemState>,
    pub(crate) discovered_primals: Arc<DashMap<String, PrimalEndpoint>>,
    // Note: HTTP removed - use Songbird via Unix sockets for any HTTP needs
    pub(crate) shutdown_notify: Arc<Notify>,
    pub(crate) monitoring: Arc<MonitoringService>,
}

impl EcosystemService {
    /// Create a new ecosystem service
    ///
    /// # Errors
    ///
    /// Returns an error if the service cannot be constructed.
    pub fn new(config: EcosystemConfig, monitoring: Arc<MonitoringService>) -> Result<Self> {
        let service_id = format!("squirrel-{}", uuid::Uuid::new_v4());
        let node_id =
            std::env::var("NODE_ID").unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4()));

        let state = Arc::new(EcosystemState {
            service_id,
            node_id,
            status: RwLock::new(ServiceStatus::Starting),
            registration_time: Utc::now(),
            last_health_check: RwLock::new(Utc::now()),
            coordination_stats: RwLock::new(CoordinationStats::default()),
        });

        // Note: HTTP client removed - delegate to Songbird via Unix sockets

        Ok(Self {
            config,
            state,
            discovered_primals: Arc::new(DashMap::new()),
            shutdown_notify: Arc::new(Notify::new()),
            monitoring,
        })
    }

    /// Start the ecosystem service
    ///
    /// # Errors
    ///
    /// Returns an error if coordinated or sovereign startup fails.
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting Squirrel MCP ecosystem service");

        // Record service startup
        let _ = self
            .monitoring
            .record_service_started("ecosystem", SQUIRREL_MCP_VERSION)
            .await;

        // Update status to starting
        *self.state.status.write() = ServiceStatus::Starting;

        if !self.config.enabled {
            tracing::info!("Ecosystem coordination disabled, operating in standalone mode");
            *self.state.status.write() = ServiceStatus::Standalone;
            return Ok(());
        }

        match self.config.mode {
            EcosystemMode::Standalone => {
                tracing::info!("Operating in standalone mode");
                *self.state.status.write() = ServiceStatus::Standalone;
            }
            EcosystemMode::Sovereign => {
                tracing::info!(
                    "Operating in sovereign mode - autonomous with optional coordination"
                );
                self.start_sovereign_mode().await?;
            }
            EcosystemMode::Coordinated => {
                tracing::info!("Operating in coordinated mode - requires ecosystem coordination");
                self.start_coordinated_mode().await?;
            }
        }

        // Start background tasks
        self.start_background_tasks();

        tracing::info!("Squirrel MCP ecosystem service started successfully");
        Ok(())
    }

    /// Start sovereign mode - autonomous operation with optional coordination
    async fn start_sovereign_mode(&self) -> Result<()> {
        *self.state.status.write() = ServiceStatus::Discovering;

        // Attempt to discover and register with ecosystem
        match self.discover_and_register().await {
            Ok(()) => {
                tracing::info!("Successfully connected to ecosystem");
                *self.state.status.write() = ServiceStatus::Coordinating;
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to ecosystem, operating standalone: {}",
                    e
                );
                *self.state.status.write() = ServiceStatus::Standalone;
                // This is not an error in sovereign mode - we can operate alone
            }
        }

        Ok(())
    }

    /// Start coordinated mode - requires ecosystem coordination
    async fn start_coordinated_mode(&self) -> Result<()> {
        *self.state.status.write() = ServiceStatus::Discovering;

        // Must successfully connect to ecosystem
        self.discover_and_register().await?;

        *self.state.status.write() = ServiceStatus::Coordinating;
        tracing::info!("Successfully connected to ecosystem in coordinated mode");

        Ok(())
    }

    /// Discover and register with the ecosystem
    async fn discover_and_register(&self) -> Result<()> {
        // First, try to discover other primals
        self.discover_primals().await?;

        // Register with ecosystem if possible
        self.register_with_ecosystem().await?;

        Ok(())
    }

    /// Start background coordination tasks
    fn start_background_tasks(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            service.monitoring_loop().await;
        });

        let service = self.clone();
        tokio::spawn(async move {
            service.discovery_loop().await;
        });
    }

    /// Background monitoring loop - delegates health checks and stats to monitoring service
    async fn monitoring_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config
                .discovery
                .probe_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(30)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Record health status
                    let health = self.get_current_health();
                    let _ = self.monitoring.record_health("ecosystem", health).await;

                    // Record performance metrics
                    let performance_metrics = {
                        let stats = self.state.coordination_stats.read();
                        let error_rate = if stats.tasks_coordinated > 0 {
                            metric_u64_as_f64(stats.coordination_failures)
                                / metric_u64_as_f64(stats.tasks_coordinated)
                        } else {
                            0.0
                        };

                        PerformanceMetrics {
                            cpu_usage: None,
                            memory_usage: None,
                            network_usage: None,
                            response_time: None,
                            throughput: Some(metric_u64_as_f64(stats.tasks_coordinated)),
                            error_rate: Some(error_rate),
                            queue_length: None,
                            active_connections: Some(
                                u32::try_from(self.discovered_primals.len()).unwrap_or(u32::MAX),
                            ),
                            custom_metrics: {
                                let mut custom = HashMap::new();
                                custom.insert("primals_discovered".to_string(), f64::from(stats.primals_discovered));
                                custom.insert("federation_nodes".to_string(), f64::from(stats.federation_nodes));
                                custom
                            },
                        }
                    };

                    let _ = self.monitoring.record_performance("ecosystem", performance_metrics).await;

                    // Update last health check time
                    *self.state.last_health_check.write() = Utc::now();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Monitoring loop shutting down");
                    break;
                }
            }
        }
    }

    /// Background discovery loop
    async fn discovery_loop(&self) {
        if !self.config.discovery.auto_discovery {
            return;
        }

        let mut interval = tokio::time::interval(
            self.config
                .discovery
                .probe_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(60)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.discover_primals().await {
                        tracing::debug!("Discovery cycle failed: {}", e);
                        let _ = self.monitoring.record_error("discovery", &e.to_string(), "ecosystem").await;
                    }
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Discovery loop shutting down");
                    break;
                }
            }
        }
    }

    /// Get current health status for monitoring
    pub(crate) fn get_current_health(&self) -> HealthStatus {
        match self.get_status() {
            ServiceStatus::Starting => HealthStatus::Unknown,
            ServiceStatus::Standalone
            | ServiceStatus::Coordinating
            | ServiceStatus::Discovering => HealthStatus::Healthy,
            ServiceStatus::Degraded => HealthStatus::Degraded,
            ServiceStatus::Stopping => HealthStatus::Unhealthy,
        }
    }

    /// Get Squirrel MCP endpoint with multi-tier resolution
    ///
    /// Resolution tiers:
    /// 1. `SQUIRREL_MCP_ENDPOINT` (full endpoint)
    /// 2. `SQUIRREL_PORT` / `SQUIRREL_SERVER_PORT` (port override via `universal_constants`)
    /// 3. Default: `http://{discovered_host}:{discovered_port}`
    #[must_use]
    pub fn get_endpoint(&self) -> String {
        std::env::var("SQUIRREL_MCP_ENDPOINT").unwrap_or_else(|_| {
            let port = universal_constants::network::squirrel_primal_port();
            let host = universal_constants::config_helpers::get_host(
                "SQUIRREL_HOST",
                universal_constants::network::DEFAULT_LOCALHOST,
            );
            universal_constants::builders::build_http_url(&host, port)
        })
    }

    /// Get service metadata for registration
    #[must_use]
    pub fn get_service_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), SQUIRREL_MCP_VERSION.to_string());
        metadata.insert("node_id".to_string(), self.state.node_id.clone());
        metadata.insert(
            "started_at".to_string(),
            self.state.registration_time.to_rfc3339(),
        );
        metadata.insert("mode".to_string(), format!("{:?}", self.config.mode));
        metadata
    }

    /// Get current service status
    #[must_use]
    pub fn get_status(&self) -> ServiceStatus {
        self.state.status.read().clone()
    }

    /// Get discovered primals
    #[must_use]
    pub fn get_discovered_primals(&self) -> Vec<PrimalEndpoint> {
        self.discovered_primals
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Shutdown the service
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown steps fail.
    #[expect(
        clippy::unused_async,
        reason = "Public shutdown API is async for consistency with other services"
    )]
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down ecosystem service");

        // Update status
        *self.state.status.write() = ServiceStatus::Stopping;

        // Unregister from ecosystem
        Self::unregister_from_ecosystem();

        // Notify background tasks to shutdown
        self.shutdown_notify.notify_waiters();

        tracing::info!("Ecosystem service shutdown complete");
        Ok(())
    }

    /// Unregister from the ecosystem via manifest cleanup.
    ///
    /// Removes the primal manifest so discovery systems no longer advertise this
    /// instance. Socket-level cleanup (biomeOS heartbeat, Songbird) is handled by
    /// the signal handler in `main.rs`.
    fn unregister_from_ecosystem() {
        let family_id = std::env::var("FAMILY_ID").ok();
        if let Err(e) = universal_patterns::manifest_discovery::remove_manifest(
            "squirrel",
            family_id.as_deref(),
        ) {
            tracing::warn!("Failed to remove primal manifest during unregister: {e}");
        }
    }
}
