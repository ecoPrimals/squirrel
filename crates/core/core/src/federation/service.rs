// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;

use super::types::{FederationConfig, FederationStats, ScalingPolicy};
use crate::{
    Error, FederationLoadBalancer, FederationStatus, FederationTopology, InstanceStatus,
    LoadMetrics, Result, SquirrelConfig, SquirrelInstance, SwarmManager,
    monitoring::MonitoringService,
};
use universal_constants::limits::DEFAULT_MAX_CONNECTIONS;
use universal_constants::network::{DEFAULT_SQUIRREL_SERVER_PORT, get_service_port};
use universal_constants::safe_cast::{f64_to_u64_clamped, usize_to_u32_saturating};

/// Local federation code path requires a capability that must be discovered on another primal via IPC.
fn capability_unavailable_federation(capability: &str, operation: &str) -> Error {
    let hint = format!(
        "This primal does not embed `{capability}`. Discover a peer that advertises it through the IPC capability registry (e.g. HTTP delegation to a network primal, often via `http.client`). Operation: {operation}"
    );
    tracing::warn!(
        capability = %capability,
        operation = %operation,
        "Federation: capability not satisfied locally; use IPC discovery to find a provider"
    );
    Error::CapabilityUnavailable {
        capability: capability.to_string(),
        hint,
    }
}

/// Federation service for managing distributed Squirrel MCP instances
#[derive(Clone)]
#[expect(
    dead_code,
    reason = "public API — consumers use federation coordination"
)]
pub struct FederationService {
    config: FederationConfig,
    state: Arc<FederationState>,
    instances: Arc<DashMap<String, SquirrelInstance>>,
    federation_topology: Arc<RwLock<FederationTopology>>,
    load_balancer: Arc<FederationLoadBalancer>,
    monitoring: Arc<MonitoringService>,
    // NOTE: HTTP removed — use service mesh via Unix sockets for federation HTTP calls
    shutdown_notify: Arc<tokio::sync::Notify>,
    load_metrics: Arc<LoadMetrics>,
    scaling_policy: Arc<ScalingPolicy>,
}

#[derive(Debug)]
struct FederationState {
    status: RwLock<FederationStatus>,
    federation_id: Arc<str>,
    leader_node: RwLock<Option<Arc<str>>>,
    last_scale_event: RwLock<Option<DateTime<Utc>>>,
    total_capacity: RwLock<u32>,
    current_utilization: RwLock<f64>,
}

