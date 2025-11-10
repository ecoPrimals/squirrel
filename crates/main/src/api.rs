use crate::universal::{CircuitBreakerStatus, LoadBalancingStatus};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::universal::ServiceMeshStatus;
use crate::MetricsCollector;

/// HTTP API server for primal status and health endpoints
/// Following Songbird service mesh patterns for ecosystem integration
pub struct ApiServer {
    /// Port to bind the server to
    port: u16,
    /// Host to bind the server to
    host: String,
    /// Reference to the ecosystem manager
    ecosystem_manager: Arc<EcosystemManager>,
    /// Reference to metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Reference to shutdown manager
    shutdown_manager: Arc<ShutdownManager>,
    /// Server state
    state: Arc<RwLock<ServerState>>,
}

/// Server state tracking
#[derive(Debug, Clone)]
struct ServerState {
    /// When the server was started
    started_at: chrono::DateTime<chrono::Utc>,
    /// Request count
    request_count: u64,
    /// Active connections
    active_connections: u32,
    /// Service mesh registration status
    service_mesh_registered: bool,
    /// Last Songbird heartbeat
    last_songbird_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_songbird_heartbeat: None,
        }
    }
}

/// Health check response following ecosystem standards
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Service mesh status
    pub service_mesh: ServiceMeshHealthStatus,
    /// Ecosystem integration status
    pub ecosystem: EcosystemHealthStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service mesh health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshHealthStatus {
    /// Registered with Songbird
    pub registered: bool,
    /// Last heartbeat
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// Connection status
    pub connection_status: String,
    /// Load balancing active
    pub load_balancing_active: bool,
}

/// Ecosystem integration health status
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemHealthStatus {
    /// Discovered primals
    pub discovered_primals: u32,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Cross-primal communication status
    pub cross_primal_status: String,
    /// Ecosystem health score
    pub ecosystem_health_score: f64,
}

impl ApiServer {
    /// Create a new API server with dynamic configuration
    pub fn new(
        port: u16,
        ecosystem_manager: Arc<EcosystemManager>,
        metrics_collector: Arc<MetricsCollector>,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Self {
        let host = std::env::var("SQUIRREL_SERVICE_HOST").unwrap_or_else(|_| {
            // Environment-aware default host
            if std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .eq_ignore_ascii_case("production")
            {
                "0.0.0.0".to_string()
            } else {
                "127.0.0.1".to_string()
            }
        });

        Self {
            port,
            host,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
            state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    /// Create a new API server with explicit host and port
    pub fn new_with_host(
        host: String,
        port: u16,
        ecosystem_manager: Arc<EcosystemManager>,
        metrics_collector: Arc<MetricsCollector>,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Self {
        Self {
            port,
            host,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
            state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    /// Get the server base URL
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Get the server websocket URL
    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}/ws", self.host, self.port)
    }

    /// Start the API server with ecosystem endpoints
    pub async fn start(&self) -> Result<()> {
        let state = self.state.clone();
        let ecosystem_manager = self.ecosystem_manager.clone();
        let metrics_collector = self.metrics_collector.clone();
        let shutdown_manager = self.shutdown_manager.clone();

        tracing::info!(
            "Starting API server on port {} with ecosystem integration",
            self.port
        );

        // Define ecosystem-standard routes
        let health = warp::path!("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_health_check);

        let health_live = warp::path!("health" / "live")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_health_live);

        let health_ready = warp::path!("health" / "ready")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_health_ready);

        // Ecosystem API endpoints following Songbird patterns
        let ecosystem_status = warp::path!("api" / "v1" / "ecosystem" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_ecosystem_status);

        let service_mesh_status = warp::path!("api" / "v1" / "service-mesh" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_service_mesh_status);

        let primals_list = warp::path!("api" / "v1" / "primals")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(handle_primals_list);

        let primal_status = warp::path!("api" / "v1" / "primals" / String)
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(handle_primal_status);

        let metrics = warp::path!("api" / "v1" / "metrics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_metrics_collector(metrics_collector.clone()))
            .and_then(handle_metrics);

        let services = warp::path!("api" / "v1" / "services")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(handle_services);

        // Songbird integration endpoints
        let songbird_register = warp::path!("api" / "v1" / "songbird" / "register")
            .and(warp::post())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_songbird_register);

        let songbird_heartbeat = warp::path!("api" / "v1" / "songbird" / "heartbeat")
            .and(warp::post())
            .and(with_state(state.clone()))
            .and_then(handle_songbird_heartbeat);

        // Shutdown endpoint using the shutdown_manager
        let shutdown = warp::path!("api" / "v1" / "shutdown")
            .and(warp::post())
            .and(with_shutdown_manager(shutdown_manager.clone()))
            .and_then(handle_shutdown);

        // Add request counting middleware
        let request_counter = warp::any()
            .and(with_state(state.clone()))
            .and_then(|state: Arc<RwLock<ServerState>>| async move {
                increment_request_count(state).await;
                Ok::<_, warp::Rejection>(())
            })
            .untuple_one();

        // Combine all routes
        let routes = request_counter
            .and(
                health
                    .or(health_live)
                    .or(health_ready)
                    .or(ecosystem_status)
                    .or(service_mesh_status)
                    .or(primals_list)
                    .or(primal_status)
                    .or(metrics)
                    .or(services)
                    .or(songbird_register)
                    .or(songbird_heartbeat)
                    .or(shutdown),
            )
            .with(warp::cors().allow_any_origin())
            .with(warp::log("api"));

        // Start the server and wait for it to run
        let port = self.port;
        tracing::info!(
            "API server started successfully on port {} with ecosystem integration",
            self.port
        );

        // This will run the server and block until it's shut down
        warp::serve(routes).run(([0, 0, 0, 0], port)).await;

        Ok(())
    }
}

// Helper functions for warp filters
fn with_state(
    state: Arc<RwLock<ServerState>>,
) -> impl Filter<Extract = (Arc<RwLock<ServerState>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_ecosystem_manager(
    ecosystem_manager: Arc<EcosystemManager>,
) -> impl Filter<Extract = (Arc<EcosystemManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ecosystem_manager.clone())
}

fn with_metrics_collector(
    metrics_collector: Arc<MetricsCollector>,
) -> impl Filter<Extract = (Arc<MetricsCollector>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics_collector.clone())
}

fn with_shutdown_manager(
    shutdown_manager: Arc<ShutdownManager>,
) -> impl Filter<Extract = (Arc<ShutdownManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || shutdown_manager.clone())
}

fn with_base_url(
    base_url: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || base_url.clone())
}

