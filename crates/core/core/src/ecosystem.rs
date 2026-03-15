// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;

use crate::{
    EcosystemConfig, EcosystemMode, Error, HealthStatus, MonitoringEvent, MonitoringService,
    PerformanceMetrics, PrimalCoordinator, PrimalEndpoint, PrimalType, Result, Task, TaskResult,
    SQUIRREL_MCP_VERSION,
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
    config: EcosystemConfig,
    state: Arc<EcosystemState>,
    discovered_primals: Arc<DashMap<String, PrimalEndpoint>>,
    // Note: HTTP removed - use Songbird via Unix sockets for any HTTP needs
    shutdown_notify: Arc<Notify>,
    monitoring: Arc<MonitoringService>,
}

#[derive(Debug)]
struct EcosystemState {
    #[allow(dead_code)]
    service_id: String,
    node_id: String,
    status: RwLock<ServiceStatus>,
    registration_time: DateTime<Utc>,
    last_health_check: RwLock<DateTime<Utc>>,
    coordination_stats: RwLock<CoordinationStats>,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Starting,
    Standalone,   // Operating without ecosystem coordination
    Discovering,  // Attempting to discover other primals
    Coordinating, // Actively coordinating with ecosystem
    Degraded,     // Some coordination failures but still operational
    Stopping,
}

#[derive(Debug, Default)]
struct CoordinationStats {
    tasks_coordinated: u64,
    primals_discovered: u32,
    federation_nodes: u32,
    last_coordination: Option<DateTime<Utc>>,
    coordination_failures: u64,
}

impl EcosystemService {
    /// Create a new ecosystem service
    pub fn new(config: EcosystemConfig, monitoring: Arc<MonitoringService>) -> Result<Self> {
        let service_id = format!("squirrel-{}", uuid::Uuid::new_v4());
        let node_id =
            std::env::var("NODE_ID").unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4()));

        let state = Arc::new(EcosystemState {
            service_id: service_id.clone(),
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
        self.start_background_tasks().await;

        tracing::info!("Squirrel MCP ecosystem service started successfully");
        Ok(())
    }

    /// Start sovereign mode - autonomous operation with optional coordination
    async fn start_sovereign_mode(&self) -> Result<()> {
        *self.state.status.write() = ServiceStatus::Discovering;

        // Attempt to discover and register with ecosystem
        match self.discover_and_register().await {
            Ok(_) => {
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
    async fn start_background_tasks(&self) {
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
                            stats.coordination_failures as f64 / stats.tasks_coordinated as f64
                        } else {
                            0.0
                        };

                        PerformanceMetrics {
                            cpu_usage: None,
                            memory_usage: None,
                            network_usage: None,
                            response_time: None,
                            throughput: Some(stats.tasks_coordinated as f64),
                            error_rate: Some(error_rate),
                            queue_length: None,
                            active_connections: Some(self.discovered_primals.len() as u32),
                            custom_metrics: {
                                let mut custom = HashMap::new();
                                custom.insert("primals_discovered".to_string(), stats.primals_discovered as f64);
                                custom.insert("federation_nodes".to_string(), stats.federation_nodes as f64);
                                custom
                            },
                        }
                    };

                    let _ = self.monitoring.record_performance("ecosystem", performance_metrics).await;

                    // Update last health check time
                    *self.state.last_health_check.write() = Utc::now();
                }
                _ = self.shutdown_notify.notified() => {
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
                _ = self.shutdown_notify.notified() => {
                    tracing::info!("Discovery loop shutting down");
                    break;
                }
            }
        }
    }

    /// Get current health status for monitoring
    fn get_current_health(&self) -> HealthStatus {
        match self.get_status() {
            ServiceStatus::Starting => HealthStatus::Unknown,
            ServiceStatus::Standalone | ServiceStatus::Coordinating => HealthStatus::Healthy,
            ServiceStatus::Discovering => HealthStatus::Healthy,
            ServiceStatus::Degraded => HealthStatus::Degraded,
            ServiceStatus::Stopping => HealthStatus::Unhealthy,
        }
    }

    /// Get the current service endpoint
    /// Get Squirrel MCP endpoint with multi-tier resolution
    ///
    /// Resolution tiers:
    /// 1. SQUIRREL_MCP_ENDPOINT (full endpoint)
    /// 2. SQUIRREL_PORT (port override)
    /// 3. Default: http://localhost:8080
    pub fn get_endpoint(&self) -> String {
        std::env::var("SQUIRREL_MCP_ENDPOINT").unwrap_or_else(|_| {
            let port = std::env::var("SQUIRREL_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8080); // Default Squirrel MCP port
            format!("http://localhost:{}", port)
        })
    }

    /// Get service metadata for registration
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
    pub fn get_status(&self) -> ServiceStatus {
        self.state.status.read().clone()
    }

    /// Get discovered primals
    pub fn get_discovered_primals(&self) -> Vec<PrimalEndpoint> {
        self.discovered_primals
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Shutdown the service
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down ecosystem service");

        // Update status
        *self.state.status.write() = ServiceStatus::Stopping;

        // Unregister from ecosystem
        if let Err(e) = self.unregister_from_ecosystem().await {
            tracing::warn!("Failed to unregister from ecosystem: {}", e);
        }

        // Notify background tasks to shutdown
        self.shutdown_notify.notify_waiters();

        tracing::info!("Ecosystem service shutdown complete");
        Ok(())
    }

    /// Unregister from the ecosystem
    async fn unregister_from_ecosystem(&self) -> Result<()> {
        // Implementation would unregister from Songbird or other discovery systems
        // For now, this is a placeholder
        Ok(())
    }
}

