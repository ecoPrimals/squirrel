// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Autonomous IPC Client — Squirrel-owned JSON-RPC 2.0 over Unix sockets
#![allow(dead_code)] // IPC types used for JSON-RPC serialization/deserialization
//!
//! # TRUE PRIMAL Pattern
//!
//! Squirrel owns its own IPC client (primal autonomy). No shared IPC crates.
//! Socket paths are discovered at runtime via XDG-compliant conventions.
//!
//! Squirrel knows:
//! - "I need a capability" (e.g., `secure_http`)
//! - "The ecosystem socket is at this path"
//!
//! Squirrel does NOT know:
//! - Other primals' existence (Songbird, BearDog, etc.)
//! - HTTP/TLS implementation details
//! - Crypto implementation
//!
//! # Architecture
//!
//! ```text
//! Squirrel ──[JSON-RPC 2.0]──▶ Unix Socket ──▶ Ecosystem Router
//! ```
//!
//! # Zero unsafe code, zero C dependencies

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// IPC client errors — modern idiomatic Rust with `thiserror`
#[derive(Debug, Error)]
pub enum IpcClientError {
    /// Failed to connect to ecosystem socket
    #[error("connection failed: {0}")]
    Connection(String),

    /// JSON-RPC error from server
    #[error("JSON-RPC error {code}: {message}")]
    Rpc {
        /// JSON-RPC standard error code
        code: i32,
        /// Human-readable error message
        message: String,
    },

    /// Request timeout
    #[error("request timed out after {0:?}")]
    Timeout(Duration),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Socket not found at expected path
    #[error("ecosystem socket not found: {0}")]
    NotFound(PathBuf),
}

/// Standard JSON-RPC 2.0 error codes
impl IpcClientError {
    /// JSON could not be parsed (-32700)
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object (-32600)
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist or is not available (-32601)
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s) (-32602)
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error (-32603)
    pub const INTERNAL_ERROR: i32 = -32603;
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// HTTP response from proxied request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: String,
}

/// Information about a discovered capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Capability name
    pub capability: String,
    /// Atomic type (Tower, Nest, Node)
    pub atomic_type: Option<String>,
    /// Primals providing this capability
    pub providers: Vec<ProviderInfo>,
    /// Primary socket to route to
    pub primary_socket: PathBuf,
}

/// Information about a capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider identifier (runtime-discovered)
    pub id: String,
    /// Socket path
    pub socket: PathBuf,
    /// Health status
    pub healthy: bool,
    /// Capabilities this provider offers
    pub capabilities: Vec<String>,
}

/// Routing metrics for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Total number of requests routed
    pub total_requests: usize,
    /// Individual metrics
    pub entries: Vec<RoutingMetric>,
}

/// Individual routing metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetric {
    /// Request ID
    pub request_id: String,
    /// Capability requested
    pub capability: String,
    /// Method called
    pub method: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Autonomous IPC Client — JSON-RPC 2.0 over Unix sockets
///
/// # TRUE PRIMAL Pattern
///
/// This client enables squirrel to communicate with the ecosystem
/// without knowing about other primals' existence. All discovery
/// happens at runtime via capability-based routing.
///
/// # Zero unsafe code
pub struct IpcClient {
    /// Path to ecosystem Unix socket
    socket_path: PathBuf,
    /// Request timeout
    request_timeout: Duration,
    /// Connection timeout
    connection_timeout: Duration,
    /// Monotonic request ID counter
    next_id: AtomicU64,
}

