// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 request parsing and dispatch (Section 4 + Section 6 batch).
//!
//! Extracted from [`super::jsonrpc_server`] for module size management.
//! Handles parsing, validation, batch dispatch, notification semantics,
//! and metrics recording.

use serde_json::Value;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use super::jsonrpc_server::{JsonRpcError, JsonRpcResponse, JsonRpcServer, error_codes};
use super::method_gate::{CallerContext, MethodGate};
use crate::security::rate_limiter::types::EndpointType;

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

        // Pre-dispatch security: rate limiting + input validation
        if let Some(orchestrator) = &self.security_orchestrator {
            let check = crate::security::orchestrator::SecurityCheckRequest {
                client_ip: IpAddr::from([127, 0, 0, 1]),
                user_agent: None,
                endpoint: method_str.to_string(),
                endpoint_type: Self::endpoint_type_for_method(method_str),
                input_data: Self::extract_input_data(params.as_ref()),
                user_id: None,
                session_id: None,
                policy_name: None,
                correlation_id: crate::observability::CorrelationId::new(),
                metadata: std::collections::HashMap::new(),
            };
            let result = orchestrator.check_security(check).await;
            if !result.allowed {
                if is_notification {
                    return None;
                }
                let req_id = obj.get("id").cloned().unwrap_or(Value::Null);
                let reason = result
                    .denial_reason
                    .unwrap_or_else(|| "security check failed".to_string());
                return Some(JsonRpcResponse {
                    jsonrpc: Arc::from("2.0"),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::SECURITY_DENIED,
                        message: reason,
                        data: Some(serde_json::json!({
                            "risk_level": format!("{:?}", result.risk_level),
                        })),
                    }),
                    id: req_id,
                });
            }
        }

        if is_notification {
            let _ = self.dispatch_jsonrpc_method(method_str, params).await;
            return None;
        }

        let request_id = obj.get("id").cloned().unwrap_or(Value::Null);

        let span = tracing::info_span!("jsonrpc_method", method = method_str, id = ?request_id);
        let _enter = span.enter();

        let result = self.dispatch_jsonrpc_method(method_str, params).await;

        let elapsed = start_time.elapsed();
        let is_error = result.is_err();

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "RPC response times won't exceed u64 milliseconds"
        )]
        {
            metrics.total_response_time_ms += elapsed.as_millis() as u64;
        }

        self.request_tracker.record_request(elapsed, is_error);

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

    /// Map JSON-RPC method prefix to `EndpointType` for rate-limit tiering.
    pub(crate) fn endpoint_type_for_method(method: &str) -> EndpointType {
        match method.split('.').next().unwrap_or(method) {
            "health" | "ping" => EndpointType::HealthCheck,
            "ai" | "inference" => EndpointType::Compute,
            "auth" | "btsp" => EndpointType::Authentication,
            "admin" | "deploy" => EndpointType::Admin,
            _ => EndpointType::Api,
        }
    }

    /// Extract text inputs from params for security validation.
    pub(crate) fn extract_input_data(
        params: Option<&Value>,
    ) -> Option<Vec<(String, String, crate::security::input_validator::InputType)>> {
        use crate::security::input_validator::InputType;

        let obj = params?.as_object()?;
        let mut inputs = Vec::new();

        for (key, val) in obj {
            if let Some(s) = val.as_str() {
                let input_type = match key.as_str() {
                    "prompt" | "text" | "message" => InputType::Text,
                    "url" | "endpoint" => InputType::Url,
                    "path" | "file" | "socket_path" => InputType::FilePath,
                    _ => continue,
                };
                inputs.push((key.clone(), s.to_string(), input_type));
            }
        }

        if inputs.is_empty() {
            None
        } else {
            Some(inputs)
        }
    }

    #[doc(hidden)]
    pub async fn test_handle_jsonrpc_line(&self, line: &str) -> Option<String> {
        self.handle_request_or_batch(line).await
    }
}
