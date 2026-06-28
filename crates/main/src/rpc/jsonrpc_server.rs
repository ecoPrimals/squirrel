// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 Server with Universal Transport (Isomorphic IPC)
//!
//! Modern, idiomatic Rust implementation of JSON-RPC 2.0 protocol for
//! biomeOS integration. This server uses Universal Transport abstractions
//! for automatic platform adaptation (Unix sockets OR TCP with discovery files).
//!
//! Wire types, error codes, and metrics live in [`super::jsonrpc_types`].
//!
//! ## Architecture
//!
//! ```text
//! Universal Transport → JSON-RPC 2.0 → Handler → AI Router → Response
//! (Unix socket on Linux/macOS, TCP fallback on Android/constrained)
//! ```
//!
//! ## Supported Methods (Semantic Naming — wateringHole standard)
//!
//! Semantic names (preferred):
//! - `ai.query` - Send prompt to AI and get response
//! - `ai.list_providers` - List available AI providers
//! - `capability.announce` - Announce capabilities
//! - `capability.discover` - Report own capabilities for socket scanning
//! - `health.check` - Full health check (canonical, PRIMAL_IPC_PROTOCOL v3.0)
//! - `health.liveness` - Liveness probe
//! - `health.readiness` - Readiness probe
//! - `system.health` - Alias for health.check (backward compat)
//! - `system.metrics` - Server metrics
//! - `system.ping` - Alias for health.liveness (backward compat)
//! - `identity.get` - Primal self-knowledge (CAPABILITY_BASED_DISCOVERY_STANDARD v1.0)
//! - `discovery.peers` - Peer discovery
//! - `tool.execute` - Tool execution (local + forwarding to announced primals)
//! - `tool.list` - List available tools (local + remote announced)
//!
//! ## Protocol
//!
//! Standard JSON-RPC 2.0:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "ai.query",
//!   "params": {...},
//!   "id": 1
//! }
//! ```

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_patterns::transport::{UniversalListener, UniversalTransport};

pub use super::jsonrpc_types::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, ServerMetrics, error_codes,
};

/// JSON-RPC Server with Universal Transport (Isomorphic IPC)
pub struct JsonRpcServer {
    /// Service name for Universal Transport discovery
    pub(crate) service_name: String,

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) socket_path: String,

    /// Server metrics
    pub(crate) metrics: Arc<RwLock<ServerMetrics>>,

    /// AI router (optional, for actual AI calls)
    pub(crate) ai_router: Option<Arc<crate::api::ai::AiRouter>>,

    /// Registry of remote primals that announced their tools.
    /// Key: tool name → socket path for forwarding.
    /// Registry: tool name -> announced primal. Uses `Arc<str>` keys for O(1) clone.
    pub(crate) announced_tools:
        Arc<RwLock<std::collections::HashMap<Arc<str>, super::types::AnnouncedPrimal>>>,

    /// Capability registry loaded from capability_registry.toml (source of truth)
    pub capability_registry: Arc<crate::capabilities::registry::CapabilityRegistry>,

    /// When set, binds an additional TCP JSON-RPC listener on `127.0.0.1:<port>` (localhost only).
    tcp_port: Option<u16>,

    /// Provider registry for spring provider registration (`provider.*` methods)
    pub(crate) provider_registry: Arc<crate::universal_adapters::registry::InMemoryServiceRegistry>,

    /// BTSP Phase 2 sessions (from `btsp_handshake`)
    pub(crate) btsp_sessions: dashmap::DashMap<String, super::btsp_handshake::BtspSession>,

    /// BTSP Phase 3 derived session keys
    pub(crate) btsp_session_keys:
        dashmap::DashMap<String, Arc<super::btsp_encrypted_framing::SessionKeys>>,

    /// Shared context manager for `context.*` handlers — persists across requests.
    pub(crate) context_manager: Arc<squirrel_context::ContextManager>,

    /// Request tracker for real-time rate/latency metrics (shared with monitoring subsystem).
    pub(crate) request_tracker: Arc<crate::monitoring::metrics::RequestTracker>,

    /// Shared metrics collector for updating live component metrics (e.g. context sessions).
    pub(crate) metrics_collector: Option<Arc<crate::monitoring::metrics::MetricsCollector>>,

    /// Security orchestrator for pre-dispatch rate limiting and input validation.
    pub(crate) security_orchestrator:
        Option<Arc<crate::security::orchestrator::SecurityOrchestrator>>,

    /// First-byte read timeout on new UDS connections.
    pub(crate) connection_timeout: std::time::Duration,
}

