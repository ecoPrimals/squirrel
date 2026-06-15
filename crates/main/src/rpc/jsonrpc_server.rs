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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_patterns::transport::{UniversalListener, UniversalTransport};

pub(crate) use super::jsonrpc_types::normalize_method;
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
        }
    }

    /// Enable a localhost TCP JSON-RPC listener on `127.0.0.1:<port>` alongside Universal Transport.
    #[must_use]
    pub const fn with_tcp_port(mut self, port: u16) -> Self {
        self.tcp_port = Some(port);
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

    /// Handle a single UDS connection with riboCipher + BTSP auto-detect.
    ///
    /// **riboCipher (Wave 113):** Reads the first byte. If `0xEC` (clear signal),
    /// reads the second byte (protocol type), strips the 2-byte prefix, and
    /// routes to the appropriate handler. For `0x01` (NDJSON JSON-RPC), proceeds
    /// directly to JSON-RPC handling, bypassing BTSP.
    ///
    /// If the first byte is NOT a riboCipher signal, it is passed through to
    /// `maybe_handshake` which already handles `{` (JSON-RPC) and `0x00` (BTSP).
    ///
    /// If `maybe_handshake` detects plain JSON-RPC (not BTSP), the consumed first
    /// line is re-injected so health probes receive proper responses.
    async fn handle_uds_connection(
        server: Arc<Self>,
        mut transport: UniversalTransport,
    ) -> Result<()> {
        use super::ribocipher_prefix::{
            BTSP_BINARY, BTSP_JSON_LINE, CLEAR_SIGNAL, MITO_SIGNAL, NDJSON_JSONRPC, NUCLEAR_SIGNAL,
        };
        use tokio::io::AsyncReadExt;

        // Read first byte to check for riboCipher signal BEFORE BTSP.
        let mut first = [0u8; 1];
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            transport.read(&mut first),
        )
        .await
        .unwrap_or(Ok(0))
        .unwrap_or(0);

        if n == 0 {
            debug!("Client disconnected before sending data (UDS)");
            return Ok(());
        }

        match first[0] {
            CLEAR_SIGNAL => {
                let mut proto = [0u8; 1];
                transport
                    .read_exact(&mut proto)
                    .await
                    .context("failed to read riboCipher protocol type byte")?;
                let protocol_type = proto[0];

                match protocol_type {
                    NDJSON_JSONRPC => {
                        debug!("riboCipher clear signal: NDJSON JSON-RPC (0x01)");
                        server.handle_universal_connection(transport).await
                    }
                    BTSP_BINARY | BTSP_JSON_LINE => {
                        debug!(
                            protocol_type,
                            "riboCipher clear signal: BTSP — proceeding to handshake"
                        );
                        Self::run_btsp_then_jsonrpc(server, transport).await
                    }
                    other => {
                        warn!(
                            protocol_type = other,
                            "riboCipher clear signal: unsupported protocol type — closing"
                        );
                        Ok(())
                    }
                }
            }
            MITO_SIGNAL | NUCLEAR_SIGNAL => {
                warn!(
                    signal_byte = format_args!("0x{:02X}", first[0]),
                    "riboCipher tier 2/3 not yet implemented — closing"
                );
                Ok(())
            }
            // Not a riboCipher signal — pass to BTSP auto-detect with the
            // consumed byte re-injected as if maybe_handshake read it.
            first_byte => Self::run_btsp_with_first_byte(server, transport, first_byte).await,
        }
    }

    /// Run BTSP handshake then fall through to JSON-RPC.
    async fn run_btsp_then_jsonrpc(
        server: Arc<Self>,
        mut transport: UniversalTransport,
    ) -> Result<()> {
        let first_line = match super::btsp_handshake::maybe_handshake(&mut transport).await {
            Ok(session) => {
                if let Some(ref s) = session {
                    debug!(session_id = %s.session_id, "BTSP authenticated");
                }
                None
            }
            Err(super::btsp_handshake::BtspError::PlainJsonRpc { first_line }) => {
                debug!("PG-14: plain JSON-RPC on BTSP socket — proceeding unauthenticated");
                Some(first_line)
            }
            Err(e) => {
                warn!("BTSP handshake failed, refusing connection: {e}");
                return Ok(());
            }
        };

        if let Some(line) = first_line {
            let reader = tokio::io::BufReader::new(transport);
            server.handle_jsonrpc_with_first_line(reader, line).await
        } else {
            server.handle_universal_connection(transport).await
        }
    }

    /// Handle a connection where we already consumed the first byte (no riboCipher
    /// signal detected). Re-inject the byte into the BTSP/JSON-RPC classification.
    async fn run_btsp_with_first_byte(
        server: Arc<Self>,
        transport: UniversalTransport,
        first_byte: u8,
    ) -> Result<()> {
        use super::btsp_handshake;

        if btsp_handshake::is_btsp_required() {
            // Prod mode: we consumed the byte that maybe_handshake expects.
            // Reconstruct classification inline (mirrors maybe_handshake logic).
            match first_byte {
                b'{' => {
                    let mut reader = BufReader::new(transport);
                    let mut rest = String::new();
                    reader.read_line(&mut rest).await.ok();
                    let first_line = format!("{}{rest}", '{');
                    let trimmed = first_line.trim();

                    if trimmed.contains("\"protocol\"") && trimmed.contains("\"btsp\"") {
                        warn!("raw BTSP JSON-line after no riboCipher signal — PG-14 fallback");
                    } else {
                        debug!("PG-14: plain JSON-RPC on BTSP socket (first byte re-injected)");
                    }
                    server
                        .handle_jsonrpc_with_first_line(reader, first_line)
                        .await
                }
                0x00 => {
                    let mut transport = transport;
                    super::btsp_handshake::btsp_handshake_server(&mut transport, Some(first_byte))
                        .await
                        .map(|_session| ())?;
                    server.handle_universal_connection(transport).await
                }
                other => {
                    debug!(
                        first_byte = format_args!("0x{other:02x}"),
                        "non-BTSP binary preamble on BTSP-guarded socket — closing"
                    );
                    Ok(())
                }
            }
        } else {
            // Dev mode: no BTSP. Re-inject the byte as the start of a JSON-RPC
            // line and proceed to universal connection handling.
            let first_char = first_byte as char;
            if first_char == '{' || first_char == '[' || first_char.is_ascii_whitespace() {
                let mut reader = BufReader::new(transport);
                let mut line = String::new();
                line.push(first_char);
                reader
                    .read_line(&mut line)
                    .await
                    .context("failed to read rest of first line after re-injected byte")?;
                let trimmed = line.trim();
                if trimmed.starts_with("PROTOCOLS:") {
                    #[cfg(feature = "tarpc-rpc")]
                    {
                        return server.handle_protocol_negotiation(reader, &line).await;
                    }
                    #[cfg(not(feature = "tarpc-rpc"))]
                    {
                        return server.handle_jsonrpc_loop(reader).await;
                    }
                }
                server.handle_jsonrpc_with_first_line(reader, line).await
            } else {
                debug!(
                    first_byte = format_args!("0x{first_byte:02x}"),
                    "non-JSON first byte in dev mode — closing"
                );
                Ok(())
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

    /// Handle a client connection via Universal Transport with protocol negotiation
    ///
    /// This method works with ANY transport type (Unix socket, TCP, Named pipe)
    /// using polymorphic AsyncRead + AsyncWrite traits.
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

    /// Dispatch a validated JSON-RPC method name (after `normalize_method`).
    pub(crate) async fn dispatch_jsonrpc_method(
        &self,
        original_method: &str,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let method = normalize_method(original_method);
        match method {
            // AI domain — semantic names (preferred)
            "ai.query" | "ai.complete" | "ai.chat" | "signal.plan" => {
                self.handle_query_ai(params).await
            }
            "ai.list_providers" => self.handle_list_providers(params).await,

            // Inference domain — vendor-agnostic wire standard
            // (ecoPrimal inference provider abstraction)
            "inference.complete" => self.handle_inference_complete(params).await,
            "inference.embed" => self.handle_inference_embed(params).await,
            "inference.models" => self.handle_inference_models(params).await,
            "inference.register_provider" => self.handle_inference_register_provider(params).await,
            "inference.unregister_provider" => {
                self.handle_inference_unregister_provider(params).await
            }

            // Capabilities domain — SEMANTIC_METHOD_NAMING_STANDARD v2.1
            // `capabilities.list` canonical; aliases per standard + ecosystem compat.
            "capabilities.announce" | "capability.announce" | "primal.announce" => {
                self.handle_announce_capabilities(params).await
            }
            "capabilities.discover" | "capability.discover" => {
                self.handle_discover_capabilities().await
            }
            "capabilities.list" | "capability.list" | "primal.capabilities" => {
                self.handle_capability_list().await
            }

            // Identity domain — CAPABILITY_BASED_DISCOVERY_STANDARD v1.0
            "identity.get" => self.handle_identity_get().await,

            // Health domain — PRIMAL_IPC_PROTOCOL v3.0 (canonical)
            // SEMANTIC_METHOD_NAMING_STANDARD: health.* is NON-NEGOTIABLE.
            // system.health / system.status are backward-compat aliases.
            "health.check" | "system.health" | "system.status" => self.handle_health().await,
            "health.liveness" => self.handle_health_liveness().await,
            "health.readiness" => self.handle_health_readiness().await,

            // Bare "health" — Wave 113 mandatory probe method.
            // Returns minimal {status, primal, version} per overwatch contract.
            "health" => self.handle_health_bare().await,

            // System domain — backward-compat (metrics/ping have no health.* equivalent)
            "system.metrics" => self.handle_metrics().await,
            "system.ping" => self.handle_ping().await,

            // Discovery domain — semantic names (preferred)
            "discovery.peers" | "discovery.list" => self.handle_discover_peers(params).await,

            // Tool domain — semantic names (preferred)
            "tool.execute" => self.handle_execute_tool(params).await,
            "tool.list" => self.handle_list_tools().await,

            // Context domain — semantic names (preferred)
            "context.create" => self.handle_context_create(params).await,
            "context.update" => self.handle_context_update(params).await,
            "context.summarize" => self.handle_context_summarize(params).await,

            // Lifecycle domain — biomeOS registration
            "lifecycle.register" => self.handle_lifecycle_register().await,
            "lifecycle.status" => self.handle_lifecycle_status().await,

            // Graph domain — primalSpring BYOB coordination
            "graph.parse" => self.handle_graph_parse(params).await,
            "graph.validate" => self.handle_graph_validate(params).await,

            // Method not found
            _ => Err(self.method_not_found(original_method)),
        }
    }
}

#[cfg(test)]
#[path = "jsonrpc_server_unit_tests.rs"]
mod unit_tests;
