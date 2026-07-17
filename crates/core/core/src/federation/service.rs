// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use dashmap::DashMap;
use std::sync::{Arc, RwLock};

use super::service_types::{FederationState, capability_unavailable_federation};
use super::types::{FederationConfig, FederationStats, ScalingPolicy};
use crate::{
    FederationLoadBalancer, FederationStatus, FederationTopology, LoadMetrics, Result,
    SquirrelInstance, monitoring::MonitoringService,
};
use universal_constants::safe_cast::usize_to_u32_saturating;

/// Federation service for managing distributed Squirrel MCP instances
#[derive(Clone)]
#[expect(
    dead_code,
    reason = "public API — consumers use federation coordination"
)]
pub struct FederationService {
    pub(super) config: FederationConfig,
    pub(super) state: Arc<FederationState>,
    pub(super) instances: Arc<DashMap<String, SquirrelInstance>>,
    pub(super) federation_topology: Arc<RwLock<FederationTopology>>,
    pub(super) load_balancer: Arc<FederationLoadBalancer>,
    pub(super) monitoring: Arc<MonitoringService>,
    pub(super) shutdown_notify: Arc<tokio::sync::Notify>,
    pub(super) load_metrics: Arc<LoadMetrics>,
    pub(super) scaling_policy: Arc<ScalingPolicy>,
}

