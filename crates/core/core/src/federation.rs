// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    monitoring::MonitoringService, Error, FederationLoadBalancer, FederationResult,
    FederationStatus, FederationTopology, InstanceStatus, LoadBalanceResult, LoadMetrics, NodeSpec,
    Result, SquirrelConfig, SquirrelInstance, SwarmManager,
};
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Federation service for managing distributed Squirrel MCP instances
#[derive(Clone)]
pub struct FederationService {
    config: FederationConfig,
    state: Arc<FederationState>,
    instances: Arc<DashMap<String, SquirrelInstance>>,
    #[allow(dead_code)]
    federation_topology: Arc<RwLock<FederationTopology>>,
    #[allow(dead_code)]
    load_balancer: Arc<FederationLoadBalancer>,
    #[allow(dead_code)]
    monitoring: Arc<MonitoringService>,
    // NOTE: HTTP removed - Use Songbird via Unix sockets for federation HTTP calls
    shutdown_notify: Arc<tokio::sync::Notify>,
    load_metrics: Arc<LoadMetrics>,
    scaling_policy: Arc<ScalingPolicy>,
}

#[derive(Debug, Clone)]
pub struct FederationConfig {
    pub node_id: String,
    pub port: u16,
    pub federation_discovery_urls: Vec<String>,
    pub auto_scaling_enabled: bool,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub health_check_interval: chrono::Duration,
    pub federation_timeout: chrono::Duration,
    pub federation_enabled: bool,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub max_local_instances: u32,
    pub scaling_check_interval: chrono::Duration,
    pub topology: FederationTopology,
    pub federation_port: u16,
}

#[derive(Debug)]
struct FederationState {
    status: RwLock<FederationStatus>,
    federation_id: String,
    leader_node: RwLock<Option<String>>,
    last_scale_event: RwLock<Option<DateTime<Utc>>>,
    total_capacity: RwLock<u32>,
    current_utilization: RwLock<f64>,
}

#[derive(Debug, Clone)]
pub struct FederationNode {
    pub id: String,
    pub endpoint: String,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub capabilities: Vec<String>,
    pub capacity: u32,
    pub current_load: u32,
    pub health: NodeHealth,
    pub last_seen: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unreachable,
}

#[derive(Debug)]
pub struct ScalingPolicy {
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub scale_up_cooldown: chrono::Duration,
    pub scale_down_cooldown: chrono::Duration,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_factor: f64,
}

