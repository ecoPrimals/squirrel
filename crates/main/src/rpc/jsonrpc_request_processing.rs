// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 request parsing and dispatch (Section 4 + Section 6 batch).
//!
//! Extracted from [`super::jsonrpc_server`] for module size management.
//! Handles parsing, validation, batch dispatch, notification semantics,
//! and metrics recording.

use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

use super::jsonrpc_server::{JsonRpcError, JsonRpcResponse, JsonRpcServer, error_codes};
use super::method_gate::{CallerContext, MethodGate};

impl JsonRpcServer {
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
                if !is_notification && let Some(r) = resp {
                    responses.push(r);
                }
            }

            if responses.is_empty() {
                return None;
            }
            return serde_json::to_string(&responses).ok();
        }

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

    #[expect(
        clippy::too_many_lines,
        reason = "single dispatch point — splitting would fragment request lifecycle"
    )]
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

        // JH-0/JH-2: pre-dispatch capability gate
        let gate = MethodGate::permissive();
        let caller_ctx = CallerContext::anonymous();
        if let Err(gate_err) = gate.check_with_context(method_str, &caller_ctx) {
            if is_notification {
                return None;
            }
            let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
            return Some(JsonRpcResponse {
                jsonrpc: Arc::from("2.0"),
                result: None,
                error: Some(gate_err),
                id: req_id,
            });
        }

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

    #[doc(hidden)]
    pub async fn test_handle_jsonrpc_line(&self, line: &str) -> Option<String> {
        self.handle_request_or_batch(line).await
    }
}