impl FederationService {
    /// Create a new federation service
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] if the service cannot be constructed.
    pub fn new(config: FederationConfig) -> Result<Self> {
        let federation_id: Arc<str> = format!("fed-{}", uuid::Uuid::new_v4()).into();

        let state = Arc::new(FederationState {
            status: RwLock::new(FederationStatus::Forming),
            federation_id,
            leader_node: RwLock::new(None),
            last_scale_event: RwLock::new(None),
            total_capacity: RwLock::new(0),
            current_utilization: RwLock::new(0.0),
        });

        let load_metrics = Arc::new(LoadMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_usage: 0.0,
            active_tasks: 0,
            queue_length: 0,
            response_time: std::time::Duration::from_millis(0),
            error_rate: 0.0,
        });

        let scaling_policy = Arc::new(ScalingPolicy {
            scale_up_threshold: 0.7,   // 70% utilization
            scale_down_threshold: 0.3, // 30% utilization
            scale_up_cooldown: chrono::Duration::minutes(5),
            scale_down_cooldown: chrono::Duration::minutes(10),
            min_instances: 1,
            max_instances: config.max_instances,
            scale_factor: 1.5,
        });

        // Note: HTTP client removed — delegate to service mesh for any HTTP needs

        Ok(Self {
            config,
            state,
            instances: Arc::new(DashMap::new()),
            federation_topology: Arc::new(RwLock::new(FederationTopology::Star)),
            load_balancer: Arc::new(FederationLoadBalancer::new(Arc::clone(&load_metrics))),
            monitoring: Arc::new(MonitoringService::new(
                crate::monitoring::MonitoringConfig::default(),
            )),
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
            load_metrics,
            scaling_policy,
        })
    }

    /// Start the federation service
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] if federation initialization fails.
    #[expect(
        clippy::unused_async,
        reason = "Async API matches callers that await start (e.g. squirrel-mcp-server)"
    )]
    pub async fn start(&self) -> Result<()> {
        tracing::info!(
            "Starting federation service for node: {}",
            self.config.node_id
        );

        if self.config.federation_enabled {
            // Initialize federation
            self.initialize_federation()?;
        } else {
            tracing::info!("Federation disabled, operating in standalone mode");
            *self
                .state
                .status
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = FederationStatus::Active;
        }

        // Start background tasks
        self.start_background_tasks();

        tracing::info!("Federation service started successfully");
        Ok(())
    }

    /// Initialize federation
    fn initialize_federation(&self) -> Result<()> {
        *self
            .state
            .status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = FederationStatus::Forming;

        // Try to discover existing federation nodes
        self.discover_federation_nodes();

        // Determine if we should be the leader or join existing federation
        if self.instances.is_empty() {
            // No other nodes found, we become the leader
            *self
                .state
                .leader_node
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(Arc::from(self.config.node_id.as_str()));
            *self
                .state
                .status
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = FederationStatus::Active;
            tracing::info!("No existing federation found, becoming leader node");
        } else {
            // Join existing federation
            self.join_existing_federation()?;
        }

        Ok(())
    }

    /// Discover existing federation nodes
    fn discover_federation_nodes(&self) {
        // This would implement actual node discovery
        // For now, using environment variables or configuration

        if let Ok(nodes_config) = std::env::var(universal_constants::env_vars::federation::NODES) {
            for node_endpoint in nodes_config.split(',') {
                match self.probe_federation_node(node_endpoint.trim()) {
                    Ok(node) => {
                        self.instances.insert(node.id.clone(), node);
                    }
                    Err(e) => {
                        tracing::debug!("Failed to probe node {}: {}", node_endpoint, e);
                    }
                }
            }
        }
    }

    /// Probe a potential federation node
    /// NOTE: Delegates HTTP to service mesh via Unix sockets (TRUE PRIMAL pattern)
    #[expect(
        clippy::unused_self,
        reason = "Instance method for API symmetry; will use federation state when probing is implemented"
    )]
    fn probe_federation_node(&self, endpoint: &str) -> Result<SquirrelInstance> {
        Err(capability_unavailable_federation(
            "http.client",
            &format!("probe_federation_node endpoint={endpoint}"),
        ))
    }

    /// Join an existing federation
    /// NOTE: Delegates HTTP to service mesh via Unix sockets
    #[expect(
        clippy::unused_self,
        reason = "Instance method for API symmetry; will use config/state when join is implemented"
    )]
    fn join_existing_federation(&self) -> Result<()> {
        Err(capability_unavailable_federation(
            "http.client",
            "join_existing_federation",
        ))
    }

    /// Find the leader node in the federation from the **local** instance registry.
    ///
    /// Cross-node consensus and HTTP probes are Phase 2; until then this returns the
    /// lexicographically smallest registered peer as a deterministic stand-in, or
    /// [`Error::CapabilityUnavailable`] when the registry is empty (callers should use IPC
    /// discovery before relying on this).
    fn find_leader_node(&self) -> Result<SquirrelInstance> {
        // Deterministic local view: lowest instance id (not distributed consensus).

        let mut best_key: Option<String> = None;
        for entry in self.instances.iter() {
            let key = entry.key();
            if best_key
                .as_ref()
                .is_none_or(|best| key.as_str() < best.as_str())
            {
                best_key = Some(key.clone());
            }
        }

        let leader =
            best_key.and_then(|k| self.instances.get(&k).map(|entry| entry.value().clone()));

        leader.ok_or_else(|| {
            capability_unavailable_federation(
                "federation:leader",
                "find_leader_node (no peers registered locally; discover peers via IPC)",
            )
        })
    }

    /// Start background federation tasks
    /// Stop a specific instance.
    /// NOTE: Delegates to service mesh for HTTP shutdown request
    #[expect(
        clippy::unused_self,
        reason = "Instance method for API symmetry; will use federation state when HTTP shutdown is implemented"
    )]
    pub(super) fn stop_instance(&self, instance: &SquirrelInstance) -> Result<()> {
        Err(capability_unavailable_federation(
            "http.client",
            &format!("stop_instance endpoint={}", instance.endpoint),
        ))
    }

    /// Maintain federation health and topology.
    pub(super) fn maintain_federation(&self) {
        // Periodic federation maintenance tasks

        // 1. Sync federation state with other nodes
        self.sync_federation_state();

        // 2. Re-elect leader if needed
        self.check_leader_health();

        // 3. Optimize topology if needed
        self.optimize_topology();

        tracing::trace!(
            endpoint = %self.get_node_endpoint(),
            caps = ?Self::get_node_capabilities(),
            "maintain_federation: local node snapshot (operator diagnostics)"
        );
    }

    /// Sync federation state with other nodes
    ///
    /// **Phase 2**: Requires cross-node messaging (e.g. HTTP via service mesh capability
    /// `http.client`) to exchange `FederationStats` and reconcile membership.
    fn sync_federation_state(&self) {
        tracing::trace!(
            federation_id = %self.state.federation_id,
            "sync_federation_state: deferred to Phase 2 (peer IPC / service mesh)"
        );
    }

    /// Check leader health and trigger re-election if needed
    ///
    /// **Phase 2**: Requires live probes of the leader endpoint and consensus or
    /// lease-based failover. Today we only reconcile the local [`find_leader_node`] view with
    /// tracing for observability.
    fn check_leader_health(&self) {
        if self.instances.is_empty() {
            tracing::trace!("check_leader_health: no peers in local registry");
            return;
        }
        match self.find_leader_node() {
            Ok(leader) => tracing::trace!(
                leader_id = %leader.id,
                "check_leader_health: resolved leader from local registry"
            ),
            Err(e) => tracing::trace!(error = %e, "check_leader_health: no leader resolved"),
        }
    }

    /// Optimize federation topology
    ///
    /// **Phase 2**: Uses metrics from peers and `FederationTopology` to rebalance
    /// or reconfigure routing; no-op until mesh telemetry is available.
    fn optimize_topology(&self) {
        let topology = *self
            .federation_topology
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tracing::trace!(
            ?topology,
            "optimize_topology: deferred to Phase 2 (topology-aware routing)"
        );
    }

    /// Advertised HTTP endpoint for this node (config + [`resolve_node_host`]).
    ///
    /// Used for logging and future mesh handshakes; federation traffic still delegates via IPC.
    fn get_node_endpoint(&self) -> String {
        let host = Self::resolve_node_host();
        universal_constants::builders::build_http_url(&host, self.config.federation_port)
    }

    /// Resolve the host address for this node via env discovery.
    ///
    /// Tier: `NODE_IP` -> `MCP_HOST` -> `localhost`
    pub(super) fn resolve_node_host() -> String {
        std::env::var(universal_constants::env_vars::deploy::NODE_IP)
            .or_else(|_| std::env::var(universal_constants::env_vars::mcp::HOST))
            .unwrap_or_else(|_| universal_constants::network::DEFAULT_LOCALHOST.to_string())
    }

    /// Current node capabilities from niche self-knowledge ([`universal_constants::capabilities::SQUIRREL_EXPOSED_CAPABILITIES`]).
    ///
    /// Phase 2 will merge this with peer advertisements from IPC; today it is the local view only.
    fn get_node_capabilities() -> Vec<String> {
        let caps = universal_constants::capabilities::SQUIRREL_EXPOSED_CAPABILITIES;
        if caps.is_empty() {
            tracing::debug!(
                "Federation: niche self-knowledge capabilities unavailable; returning no capabilities"
            );
            return Vec::new();
        }
        caps.iter().map(|s| (*s).to_string()).collect()
    }

    // -- Scoped accessors for sibling trait impls (service_swarm.rs) --

    /// Node ID from federation config.
    pub(super) fn node_id(&self) -> &str {
        &self.config.node_id
    }

    /// Shared instance registry.
    pub(super) fn instances(&self) -> &DashMap<String, SquirrelInstance> {
        &self.instances
    }

    /// Federation identifier string.
    pub(super) fn federation_id(&self) -> &str {
        &self.state.federation_id
    }

    /// Write the current utilization gauge.
    pub(super) fn set_current_utilization(&self, value: f64) {
        *self
            .state
            .current_utilization
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }

    /// Get federation statistics
    #[must_use]
    pub fn get_federation_stats(&self) -> FederationStats {
        let instance_count = usize_to_u32_saturating(self.instances.len());
        FederationStats {
            node_id: self.config.node_id.clone(),
            federation_id: (*self.state.federation_id).to_string(),
            status: *self
                .state
                .status
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            local_instances: instance_count,
            federation_nodes: instance_count,
            total_capacity: *self
                .state
                .total_capacity
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            current_utilization: *self
                .state
                .current_utilization
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            is_leader: self
                .state
                .leader_node
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_deref()
                .is_some_and(|leader| leader == self.config.node_id),
        }
    }

    /// Shutdown the federation service
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] if teardown steps fail.
    #[expect(
        clippy::unused_async,
        reason = "Async API matches callers that await shutdown (e.g. squirrel-mcp-server) and future async teardown"
    )]
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down federation service");

        // Notify background tasks to shutdown
        self.shutdown_notify.notify_waiters();

        // Stop all local instances
        for entry in self.instances.iter() {
            let instance = entry.value();
            if let Err(e) = self.stop_instance(instance) {
                tracing::warn!("Failed to stop instance during shutdown: {}", e);
            }
        }

        // Leave federation if we're part of one
        if self.config.federation_enabled {
            self.leave_federation();
        }

        tracing::info!("Federation service shutdown complete");
        Ok(())
    }

    /// Leave the federation
    fn leave_federation(&self) {
        // Implementation would properly leave the federation
        *self
            .state
            .status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = FederationStatus::Inactive;
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
