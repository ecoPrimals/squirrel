// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 Server with Universal Transport (Isomorphic IPC)
#![allow(dead_code)] // JSON-RPC request/response fields used in deserialization
//!
//! Modern, idiomatic Rust implementation of JSON-RPC 2.0 protocol for
//! biomeOS integration. This server uses Universal Transport abstractions
//! for automatic platform adaptation (Unix sockets OR TCP with discovery files).
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
//! - `system.health` - Health check endpoint
//! - `system.metrics` - Server metrics
//! - `system.ping` - Connectivity test
//! - `discovery.peers` - Peer discovery
//! - `tool.execute` - Tool execution (local + forwarding to announced primals)
//! - `tool.list` - List available tools (local + remote announced)
//!
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

// Serde helpers for Arc<str> (zero-copy for hot-path jsonrpc/method fields)
fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_patterns::transport::{UniversalListener, UniversalTransport};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0") — `Arc<str>` for zero-copy (always "2.0")
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub jsonrpc: Arc<str>,

    /// Method name — `Arc<str>` for zero-copy (method names reused constantly)
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub method: Arc<str>,

    /// Parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// Request ID (optional for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version — `Arc<str>` for zero-copy (always "2.0")
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub jsonrpc: Arc<str>,

    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request ID (echoed from request)
    pub id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC error codes (standard)
pub mod error_codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist or is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Server metrics
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    /// Total requests handled
    pub requests_handled: u64,

    /// Total errors
    pub errors: u64,

    /// Server start time
    pub start_time: Instant,

    /// Total response time (for averaging)
    pub total_response_time_ms: u64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    /// Creates a new server metrics instance with default values.
    pub fn new() -> Self {
        Self {
            requests_handled: 0,
            errors: 0,
            start_time: Instant::now(),
            total_response_time_ms: 0,
        }
    }

    /// Returns the server uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Returns the average response time in milliseconds, if any requests were handled.
    pub fn avg_response_time_ms(&self) -> Option<f64> {
        if self.requests_handled > 0 {
            Some(self.total_response_time_ms as f64 / self.requests_handled as f64)
        } else {
            None
        }
    }
}

/// JSON-RPC Server with Universal Transport (Isomorphic IPC)
pub struct JsonRpcServer {
    /// Service name for Universal Transport discovery
    pub(crate) service_name: String,

    /// Legacy socket path (kept for backward compatibility, used as fallback)
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
}