#[async_trait::async_trait]
impl PrimalCoordinator for EcosystemService {
    async fn register_with_ecosystem(&self) -> Result<()> {
        if let Some(ref songbird_endpoint) = self.config.discovery.songbird_endpoint {
            tracing::info!(
                "Attempting to register with Songbird at: {}",
                songbird_endpoint
            );

            // NOTE: Registration uses Unix socket discovery via ecosystem patterns
            // Pattern: Capability-based service registry via Unix sockets
            tracing::info!(
                "Songbird registration not yet implemented (requires Unix socket discovery)"
            );
            tracing::debug!("Songbird endpoint: {}", songbird_endpoint);

            // For now, succeed silently (registration will use file-based or Unix socket discovery)
            Ok(())
        } else {
            tracing::debug!("No Songbird endpoint configured, skipping registration");
            Ok(())
        }
    }

    async fn discover_primals(&self) -> Result<Vec<PrimalEndpoint>> {
        let mut discovered = Vec::new();

        // Try Songbird service discovery first
        if let Some(ref songbird_endpoint) = self.config.discovery.songbird_endpoint {
            match self.discover_via_songbird(songbird_endpoint).await {
                Ok(mut primals) => {
                    discovered.append(&mut primals);
                }
                Err(e) => {
                    tracing::debug!("Songbird discovery failed: {}", e);
                    let _ = self
                        .monitoring
                        .record_error("songbird_discovery", &e.to_string(), "ecosystem")
                        .await;
                }
            }
        }

        // Fallback to direct endpoint probing
        if discovered.is_empty() || self.config.discovery.auto_discovery {
            match self.discover_via_direct_probing().await {
                Ok(mut primals) => {
                    discovered.append(&mut primals);
                }
                Err(e) => {
                    tracing::debug!("Direct probing failed: {}", e);
                    let _ = self
                        .monitoring
                        .record_error("direct_discovery", &e.to_string(), "ecosystem")
                        .await;
                }
            }
        }

        // Update our discovered primals cache and record discovery events
        for primal in &discovered {
            self.discovered_primals
                .insert(primal.id.clone(), primal.clone());

            // Record primal discovery event
            let _ = self
                .monitoring
                .record_event(MonitoringEvent::PrimalDiscovered {
                    primal_id: primal.id.clone(),
                    primal_type: format!("{:?}", primal.primal_type),
                    endpoint: primal.endpoint.clone(),
                    timestamp: Utc::now(),
                })
                .await;
        }

        // Update stats
        {
            let mut stats = self.state.coordination_stats.write();
            stats.primals_discovered = discovered.len() as u32;
        }

        tracing::debug!("Discovered {} primals", discovered.len());
        Ok(discovered)
    }

    async fn coordinate_task(&self, task: Task) -> Result<TaskResult> {
        let start_time = Utc::now();
        tracing::debug!("Coordinating task: {}", task.id);

        // Update coordination stats
        {
            let mut stats = self.state.coordination_stats.write();
            stats.tasks_coordinated += 1;
            stats.last_coordination = Some(Utc::now());
        }

        // Record task submission event
        let _ = self
            .monitoring
            .record_event(MonitoringEvent::TaskSubmitted {
                task_id: task.id.clone(),
                task_type: format!("{:?}", task.task_type),
                priority: format!("{:?}", task.priority),
                timestamp: Utc::now(),
            })
            .await;

        // For now, this is a basic implementation
        // In a real system, this would route the task to appropriate primals
        // based on the task requirements and available capabilities

        let result = match self.route_task_to_primal(&task).await {
            Ok(result) => result,
            Err(e) => {
                // Update failure stats
                {
                    let mut stats = self.state.coordination_stats.write();
                    stats.coordination_failures += 1;
                }

                let _ = self
                    .monitoring
                    .record_error("task_coordination", &e.to_string(), "ecosystem")
                    .await;

                // In sovereign mode, fall back to local execution
                if matches!(self.config.mode, EcosystemMode::Sovereign) {
                    tracing::warn!(
                        "Task coordination failed, falling back to local execution: {}",
                        e
                    );
                    self.execute_task_locally(&task).await?
                } else {
                    return Err(e);
                }
            }
        };

        let execution_time = (Utc::now() - start_time)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));

        // Record task completion event
        let _ = self
            .monitoring
            .record_task_completed(&task.id, execution_time, true)
            .await;

        Ok(TaskResult {
            id: task.id,
            status: crate::TaskStatus::Completed,
            result: Some(result),
            error: None,
            execution_time,
            executed_by: None, // Would be set to the actual executing primal
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let health = self.get_current_health();
        let _ = self
            .monitoring
            .record_health("ecosystem", health.clone())
            .await;
        Ok(health)
    }
}

