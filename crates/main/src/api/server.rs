//! API server core implementation
//!
//! Main server struct and routing configuration.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::MetricsCollector;

use super::ai::{
    ai_routes, handle_compatible_models, handle_model_load, provider_routes, ActionRegistry,
    AiRouter, ServiceMeshAiIntegration,
};
use super::ecosystem;
use super::health;
use super::management;
use super::metrics;
use super::service_mesh;

/// Server state tracking
#[derive(Debug, Clone)]
pub struct ServerState {
    /// When the server was started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Request count
    pub request_count: u64,
    /// Active connections
    pub active_connections: u32,
    /// Service mesh registration status
    pub service_mesh_registered: bool,
    /// Last service mesh heartbeat timestamp
    pub last_service_mesh_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }
    }
}

/// HTTP API server for primal status and health endpoints
///
/// `ApiServer` provides a `RESTful` API following service mesh patterns
/// for ecosystem integration via capability discovery. It exposes endpoints for:
///
/// - **Health checks**: `/health`, `/health/live`, `/health/ready`
/// - **Ecosystem status**: `/api/v1/ecosystem/status`
/// - **Service mesh**: `/api/v1/service-mesh/status`
/// - **Primal discovery**: `/api/v1/primals`
/// - **Metrics**: `/api/v1/metrics`
/// - **Service management**: `/api/v1/services`
///
/// # Architecture
///
/// The server integrates with:
/// - `EcosystemManager`: For service discovery and registration
/// - `MetricsCollector`: For performance tracking
/// - `ShutdownManager`: For graceful shutdown coordination
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

