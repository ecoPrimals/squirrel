// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Autonomous IPC Client — Squirrel-owned JSON-RPC 2.0 over Unix sockets
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

mod connection;
mod discovery;
mod messaging;
mod types;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;

pub use messaging::{extract_rpc_error, extract_rpc_result, parse_capabilities_from_response};
pub use types::{
    CapabilityInfo, HttpResponse, IpcClientError, IpcErrorPhase, ProviderInfo, RoutingMetric,
    RoutingMetrics, RpcError,
};

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
    next_id: std::sync::atomic::AtomicU64,
}

impl IpcClient {
    /// Create client with explicit socket path
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Discover ecosystem socket by family ID at runtime
    ///
    /// # XDG-compliant discovery
    ///
    /// 1. `$XDG_RUNTIME_DIR/biomeos/{service_id}.sock`
    /// 2. `/tmp/biomeos-$USER/{service_id}.sock`  (fallback)
    #[must_use = "discovery may fail; the result should be checked"]
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
    // Socket discovery
    // -----------------------------------------------------------------------

    /// XDG-compliant socket path discovery
    fn discover_socket(service_id: &str) -> PathBuf {
        discovery::discover_socket(service_id)
    }
}