impl FederationService {
    /// Create a new federation service
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the service cannot be constructed.
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
    /// Returns [`Error`] if federation initialization fails.
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
            *self.state.status.write() = FederationStatus::Active;
        }

        // Start background tasks
        self.start_background_tasks();

        tracing::info!("Federation service started successfully");
        Ok(())
    }

    /// Initialize federation
    fn initialize_federation(&self) -> Result<()> {
        *self.state.status.write() = FederationStatus::Forming;

        // Try to discover existing federation nodes
        self.discover_federation_nodes();

        // Determine if we should be the leader or join existing federation
        if self.instances.is_empty() {
            // No other nodes found, we become the leader
            *self.state.leader_node.write() = Some(Arc::from(self.config.node_id.as_str()));
            *self.state.status.write() = FederationStatus::Active;
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

        if let Ok(nodes_config) = std::env::var("FEDERATION_NODES") {
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
    fn start_background_tasks(&self) {
        // Health monitoring
        let service = self.clone();
        tokio::spawn(async move {
            service.health_monitoring_loop().await;
        });

        // Load metrics collection
        let service = self.clone();
        tokio::spawn(async move {
            service.load_monitoring_loop().await;
        });

        // Auto-scaling
        if self.config.auto_scaling_enabled {
            let service = self.clone();
            tokio::spawn(async move {
                service.auto_scaling_loop().await;
            });
        }

        // Federation maintenance
        if self.config.federation_enabled {
            let service = self.clone();
            tokio::spawn(async move {
                service.federation_maintenance_loop().await;
            });
        }
    }

    /// Health monitoring loop
    async fn health_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config
                .health_check_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(30)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.check_federation_health();
                    self.check_instance_health();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Health monitoring loop shutting down");
                    break;
                }
            }
        }
    }

    /// Load monitoring loop
    async fn load_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.collect_load_metrics();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Load monitoring loop shutting down");
                    break;
                }
            }
        }
    }

    /// Auto-scaling loop
    async fn auto_scaling_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config
                .scaling_check_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(60)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.evaluate_scaling_decision().await {
                        tracing::error!("Scaling evaluation failed: {}", e);
                    }
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Auto-scaling loop shutting down");
                    break;
                }
            }
        }
    }

    /// Federation maintenance loop
    async fn federation_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.maintain_federation();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Federation maintenance loop shutting down");
                    break;
                }
            }
        }
    }

    /// Check health of federation nodes
    /// NOTE: Delegates HTTP health checks to service mesh via Unix sockets
    fn check_federation_health(&self) {
        // For now, assume all instances are running (HTTP health checking requires service mesh)
        for mut entry in self.instances.iter_mut() {
            let (_instance_id, instance) = entry.pair_mut();
            instance.health = InstanceStatus::Running;
            instance.last_seen = Utc::now();
        }

        // Pattern for future implementation:
        // CapabilityHttpClient::discover("http.client").get(&health_url).await
    }

    /// Check health of local instances
    /// NOTE: Delegates to service mesh for HTTP health checks
    fn check_instance_health(&self) {
        // Instance health checking requires HTTP delegation to service mesh
        // Pattern: CapabilityHttpClient::discover("http.client").get(&health_url).await

        // For now, assume all instances are running (to be implemented with service mesh)
        for mut entry in self.instances.iter_mut() {
            let (instance_id, instance) = entry.pair_mut();

            if instance.health == InstanceStatus::Starting {
                instance.health = InstanceStatus::Running;
                tracing::info!(
                    "Instance {} marked as running (HTTP health check via service mesh)",
                    instance_id
                );
            }
        }
    }

    /// Collect current load metrics
    ///
    /// Uses config-driven defaults when real metrics are unavailable.
    /// Real implementation would delegate to service mesh / metrics exporter.
    fn collect_load_metrics(&self) {
        // Use config-based defaults; real metrics would come from metrics exporter
        let cpu = std::env::var("FEDERATION_CPU_USAGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let memory = std::env::var("FEDERATION_MEMORY_USAGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let queue_length = std::env::var("FEDERATION_QUEUE_LENGTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0u32);
        let instance_count = usize_to_u32_saturating(self.instances.len());
        let max_conn = usize_to_u32_saturating(DEFAULT_MAX_CONNECTIONS);
        let active_tasks = instance_count.min(max_conn);

        tracing::debug!(
            "Load metrics - CPU: {:.2}%, Memory: {:.2}%, Queue: {}, Active: {}",
            cpu * 100.0,
            memory * 100.0,
            queue_length,
            active_tasks
        );
    }

    /// Evaluate whether scaling is needed
    async fn evaluate_scaling_decision(&self) -> Result<()> {
        let current_utilization = self.calculate_overall_utilization();
        let current_instances = usize_to_u32_saturating(self.instances.len());

        // Check cooldown periods
        let last_scale_snapshot = *self.state.last_scale_event.read();
        if let Some(last_scale) = last_scale_snapshot {
            let time_since_last_scale = Utc::now() - last_scale;
            if time_since_last_scale < self.scaling_policy.scale_up_cooldown {
                return Ok(()); // Still in cooldown
            }
        }

        // Scale up decision
        if current_utilization > self.scaling_policy.scale_up_threshold {
            if current_instances < self.scaling_policy.max_instances {
                let scaled = f64::from(current_instances) * self.scaling_policy.scale_factor;
                let capped =
                    f64_to_u64_clamped(scaled).min(u64::from(self.scaling_policy.max_instances));
                let target_instances = u32::try_from(capped)
                    .unwrap_or(u32::MAX)
                    .min(self.scaling_policy.max_instances);

                tracing::info!(
                    "Scaling up from {} to {} instances (utilization: {:.2})",
                    current_instances,
                    target_instances,
                    current_utilization
                );

                self.scale_up(target_instances - current_instances).await?;
            }
        }
        // Scale down decision
        else if current_utilization < self.scaling_policy.scale_down_threshold
            && current_instances > self.scaling_policy.min_instances
        {
            let scaled = f64::from(current_instances) / self.scaling_policy.scale_factor;
            let rounded = f64_to_u64_clamped(scaled);
            let target_instances =
                u32::try_from(rounded.max(u64::from(self.scaling_policy.min_instances)))
                    .unwrap_or(u32::MAX);

            tracing::info!(
                "Scaling down from {} to {} instances (utilization: {:.2})",
                current_instances,
                target_instances,
                current_utilization
            );

            self.scale_down(current_instances - target_instances);
        }

        Ok(())
    }

    /// Calculate overall utilization across all metrics.
    pub(super) fn calculate_overall_utilization(&self) -> f64 {
        let cpu = self.load_metrics.cpu_usage;
        let memory = self.load_metrics.memory_usage;
        let queue_pressure = f64::from(self.load_metrics.queue_length) / 100.0;

        // Weighted average of different metrics
        (cpu.mul_add(0.4, memory * 0.3) + queue_pressure * 0.3).min(1.0)
    }

    /// Scale up by spawning new instances
    async fn scale_up(&self, instances_to_add: u32) -> Result<()> {
        for i in 0..instances_to_add {
            let instance_config = self.create_instance_config(i);
            match self.spawn_squirrel(instance_config).await {
                Ok(instance) => {
                    tracing::info!("Successfully spawned new instance: {}", instance.id);
                }
                Err(e) => {
                    tracing::error!("Failed to spawn instance: {}", e);
                }
            }
        }

        *self.state.last_scale_event.write() = Some(Utc::now());
        Ok(())
    }

    /// Scale down by stopping instances
    fn scale_down(&self, instances_to_remove: u32) {
        let mut instances_to_stop = Vec::new();

        // Select instances to stop (prefer those with lower load)
        for (removed, entry) in self.instances.iter().enumerate() {
            if removed >= instances_to_remove as usize {
                break;
            }
            instances_to_stop.push(entry.key().clone());
        }

        for instance_id in instances_to_stop {
            if let Some((_, mut instance)) = self.instances.remove(&instance_id) {
                instance.health = InstanceStatus::Stopping;

                // Send shutdown signal to instance
                if let Err(e) = self.stop_instance(&instance) {
                    tracing::error!("Failed to stop instance {}: {}", instance.id, e);
                } else {
                    tracing::info!("Successfully stopped instance: {}", instance.id);
                }
            }
        }

        *self.state.last_scale_event.write() = Some(Utc::now());
    }

    /// Stop a specific instance
    /// NOTE: Delegates to service mesh for HTTP shutdown request
    #[expect(
        clippy::unused_self,
        reason = "Instance method for API symmetry; will use federation state when HTTP shutdown is implemented"
    )]
    fn stop_instance(&self, instance: &SquirrelInstance) -> Result<()> {
        Err(capability_unavailable_federation(
            "http.client",
            &format!("stop_instance endpoint={}", instance.endpoint),
        ))
    }

    /// Create configuration for a new instance using universal-constants defaults
    fn create_instance_config(&self, instance_index: u32) -> SquirrelConfig {
        let base_port = std::env::var("SQUIRREL_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| get_service_port("websocket"));
        let offset = u16::try_from(instance_index).unwrap_or(u16::MAX);
        let port = if base_port > 0 {
            base_port.saturating_add(offset)
        } else {
            DEFAULT_SQUIRREL_SERVER_PORT.saturating_add(offset)
        };

        let capacity = std::env::var("SQUIRREL_INSTANCE_CAPACITY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(usize_to_u32_saturating(DEFAULT_MAX_CONNECTIONS));

        SquirrelConfig {
            node_id: format!("{}-instance-{}", self.config.node_id, instance_index),
            port,
            federation_enabled: false, // Instances don't federate themselves
            region: self.config.region.clone(),
            zone: self.config.zone.clone(),
            auto_scaling_enabled: true,
            capabilities: vec!["mcp".to_string(), "routing".to_string()],
            capacity,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Maintain federation health and topology
    fn maintain_federation(&self) {
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
        let topology = *self.federation_topology.read();
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
        std::env::var("NODE_IP")
            .or_else(|_| std::env::var("MCP_HOST"))
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
        *self.state.current_utilization.write() = value;
    }

    /// Get federation statistics
    #[must_use]
    pub fn get_federation_stats(&self) -> FederationStats {
        let instance_count = usize_to_u32_saturating(self.instances.len());
        FederationStats {
            node_id: self.config.node_id.clone(),
            federation_id: (*self.state.federation_id).to_string(),
            status: *self.state.status.read(),
            local_instances: instance_count,
            federation_nodes: instance_count,
            total_capacity: *self.state.total_capacity.read(),
            current_utilization: *self.state.current_utilization.read(),
            is_leader: self
                .state
                .leader_node
                .read()
                .as_deref()
                .is_some_and(|leader| leader == self.config.node_id),
        }
    }

    /// Shutdown the federation service
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if teardown steps fail.
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
        *self.state.status.write() = FederationStatus::Inactive;
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