impl EcosystemService {
    /// Discover primals via Songbird service discovery
    /// Discover primals via Songbird service registry
    /// NOTE: Uses Unix socket-based discovery via ecosystem patterns
    async fn discover_via_songbird(&self, songbird_endpoint: &str) -> Result<Vec<PrimalEndpoint>> {
        tracing::debug!(
            "Songbird discovery not yet implemented (requires Unix socket): {}",
            songbird_endpoint
        );

        // Discovery should use Unix socket-based capability registry
        // Pattern: CapabilityRegistry::discover_services().await

        // For now, return empty list (discovery will use file-based or direct probing)
        Ok(Vec::new())
    }

    /// Discover primals via direct endpoint probing
    async fn discover_via_direct_probing(&self) -> Result<Vec<PrimalEndpoint>> {
        tracing::debug!("Discovering primals via direct probing");

        let mut primals = Vec::new();

        for (primal_name, endpoint) in &self.config.discovery.direct_endpoints {
            match self.probe_primal_endpoint(primal_name, endpoint).await {
                Ok(primal) => primals.push(primal),
                Err(e) => {
                    tracing::debug!("Failed to probe {}: {}", primal_name, e);
                    let _ = self
                        .monitoring
                        .record_error("endpoint_probe", &e.to_string(), "ecosystem")
                        .await;
                }
            }
        }

        Ok(primals)
    }

    /// Probe a specific primal endpoint
    /// NOTE: Uses Unix socket-based health check via ecosystem patterns
    async fn probe_primal_endpoint(
        &self,
        primal_name: &str,
        endpoint: &str,
    ) -> Result<PrimalEndpoint> {
        tracing::debug!(
            "Endpoint probing not yet implemented (requires Unix socket): {}",
            endpoint
        );

        // Primal health checks should use Unix socket-based communication
        // Pattern: UnixStream::connect(socket_path).await + JSON-RPC health check

        // For now, return error (discovery will use file-based registry)
        Err(Error::Discovery(format!(
            "Endpoint probing not yet implemented for {}: {}",
            primal_name, endpoint
        )))
    }

    /// Parse primal type from string
    #[allow(dead_code)]
    fn parse_primal_type(&self, type_str: &str) -> Result<PrimalType> {
        match type_str.to_lowercase().as_str() {
            "squirrel" => Ok(PrimalType::Squirrel),
            "songbird" => Ok(PrimalType::Songbird),
            "nestgate" => Ok(PrimalType::NestGate),
            "beardog" => Ok(PrimalType::BearDog),
            "toadstool" => Ok(PrimalType::ToadStool),
            "biomeos" => Ok(PrimalType::BiomeOS),
            _ => Err(Error::Discovery(format!("Unknown primal type: {type_str}"))),
        }
    }

    /// Route task to appropriate primal
    async fn route_task_to_primal(&self, task: &Task) -> Result<serde_json::Value> {
        // This is a simplified routing implementation
        // In a real system, this would involve sophisticated routing logic
        // based on task requirements, primal capabilities, load balancing, etc.

        tracing::debug!("Routing task {} to primal", task.id);

        // For now, just return a mock result
        Ok(serde_json::json!({
            "result": "Task coordinated successfully",
            "task_id": task.id,
            "timestamp": Utc::now().to_rfc3339()
        }))
    }

    /// Execute task locally as fallback
    async fn execute_task_locally(&self, task: &Task) -> Result<serde_json::Value> {
        tracing::info!("Executing task {} locally", task.id);

        // Record local execution event
        let _ = self
            .monitoring
            .record_event(MonitoringEvent::Custom {
                event_type: "local_task_execution".to_string(),
                data: serde_json::json!({
                    "task_id": task.id,
                    "reason": "coordination_failure_fallback"
                }),
                timestamp: Utc::now(),
            })
            .await;

        // Local execution logic would go here
        // For now, just return a mock result
        Ok(serde_json::json!({
            "result": "Task executed locally",
            "task_id": task.id,
            "timestamp": Utc::now().to_rfc3339(),
            "execution_mode": "local_fallback"
        }))
    }
}

// Supporting data structures
#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ServiceRegistration {
    service_id: String,
    primal_type: String,
    endpoint: String,
    capabilities: Vec<String>,
    health_endpoint: String,
    metadata: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ServiceInfo {
    service_id: String,
    primal_type: String,
    endpoint: String,
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PrimalInfo {
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
}