impl JsonRpcServer {
    /// Load the capability registry from the config directory or use compiled defaults
    fn load_registry() -> Arc<crate::capabilities::registry::CapabilityRegistry> {
        let candidates = [
            std::path::PathBuf::from("config/capability_registry.toml"),
            std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../config/capability_registry.toml"
            )),
        ];
        for path in &candidates {
            if path.exists() {
                return Arc::new(crate::capabilities::registry::CapabilityRegistry::load(
                    path,
                ));
            }
        }
        Arc::new(crate::capabilities::registry::CapabilityRegistry::load(
            &candidates[0],
        ))
    }

    /// Create a new JSON-RPC server with Universal Transport
    #[must_use]
    pub fn new(socket_path: String) -> Self {
        Self {
            service_name: crate::niche::PRIMAL_ID.to_string(),
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: None,
            announced_tools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            capability_registry: Self::load_registry(),
            tcp_port: None,
            provider_registry: Arc::new(
                crate::universal_adapters::registry::InMemoryServiceRegistry::new(),
            ),
            btsp_sessions: dashmap::DashMap::new(),
            btsp_session_keys: dashmap::DashMap::new(),
            context_manager: Arc::new(squirrel_context::ContextManager::new()),
            request_tracker: Arc::new(crate::monitoring::metrics::RequestTracker::new()),
            metrics_collector: None,
            security_orchestrator: None,
            connection_timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Create server with AI router
    #[must_use]
    pub fn with_ai_router(socket_path: String, ai_router: Arc<crate::api::ai::AiRouter>) -> Self {
        Self {
            service_name: crate::niche::PRIMAL_ID.to_string(),
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: Some(ai_router),
            announced_tools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            capability_registry: Self::load_registry(),
            tcp_port: None,
            provider_registry: Arc::new(
                crate::universal_adapters::registry::InMemoryServiceRegistry::new(),
            ),
            btsp_sessions: dashmap::DashMap::new(),
            btsp_session_keys: dashmap::DashMap::new(),
            context_manager: Arc::new(squirrel_context::ContextManager::new()),
            request_tracker: Arc::new(crate::monitoring::metrics::RequestTracker::new()),
            metrics_collector: None,
            security_orchestrator: None,
            connection_timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Share a `RequestTracker` with the monitoring subsystem so component
    /// metrics reflect real RPC traffic.
    #[must_use]
    pub fn with_request_tracker(
        mut self,
        tracker: Arc<crate::monitoring::metrics::RequestTracker>,
    ) -> Self {
        self.request_tracker = tracker;
        self
    }

    /// Attach a `MetricsCollector` for live component metric updates
    /// (e.g. context session counts).
    #[must_use]
    pub fn with_metrics_collector(
        mut self,
        collector: Arc<crate::monitoring::metrics::MetricsCollector>,
    ) -> Self {
        self.metrics_collector = Some(collector);
        self
    }

    /// Attach a `SecurityOrchestrator` for pre-dispatch rate limiting and input validation.
    #[must_use]
    pub fn with_security_orchestrator(
        mut self,
        orchestrator: Arc<crate::security::orchestrator::SecurityOrchestrator>,
    ) -> Self {
        self.security_orchestrator = Some(orchestrator);
        self
    }

    /// Enable a localhost TCP JSON-RPC listener on `127.0.0.1:<port>` alongside Universal Transport.
    #[must_use]
    pub const fn with_tcp_port(mut self, port: u16) -> Self {
        self.tcp_port = Some(port);
        self
    }

    /// Set the first-byte read timeout for new UDS connections.
    #[must_use]
    pub const fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Start the JSON-RPC server with Universal Transport (Isomorphic IPC)
    ///
    /// This method uses Universal Transport abstractions for automatic platform adaptation:
    /// - Linux/macOS: Unix sockets (preferred)
    /// - Android (SELinux): TCP fallback with discovery files
    /// - Windows: Named pipes (when available)
    ///
    /// ## SQ-01: Dual Socket Binding (Linux)
    ///
    /// On Linux the primary listener uses an abstract namespace socket (`\0squirrel`),
    /// which is invisible to `readdir()`. biomeOS filesystem socket scanning therefore
    /// cannot discover abstract-only primals. To comply with IPC_COMPLIANCE_MATRIX
    /// and PRIMAL_IPC_PROTOCOL, we **also** bind a filesystem socket at
    /// `$XDG_RUNTIME_DIR/biomeos/squirrel.sock` so biomeOS can find us.
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🔌 Starting JSON-RPC server with Universal Transport...");

        // Bind using Universal Transport (automatic fallback)
        let listener = UniversalListener::bind(&self.service_name, None)
            .await
            .context("Failed to bind Universal Transport listener")?;

        // SQ-01: On Linux, also bind a filesystem socket at the biomeOS standard
        // path. Abstract namespace sockets are invisible to readdir()-based
        // discovery used by biomeOS socket scanning.
        // Uses self.socket_path (resolved from --socket CLI / config / env)
        // so the launcher's expected path is honoured.
        #[cfg(target_os = "linux")]
        {
            let fs_path = self.socket_path.clone();
            if let Err(e) = super::unix_socket::prepare_socket_path(&fs_path) {
                warn!(
                    "Failed to prepare filesystem socket {}: {} (abstract-only mode)",
                    fs_path, e
                );
            } else {
                match tokio::net::UnixListener::bind(&fs_path) {
                    Ok(fs_listener) => {
                        info!(
                            "✅ Filesystem socket bound: {} (biomeOS discovery)",
                            fs_path
                        );
                        #[cfg(unix)]
                        {
                            if let Err(e) =
                                super::unix_socket::try_create_capability_domain_symlink(&fs_path)
                            {
                                warn!(
                                    "Could not create capability-domain symlink ai.sock → {} (PRIMAL_IPC_PROTOCOL): {}",
                                    std::path::Path::new(&fs_path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(concat!(env!("CARGO_PKG_NAME"), ".sock")),
                                    e
                                );
                            }
                        }
                        let server = Arc::clone(&self);
                        tokio::spawn(async move {
                            Self::accept_filesystem_socket(server, fs_listener).await;
                        });
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ Could not bind filesystem socket {}: {} (abstract-only mode)",
                            fs_path, e
                        );
                    }
                }
            }
        }

        if let Some(port) = self.tcp_port {
            let addr = format!("{}:{port}", universal_constants::network::LOCALHOST_IPV4);
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(tcp_listener) => {
                    info!("TCP JSON-RPC listener on {addr}");
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        Self::accept_tcp_jsonrpc(server, tcp_listener).await;
                    });
                }
                Err(e) => {
                    warn!(
                        "Could not bind TCP JSON-RPC on {}: {} (continuing without TCP)",
                        addr, e
                    );
                }
            }
        }

        info!("✅ JSON-RPC server ready (service: {})", self.service_name);

        // Accept connections loop (primary transport)
        loop {
            match listener.accept().await {
                Ok((transport, _remote_addr)) => {
                    debug!("📥 New connection accepted");
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_uds_connection(server, transport).await {
                            error!("Error handling connection: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Accept loop for the biomeOS filesystem socket (SQ-01 compliance).
    ///
    /// Runs as a spawned task alongside the primary abstract/TCP listener.
    /// Connections are handed off to the same `handle_universal_connection`
    /// pipeline so both transports share handler logic.
    #[cfg(target_os = "linux")]
    async fn accept_filesystem_socket(server: Arc<Self>, fs_listener: tokio::net::UnixListener) {
        loop {
            match fs_listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("📥 Filesystem socket connection accepted");
                    let srv = Arc::clone(&server);
                    tokio::spawn(async move {
                        let transport = UniversalTransport::UnixSocket(stream);
                        if let Err(e) = Self::handle_uds_connection(srv, transport).await {
                            error!("Error handling filesystem socket connection: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept on filesystem socket: {}", e);
                }
            }
        }
    }

    /// Accept loop for localhost TCP JSON-RPC (newline-delimited, same handler as Unix).
    pub(super) async fn accept_tcp_jsonrpc(server: Arc<Self>, listener: tokio::net::TcpListener) {
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("📥 TCP JSON-RPC connection accepted");
                    let srv = Arc::clone(&server);
                    tokio::spawn(async move {
                        let transport = UniversalTransport::Tcp(stream);
                        if let Err(e) = srv.handle_universal_connection(transport).await {
                            error!("Error handling TCP JSON-RPC connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept on TCP JSON-RPC listener: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "jsonrpc_server_unit_tests.rs"]
mod unit_tests;
