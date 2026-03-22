// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::time::Duration;

/// Phase of the IPC call where an error occurred.
///
/// Absorbed from rhizoCrypt v0.13 — enables targeted retry logic:
/// retry on `Connect`/`Write`, do NOT retry on `JsonRpcError` with `-32601`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IpcErrorPhase {
    /// Socket connection failed (retryable)
    Connect,
    /// Writing request to socket failed (retryable)
    Write,
    /// Reading response from socket failed (retryable)
    Read,
    /// Response was not valid JSON
    InvalidJson,
    /// Server returned a JSON-RPC error (check code before retrying)
    JsonRpcError,
    /// No `result` field in a successful-looking response
    NoResult,
}

/// IPC client errors — modern idiomatic Rust with `thiserror`
///
/// Each variant carries an [`IpcErrorPhase`] for retry-aware error handling.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IpcClientError {
    /// Failed to connect to ecosystem socket
    #[error("[{phase:?}] connection failed: {message}")]
    Connection {
        /// Error phase (always `Connect`)
        phase: IpcErrorPhase,
        /// Human-readable detail
        message: String,
    },

    /// JSON-RPC error from server
    #[error("[{phase:?}] JSON-RPC error {code}: {message}")]
    Rpc {
        /// Error phase (always `JsonRpcError`)
        phase: IpcErrorPhase,
        /// JSON-RPC standard error code
        code: i32,
        /// Human-readable error message
        message: String,
    },

    /// Request timeout
    #[error("[{phase:?}] request timed out after {duration:?}")]
    Timeout {
        /// Which phase timed out
        phase: IpcErrorPhase,
        /// How long we waited
        duration: Duration,
    },

    /// I/O error
    #[error("[{phase:?}] I/O error: {source}")]
    Io {
        /// Which phase the I/O error occurred in
        phase: IpcErrorPhase,
        /// Underlying I/O error
        source: std::io::Error,
    },

    /// Serialization error
    #[error("[InvalidJson] serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Socket not found at expected path
    #[error("[Connect] ecosystem socket not found: {0}")]
    NotFound(PathBuf),
}

impl IpcClientError {
    /// The phase in which this error occurred — use for retry decisions.
    #[must_use]
    pub fn phase(&self) -> IpcErrorPhase {
        match self {
            Self::Connection { phase, .. }
            | Self::Rpc { phase, .. }
            | Self::Timeout { phase, .. }
            | Self::Io { phase, .. } => *phase,
            Self::Serialization(_) => IpcErrorPhase::InvalidJson,
            Self::NotFound(_) => IpcErrorPhase::Connect,
        }
    }

    /// Whether this error is safe to retry without side effects.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.phase(),
            IpcErrorPhase::Connect | IpcErrorPhase::Write | IpcErrorPhase::Read
        )
    }
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

/// Structured JSON-RPC error extracted from a response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    /// JSON-RPC error code (e.g., -32601 for method not found).
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional data.
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    /// Whether this is a "method not found" error.
    #[must_use]
    pub fn is_method_not_found(&self) -> bool {
        self.code == IpcClientError::METHOD_NOT_FOUND
    }

    /// Whether this is an internal error.
    #[must_use]
    pub fn is_internal(&self) -> bool {
        self.code == IpcClientError::INTERNAL_ERROR
    }

    /// Whether this error code is in the reserved JSON-RPC range (-32000 to -32099).
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        (-32099..=-32000).contains(&self.code)
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}