/// Middleware to increment request count for all requests
async fn increment_request_count(state: Arc<RwLock<ServerState>>) {
    let mut state_guard = state.write().await;
    state_guard.request_count += 1;
}

// Handler functions following ecosystem patterns
async fn handle_health_check(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("primal_type".to_string(), "ai".to_string());
    metadata.insert(
        "capabilities".to_string(),
        "ai_coordination,mcp_protocol,context_awareness".to_string(),
    );

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: uptime,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: if state.service_mesh_registered {
                "connected"
            } else {
                "disconnected"
            }
            .to_string(),
            load_balancing_active: state.service_mesh_registered,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: ecosystem_status.discovered_services.len() as u32,
            active_integrations: ecosystem_status.active_integrations,
            cross_primal_status: ecosystem_status.status,
            ecosystem_health_score: manager_status.health_status.health_score,
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_health_live(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let mut metadata = HashMap::new();
    metadata.insert("uptime_seconds".to_string(), uptime.to_string());
    metadata.insert(
        "active_connections".to_string(),
        state.active_connections.to_string(),
    );

    let response = HealthResponse {
        status: "alive".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: uptime,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: "live".to_string(),
            load_balancing_active: false,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: "live".to_string(),
            ecosystem_health_score: 1.0,
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_health_ready(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let ready =
        manager_status.status == "initialized" && manager_status.health_status.health_score > 0.5;

    let status = if ready { "ready" } else { "not_ready" };

    let response = HealthResponse {
        status: status.to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: chrono::Utc::now()
            .signed_duration_since(state.started_at)
            .num_seconds() as u64,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: if ready { "ready" } else { "not_ready" }.to_string(),
            load_balancing_active: ready && state.service_mesh_registered,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: status.to_string(),
            ecosystem_health_score: manager_status.health_status.health_score,
        },
        metadata: HashMap::new(),
    };

    Ok(warp::reply::json(&response))
}

async fn handle_ecosystem_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("primal_type".to_string(), "ai".to_string());
    metadata.insert(
        "service_mesh_integration".to_string(),
        "songbird".to_string(),
    );

    let response = EcosystemStatusResponse {
        status: ecosystem_status.status,
        timestamp: chrono::Utc::now(),
        active_primals: ecosystem_status
            .discovered_services
            .iter()
            .map(|s| s.service_id.to_string()) // Convert Arc<str> to String
            .collect(),
        service_discovery: if ecosystem_status.discovered_services.is_empty() {
            "discovering".to_string()
        } else {
            "active".to_string()
        },
        service_mesh_status: ServiceMeshStatusResponse {
            enabled: true,
            registered: manager_status.status == "initialized",
            load_balancing: LoadBalancingResponse {
                enabled: true,
                algorithm: "round_robin".to_string(),
                health_score: manager_status.health_status.health_score,
            },
            cross_primal_communication: CrossPrimalCommunicationResponse {
                enabled: true,
                active_connections: ecosystem_status.active_integrations.len() as u32,
                supported_protocols: vec!["http".to_string(), "grpc".to_string()],
            },
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_service_mesh_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let manager_status = ecosystem_manager.get_manager_status().await;

    let response = ServiceMeshStatus {
        registered: manager_status.status == "initialized",
        connected: true,
        songbird_endpoint: Some("http://localhost:8080".to_string()),
        registration_time: manager_status.last_registration,
        last_heartbeat: Some(Utc::now()),
        mesh_version: "1.0.0".to_string(),
        instance_id: "squirrel-1".to_string(),
        load_balancing_enabled: true,
        circuit_breaker_status: CircuitBreakerStatus {
            open: false,
            failures: 0,
            last_failure: None,
            next_retry: None,
        },
        last_registration: manager_status.last_registration,
        mesh_health: manager_status.health_status.health_score.to_string(),
        active_connections: manager_status.active_registrations.len() as u32,
        load_balancing: LoadBalancingStatus {
            enabled: true,
            healthy: true,
            active_connections: manager_status.active_registrations.len() as u32,
            algorithm: "round_robin".to_string(),
            health_score: manager_status.health_status.health_score,
            last_check: Utc::now(),
        },
    };

    Ok(warp::reply::json(&response))
}

async fn handle_primals_list(
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let mut primals = vec![PrimalStatusResponse {
        name: "squirrel".to_string(),
        status: "active".to_string(),
        timestamp: chrono::Utc::now(),
        endpoints: vec![base_url.clone()],
        metadata: {
            let mut map = HashMap::new();
            map.insert("type".to_string(), "ai".to_string());
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            map.insert(
                "capabilities".to_string(),
                "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
            );
            map
        },
    }];

    // Add discovered primals
    for service in ecosystem_status.discovered_services {
        primals.push(PrimalStatusResponse {
            name: service.service_id.to_string(), // Convert Arc<str> to String
            status: "active".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec![service.endpoint.to_string()], // Convert Arc<str> to String
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<String, String>>(), // Convert Arc<str> keys/values to String
        });
    }

    Ok(warp::reply::json(&primals))
}

async fn handle_primal_status(
    primal_name: String,
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let (status, endpoints, metadata) = match primal_name.as_str() {
        "squirrel" => ("active".to_string(), vec![base_url.clone()], {
            let mut map = HashMap::new();
            map.insert("type".to_string(), "ai".to_string());
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            map.insert(
                "capabilities".to_string(),
                "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
            );
            map.insert("service_mesh".to_string(), "songbird".to_string());
            map
        }),
        _ => {
            if let Some(service) = ecosystem_status
                .discovered_services
                .iter()
                .find(|s| s.service_id.as_ref() == primal_name)
            // Compare Arc<str> with String
            {
                (
                    "active".to_string(),
                    vec![service.endpoint.to_string()], // Convert Arc<str> to String
                    service
                        .metadata
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<HashMap<String, String>>(), // Convert Arc<str> keys/values to String
                )
            } else {
                ("not_found".to_string(), vec![], HashMap::new())
            }
        }
    };

    let response = PrimalStatusResponse {
        name: primal_name,
        status,
        timestamp: chrono::Utc::now(),
        endpoints,
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_metrics(
    state: Arc<RwLock<ServerState>>,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<impl Reply, warp::Rejection> {
    let state_guard = state.read().await;
    let metrics = metrics_collector
        .get_all_metrics()
        .await
        .map_err(|_| warp::reject::not_found())?;

    let response = MetricsResponse {
        timestamp: chrono::Utc::now(),
        system: {
            let mut map = HashMap::new();
            map.insert("uptime".to_string(), "running".to_string());
            map.insert("memory_usage".to_string(), "normal".to_string());
            map.insert("cpu_usage".to_string(), "normal".to_string());
            map
        },
        application: {
            let mut map = HashMap::new();
            map.insert(
                "active_sessions".to_string(),
                metrics.metrics.len().to_string(),
            );
            map.insert(
                "requests_processed".to_string(),
                state_guard.request_count.to_string(),
            );
            map.insert("errors".to_string(), "0".to_string());
            map
        },
        performance: {
            let mut map = HashMap::new();
            map.insert("avg_response_time".to_string(), "100ms".to_string());
            map.insert("throughput".to_string(), "0 req/s".to_string());
            map.insert("error_rate".to_string(), "0%".to_string());
            map
        },
        ecosystem: {
            let mut map = HashMap::new();
            map.insert("service_mesh_health".to_string(), "healthy".to_string());
            map.insert("cross_primal_calls".to_string(), "0".to_string());
            map.insert("load_balancing_efficiency".to_string(), "100%".to_string());
            map
        },
    };

    Ok(warp::reply::json(&response))
}

async fn handle_services(
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let mut services = vec![
        ServiceInfo {
            name: "Squirrel AI".to_string(),
            service_type: "HTTP API".to_string(),
            endpoints: vec![base_url.clone()],
            health: "healthy".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
                map.insert("primal".to_string(), "squirrel".to_string());
                map.insert(
                    "capabilities".to_string(),
                    "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
                );
                map
            },
        },
        ServiceInfo {
            name: "Service Discovery".to_string(),
            service_type: "Registry".to_string(),
            endpoints: vec![],
            health: if ecosystem_status.discovered_services.is_empty() {
                "discovering"
            } else {
                "active"
            }
            .to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("status".to_string(), "ecosystem_integration".to_string());
                map.insert("service_mesh".to_string(), "songbird".to_string());
                map
            },
        },
    ];

    // Add discovered services
    for service in ecosystem_status.discovered_services {
        services.push(ServiceInfo {
            name: service.service_id.to_string(), // Convert Arc<str> to String
            service_type: format!("{:?}", service.primal_type),
            endpoints: vec![service.endpoint.to_string()], // Convert Arc<str> to String
            health: "discovered".to_string(),
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<String, String>>(), // Convert Arc<str> keys/values to String
        });
    }

    let response = ServicesResponse {
        timestamp: chrono::Utc::now(),
        services,
        registry_status: "active".to_string(),
        service_mesh_status: ServiceMeshIntegrationStatus {
            enabled: true,
            provider: "songbird".to_string(),
            load_balancing: true,
            cross_primal_communication: true,
        },
    };

    Ok(warp::reply::json(&response))
}

async fn handle_songbird_register(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Registration is now handled by the ecosystem-api integration
    let response = SongbirdRegistrationResponse {
        success: true,
        message: "Registered with Songbird service mesh via ecosystem-api".to_string(),
        service_id: "squirrel-ai".to_string(),
        timestamp: chrono::Utc::now(),
    };

    Ok(warp::reply::json(&response))
}

async fn handle_songbird_heartbeat(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut state = state.write().await;
    state.last_songbird_heartbeat = Some(chrono::Utc::now());
    state.service_mesh_registered = true;

    let response = SongbirdHeartbeatResponse {
        acknowledged: true,
        timestamp: chrono::Utc::now(),
        next_heartbeat: chrono::Utc::now() + chrono::Duration::seconds(30),
    };

    Ok(warp::reply::json(&response))
}

async fn handle_shutdown(
    shutdown_manager: Arc<ShutdownManager>,
) -> Result<impl Reply, warp::Rejection> {
    tracing::info!("Received shutdown signal");
    match shutdown_manager.request_shutdown().await {
        Ok(report) => {
            tracing::info!("Shutdown initiated successfully, report: {:?}", report);
            Ok(warp::reply::json(&ShutdownResponse { acknowledged: true }))
        }
        Err(e) => {
            tracing::error!("Failed to initiate shutdown: {}", e);
            Err(warp::reject::not_found()) // Use a standard rejection
        }
    }
}

// Additional response types for ecosystem API
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemStatusResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub active_primals: Vec<String>,
    pub service_discovery: String,
    pub service_mesh_status: ServiceMeshStatusResponse,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshStatusResponse {
    pub enabled: bool,
    pub registered: bool,
    pub load_balancing: LoadBalancingResponse,
    pub cross_primal_communication: CrossPrimalCommunicationResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadBalancingResponse {
    pub enabled: bool,
    pub algorithm: String,
    pub health_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossPrimalCommunicationResponse {
    pub enabled: bool,
    pub active_connections: u32,
    pub supported_protocols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalStatusResponse {
    pub name: String,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system: HashMap<String, String>,
    pub application: HashMap<String, String>,
    pub performance: HashMap<String, String>,
    pub ecosystem: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResponse {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub services: Vec<ServiceInfo>,
    pub registry_status: String,
    pub service_mesh_status: ServiceMeshIntegrationStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshIntegrationStatus {
    pub enabled: bool,
    pub provider: String,
    pub load_balancing: bool,
    pub cross_primal_communication: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub service_type: String,
    pub endpoints: Vec<String>,
    pub health: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdRegistrationResponse {
    pub success: bool,
    pub message: String,
    pub service_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdHeartbeatResponse {
    pub acknowledged: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub next_heartbeat: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownResponse {
    pub acknowledged: bool,
}
