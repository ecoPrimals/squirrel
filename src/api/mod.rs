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
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Discovered services
    pub discovered_services: u32,
    /// Healthy services
    pub healthy_services: u32,
    /// Service details
    pub services: Vec<ServiceInfo>,
}

/// Service information
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Health status
    pub health: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Last seen
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Ecosystem overview response
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemOverviewResponse {
    /// Total primals
    pub total_primals: u32,
    /// Active primals
    pub active_primals: u32,
    /// Service summary
    pub service_summary: ServiceSummary,
    /// System health
    pub system_health: String,
}

/// Service summary
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceSummary {
    /// Total services
    pub total_services: u32,
    /// Healthy services
    pub healthy_services: u32,
    /// Unhealthy services
    pub unhealthy_services: u32,
}

/// Primal information
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name
    pub name: String,
    /// Primal type
    pub primal_type: String,
    /// Health status
    pub health: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request count
    pub request_count: u64,
    /// Active connections
    pub active_connections: u32,
    /// Uptime seconds
    pub uptime_seconds: u64,
    /// System metrics
    pub system_metrics: HashMap<String, f64>,
}

/// Shutdown status response
#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownStatusResponse {
    /// Shutdown initiated
    pub shutdown_initiated: bool,
    /// Shutdown progress
    pub shutdown_progress: f32,
    /// Components status
    pub components: Vec<ComponentStatus>,
}

/// Component status
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Status
    pub status: String,
    /// Shutdown duration
    pub shutdown_duration_ms: Option<u64>,
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
        let shutdown_manager = self.shutdown_manager.clone();

        tracing::info!("Starting API server on port {}", self.port);

        // Define routes
        let health = warp::path!("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_health_check);

        let health_live = warp::path!("health" / "live")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_liveness_check);

        let health_ready = warp::path!("health" / "ready")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(handle_readiness_check);

        let ecosystem_status = warp::path!("api" / "v1" / "ecosystem" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_ecosystem_status);

        let ecosystem_overview = warp::path!("api" / "v1" / "ecosystem" / "overview")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_ecosystem_overview);

        let primals_list = warp::path!("api" / "v1" / "primals")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_primals_list);

        let metrics = warp::path!("api" / "v1" / "metrics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_metrics_collector(metrics_collector.clone()))
            .and_then(handle_metrics);

        let shutdown_status = warp::path!("api" / "v1" / "shutdown" / "status")
            .and(warp::get())
            .and(with_shutdown_manager(shutdown_manager.clone()))
            .and_then(handle_shutdown_status);

        // Combine all routes
        let routes = health
            .or(health_live)
            .or(health_ready)
            .or(ecosystem_status)
            .or(ecosystem_overview)
            .or(primals_list)
            .or(metrics)
            .or(shutdown_status)
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

fn with_shutdown_manager(
    shutdown_manager: Arc<ShutdownManager>,
) -> impl Filter<Extract = (Arc<ShutdownManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || shutdown_manager.clone())
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

async fn handle_liveness_check(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let _state = state.read().await;
    let response = serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now()
    });
    Ok(warp::reply::json(&response))
}

async fn handle_readiness_check(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let _state = state.read().await;
    let response = serde_json::json!({
        "status": "ready",
        "timestamp": chrono::Utc::now()
    });
    Ok(warp::reply::json(&response))
}

async fn handle_ecosystem_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let response = EcosystemStatusResponse {
        status: "operational".to_string(),
        timestamp: chrono::Utc::now(),
        discovered_services: 0,
        healthy_services: 0,
        services: vec![],
    };
    Ok(warp::reply::json(&response))
}

async fn handle_ecosystem_overview(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let response = EcosystemOverviewResponse {
        total_primals: 5,
        active_primals: 1,
        service_summary: ServiceSummary {
            total_services: 1,
            healthy_services: 1,
            unhealthy_services: 0,
        },
        system_health: "healthy".to_string(),
    };
    Ok(warp::reply::json(&response))
}

async fn handle_primals_list(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let primals = vec![
        PrimalInfo {
            name: "Squirrel".to_string(),
            primal_type: "AI".to_string(),
            health: "healthy".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![
                "context_management".to_string(),
                "ai_coordination".to_string(),
                "ecosystem_management".to_string(),
            ],
            last_seen: chrono::Utc::now(),
        }
    ];
    Ok(warp::reply::json(&primals))
}

async fn handle_metrics(
    state: Arc<RwLock<ServerState>>,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let mut system_metrics = HashMap::new();
    system_metrics.insert("cpu_usage".to_string(), 0.0);
    system_metrics.insert("memory_usage".to_string(), 0.0);
    system_metrics.insert("disk_usage".to_string(), 0.0);

    let response = MetricsResponse {
        timestamp: chrono::Utc::now(),
        request_count: state.request_count,
        active_connections: state.active_connections,
        uptime_seconds: uptime,
        system_metrics,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_shutdown_status(
    shutdown_manager: Arc<ShutdownManager>,
) -> Result<impl Reply, warp::Rejection> {
    let response = ShutdownStatusResponse {
        shutdown_initiated: false,
        shutdown_progress: 0.0,
        components: vec![
            ComponentStatus {
                name: "API Server".to_string(),
                status: "running".to_string(),
                shutdown_duration_ms: None,
            },
            ComponentStatus {
                name: "Ecosystem Manager".to_string(),
                status: "running".to_string(),
                shutdown_duration_ms: None,
            },
        ],
    };
    Ok(warp::reply::json(&response))
} 