impl IpcClient {
    /// Create client with explicit socket path
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
            next_id: AtomicU64::new(1),
        }
    }

    /// Discover ecosystem socket by family ID at runtime
    ///
    /// # XDG-compliant discovery
    ///
    /// 1. `$XDG_RUNTIME_DIR/biomeos/{service_id}.sock`
    /// 2. `/tmp/biomeos-$USER/{service_id}.sock`  (fallback)
    pub fn discover(service_id: &str) -> Result<Self> {
        let socket_path = Self::discover_socket(service_id);

        if !socket_path.exists() {
            return Err(IpcClientError::NotFound(socket_path).into());
        }

        Ok(Self::new(socket_path))
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, dur: Duration) -> Self {
        self.request_timeout = dur;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, dur: Duration) -> Self {
        self.connection_timeout = dur;
        self
    }

    // -----------------------------------------------------------------------
    // High-level API
    // -----------------------------------------------------------------------

    /// Proxy HTTP request through the ecosystem (capability-based)
    ///
    /// Squirrel asks for "http proxy capability" — the ecosystem routes it.
    /// No reqwest, no ring, no C dependencies.
    pub async fn proxy_http(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Value>,
    ) -> Result<HttpResponse> {
        let params = serde_json::json!({
            "method": method,
            "url": url,
            "headers": headers.unwrap_or_default(),
            "body": body
        });

        let result = self.call("neural_api.proxy_http", &params).await?;
        serde_json::from_value(result).context("failed to parse HTTP response")
    }

    /// Discover capability providers at runtime
    pub async fn discover_capability(&self, capability: &str) -> Result<CapabilityInfo> {
        let params = serde_json::json!({ "capability": capability });
        let result = self.call("neural_api.discover_capability", &params).await?;
        serde_json::from_value(result).context("failed to parse capability info")
    }

    /// Route JSON-RPC request to a primal by capability (not by name)
    pub async fn route_by_capability(
        &self,
        capability: &str,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        let request_params = serde_json::json!({
            "capability": capability,
            "method": method,
            "params": params
        });
        self.call("neural_api.route_to_primal", &request_params)
            .await
    }

    /// Get routing metrics (observability)
    pub async fn get_metrics(&self) -> Result<RoutingMetrics> {
        let result = self
            .call("neural_api.get_routing_metrics", &Value::Null)
            .await?;
        serde_json::from_value(result).context("failed to parse routing metrics")
    }

    // -----------------------------------------------------------------------
    // Core JSON-RPC 2.0
    // -----------------------------------------------------------------------

    /// Make a raw JSON-RPC 2.0 call
    pub async fn call(&self, method: &str, params: &Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let mut stream = timeout(
            self.connection_timeout,
            UnixStream::connect(&self.socket_path),
        )
        .await
        .map_err(|_| IpcClientError::Timeout(self.connection_timeout))?
        .with_context(|| {
            format!(
                "failed to connect to ecosystem at {}",
                self.socket_path.display()
            )
        })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Shutdown write side to signal end-of-request
        stream.shutdown().await?;

        let mut response_bytes = Vec::with_capacity(4096);
        timeout(
            self.request_timeout,
            stream.read_to_end(&mut response_bytes),
        )
        .await
        .map_err(|_| IpcClientError::Timeout(self.request_timeout))?
        .context("failed to read response")?;

        let response: Value =
            serde_json::from_slice(&response_bytes).context("failed to parse JSON-RPC response")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            let code = error
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or(IpcClientError::INTERNAL_ERROR as i64) as i32;
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            return Err(IpcClientError::Rpc { code, message }.into());
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("response missing 'result' field"))
    }

    // -----------------------------------------------------------------------
    // Socket discovery
    // -----------------------------------------------------------------------

    /// XDG-compliant socket path discovery
    fn discover_socket(service_id: &str) -> PathBuf {
        let sock_name = format!("{service_id}.sock");

        // Try XDG_RUNTIME_DIR first
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let path = Path::new(&xdg_runtime).join("biomeos").join(&sock_name);
            if path.exists() {
                return path;
            }
        }

        // Fallback: /tmp/biomeos/{service_id}.sock (ecosystem convention)
        PathBuf::from(universal_constants::network::BIOMEOS_SOCKET_FALLBACK_DIR).join(sock_name)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_construction_does_not_panic() {
        let client = IpcClient::new("/tmp/test-squirrel.sock");
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test-squirrel.sock"));
        assert_eq!(client.request_timeout, Duration::from_secs(30));
        assert_eq!(client.connection_timeout, Duration::from_secs(5));
    }

    #[test]
    fn client_timeout_configuration() {
        let client = IpcClient::new("/tmp/test.sock")
            .with_request_timeout(Duration::from_secs(60))
            .with_connection_timeout(Duration::from_secs(10));

        assert_eq!(client.request_timeout, Duration::from_secs(60));
        assert_eq!(client.connection_timeout, Duration::from_secs(10));
    }

    #[test]
    fn socket_discovery_returns_xdg_path() {
        let path = IpcClient::discover_socket("test-service");
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("biomeos") && path_str.ends_with("test-service.sock"),
            "expected XDG-compliant path, got: {path_str}"
        );
    }

    #[test]
    fn request_id_increments() {
        let client = IpcClient::new("/tmp/test.sock");
        let id1 = client.next_id.fetch_add(1, Ordering::Relaxed);
        let id2 = client.next_id.fetch_add(1, Ordering::Relaxed);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn discover_fails_for_nonexistent_socket() {
        let result = IpcClient::discover("nonexistent-service-xyz");
        assert!(result.is_err());
    }

    #[test]
    fn error_display_formatting() {
        let err = IpcClientError::Rpc {
            code: -32601,
            message: "method not found".to_string(),
        };
        assert_eq!(err.to_string(), "JSON-RPC error -32601: method not found");
    }

    #[test]
    fn error_display_connection() {
        let err = IpcClientError::Connection("refused".to_string());
        assert_eq!(err.to_string(), "connection failed: refused");
    }

    #[test]
    fn error_display_timeout() {
        let err = IpcClientError::Timeout(Duration::from_secs(5));
        assert_eq!(err.to_string(), "request timed out after 5s");
    }

    #[test]
    fn error_display_not_found() {
        let err = IpcClientError::NotFound(PathBuf::from("/tmp/missing.sock"));
        assert_eq!(
            err.to_string(),
            "ecosystem socket not found: /tmp/missing.sock"
        );
    }

    #[test]
    fn error_constants() {
        assert_eq!(IpcClientError::PARSE_ERROR, -32700);
        assert_eq!(IpcClientError::INVALID_REQUEST, -32600);
        assert_eq!(IpcClientError::METHOD_NOT_FOUND, -32601);
        assert_eq!(IpcClientError::INVALID_PARAMS, -32602);
        assert_eq!(IpcClientError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn http_response_serde() {
        let resp = HttpResponse {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("content-type".to_string(), "application/json".to_string());
                h
            },
            body: r#"{"ok": true}"#.to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deser: HttpResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.status, 200);
        assert_eq!(
            deser.headers.get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn capability_info_serde() {
        let info = CapabilityInfo {
            capability: "secure_http".to_string(),
            atomic_type: Some("Tower".to_string()),
            providers: vec![ProviderInfo {
                id: "provider-1".to_string(),
                socket: PathBuf::from("/tmp/p1.sock"),
                healthy: true,
                capabilities: vec!["http_proxy".to_string()],
            }],
            primary_socket: PathBuf::from("/tmp/p1.sock"),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deser: CapabilityInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.capability, "secure_http");
        assert_eq!(deser.providers.len(), 1);
        assert!(deser.providers[0].healthy);
    }

    #[test]
    fn routing_metrics_serde() {
        let metrics = RoutingMetrics {
            total_requests: 42,
            entries: vec![RoutingMetric {
                request_id: "req-1".to_string(),
                capability: "ai.query".to_string(),
                method: "POST".to_string(),
                latency_ms: 150,
                success: true,
                error: None,
            }],
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deser: RoutingMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_requests, 42);
        assert_eq!(deser.entries.len(), 1);
        assert!(deser.entries[0].success);
    }

    #[test]
    fn routing_metric_with_error() {
        let metric = RoutingMetric {
            request_id: "req-2".to_string(),
            capability: "storage".to_string(),
            method: "GET".to_string(),
            latency_ms: 5000,
            success: false,
            error: Some("timeout".to_string()),
        };
        let json = serde_json::to_string(&metric).unwrap();
        let deser: RoutingMetric = serde_json::from_str(&json).unwrap();
        assert!(!deser.success);
        assert_eq!(deser.error.as_deref(), Some("timeout"));
    }

    #[test]
    fn client_builder_pattern() {
        let client = IpcClient::new("/tmp/test.sock")
            .with_request_timeout(Duration::from_secs(120))
            .with_connection_timeout(Duration::from_millis(500));

        assert_eq!(client.request_timeout, Duration::from_secs(120));
        assert_eq!(client.connection_timeout, Duration::from_millis(500));
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
    }
}
