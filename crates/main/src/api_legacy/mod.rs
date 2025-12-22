//! Legacy API server implementation
//!
//! This module provides backward-compatible REST API endpoints following
//! the original ecosystem integration patterns. It maintains compatibility
//! while the system evolves toward newer standards.
//!
//! # Architecture
//!
//! The API server integrates with:
//! - `EcosystemManager`: Service discovery and registration
//! - `MetricsCollector`: Performance tracking
//! - `ShutdownManager`: Graceful shutdown coordination
//!
//! # Organization
//!
//! - `types`: Response type definitions
//! - `state`: Server state management
//! - `filters`: Warp filter helpers for dependency injection
//! - `handlers`: HTTP request handlers
//! - `routes`: Route definitions and composition

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::MetricsCollector;

// Public modules
pub mod filters;
pub mod handlers;
pub mod routes;
pub mod state;
pub mod types;

// Re-export commonly used types for backward compatibility
pub use state::ServerState;
pub use types::*;

/// HTTP API server for primal status and health endpoints
///
/// `ApiServer` provides a RESTful API following Songbird service mesh patterns
/// for ecosystem integration. It exposes endpoints for:
///
/// - **Health checks**: `/health`, `/health/live`, `/health/ready`
/// - **Ecosystem status**: `/api/v1/ecosystem/status`
/// - **Service mesh**: `/api/v1/service-mesh/status`
/// - **Primal discovery**: `/api/v1/primals`
/// - **Metrics**: `/api/v1/metrics`
/// - **Service management**: `/api/v1/services`
///
/// # Example
///
/// ```rust,no_run
/// use squirrel::api_legacy::ApiServer;
/// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
/// use squirrel::shutdown::ShutdownManager;
/// use squirrel::MetricsCollector;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let port = 9010;
/// let config = EcosystemConfig::default();
/// let metrics = Arc::new(MetricsCollector::new());
/// let ecosystem = Arc::new(EcosystemManager::new(config, metrics.clone()));
/// let shutdown = Arc::new(ShutdownManager::new());
///
/// let api_server = ApiServer::new(port, ecosystem, metrics, shutdown);
/// api_server.start().await?;
/// # Ok(())
/// # }
/// ```
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

    /// Starts the API server and begins serving requests
    ///
    /// This method initializes all ecosystem-standard routes and starts the HTTP server.
    /// It will block until the server is shut down (either via the shutdown endpoint
    /// or external termination).
    ///
    /// # Routes
    ///
    /// The following endpoints are exposed:
    ///
    /// ## Health Checks
    /// - `GET /health` - Overall health status with ecosystem integration
    /// - `GET /health/live` - Liveness probe (always returns 200 if running)
    /// - `GET /health/ready` - Readiness probe (checks ecosystem connectivity)
    ///
    /// ## Ecosystem API (v1)
    /// - `GET /api/v1/ecosystem/status` - Ecosystem connection status
    /// - `GET /api/v1/service-mesh/status` - Songbird service mesh status
    /// - `GET /api/v1/primals` - List discovered primals
    /// - `GET /api/v1/primals/{name}` - Get specific primal status
    /// - `GET /api/v1/metrics` - Performance metrics
    /// - `GET /api/v1/services` - List registered services
    ///
    /// ## Songbird Integration
    /// - `POST /api/v1/songbird/register` - Register with Songbird
    /// - `POST /api/v1/songbird/heartbeat` - Send heartbeat to Songbird
    ///
    /// ## Management
    /// - `POST /api/v1/shutdown` - Initiate graceful shutdown
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` if:
    /// - Port binding fails (port already in use, insufficient permissions)
    /// - Server initialization fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel::api_legacy::ApiServer;
    /// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
    /// use squirrel::shutdown::ShutdownManager;
    /// use squirrel::MetricsCollector;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let port = 9010;
    /// let config = EcosystemConfig::default();
    /// let metrics = Arc::new(MetricsCollector::new());
    /// let ecosystem = Arc::new(EcosystemManager::new(config, metrics.clone()));
    /// let shutdown = Arc::new(ShutdownManager::new());
    ///
    /// let api_server = ApiServer::new(port, ecosystem, metrics, shutdown);
    ///
    /// // This will block until shutdown
    /// api_server.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<()> {
        tracing::info!(
            "Starting API server on {}:{} with ecosystem integration",
            self.host,
            self.port
        );

        // Build all routes
        let routes = routes::build_routes(
            self.state.clone(),
            self.ecosystem_manager.clone(),
            self.metrics_collector.clone(),
            self.shutdown_manager.clone(),
            self.base_url(),
        );

        tracing::info!(
            "API server started successfully on {}:{} with ecosystem integration",
            self.host,
            self.port
        );

        // Start the server and wait for it to run
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;

        Ok(())
    }

    /// Stop the API server gracefully
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping API server");
        Ok(())
    }

    /// Get current server state
    pub async fn get_state(&self) -> ServerState {
        self.state.read().await.clone()
    }
}
