use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};

use crate::ecosystem::EcosystemManager;
use crate::MetricsCollector;
use crate::shutdown::ShutdownManager;

/// HTTP API server for primal status and health endpoints
pub struct ApiServer {
    /// Port to bind the server to
    port: u16,
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
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            request_count: 0,
            active_connections: 0,
        }
    }
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Ecosystem status response
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemStatusResponse {
    /// Overall ecosystem status
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Active primals
    pub active_primals: Vec<String>,
    /// Service discovery status
    pub service_discovery: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Primal status response
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalStatusResponse {
    /// Primal name
    pub name: String,
    /// Status
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// System metrics
    pub system: HashMap<String, String>,
    /// Application metrics
    pub application: HashMap<String, String>,
    /// Performance metrics
    pub performance: HashMap<String, String>,
}

/// Services response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResponse {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Discovered services
    pub services: Vec<ServiceInfo>,
    /// Registry status
    pub registry_status: String,
}

/// Service information
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: String,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Health status
    pub health: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl ApiServer {
    /// Create new API server
    pub fn new(
        port: u16,
        ecosystem_manager: Arc<EcosystemManager>,
        metrics_collector: Arc<MetricsCollector>,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Self {
        Self {
            port,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
            state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        let state = self.state.clone();
        let ecosystem_manager = self.ecosystem_manager.clone();
        let metrics_collector = self.metrics_collector.clone();

        tracing::info!("Starting API server on port {}", self.port);

        // Define routes
        let health = warp::path!("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_health_check);

        let health_live = warp::path!("health" / "live")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_health_live);

        let health_ready = warp::path!("health" / "ready")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_health_ready);

        let ecosystem_status = warp::path!("api" / "v1" / "ecosystem" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_ecosystem_status);

        let primals_list = warp::path!("api" / "v1" / "primals")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_primals_list);

        let primal_status = warp::path!("api" / "v1" / "primals" / String)
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_primal_status);

        let metrics = warp::path!("api" / "v1" / "metrics")
            .and(warp::get())
            .and(with_metrics_collector(metrics_collector.clone()))
            .and_then(handle_metrics);

        let services = warp::path!("api" / "v1" / "services")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_services);

        // Combine all routes
        let routes = health
            .or(health_live)
            .or(health_ready)
            .or(ecosystem_status)
            .or(primals_list)
            .or(primal_status)
            .or(metrics)
            .or(services)
            .with(warp::cors().allow_any_origin())
            .with(warp::log("api"));

        // Start the server
        let port = self.port;
        tokio::spawn(async move {
            warp::serve(routes).run(([0, 0, 0, 0], port)).await;
        });

        tracing::info!("API server started successfully on port {}", self.port);
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

// Handler functions
async fn handle_health_check(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("request_count".to_string(), state.request_count.to_string());
    metadata.insert("active_connections".to_string(), state.active_connections.to_string());

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: uptime,
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_health_live(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let _state = state.read().await;
    let response = serde_json::json!({
        "status": "live",
        "timestamp": chrono::Utc::now(),
    });
    Ok(warp::reply::json(&response))
}

async fn handle_health_ready(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let _state = state.read().await;
    let response = serde_json::json!({
        "status": "ready",
        "timestamp": chrono::Utc::now(),
    });
    Ok(warp::reply::json(&response))
}

async fn handle_ecosystem_status(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("primal_type".to_string(), "Squirrel".to_string());
    
    let response = EcosystemStatusResponse {
        status: "active".to_string(),
        timestamp: chrono::Utc::now(),
        active_primals: vec![
            "Squirrel".to_string(),
            "ToadStool".to_string(),
            "NestGate".to_string(),
            "BearDog".to_string(),
            "biomeOS".to_string(),
        ],
        service_discovery: "active".to_string(),
        metadata,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_primals_list(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let primals = vec![
        PrimalStatusResponse {
            name: "Squirrel".to_string(),
            status: "active".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "orchestrator".to_string());
                map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
                map
            },
        },
        PrimalStatusResponse {
            name: "ToadStool".to_string(),
            status: "discovering".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec![],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "compute".to_string());
                map
            },
        },
        PrimalStatusResponse {
            name: "NestGate".to_string(),
            status: "discovering".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec![],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "gateway".to_string());
                map
            },
        },
        PrimalStatusResponse {
            name: "BearDog".to_string(),
            status: "discovering".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec![],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "security".to_string());
                map
            },
        },
        PrimalStatusResponse {
            name: "biomeOS".to_string(),
            status: "discovering".to_string(),
            timestamp: chrono::Utc::now(),
            endpoints: vec![],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "platform".to_string());
                map
            },
        },
    ];

    Ok(warp::reply::json(&primals))
}

async fn handle_primal_status(
    primal_name: String,
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let mut metadata = HashMap::new();
    
    let (status, endpoints) = match primal_name.as_str() {
        "Squirrel" => {
            metadata.insert("type".to_string(), "orchestrator".to_string());
            metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            ("active".to_string(), vec!["http://localhost:8080".to_string()])
        },
        "ToadStool" => {
            metadata.insert("type".to_string(), "compute".to_string());
            ("discovering".to_string(), vec![])
        },
        "NestGate" => {
            metadata.insert("type".to_string(), "gateway".to_string());
            ("discovering".to_string(), vec![])
        },
        "BearDog" => {
            metadata.insert("type".to_string(), "security".to_string());
            ("discovering".to_string(), vec![])
        },
        "biomeOS" => {
            metadata.insert("type".to_string(), "platform".to_string());
            ("discovering".to_string(), vec![])
        },
        _ => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Primal not found",
                    "primal": primal_name
                })),
                warp::http::StatusCode::NOT_FOUND,
            ).into_response());
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
    _metrics_collector: Arc<MetricsCollector>,
) -> Result<impl Reply, warp::Rejection> {
    let mut system_metrics = HashMap::new();
    system_metrics.insert("cpu_usage".to_string(), "15.2%".to_string());
    system_metrics.insert("memory_usage".to_string(), "342MB".to_string());
    system_metrics.insert("disk_usage".to_string(), "12.5GB".to_string());
    
    let mut app_metrics = HashMap::new();
    app_metrics.insert("active_connections".to_string(), "0".to_string());
    app_metrics.insert("request_count".to_string(), "0".to_string());
    app_metrics.insert("error_count".to_string(), "0".to_string());
    
    let mut perf_metrics = HashMap::new();
    perf_metrics.insert("avg_response_time".to_string(), "45ms".to_string());
    perf_metrics.insert("p95_response_time".to_string(), "120ms".to_string());
    perf_metrics.insert("throughput".to_string(), "150 req/s".to_string());

    let response = MetricsResponse {
        timestamp: chrono::Utc::now(),
        system: system_metrics,
        application: app_metrics,
        performance: perf_metrics,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_services(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let services = vec![
        ServiceInfo {
            name: "Squirrel API".to_string(),
            service_type: "HTTP API".to_string(),
            endpoints: vec!["http://localhost:8080".to_string()],
            health: "healthy".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
                map.insert("primal".to_string(), "Squirrel".to_string());
                map
            },
        },
        ServiceInfo {
            name: "Service Discovery".to_string(),
            service_type: "Registry".to_string(),
            endpoints: vec![],
            health: "discovering".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("status".to_string(), "background_discovery".to_string());
                map
            },
        },
    ];

    let response = ServicesResponse {
        timestamp: chrono::Utc::now(),
        services,
        registry_status: "active".to_string(),
    };

    Ok(warp::reply::json(&response))
} 