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
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_constants::network::LOCALHOST_IPV4;
use universal_patterns::transport::{UniversalListener, UniversalTransport};

pub use super::jsonrpc_types::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, ServerMetrics, error_codes,
};

/// JSON-RPC Server with Universal Transport (Isomorphic IPC)
pub struct JsonRpcServer {
    /// Service name for Universal Transport discovery
    pub(crate) service_name: String,

    /// Resolved filesystem socket path (manifest, lifecycle cleanup, SQ-01 dual bind on Linux).
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

    /// When set, binds an additional TCP JSON-RPC listener on `<tcp_bind_host>:<port>`.
    tcp_port: Option<u16>,

    /// Bind address for the TCP listener. Defaults to `127.0.0.1` (localhost only).
    /// Set to `0.0.0.0` for Docker/benchScale deployments.
    tcp_bind_host: String,
}

impl JsonRpcServer {
    /// Load the capability registry from CWD or fall back to the compiled-in
    /// embedded copy (no absolute host paths baked into the binary).
    fn load_registry() -> Arc<crate::capabilities::registry::CapabilityRegistry> {
        let cwd_candidate = std::path::PathBuf::from("capability_registry.toml");
        if cwd_candidate.exists() {
            return Arc::new(crate::capabilities::registry::CapabilityRegistry::load(
                &cwd_candidate,
            ));
        }
        Arc::new(crate::capabilities::registry::CapabilityRegistry::load(
            &cwd_candidate,
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
            tcp_bind_host: LOCALHOST_IPV4.to_string(),
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
            tcp_bind_host: LOCALHOST_IPV4.to_string(),
        }
    }

    /// Enable a TCP JSON-RPC listener on `<bind_host>:<port>` alongside Universal Transport.
    #[must_use]
    pub fn with_tcp(mut self, port: u16, bind_host: String) -> Self {
        self.tcp_port = Some(port);
        self.tcp_bind_host = bind_host;
        self
    }

    /// Enable a localhost TCP JSON-RPC listener on `127.0.0.1:<port>` alongside Universal Transport.
    #[must_use]
    pub fn with_tcp_port(self, port: u16) -> Self {
        self.with_tcp(port, LOCALHOST_IPV4.to_string())
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
    /// the same path as `socket_path` (CLI / config / auto-detection, after
    /// [`super::unix_socket::resolve_socket_path_for_ipc`]) so biomeOS can find us.
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🔌 Starting JSON-RPC server with Universal Transport...");

        // Bind using Universal Transport (automatic fallback)
        let listener = UniversalListener::bind(&self.service_name, None)
            .await
            .context("Failed to bind Universal Transport listener")?;

        // SQ-01: On Linux, also bind a filesystem socket at the biomeOS standard
        // path. Abstract namespace sockets are invisible to readdir()-based
        // discovery used by biomeOS socket scanning.
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
                            "✅ Filesystem socket bound: {} (orchestrator discovery)",
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
                                        .unwrap_or("squirrel.sock"),
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
            let bind_host = &self.tcp_bind_host;
            let addr = format!("{bind_host}:{port}");
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(tcp_listener) => {
                    info!("TCP JSON-RPC listener on {addr}");
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        Self::accept_tcp_jsonrpc(server, tcp_listener).await;
                    });
                }
                Err(e) => {
                    warn!("Could not bind TCP JSON-RPC on {addr}: {e} (continuing without TCP)");
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
                        Self::accept_with_btsp(server, transport).await;
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
                        Self::accept_with_btsp(srv, transport).await;
                    });
                }
                Err(e) => {
                    error!("Failed to accept on filesystem socket: {}", e);
                }
            }
        }
    }

    /// BTSP Phase 2 with auto-detect: handshake when `FAMILY_ID` is set,
    /// graceful fallback for plain JSON-RPC clients (PG-14 resolution).
    ///
    /// When the first byte is `{`, the client is sending plain JSON-RPC
    /// without BTSP framing. The consumed byte is prepended via
    /// `std::io::Cursor` + `tokio::io::chain` so the JSON-RPC handler
    /// sees the complete request.
    async fn accept_with_btsp(server: Arc<Self>, mut transport: UniversalTransport) {
        match super::btsp_handshake::maybe_handshake(&mut transport).await {
            Ok(session) => {
                if let Some(ref s) = session {
                    debug!(session_id = %s.session_id, "BTSP authenticated");
                }
                if let Err(e) = server.clone().handle_universal_connection(transport).await {
                    error!("Error handling connection: {}", e);
                }
            }
            Err(super::btsp_handshake::BtspError::PlainJsonRpc) => {
                if let Err(e) = server
                    .clone()
                    .handle_universal_connection_with_prefix(transport, b'{')
                    .await
                {
                    error!("Error handling plain JSON-RPC connection: {}", e);
                }
            }
            Err(e) => {
                warn!("BTSP handshake failed, refusing connection: {e}");
            }
        }
    }

    /// Accept loop for localhost TCP JSON-RPC (newline-delimited, same handler as Unix).
    async fn accept_tcp_jsonrpc(server: Arc<Self>, listener: tokio::net::TcpListener) {
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("📥 TCP JSON-RPC connection accepted");
                    let srv = Arc::clone(&server);
                    tokio::spawn(async move {
                        let transport = UniversalTransport::Tcp(stream);
                        if let Err(e) = srv.clone().handle_universal_connection(transport).await {
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

    /// Handle a plain JSON-RPC connection where the first byte (`{`) was
    /// already consumed by BTSP auto-detect (PG-14 fallback path).
    ///
    /// Prepends the consumed byte to reconstruct the complete JSON-RPC
    /// request, then reads the rest of the first line and processes it
    /// through the standard JSON-RPC handler.
    async fn handle_universal_connection_with_prefix(
        self: std::sync::Arc<Self>,
        transport: UniversalTransport,
        prefix_byte: u8,
    ) -> Result<()> {
        let mut reader = BufReader::new(transport);
        let mut line = String::from(char::from(prefix_byte));
        let mut rest = String::new();

        match reader.read_line(&mut rest).await {
            Ok(0) => {
                debug!("Client disconnected after sending single byte");
                Ok(())
            }
            Ok(_) => {
                line.push_str(&rest);
                self.handle_jsonrpc_with_first_line(reader, line).await
            }
            Err(e) => {
                warn!("Error reading from plain JSON-RPC connection: {}", e);
                Err(e).context("Failed to read from plain JSON-RPC connection")
            }
        }
    }

    /// Handle a client connection via Universal Transport with protocol negotiation.
    ///
    /// This method works with ANY transport type (Unix socket, TCP, Named pipe)
    /// using polymorphic `AsyncRead` + `AsyncWrite` traits.
    ///
    /// ## Protocol Negotiation
    ///
    /// The server supports both JSON-RPC and tarpc protocols:
    /// - If client sends "PROTOCOLS: ..." → negotiate and route to selected protocol
    /// - If client sends JSON-RPC request → default to JSON-RPC
    /// - tarpc provides higher performance for bulk operations
    async fn handle_universal_connection(
        self: std::sync::Arc<Self>,
        transport: UniversalTransport,
    ) -> Result<()> {
        // Wrap transport in BufReader for line-oriented protocol
        let mut reader = BufReader::new(transport);
        let mut line = String::new();

        // Read first line with timeout to detect protocol negotiation
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - client disconnected immediately
                debug!("Client disconnected before sending data");
                Ok(())
            }
            Ok(_) => {
                let trimmed = line.trim();

                // Check if this is a protocol negotiation request
                if trimmed.starts_with("PROTOCOLS:") {
                    #[cfg(feature = "tarpc-rpc")]
                    {
                        self.handle_protocol_negotiation(reader, &line).await
                    }
                    #[cfg(not(feature = "tarpc-rpc"))]
                    {
                        // tarpc not available, respond with JSON-RPC only
                        info!(
                            "Protocol negotiation requested, tarpc not enabled, selecting JSON-RPC"
                        );
                        let response = "PROTOCOL: jsonrpc\n";
                        reader
                            .get_mut()
                            .write_all(response.as_bytes())
                            .await
                            .context("Failed to write protocol response")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush protocol response")?;
                        // Continue with JSON-RPC handling below
                        self.handle_jsonrpc_loop(reader).await
                    }
                } else {
                    // Not a protocol request - treat as JSON-RPC request
                    // Process this first line and continue with JSON-RPC loop
                    self.handle_jsonrpc_with_first_line(reader, line).await
                }
            }
            Err(e) => {
                warn!("Error reading from connection: {}", e);
                Err(e).context("Failed to read first line from connection")
            }
        }
    }

    /// Handle JSON-RPC loop after processing first line (shared with protocol negotiation)
    async fn handle_jsonrpc_with_first_line(
        &self,
        mut reader: BufReader<UniversalTransport>,
        first_line: String,
    ) -> Result<()> {
        if let Some(response_json) = self.handle_request_or_batch(&first_line).await {
            let mut out = response_json;
            out.push('\n');
            reader
                .get_mut()
                .write_all(out.as_bytes())
                .await
                .context("Failed to write JSON-RPC response")?;
            reader
                .get_mut()
                .flush()
                .await
                .context("Failed to flush JSON-RPC response")?;
        }

        self.handle_jsonrpc_loop(reader).await
    }

    /// Handle protocol negotiation for multi-protocol support
    #[cfg(feature = "tarpc-rpc")]
    async fn handle_protocol_negotiation(
        self: std::sync::Arc<Self>,
        mut reader: BufReader<UniversalTransport>,
        first_line: &str,
    ) -> Result<()> {
        use super::protocol::IpcProtocol;
        use super::protocol_negotiation::{ProtocolRequest, ProtocolResponse, select_protocol};
        use super::tarpc_server::TarpcRpcServer;

        info!("🔄 Protocol negotiation requested");

        // Parse the protocol request
        let request = match ProtocolRequest::from_wire(first_line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid protocol request: {}", e);
                // Fallback to JSON-RPC
                let response = "PROTOCOL: jsonrpc\n";
                reader
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await
                    .context("Failed to write protocol fallback response")?;
                reader
                    .get_mut()
                    .flush()
                    .await
                    .context("Failed to flush protocol fallback response")?;
                return self.handle_jsonrpc_loop(reader).await;
            }
        };

        // Server supports both protocols
        let server_supported = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let selected = select_protocol(&request.supported, &server_supported);

        // Send response
        let response = ProtocolResponse::new(selected);
        let response_line = response.to_wire();
        reader
            .get_mut()
            .write_all(response_line.as_bytes())
            .await
            .context("Failed to write protocol negotiation response")?;
        reader
            .get_mut()
            .flush()
            .await
            .context("Failed to flush protocol negotiation response")?;

        info!("✅ Protocol negotiated: {}", selected);

        // Route to appropriate handler
        match selected {
            IpcProtocol::Tarpc => {
                // Extract the transport from the reader to pass to tarpc
                let transport = reader.into_inner();

                // Create tarpc server that delegates to this JSON-RPC server
                let tarpc_server = TarpcRpcServer::from_jsonrpc(self);

                // Handle connection with tarpc
                tarpc_server.handle_connection(transport).await
            }
            IpcProtocol::JsonRpc => {
                // Continue with JSON-RPC
                self.handle_jsonrpc_loop(reader).await
            }
        }
    }

    /// Standard JSON-RPC request/response loop (supports batch per Section 6).
    async fn handle_jsonrpc_loop(&self, mut reader: BufReader<UniversalTransport>) -> Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    if let Some(response_json) = self.handle_request_or_batch(&line).await {
                        let mut out = response_json;
                        out.push('\n');
                        reader
                            .get_mut()
                            .write_all(out.as_bytes())
                            .await
                            .context("Failed to write JSON-RPC response in loop")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush JSON-RPC response in loop")?;
                    }
                    // None means all-notification batch — no response per spec
                }
                Err(e) => {
                    warn!("Error reading from connection: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request or batch (JSON-RPC 2.0 Section 6).
    ///
    /// Parses the raw JSON. If it's an array, dispatches each element as a
    /// separate request and collects responses. Notifications (no `id`) produce
    /// no response. If the batch is empty, returns a single Invalid Request
    /// error per spec.
    pub(crate) async fn handle_request_or_batch(&self, request_str: &str) -> Option<String> {
        let trimmed = request_str.trim();

        let parsed: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::PARSE_ERROR,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                    id: Value::Null,
                };
                return serde_json::to_string(&resp).ok();
            }
        };

        if let Value::Array(items) = parsed {
            if items.is_empty() {
                let resp = JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::INVALID_REQUEST,
                        message: "Empty batch".to_string(),
                        data: None,
                    }),
                    id: Value::Null,
                };
                return serde_json::to_string(&resp).ok();
            }

            let mut responses = Vec::with_capacity(items.len());
            for item in items {
                let single = serde_json::to_string(&item).unwrap_or_default();
                let is_notification = item.as_object().is_some_and(|m| !m.contains_key("id"));
                let resp = self.handle_single_request(&single).await;
                // Per spec: notifications (no id) produce no response element
                if !is_notification && let Some(r) = resp {
                    responses.push(r);
                }
            }

            if responses.is_empty() {
                // All were notifications — per spec, no response at all
                return None;
            }
            return serde_json::to_string(&responses).ok();
        }

        // Single request
        match self.handle_single_request(trimmed).await {
            Some(resp) => serde_json::to_string(&resp).ok(),
            None => None,
        }
    }

    /// Handle a single JSON-RPC request (non-batch).
    /// Returns `None` for successful notifications (no response per JSON-RPC 2.0).
    async fn handle_single_request(&self, request_str: &str) -> Option<JsonRpcResponse> {
        let start_time = Instant::now();

        let value: Value = match serde_json::from_str(request_str.trim()) {
            Ok(v) => v,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::PARSE_ERROR,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                    id: Value::Null,
                });
            }
        };

        let Some(obj) = value.as_object() else {
            return Some(self.error_response(
                Value::Null,
                error_codes::INVALID_REQUEST,
                "JSON-RPC request must be a JSON object",
            ));
        };

        self.handle_single_request_object(obj, start_time).await
    }

    async fn handle_single_request_object(
        &self,
        obj: &serde_json::Map<String, Value>,
        start_time: Instant,
    ) -> Option<JsonRpcResponse> {
        let is_notification = !obj.contains_key("id");

        if obj.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
            if is_notification {
                return None;
            }
            let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
            return Some(self.error_response(
                req_id,
                error_codes::INVALID_REQUEST,
                "Invalid JSON-RPC version (must be 2.0)",
            ));
        }

        let method_str: &str = match obj.get("method") {
            None => {
                if is_notification {
                    return None;
                }
                let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
                return Some(self.error_response(
                    req_id,
                    error_codes::INVALID_REQUEST,
                    "Missing method",
                ));
            }
            Some(Value::String(s)) if !s.is_empty() => s.as_str(),
            Some(Value::String(_)) => {
                if is_notification {
                    return None;
                }
                let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
                return Some(self.error_response(
                    req_id,
                    error_codes::INVALID_REQUEST,
                    "Empty method name",
                ));
            }
            _ => {
                if is_notification {
                    return None;
                }
                let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
                return Some(self.error_response(
                    req_id,
                    error_codes::INVALID_REQUEST,
                    "Invalid method (must be a non-empty string)",
                ));
            }
        };

        if let Some(p) = obj.get("params")
            && !p.is_object()
            && !p.is_array()
        {
            if is_notification {
                return None;
            }
            let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
            return Some(self.error_response(
                req_id,
                error_codes::INVALID_PARAMS,
                "params must be a structured value (object or array)",
            ));
        }

        let params = obj.get("params").cloned();

        if is_notification {
            let _ = self.dispatch_jsonrpc_method(method_str, params).await;
            return None;
        }

        let request_id = obj.get("id").cloned().unwrap_or(Value::Null);

        let span = tracing::info_span!("jsonrpc_method", method = method_str, id = ?request_id);
        let _enter = span.enter();

        let result = self.dispatch_jsonrpc_method(method_str, params).await;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        metrics.total_response_time_ms += elapsed_ms;

        Some(match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: Arc::from("2.0"),
                result: Some(value),
                error: None,
                id: request_id,
            },
            Err(error) => {
                metrics.errors += 1;
                JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(error),
                    id: request_id,
                }
            }
        })
    }

    // Handler methods are in jsonrpc_handlers.rs (organized by domain)

    /// Hidden API for `crates/main/tests/*` JSON-RPC wire tests (library is built without `cfg(test)` for integration tests).
    #[doc(hidden)]
    pub async fn test_handle_jsonrpc_line(&self, line: &str) -> Option<String> {
        self.handle_request_or_batch(line).await
    }
}

#[cfg(test)]
#[path = "jsonrpc_server_unit_tests.rs"]
mod unit_tests;