impl ApiServer {
    /// Create a new API server with dynamic configuration
    #[must_use]
    pub fn new(
        port: u16,
        ecosystem_manager: Arc<EcosystemManager>,
        metrics_collector: Arc<MetricsCollector>,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Self {
        let host = std::env::var("SQUIRREL_SERVICE_HOST").unwrap_or_else(|_| {
            // Environment-aware default host using universal-constants
            use universal_constants::network;
            if std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .eq_ignore_ascii_case("production")
            {
                network::DEFAULT_BIND_ADDRESS.to_string()
            } else {
                network::LOCALHOST_IPV4.to_string()
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
    #[must_use]
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
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Get the server websocket URL
    #[must_use]
    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}/ws", self.host, self.port)
    }

    /// Starts the API server and begins serving requests
    ///
    /// This method initializes all ecosystem-standard routes and starts the HTTP server.
    pub async fn start(&self) -> Result<()> {
        let state = self.state.clone();
        let ecosystem_manager = self.ecosystem_manager.clone();
        let metrics_collector = self.metrics_collector.clone();
        let shutdown_manager = self.shutdown_manager.clone();

        tracing::info!(
            "Starting API server on {}:{} with ecosystem integration",
            self.host,
            self.port
        );

        // Initialize AI router (capability-based discovery)
        tracing::info!("🤖 Initializing AI capability router...");
        let ai_router = match AiRouter::new_with_discovery(None).await {
            Ok(router) => {
                let provider_count = router.provider_count().await;
                tracing::info!(
                    "✅ AI router initialized successfully with {} provider(s)",
                    provider_count
                );
                Arc::new(router)
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  AI router initialization failed: {}. AI endpoints will return errors.",
                    e
                );
                tracing::warn!(
                    "💡 Set OPENAI_API_KEY or HUGGINGFACE_API_KEY to enable AI capabilities"
                );
                Arc::new(AiRouter::default())
            }
        };

        // Register AI capabilities with service mesh (non-blocking, optional)
        // Uses capability discovery to find available service mesh providers
        let ai_router_clone = ai_router.clone();
        let base_url = self.base_url();
        tokio::spawn(async move {
            let mut service_mesh_integration = ServiceMeshAiIntegration::new(
                ai_router_clone.clone(),
                "squirrel".to_string(), // Service ID
                base_url.clone(),
            );

            // Register capabilities
            if let Err(e) = service_mesh_integration.register_capabilities().await {
                tracing::warn!("⚠️  Service mesh AI registration failed: {}", e);
                tracing::info!(
                    "💡 AI capabilities available locally without service mesh coordination"
                );
            }

            // Start heartbeat loop (30s interval)
            let integration = Arc::new(service_mesh_integration);
            integration.start_heartbeat_loop(30).await;
        });

        // Start JSON-RPC server on Unix socket (for biomeOS integration)
        let node_id = std::env::var("SQUIRREL_NODE_ID")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "squirrel".to_string());

        tracing::info!("🔌 Starting JSON-RPC server on Unix socket...");
        let rpc_server = crate::rpc::RpcServer::with_ai_router(&node_id, ai_router.clone());
        let rpc_socket_path = rpc_server.socket_path().to_string();
        tracing::info!("✅ JSON-RPC server will listen on: {}", rpc_socket_path);

        // Start RPC server in background task
        tokio::spawn(async move {
            if let Err(e) = rpc_server.start().await {
                tracing::error!("❌ JSON-RPC server error: {}", e);
            }
        });

        // Create AI routes
        let ai = ai_routes(ai_router);

        // Create action registry for Phase 6 (Ultimate Vision)
        tracing::info!("🌟 Initializing ActionRegistry for dynamic provider registration...");
        let action_registry = Arc::new(ActionRegistry::new());

        // Register PrimalPulse tools
        tracing::info!("🌊 Registering PrimalPulse AI tools...");
        crate::primal_pulse::register_primal_pulse_tools(action_registry.clone()).await;

        let provider = provider_routes(action_registry);
        tracing::info!("✅ ActionRegistry initialized - Phase 6 ULTIMATE endpoints ready!");

        // Define health check routes
        let health_check = warp::path!("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(health::handle_health_check);

        let health_live = warp::path!("health" / "live")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(health::handle_health_live);

        let health_ready = warp::path!("health" / "ready")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(health::handle_health_ready);

        // Ecosystem API endpoints
        let ecosystem_status = warp::path!("api" / "v1" / "ecosystem" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(ecosystem::handle_ecosystem_status);

        let service_mesh_status = warp::path!("api" / "v1" / "service-mesh" / "status")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(ecosystem::handle_service_mesh_status);

        let primals_list = warp::path!("api" / "v1" / "primals")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(ecosystem::handle_primals_list);

        let primal_status = warp::path!("api" / "v1" / "primals" / String)
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(ecosystem::handle_primal_status);

        let services = warp::path!("api" / "v1" / "services")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and(with_base_url(self.base_url()))
            .and_then(ecosystem::handle_services);

        // Tower discovery endpoint (for cross-tower AI mesh)
        let towers = warp::path!("api" / "ecosystem" / "towers")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(ecosystem::handle_towers_discovery);

        // Model management endpoints (Phase 2: Model Coordination)
        let models_compatible = warp::path!("api" / "ai" / "models" / "compatible")
            .and(warp::get())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_compatible_models);

        let model_load = warp::path!("api" / "ai" / "model" / "load")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(handle_model_load);

        // Metrics endpoint
        let metrics_endpoint = warp::path!("api" / "v1" / "metrics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and(with_metrics_collector(metrics_collector.clone()))
            .and_then(metrics::handle_metrics);

        // ===================================================================
        // DEPRECATED: Legacy service mesh endpoints (backward compatibility)
        // These endpoints use hardcoded paths for compatibility with older
        // service mesh providers. New code should use the generic paths below.
        // ===================================================================
        #[allow(deprecated)]
        let songbird_register = warp::path!("api" / "v1" / "songbird" / "register")
            .and(warp::post())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(service_mesh::handle_service_mesh_register);

        #[allow(deprecated)]
        let songbird_heartbeat = warp::path!("api" / "v1" / "songbird" / "heartbeat")
            .and(warp::post())
            .and(with_state(state.clone()))
            .and_then(service_mesh::handle_service_mesh_heartbeat);

        // ===================================================================
        // NEW: Capability-based service mesh endpoints (protocol-agnostic)
        // These work with ANY service mesh provider discovered at runtime.
        // ===================================================================
        let service_mesh_register = warp::path!("api" / "v1" / "service_mesh" / "register")
            .and(warp::post())
            .and(with_ecosystem_manager(ecosystem_manager.clone()))
            .and_then(service_mesh::handle_service_mesh_register);

        let service_mesh_heartbeat = warp::path!("api" / "v1" / "service_mesh" / "heartbeat")
            .and(warp::post())
            .and(with_state(state.clone()))
            .and_then(service_mesh::handle_service_mesh_heartbeat);

        // Management endpoints
        let shutdown = warp::path!("api" / "v1" / "shutdown")
            .and(warp::post())
            .and(with_shutdown_manager(shutdown_manager.clone()))
            .and_then(management::handle_shutdown);

        // Request counting middleware
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
                health_check
                    .or(health_live)
                    .or(health_ready)
                    .or(ecosystem_status)
                    .or(service_mesh_status)
                    .or(primals_list)
                    .or(primal_status)
                    .or(metrics_endpoint)
                    .or(services)
                    .or(towers) // Tower discovery for cross-tower AI mesh
                    .or(models_compatible) // Model compatibility check (Phase 2)
                    .or(model_load) // Model loading (Phase 2)
                    // Deprecated legacy endpoints (backward compatibility only)
                    .or(songbird_register)
                    .or(songbird_heartbeat)
                    // NEW: Capability-based service mesh endpoints
                    .or(service_mesh_register)
                    .or(service_mesh_heartbeat)
                    .or(shutdown)
                    .or(ai) // AI routes (Phase 1-4)
                    .or(provider), // Provider registration routes (Phase 6)
            )
            .with(warp::cors().allow_any_origin())
            .with(warp::log("api"));

        tracing::info!(
            "API server started successfully on {}:{}",
            self.host,
            self.port
        );

        // Start the server
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;

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

async fn increment_request_count(state: Arc<RwLock<ServerState>>) {
    let mut state_guard = state.write().await;
    state_guard.request_count += 1;
}
