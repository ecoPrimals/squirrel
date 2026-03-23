// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use super::IpcClient;
use super::connection;
use super::types::{IpcClientError, IpcErrorPhase, RpcError};

impl IpcClient {
    /// Make a raw JSON-RPC 2.0 call
    pub async fn call(&self, method: &str, params: &Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let mut stream =
            connection::connect_unix_stream(&self.socket_path, self.connection_timeout).await?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let request_bytes =
            serde_json::to_vec(&request).context("Failed to serialize JSON-RPC request")?;

        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| IpcClientError::Io {
                phase: IpcErrorPhase::Write,
                source: e,
            })?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| IpcClientError::Io {
                phase: IpcErrorPhase::Write,
                source: e,
            })?;
        stream.flush().await.map_err(|e| IpcClientError::Io {
            phase: IpcErrorPhase::Write,
            source: e,
        })?;
        stream.shutdown().await.map_err(|e| IpcClientError::Io {
            phase: IpcErrorPhase::Write,
            source: e,
        })?;

        let mut response_bytes = Vec::with_capacity(4096);
        timeout(
            self.request_timeout,
            stream.read_to_end(&mut response_bytes),
        )
        .await
        .map_err(|_| IpcClientError::Timeout {
            phase: IpcErrorPhase::Read,
            duration: self.request_timeout,
        })?
        .map_err(|e| IpcClientError::Io {
            phase: IpcErrorPhase::Read,
            source: e,
        })?;

        let response: Value =
            serde_json::from_slice(&response_bytes).context("Failed to parse JSON-RPC response")?;

        if let Some(error) = response.get("error") {
            let code = error
                .get("code")
                .and_then(Value::as_i64)
                .map_or(IpcClientError::INTERNAL_ERROR, |c| {
                    i32::try_from(c).unwrap_or(IpcClientError::INTERNAL_ERROR)
                });
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            return Err(IpcClientError::Rpc {
                phase: IpcErrorPhase::JsonRpcError,
                code,
                message,
            }
            .into());
        }

        response.get("result").cloned().ok_or_else(|| {
            IpcClientError::Rpc {
                phase: IpcErrorPhase::NoResult,
                code: IpcClientError::INTERNAL_ERROR,
                message: "response missing 'result' field".to_string(),
            }
            .into()
        })
    }
}

/// Parse capabilities from a `capability.list` response in any of 4 formats.
///
/// Absorbed from airSpring v0.8.7 — handles all ecosystem response shapes:
///
/// 1. **Flat array** (ecosystem consensus): `{ "capabilities": ["a.b", "c.d"] }`
/// 2. **Legacy object** (method keys): `{ "methods": { "a.b": {...} } }`
/// 3. **Nested** (wrapped in result): `{ "result": { "capabilities": [...] } }`
/// 4. **Double-nested** (JSON-RPC + result wrapper): `{ "result": { "result": { "capabilities": [...] } } }`
pub fn parse_capabilities_from_response(response: &serde_json::Value) -> Vec<String> {
    // Try flat-array format first (most common)
    if let Some(caps) = extract_string_array(response, "capabilities") {
        return caps;
    }

    // Try legacy "methods" keys
    if let Some(methods) = response.get("methods").and_then(|v| v.as_object()) {
        return methods.keys().cloned().collect();
    }

    // Try single-nested: { "result": { "capabilities": [...] } }
    if let Some(inner) = response.get("result") {
        if let Some(caps) = extract_string_array(inner, "capabilities") {
            return caps;
        }
        if let Some(methods) = inner.get("methods").and_then(|v| v.as_object()) {
            return methods.keys().cloned().collect();
        }

        // Try double-nested: { "result": { "result": { "capabilities": [...] } } }
        if let Some(inner2) = inner.get("result") {
            if let Some(caps) = extract_string_array(inner2, "capabilities") {
                return caps;
            }
            if let Some(methods) = inner2.get("methods").and_then(|v| v.as_object()) {
                return methods.keys().cloned().collect();
            }
        }
    }

    Vec::new()
}

fn extract_string_array(value: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    value.get(key).and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
}

/// Extract the `"result"` field from a JSON-RPC success response.
///
/// Returns `Err` if the response contains an `"error"` field (converted to
/// [`RpcError`]) or if the `"result"` field is missing entirely.
pub fn extract_rpc_result(response: &serde_json::Value) -> Result<serde_json::Value, RpcError> {
    if let Some(err) = extract_rpc_error(response) {
        return Err(err);
    }
    response.get("result").cloned().ok_or_else(|| RpcError {
        code: IpcClientError::INTERNAL_ERROR,
        message: "response missing 'result' field".to_string(),
        data: None,
    })
}

/// Extract a structured error from a JSON-RPC error response.
///
/// Absorbed from loamSpine v0.9.3 / petalTongue v1.6.6. Extracts the
/// code, message, and optional data from a JSON-RPC `-32xxx` error.
///
/// Returns `None` if the response is not an error or is malformed.
pub fn extract_rpc_error(response: &serde_json::Value) -> Option<RpcError> {
    let error = response.get("error")?;
    let code = error.get("code").and_then(|v| v.as_i64())? as i32;
    let message = error
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown error")
        .to_string();
    let data = error.get("data").cloned();

    Some(RpcError {
        code,
        message,
        data,
    })
}