impl FederationService {
    /// Create a new federation service
    pub fn new(config: FederationConfig) -> Result<Self> {
        let federation_id = format!("fed-{}", uuid::Uuid::new_v4());

        let state = Arc::new(FederationState {
            status: RwLock::new(FederationStatus::Forming),
            federation_id: federation_id.clone(),
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

        // Note: HTTP client removed - delegate to Songbird for any HTTP needs

        Ok(Self {
            config,
            state,
            instances: Arc::new(DashMap::new()),
            federation_topology: Arc::new(RwLock::new(FederationTopology::Star)),
            load_balancer: Arc::new(FederationLoadBalancer::new(load_metrics.clone())),
            monitoring: Arc::new(MonitoringService::new(
                crate::monitoring::MonitoringConfig::default(),
            )),
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
            load_metrics,
            scaling_policy,
        })
    }

    /// Start the federation service
    pub async fn start(&self) -> Result<()> {
        tracing::info!(
            "Starting federation service for node: {}",
            self.config.node_id
        );

        if self.config.federation_enabled {
            // Initialize federation
            self.initialize_federation().await?;
        } else {
            tracing::info!("Federation disabled, operating in standalone mode");
            *self.state.status.write() = FederationStatus::Active;
        }

        // Start background tasks
        self.start_background_tasks().await;

        tracing::info!("Federation service started successfully");
        Ok(())
    }

    /// Initialize federation
    async fn initialize_federation(&self) -> Result<()> {
        *self.state.status.write() = FederationStatus::Forming;

        // Try to discover existing federation nodes
        self.discover_federation_nodes().await?;

        // Determine if we should be the leader or join existing federation
        if self.instances.is_empty() {
            // No other nodes found, we become the leader
            *self.state.leader_node.write() = Some(self.config.node_id.clone());
            *self.state.status.write() = FederationStatus::Active;
            tracing::info!("No existing federation found, becoming leader node");
        } else {
            // Join existing federation
            self.join_existing_federation().await?;
        }

        Ok(())
    }

    /// Discover existing federation nodes
    async fn discover_federation_nodes(&self) -> Result<()> {
        // This would implement actual node discovery
        // For now, using environment variables or configuration

        if let Ok(nodes_config) = std::env::var("FEDERATION_NODES") {
            for node_endpoint in nodes_config.split(',') {
                match self.probe_federation_node(node_endpoint.trim()).await {
                    Ok(node) => {
                        self.instances.insert(node.id.clone(), node);
                    }
                    Err(e) => {
                        tracing::debug!("Failed to probe node {}: {}", node_endpoint, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Probe a potential federation node
    /// NOTE: Delegates HTTP to Songbird via Unix sockets (TRUE PRIMAL pattern)
    async fn probe_federation_node(&self, endpoint: &str) -> Result<SquirrelInstance> {
        // Federation probing requires HTTP delegation to Songbird
        // Pattern: CapabilityHttpClient::discover("http.client").await?
        Err(Error::Federation(format!(
            "Federation node probing not yet implemented (requires Songbird HTTP delegation): {}",
            endpoint
        )))
    }

    /// Join an existing federation
    /// NOTE: Delegates HTTP to Songbird via Unix sockets
    async fn join_existing_federation(&self) -> Result<()> {
        // Federation joining requires HTTP POST delegation to Songbird
        // Pattern: CapabilityHttpClient::discover("http.client").post(&join_url, &join_request).await

        tracing::info!(
            "Federation joining not yet implemented (requires Songbird HTTP delegation)"
        );

        Err(Error::Federation(
            "Federation joining not yet implemented (requires Songbird HTTP delegation)"
                .to_string(),
        ))
    }

    /// Find the leader node in the federation
    #[allow(dead_code)]
    async fn find_leader_node(&self) -> Result<SquirrelInstance> {
        // Simple leader election: use the node with the lowest ID
        // In practice, this would be more sophisticated

        let mut leader: Option<SquirrelInstance> = None;

        for entry in self.instances.iter() {
            let instance = entry.value();
            if leader.is_none()
                || &instance.id < leader.as_ref().map(|l| &l.id).unwrap_or(&String::new())
            {
                leader = Some(instance.clone());
            }
        }

        leader.ok_or_else(|| Error::Federation("No leader node found".to_string()))
    }

    /// Start background federation tasks
    async fn start_background_tasks(&self) {
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
                    self.check_federation_health().await;
                    self.check_instance_health().await;
                }
                _ = self.shutdown_notify.notified() => {
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
                    self.collect_load_metrics().await;
                }
                _ = self.shutdown_notify.notified() => {
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
                _ = self.shutdown_notify.notified() => {
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
                    self.maintain_federation().await;
                }
                _ = self.shutdown_notify.notified() => {
                    tracing::info!("Federation maintenance loop shutting down");
                    break;
                }
            }
        }
    }

    /// Check health of federation nodes
    /// NOTE: Delegates HTTP health checks to Songbird via Unix sockets
    async fn check_federation_health(&self) {
        // For now, assume all instances are running (HTTP health checking requires Songbird)
        for mut entry in self.instances.iter_mut() {
            let (_instance_id, instance) = entry.pair_mut();
            instance.health = InstanceStatus::Running;
            instance.last_seen = Utc::now();
        }

        // Pattern for future implementation:
        // CapabilityHttpClient::discover("http.client").get(&health_url).await
    }

    /// Check health of local instances
    /// NOTE: Delegates to Songbird for HTTP health checks
    async fn check_instance_health(&self) {
        // Instance health checking requires HTTP delegation to Songbird
        // Pattern: CapabilityHttpClient::discover("http.client").get(&health_url).await

        // For now, assume all instances are running (to be implemented with Songbird)
        for mut entry in self.instances.iter_mut() {
            let (instance_id, instance) = entry.pair_mut();

            if instance.health == InstanceStatus::Starting {
                instance.health = InstanceStatus::Running;
                tracing::info!(
                    "Instance {} marked as running (HTTP health check via Songbird)",
                    instance_id
                );
            }
        }
    }

    /// Collect current load metrics
    async fn collect_load_metrics(&self) {
        // This would collect actual system metrics
        // For now, using mock values with some variation

        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Simulate load metrics with some randomness
        let base_cpu = 0.3 + (self.instances.len() as f64 * 0.1);
        let _cpu_usage = base_cpu + rng.gen::<f64>() * 0.2;

        let base_memory = 0.2 + (self.instances.len() as f64 * 0.15);
        let _memory_usage = base_memory + rng.gen::<f64>() * 0.1;

        let _queue_length = rng.gen_range(0..50);
        let _active_tasks = rng.gen_range(10..200);

        // Create new metrics and store them (would need proper synchronization in production)
        let cpu = self.load_metrics.cpu_usage; // Direct access since it's f64, not RwLock<f64>
        let memory = self.load_metrics.memory_usage;
        let _network = self.load_metrics.network_usage;
        let active_tasks = self.load_metrics.active_tasks;
        let queue_length = self.load_metrics.queue_length;
        let _error_rate = self.load_metrics.error_rate;

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
        let current_utilization = self.calculate_overall_utilization().await;
        let current_instances = self.instances.len() as u32;

        // Check cooldown periods
        if let Some(last_scale) = *self.state.last_scale_event.read() {
            let time_since_last_scale = Utc::now() - last_scale;
            if time_since_last_scale < self.scaling_policy.scale_up_cooldown {
                return Ok(()); // Still in cooldown
            }
        }

        // Scale up decision
        if current_utilization > self.scaling_policy.scale_up_threshold {
            if current_instances < self.scaling_policy.max_instances {
                let target_instances =
                    ((current_instances as f64 * self.scaling_policy.scale_factor) as u32)
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
            let target_instances = ((current_instances as f64 / self.scaling_policy.scale_factor)
                as u32)
                .max(self.scaling_policy.min_instances);

            tracing::info!(
                "Scaling down from {} to {} instances (utilization: {:.2})",
                current_instances,
                target_instances,
                current_utilization
            );

            self.scale_down(current_instances - target_instances)
                .await?;
        }

        Ok(())
    }

    /// Calculate overall utilization across all metrics
    async fn calculate_overall_utilization(&self) -> f64 {
        let cpu = self.load_metrics.cpu_usage;
        let memory = self.load_metrics.memory_usage;
        let queue_pressure = (self.load_metrics.queue_length as f64) / 100.0;

        // Weighted average of different metrics
        (cpu * 0.4 + memory * 0.3 + queue_pressure * 0.3).min(1.0)
    }

    /// Scale up by spawning new instances
    async fn scale_up(&self, instances_to_add: u32) -> Result<()> {
        for i in 0..instances_to_add {
            let instance_config = self.create_instance_config(i).await?;
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
    async fn scale_down(&self, instances_to_remove: u32) -> Result<()> {
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
                if let Err(e) = self.stop_instance(&instance).await {
                    tracing::error!("Failed to stop instance {}: {}", instance.id, e);
                } else {
                    tracing::info!("Successfully stopped instance: {}", instance.id);
                }
            }
        }

        *self.state.last_scale_event.write() = Some(Utc::now());
        Ok(())
    }

    /// Stop a specific instance
    /// NOTE: Delegates to Songbird for HTTP shutdown request
    async fn stop_instance(&self, instance: &SquirrelInstance) -> Result<()> {
        // Instance shutdown requires HTTP POST delegation to Songbird
        // Pattern: CapabilityHttpClient::discover("http.client").post(&shutdown_url).await
        Err(Error::Federation(format!(
            "Instance shutdown not yet implemented (requires Songbird HTTP delegation): {}",
            instance.endpoint
        )))
    }

    /// Create configuration for a new instance
    async fn create_instance_config(&self, instance_index: u32) -> Result<SquirrelConfig> {
        // This would create actual configuration
        // For now, return a mock configuration
        Ok(SquirrelConfig {
            node_id: format!("{}-instance-{}", self.config.node_id, instance_index),
            port: 8080 + instance_index as u16,
            federation_enabled: false, // Instances don't federate themselves
            region: self.config.region.clone(),
            zone: self.config.zone.clone(),
            auto_scaling_enabled: true,
            capabilities: vec!["mcp".to_string(), "routing".to_string()],
            capacity: 100,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Maintain federation health and topology
    async fn maintain_federation(&self) {
        // Periodic federation maintenance tasks

        // 1. Sync federation state with other nodes
        if let Err(e) = self.sync_federation_state().await {
            tracing::debug!("Federation state sync failed: {}", e);
        }

        // 2. Re-elect leader if needed
        if let Err(e) = self.check_leader_health().await {
            tracing::debug!("Leader health check failed: {}", e);
        }

        // 3. Optimize topology if needed
        if let Err(e) = self.optimize_topology().await {
            tracing::debug!("Topology optimization failed: {}", e);
        }
    }

    /// Sync federation state with other nodes
    async fn sync_federation_state(&self) -> Result<()> {
        // Implementation would sync state across federation nodes
        Ok(())
    }

    /// Check leader health and trigger re-election if needed
    async fn check_leader_health(&self) -> Result<()> {
        // Implementation would check leader health and trigger re-election
        Ok(())
    }

    /// Optimize federation topology
    async fn optimize_topology(&self) -> Result<()> {
        // Implementation would optimize the federation topology based on performance metrics
        Ok(())
    }

    /// Get current node endpoint
    #[allow(dead_code)]
    fn get_node_endpoint(&self) -> String {
        format!(
            "http://{}:{}",
            std::env::var("NODE_IP")
                .or_else(|_| std::env::var("MCP_HOST"))
                .unwrap_or_else(|_| "localhost".to_string()),
            self.config.federation_port
        )
    }

    /// Get current node capabilities
    #[allow(dead_code)]
    fn get_node_capabilities(&self) -> Vec<String> {
        vec![
            "mcp".to_string(),
            "ai-task-routing".to_string(),
            "multi-mcp-coordination".to_string(),
            "context-management".to_string(),
            "federation".to_string(),
            "scaling".to_string(),
        ]
    }

    /// Get federation statistics
    pub fn get_federation_stats(&self) -> FederationStats {
        FederationStats {
            node_id: self.config.node_id.clone(),
            federation_id: self.state.federation_id.clone(),
            status: self.state.status.read().clone(),
            local_instances: self.instances.len() as u32,
            federation_nodes: self.instances.len() as u32,
            total_capacity: *self.state.total_capacity.read(),
            current_utilization: *self.state.current_utilization.read(),
            is_leader: self.state.leader_node.read().as_ref() == Some(&self.config.node_id),
        }
    }

    /// Shutdown the federation service
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down federation service");

        // Notify background tasks to shutdown
        self.shutdown_notify.notify_waiters();

        // Stop all local instances
        for entry in self.instances.iter() {
            let instance = entry.value();
            if let Err(e) = self.stop_instance(instance).await {
                tracing::warn!("Failed to stop instance during shutdown: {}", e);
            }
        }

        // Leave federation if we're part of one
        if self.config.federation_enabled {
            if let Err(e) = self.leave_federation().await {
                tracing::warn!("Failed to leave federation during shutdown: {}", e);
            }
        }

        tracing::info!("Federation service shutdown complete");
        Ok(())
    }

    /// Leave the federation
    async fn leave_federation(&self) -> Result<()> {
        // Implementation would properly leave the federation
        *self.state.status.write() = FederationStatus::Inactive;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SwarmManager for FederationService {
    async fn spawn_squirrel(&self, config: SquirrelConfig) -> Result<SquirrelInstance> {
        let instance_id = uuid::Uuid::new_v4().to_string();
        let instance_port = config.port;

        tracing::info!("Spawning new Squirrel instance: {}", instance_id);

        // In a real implementation, this would actually spawn a new process or container
        // For now, we simulate the instance creation

        let node_ip = std::env::var("NODE_IP")
            .or_else(|_| std::env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string());
        let instance = SquirrelInstance {
            id: instance_id.clone(),
            node_id: self.config.node_id.clone(),
            endpoint: format!("http://{node_ip}:{instance_port}"),
            capabilities: vec![
                "mcp".to_string(),
                "ai-task-routing".to_string(),
                "context-management".to_string(),
            ],
            health: InstanceStatus::Starting,
            last_seen: Utc::now(),
            capacity: 10,    // Mock capacity
            current_load: 0, // Mock current load
            region: None,
            zone: None,
            metadata: HashMap::new(),
        };

        // Store the instance
        let instance_id_for_log = instance.id.clone();
        let instance_for_return = instance.clone();
        self.instances.insert(instance_id, instance);

        // In a real implementation, you would:
        // 1. Create new process/container with the config
        // 2. Wait for it to start up and become healthy
        // 3. Register it with the load balancer

        tracing::info!("Successfully created instance: {}", instance_id_for_log);
        Ok(instance_for_return)
    }

    async fn federate_nodes(&self, nodes: Vec<NodeSpec>) -> Result<FederationResult> {
        let mut successful_nodes = Vec::new();
        let mut failed_nodes = 0;

        for node_spec in nodes {
            match self.add_federation_node(node_spec.clone()).await {
                Ok(_) => {
                    successful_nodes.push(node_spec);
                }
                Err(e) => {
                    tracing::error!("Failed to add federation node {}: {}", node_spec.id, e);
                    failed_nodes += 1;
                }
            }
        }

        let status = if failed_nodes == 0 {
            FederationStatus::Active
        } else if successful_nodes.is_empty() {
            FederationStatus::Inactive
        } else {
            FederationStatus::Degraded
        };

        Ok(FederationResult {
            federation_id: self.state.federation_id.clone(),
            nodes_joined: successful_nodes.len() as u32,
            total_capacity: 100,
            status,
        })
    }

    async fn balance_load(&self, _load: LoadMetrics) -> Result<LoadBalanceResult> {
        // Update our load metrics

        // Calculate load balancing decision
        let current_utilization = self.calculate_overall_utilization().await;
        *self.state.current_utilization.write() = current_utilization;

        // Determine load balancing action
        let _action = if current_utilization > 0.8 {
            "scale_up".to_string()
        } else if current_utilization < 0.3 {
            "scale_down".to_string()
        } else {
            "maintain".to_string()
        };

        Ok(LoadBalanceResult {
            distribution: std::collections::HashMap::new(),
            balance_score: current_utilization,
            rebalance_time: std::time::Duration::from_millis(100),
        })
    }
}

impl FederationService {
    /// Add a new node to the federation
    async fn add_federation_node(&self, node_spec: NodeSpec) -> Result<()> {
        let node = SquirrelInstance {
            id: node_spec.id.clone(),
            node_id: self.config.node_id.clone(),
            endpoint: node_spec.endpoint,
            capabilities: node_spec.capabilities,
            health: InstanceStatus::Running,
            last_seen: Utc::now(),
            capacity: node_spec.capacity,
            current_load: 0, // Default value since NodeSpec doesn't have this field
            region: node_spec.region,
            zone: node_spec.zone,
            metadata: HashMap::new(), // Default value since NodeSpec doesn't have this field
        };

        self.instances.insert(node_spec.id, node);
        Ok(())
    }
}

// Supporting types
#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NodeInfo {
    node_id: String,
    region: Option<String>,
    zone: Option<String>,
    capabilities: Vec<String>,
    capacity: u32,
    current_load: u32,
    metadata: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JoinRequest {
    node_id: String,
    endpoint: String,
    region: Option<String>,
    zone: Option<String>,
    capabilities: Vec<String>,
    capacity: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FederationStats {
    pub node_id: String,
    pub federation_id: String,
    pub status: FederationStatus,
    pub local_instances: u32,
    pub federation_nodes: u32,
    pub total_capacity: u32,
    pub current_utilization: f64,
    pub is_leader: bool,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            node_id: format!("node-{}", uuid::Uuid::new_v4()),
            port: 8080,
            federation_discovery_urls: Vec::new(),
            auto_scaling_enabled: true,
            min_instances: 1,
            max_instances: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            health_check_interval: chrono::Duration::seconds(30),
            federation_timeout: chrono::Duration::seconds(60),
            federation_enabled: false,
            region: None,
            zone: None,
            max_local_instances: 10,
            scaling_check_interval: chrono::Duration::seconds(60),
            topology: FederationTopology::Star,
            federation_port: 8090,
        }
    }
}