impl JsonRpcServer {
    /// Load the capability registry from the workspace root or use compiled defaults
    fn load_registry() -> Arc<crate::capabilities::registry::CapabilityRegistry> {
        let candidates = [
            std::path::PathBuf::from("capability_registry.toml"),
            std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../capability_registry.toml"
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
    pub fn new(socket_path: String) -> Self {
        Self {
            service_name: "squirrel".to_string(),
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: None,
            announced_tools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            capability_registry: Self::load_registry(),
        }
    }

    /// Create server with AI router
    pub fn with_ai_router(socket_path: String, ai_router: Arc<crate::api::ai::AiRouter>) -> Self {
        Self {
            service_name: "squirrel".to_string(),
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: Some(ai_router),
            announced_tools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            capability_registry: Self::load_registry(),
        }
    }

    /// Start the JSON-RPC server with Universal Transport (Isomorphic IPC)
    ///
    /// This method uses Universal Transport abstractions for automatic platform adaptation:
    /// - Linux/macOS: Unix sockets (preferred)
    /// - Android (SELinux): TCP fallback with discovery files
    /// - Windows: Named pipes (when available)
    ///
    /// The server will automatically:
    /// 1. Try Unix socket first
    /// 2. Detect platform constraints (SELinux, AppArmor, etc.)
    /// 3. Adapt to TCP fallback if needed
    /// 4. Write discovery files for client auto-discovery
    ///
    /// This method will block until the server is stopped (Ctrl+C).
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🔌 Starting JSON-RPC server with Universal Transport...");

        // Bind using Universal Transport (automatic fallback)
        let listener = UniversalListener::bind(&self.service_name, None)
            .await
            .context("Failed to bind Universal Transport listener")?;

        info!("✅ JSON-RPC server ready (service: {})", self.service_name);

        // Accept connections loop
        loop {
            match listener.accept().await {
                Ok((transport, _remote_addr)) => {
                    debug!("📥 New connection accepted");
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.clone().handle_universal_connection(transport).await
                        {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
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

    /// Handle a client connection (LEGACY - kept for backward compatibility)
    ///
    /// Note: New code should use handle_universal_connection() instead.
    /// This method is kept for any legacy direct Unix socket usage.
    #[deprecated(note = "Use handle_universal_connection() with UniversalTransport instead")]
    async fn handle_connection<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(stream);
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
                            .context("Failed to write JSON-RPC response (legacy)")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush JSON-RPC response (legacy)")?;
                    }
                }
                Err(e) => {
                    warn!("Error reading from socket: {}", e);
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
                let resp = self.handle_single_request(&single).await;
                // Per spec: notifications (no id) produce no response element
                let is_notification = item.get("id").is_none();
                if !is_notification {
                    responses.push(resp);
                }
            }

            if responses.is_empty() {
                // All were notifications — per spec, no response at all
                return None;
            }
            return serde_json::to_string(&responses).ok();
        }

        // Single request
        let resp = self.handle_single_request(trimmed).await;
        serde_json::to_string(&resp).ok()
    }

    /// Handle a single JSON-RPC request (non-batch).
    async fn handle_single_request(&self, request_str: &str) -> JsonRpcResponse {
        let start_time = Instant::now();

        let mut request: JsonRpcRequest = match serde_json::from_str(request_str.trim()) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::PARSE_ERROR,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                    id: Value::Null,
                };
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc.as_ref() != "2.0" {
            return self.error_response(
                request.id.unwrap_or(Value::Null),
                error_codes::INVALID_REQUEST,
                "Invalid JSON-RPC version (must be 2.0)",
            );
        }

        let request_id = request.id.take().unwrap_or(Value::Null);

        // Dispatch to method handler with tracing span

        let span =
            tracing::info_span!("jsonrpc_method", method = %request.method, id = ?request.id);
        let _enter = span.enter();

        // Method dispatch with semantic naming (wateringHole standard: {domain}.{operation})
        // Semantic names are preferred; legacy aliases emit deprecation warnings (Phase 2)
        let result = match request.method.as_ref() {
            // AI domain — semantic names (preferred)
            "ai.query" | "ai.complete" | "ai.chat" => self.handle_query_ai(request.params).await,
            "ai.list_providers" => self.handle_list_providers(request.params).await,

            // Capability domain — semantic names (preferred)
            "capability.announce" => self.handle_announce_capabilities(request.params).await,
            "capability.discover" => self.handle_discover_capabilities().await,
            "capability.list" => self.handle_capability_list().await,

            // System domain — semantic names (preferred)
            "system.health" | "system.status" => self.handle_health().await,
            "system.metrics" => self.handle_metrics().await,
            "system.ping" => self.handle_ping().await,

            // Health domain — PRIMAL_IPC_PROTOCOL v3.0
            "health.liveness" => self.handle_health_liveness().await,
            "health.readiness" => self.handle_health_readiness().await,

            // Discovery domain — semantic names (preferred)
            "discovery.peers" => self.handle_discover_peers(request.params).await,

            // Tool domain — semantic names (preferred)
            "tool.execute" => self.handle_execute_tool(request.params).await,
            "tool.list" => self.handle_list_tools().await,

            // Context domain — semantic names (preferred)
            "context.create" => self.handle_context_create(request.params).await,
            "context.update" => self.handle_context_update(request.params).await,
            "context.summarize" => self.handle_context_summarize(request.params).await,

            // Lifecycle domain — biomeOS registration
            "lifecycle.register" => self.handle_lifecycle_register().await,
            "lifecycle.status" => self.handle_lifecycle_status().await,

            // Method not found
            _ => Err(self.method_not_found(request.method.as_ref())),
        };

        // Update metrics
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        metrics.total_response_time_ms += elapsed_ms;

        match result {
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
        }
    }

    // Handler methods are in jsonrpc_handlers.rs (organized by domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: Arc::from("2.0"),
            method: Arc::from("ai.query"),
            params: Some(json!({"prompt": "Hello"})),
            id: Some(json!(1)),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: Arc::from("2.0"),
            result: Some(json!({"status": "ok"})),
            error: None,
            id: json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: Arc::from("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: None,
            }),
            id: json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.result.is_none());
        assert!(deserialized.error.is_some());
        assert_eq!(
            deserialized.error.unwrap().code,
            error_codes::METHOD_NOT_FOUND
        );
    }

    #[test]
    fn test_metrics_uptime() {
        let metrics = ServerMetrics::new();
        // uptime_seconds() returns u64, always >= 0
        let _ = metrics.uptime_seconds();
    }

    #[test]
    fn test_metrics_avg_response_time() {
        let mut metrics = ServerMetrics::new();
        assert!(metrics.avg_response_time_ms().is_none());

        metrics.requests_handled = 10;
        metrics.total_response_time_ms = 1000;
        assert_eq!(metrics.avg_response_time_ms(), Some(100.0));
    }
}
