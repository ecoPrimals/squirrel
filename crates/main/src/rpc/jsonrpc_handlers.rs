// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 method handlers — utility, AI, Capability, System,
//! Discovery, and Lifecycle domains.
//!
//! Context handlers live in `handlers_context.rs`.
//! Tool handlers live in `handlers_tool.rs`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckResponse,
    ListProvidersResponse, ProviderInfo, QueryAiRequest, QueryAiResponse,
};
use crate::niche;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

impl JsonRpcServer {
    // -----------------------------------------------------------------------
    // Utility methods
    // -----------------------------------------------------------------------

    /// Parse parameters into expected type
    pub(crate) fn parse_params<T: serde::de::DeserializeOwned>(
        &self,
        params: Option<Value>,
    ) -> Result<T, JsonRpcError> {
        match params {
            Some(value) => serde_json::from_value(value).map_err(|e| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Invalid parameters: {e}"),
                data: None,
            }),
            None => Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing parameters".to_string(),
                data: None,
            }),
        }
    }

    /// Create method not found error
    pub(crate) fn method_not_found(&self, method: &str) -> JsonRpcError {
        JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    /// Create error response
    pub(crate) fn error_response(
        &self,
        id: Value,
        code: i32,
        message: &str,
    ) -> super::jsonrpc_server::JsonRpcResponse {
        super::jsonrpc_server::JsonRpcResponse {
            jsonrpc: std::sync::Arc::from("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }

    // -----------------------------------------------------------------------
    // AI domain
    // -----------------------------------------------------------------------

    /// Handle `ai.query` / `query_ai` method
    pub(crate) async fn handle_query_ai(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: QueryAiRequest = self.parse_params(params)?;

        info!("query_ai - prompt length: {}", request.prompt.len());

        if let Some(router) = &self.ai_router {
            use crate::api::ai::types::TextGenerationRequest;

            let ai_request = TextGenerationRequest {
                prompt: request.prompt,
                system: None,
                max_tokens: request.max_tokens.map_or(1024, |v| v as u32),
                temperature: request.temperature.unwrap_or(0.7),
                model: request.model,
                constraints: vec![],
                params: std::collections::HashMap::new(),
            };

            let start = Instant::now();
            match router.generate_text(ai_request, None).await {
                Ok(ai_response) => {
                    let response = QueryAiResponse {
                        response: ai_response.text,
                        provider: ai_response.provider_id.clone(),
                        model: ai_response.model.clone(),
                        tokens_used: ai_response.usage.map(|u| u.total_tokens as usize),
                        latency_ms: start.elapsed().as_millis() as u64,
                        success: true,
                    };
                    serde_json::to_value(response).map_err(|e| JsonRpcError {
                        code: error_codes::INTERNAL_ERROR,
                        message: format!("Serialization error: {e}"),
                        data: None,
                    })
                }
                Err(e) => Err(JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("AI router error: {e}"),
                    data: None,
                }),
            }
        } else {
            Err(JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: "AI router not configured. Configure providers to enable AI inference."
                    .to_string(),
                data: None,
            })
        }
    }

    /// Handle `ai.list_providers` / `list_providers` method
    pub(crate) async fn handle_list_providers(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("list_providers");

        if let Some(router) = &self.ai_router {
            let providers: Vec<ProviderInfo> = router
                .list_providers()
                .await
                .into_iter()
                .map(|p| {
                    let cost_tier = match p.cost_per_unit {
                        Some(cost) if cost > 0.01 => "high",
                        Some(cost) if cost > 0.0 => "medium",
                        _ => "free",
                    }
                    .to_string();

                    ProviderInfo {
                        id: p.provider_id.clone(),
                        name: p.provider_name,
                        models: p.capabilities.clone(),
                        capabilities: p.capabilities,
                        online: p.is_available,
                        avg_latency_ms: if p.avg_latency_ms > 0 {
                            Some(p.avg_latency_ms)
                        } else {
                            None
                        },
                        cost_tier,
                    }
                })
                .collect();

            let response = ListProvidersResponse {
                total: providers.len(),
                providers,
            };

            serde_json::to_value(response).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Serialization error: {e}"),
                data: None,
            })
        } else {
            let response = ListProvidersResponse {
                total: 0,
                providers: vec![],
            };
            serde_json::to_value(response).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Serialization error: {e}"),
                data: None,
            })
        }
    }

    // -----------------------------------------------------------------------
    // Capability domain
    // -----------------------------------------------------------------------

    /// Handle `capability.announce` — register remote tools for routing.
    pub(crate) async fn handle_announce_capabilities(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: AnnounceCapabilitiesRequest = self.parse_params(params)?;

        info!(
            "announce_capabilities - {} capabilities from {:?}",
            request.capabilities.len(),
            request.primal
        );

        let mut tools_registered = 0usize;

        if let (Some(primal), Some(socket_path)) = (&request.primal, &request.socket_path) {
            let tools: Vec<Arc<str>> = match &request.tools {
                Some(t) if !t.is_empty() => t.iter().map(|s| Arc::from(s.as_str())).collect(),
                _ => request
                    .capabilities
                    .iter()
                    .map(|s| Arc::from(s.as_str()))
                    .collect(),
            };
            tools_registered = tools.len();

            let capabilities: Vec<Arc<str>> = request
                .capabilities
                .iter()
                .map(|s| Arc::from(s.as_str()))
                .collect();

            let announced = super::types::AnnouncedPrimal {
                primal: Arc::from(primal.as_str()),
                socket_path: Arc::from(socket_path.as_str()),
                capabilities: capabilities.clone(),
                tools: tools.clone(),
                announced_at: chrono::Utc::now(),
            };

            let mut registry = self.announced_tools.write().await;
            for tool_name in &tools {
                info!(
                    "Registered remote tool '{}' -> {} at {}",
                    tool_name, primal, socket_path
                );
                registry.insert(Arc::clone(tool_name), announced.clone());
            }
        }

        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: format!("Acknowledged {} capabilities", request.capabilities.len()),
            announced_at: chrono::Utc::now().to_rfc3339(),
            tools_registered,
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }

    /// Handle `capability.list` — per-method cost, dependency, and schema info.
    ///
    /// Richer than `capability.discover`: returns structured per-operation
    /// data that biomeOS PathwayLearner and other primals use for scheduling.
    pub(crate) async fn handle_capability_list(&self) -> Result<Value, JsonRpcError> {
        debug!("capability.list request");

        let mut methods = serde_json::Map::new();
        let costs = niche::cost_estimates_json();
        let deps = niche::operation_dependencies();

        let cost_map = costs.as_object();
        let dep_map = deps.as_object();

        for cap in niche::CAPABILITIES {
            let mut entry = serde_json::Map::new();
            if let Some(cm) = cost_map
                && let Some(cost) = cm.get(*cap)
            {
                entry.insert("cost".to_string(), cost.clone());
            }
            if let Some(dm) = dep_map
                && let Some(dep) = dm.get(*cap)
            {
                entry.insert("depends_on".to_string(), dep.clone());
            }
            methods.insert(cap.to_string(), Value::Object(entry));
        }

        Ok(serde_json::json!({
            "primal": niche::PRIMAL_ID,
            "version": niche::PRIMAL_VERSION,
            "domain": niche::DOMAIN,
            "methods": methods,
            "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        }))
    }

    /// Handle `capability.discover` — return capabilities for socket scanning.
    pub(crate) async fn handle_discover_capabilities(&self) -> Result<Value, JsonRpcError> {
        debug!("discover_capabilities request");

        let mut capabilities: Vec<&str> = self.capability_registry.method_names();

        if let Some(router) = &self.ai_router
            && router.provider_count().await > 0
        {
            if !capabilities.contains(&"ai.inference") {
                capabilities.push("ai.inference");
            }
            if !capabilities.contains(&"ai.text_generation") {
                capabilities.push("ai.text_generation");
            }
        }

        let reg = &self.capability_registry.primal;
        let response = serde_json::json!({
            "primal": reg.name,
            "capabilities": capabilities,
            "version": reg.version,
            "metadata": {
                "transport": reg.transport,
                "protocol": reg.protocol,
                "service_name": self.service_name,
                "domain": reg.domain,
            },
            "cost_estimates": niche::cost_estimates_json(),
            "operation_dependencies": niche::operation_dependencies(),
            "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        });

        Ok(response)
    }

    // -----------------------------------------------------------------------
    // System domain
    // -----------------------------------------------------------------------

    /// Handle `system.health` / `health` method
    pub(crate) async fn handle_health(&self) -> Result<Value, JsonRpcError> {
        debug!("health check");

        let metrics = self.metrics.read().await;

        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: metrics.uptime_seconds(),
            active_providers: if let Some(router) = &self.ai_router {
                router.provider_count().await
            } else {
                0
            },
            requests_processed: metrics.requests_handled,
            avg_response_time_ms: metrics.avg_response_time_ms(),
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }

    /// Handle `system.metrics` / `metrics` method
    pub(crate) async fn handle_metrics(&self) -> Result<Value, JsonRpcError> {
        debug!("metrics request");

        let metrics = self.metrics.read().await;

        let response = serde_json::json!({
            "requests_handled": metrics.requests_handled,
            "errors": metrics.errors,
            "uptime_seconds": metrics.uptime_seconds(),
            "avg_response_time_ms": metrics.avg_response_time_ms(),
            "success_rate": if metrics.requests_handled > 0 {
                (metrics.requests_handled - metrics.errors) as f64 / metrics.requests_handled as f64
            } else {
                1.0
            }
        });

        Ok(response)
    }

    /// Handle `system.ping` method
    pub(crate) async fn handle_ping(&self) -> Result<Value, JsonRpcError> {
        debug!("ping");

        Ok(serde_json::json!({
            "pong": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    // -----------------------------------------------------------------------
    // Discovery domain
    // -----------------------------------------------------------------------

    /// Handle `discovery.peers` — scan sockets for running primals.
    pub(crate) async fn handle_discover_peers(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("discover_peers request");

        let peers = match crate::capabilities::discovery::discover_all_capabilities().await {
            Ok(capabilities_map) => {
                let mut seen_sockets = std::collections::HashSet::new();
                let mut peer_list = Vec::new();

                for providers in capabilities_map.values() {
                    for provider in providers {
                        if seen_sockets.insert(provider.socket.clone()) {
                            peer_list.push(serde_json::json!({
                                "id": provider.id,
                                "socket": provider.socket.display().to_string(),
                                "capabilities": provider.capabilities,
                                "discovered_via": provider.discovered_via,
                            }));
                        }
                    }
                }

                peer_list
            }
            Err(e) => {
                warn!("Peer discovery scan failed: {e}");
                Vec::new()
            }
        };

        let response = serde_json::json!({
            "peers": peers,
            "total": peers.len(),
            "discovery_method": "socket_scan"
        });

        Ok(response)
    }

    // -----------------------------------------------------------------------
    // Lifecycle domain (biomeOS)
    // -----------------------------------------------------------------------

    /// Handle `lifecycle.register` — acknowledge registration requests.
    pub(crate) async fn handle_lifecycle_register(&self) -> Result<Value, JsonRpcError> {
        debug!("lifecycle.register request");

        Ok(serde_json::json!({
            "success": true,
            "message": format!("{} registered", self.service_name),
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// Handle `lifecycle.status` — heartbeat status report
    pub(crate) async fn handle_lifecycle_status(&self) -> Result<Value, JsonRpcError> {
        debug!("lifecycle.status request");

        let metrics = self.metrics.read().await;

        Ok(serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": metrics.uptime_seconds(),
            "service": self.service_name,
        }))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_server() -> JsonRpcServer {
        JsonRpcServer::new("/tmp/test.sock".to_string())
    }

    #[derive(Debug, serde::Deserialize)]
    struct TestParams {
        name: String,
        count: u32,
    }

    #[tokio::test]
    async fn test_parse_params_valid() {
        let server = make_server();
        let params = Some(json!({"name": "test", "count": 42}));
        let result: TestParams = server.parse_params(params).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.count, 42);
    }

    #[tokio::test]
    async fn test_parse_params_missing() {
        let server = make_server();
        let result: Result<TestParams, _> = server.parse_params(None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_parse_params_invalid_type() {
        let server = make_server();
        let params = Some(json!({"name": "test", "count": "not-a-number"}));
        let result: Result<TestParams, _> = server.parse_params(params);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_parse_params_wrong_structure() {
        let server = make_server();
        let params = Some(json!({"wrong": "structure"}));
        let result: Result<TestParams, _> = server.parse_params(params);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let server = make_server();
        let err = server.method_not_found("nonexistent.method");
        assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
        assert!(err.message.contains("nonexistent.method"));
    }

    #[tokio::test]
    async fn test_error_response() {
        let server = make_server();
        let response = server.error_response(json!(1), -32000, "Custom error");
        assert_eq!(response.jsonrpc.as_ref(), "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32000);
        assert_eq!(err.message, "Custom error");
        assert_eq!(response.id, json!(1));
    }

    #[tokio::test]
    async fn test_handle_health() {
        let server = make_server();
        let result = server.handle_health().await.unwrap();
        assert!(result.get("status").and_then(|v| v.as_str()) == Some("healthy"));
        assert!(result.get("version").is_some());
        assert!(result.get("uptime_seconds").is_some());
    }

    #[tokio::test]
    async fn test_handle_metrics() {
        let server = make_server();
        let result = server.handle_metrics().await.unwrap();
        assert!(result.get("requests_handled").is_some());
        assert!(result.get("errors").is_some());
        assert!(result.get("uptime_seconds").is_some());
        assert!(result.get("success_rate").is_some());
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let server = make_server();
        let result = server.handle_ping().await.unwrap();
        assert_eq!(result.get("pong").and_then(|v| v.as_bool()), Some(true));
        assert!(result.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_handle_discover_capabilities() {
        let server = make_server();
        let result = server.handle_discover_capabilities().await.unwrap();
        assert_eq!(
            result.get("primal").and_then(|v| v.as_str()),
            Some("squirrel")
        );
        let caps = result
            .get("capabilities")
            .and_then(|v| v.as_array())
            .expect("test: capabilities must be array");
        assert!(caps.iter().any(|c| c.as_str() == Some("ai.query")));
        assert!(caps.iter().any(|c| c.as_str() == Some("system.health")));

        assert!(
            result.get("cost_estimates").is_some(),
            "response must include cost_estimates"
        );
        assert!(
            result.get("operation_dependencies").is_some(),
            "response must include operation_dependencies"
        );
        let consumed = result
            .get("consumed_capabilities")
            .and_then(|v| v.as_array())
            .expect("test: consumed_capabilities must be array");
        assert!(
            consumed
                .iter()
                .any(|c| c.as_str() == Some("discovery.register")),
            "consumed_capabilities must include discovery.register"
        );
    }

    #[tokio::test]
    async fn test_handle_capability_list() {
        let server = make_server();
        let result = server.handle_capability_list().await.unwrap();
        assert_eq!(
            result.get("primal").and_then(|v| v.as_str()),
            Some("squirrel")
        );
        let methods = result
            .get("methods")
            .and_then(|v| v.as_object())
            .expect("test: methods must be object");
        assert!(methods.contains_key("ai.query"));
        assert!(methods.contains_key("capability.list"));

        let ai_query = methods.get("ai.query").unwrap().as_object().unwrap();
        assert!(ai_query.contains_key("cost"));
        assert!(ai_query.contains_key("depends_on"));
    }

    #[tokio::test]
    async fn test_handle_announce_capabilities_valid() {
        let server = make_server();
        let params = Some(json!({"capabilities": ["ai.inference", "tool.execute"]}));
        let result = server.handle_announce_capabilities(params).await.unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert!(
            result
                .get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("2")
        );
    }

    #[tokio::test]
    async fn test_handle_announce_capabilities_missing_params() {
        let server = make_server();
        let result = server.handle_announce_capabilities(None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_list_providers_no_router() {
        let server = make_server();
        let result = server.handle_list_providers(None).await.unwrap();
        assert_eq!(result.get("total").and_then(|v| v.as_u64()), Some(0));
        assert!(
            result
                .get("providers")
                .and_then(|v| v.as_array())
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_handle_query_ai_no_params() {
        let server = make_server();
        let result = server.handle_query_ai(None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_query_ai_no_router() {
        let server = make_server();
        let params = Some(json!({"prompt": "Hello"}));
        let result = server.handle_query_ai(params).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("AI router not configured")
        );
    }

    #[tokio::test]
    async fn test_handle_batch_empty() {
        let server = make_server();
        let result = server.handle_request_or_batch("[]").await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn test_handle_batch_single() {
        let server = make_server();
        let batch = r#"[{"jsonrpc":"2.0","method":"system.ping","id":1}]"#;
        let result = server.handle_request_or_batch(batch).await;
        assert!(result.is_some());
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_batch_multi() {
        let server = make_server();
        let batch = r#"[
            {"jsonrpc":"2.0","method":"system.ping","id":1},
            {"jsonrpc":"2.0","method":"system.health","id":2}
        ]"#;
        let result = server.handle_request_or_batch(batch).await;
        assert!(result.is_some());
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_lifecycle_register() {
        let server = make_server();
        let result = server.handle_lifecycle_register().await.unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert!(
            result
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap()
                .contains("squirrel")
        );
        assert!(result.get("version").is_some());
    }

    #[tokio::test]
    async fn test_handle_lifecycle_status() {
        let server = make_server();
        let result = server.handle_lifecycle_status().await.unwrap();
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("healthy")
        );
        assert!(result.get("version").is_some());
        assert!(result.get("uptime_seconds").is_some());
        assert_eq!(
            result.get("service").and_then(|v| v.as_str()),
            Some("squirrel")
        );
    }

    #[tokio::test]
    async fn test_handle_discover_peers() {
        let server = make_server();
        let result = server.handle_discover_peers(None).await.unwrap();
        assert!(result.get("peers").and_then(|v| v.as_array()).is_some());
        assert!(result.get("total").and_then(|v| v.as_u64()).is_some());
        assert_eq!(
            result.get("discovery_method").and_then(|v| v.as_str()),
            Some("socket_scan")
        );
    }

    #[tokio::test]
    async fn test_handle_single_request_ping() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"system.ping","id":42}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(
            parsed
                .get("result")
                .and_then(|r| r.get("pong"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(parsed.get("id").and_then(|v| v.as_i64()), Some(42));
    }

    #[tokio::test]
    async fn test_handle_parse_error() {
        let server = make_server();
        let result = server.handle_request_or_batch("not valid json {{{").await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("error").is_some());
        assert_eq!(
            parsed
                .get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_i64()),
            Some(error_codes::PARSE_ERROR as i64)
        );
    }

    #[tokio::test]
    async fn test_handle_invalid_jsonrpc_version() {
        let server = make_server();
        let req = r#"{"jsonrpc":"1.0","method":"system.ping","id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("error").is_some());
        assert!(
            parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .contains("2.0")
        );
    }

    #[tokio::test]
    async fn test_handle_announce_with_primal_and_socket() {
        let server = make_server();
        let params = Some(json!({
            "capabilities": ["tool.remote"],
            "primal": "songbird-1",
            "socket_path": "/tmp/songbird.sock",
            "tools": ["tool.remote"]
        }));
        let result = server.handle_announce_capabilities(params).await.unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            result.get("tools_registered").and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[tokio::test]
    async fn test_handle_list_tools() {
        let server = make_server();
        let result = server.handle_list_tools().await.unwrap();
        assert!(result.get("tools").and_then(|v| v.as_array()).is_some());
        assert!(result.get("total").and_then(|v| v.as_u64()).is_some());
    }

    #[tokio::test]
    async fn test_handle_context_create() {
        let server = make_server();
        let params = Some(json!({"session_id": "test-session-123", "metadata": {"key": "value"}}));
        let result = server.handle_context_create(params).await.unwrap();
        assert!(result.get("id").is_some());
        assert!(result.get("version").is_some());
        assert!(result.get("created_at").is_some());
        assert_eq!(
            result
                .get("metadata")
                .and_then(|m| m.get("key"))
                .and_then(|v| v.as_str()),
            Some("value")
        );
    }

    #[tokio::test]
    async fn test_handle_context_create_auto_session_id() {
        let server = make_server();
        let result = server.handle_context_create(None).await.unwrap();
        assert!(
            result
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn test_handle_context_update_missing_params() {
        let server = make_server();
        let result = server.handle_context_update(None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_context_summarize_missing_params() {
        let server = make_server();
        let result = server.handle_context_summarize(None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_execute_tool_missing_params() {
        let server = make_server();
        let result = server.handle_execute_tool(None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_execute_tool_missing_tool_param() {
        let server = make_server();
        let params = Some(json!({"args": {}}));
        let result = server.handle_execute_tool(params).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_request_context_create() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"context.create","params":{"session_id":"req-test"},"id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("result").is_some());
        assert!(parsed.get("result").and_then(|r| r.get("id")).is_some());
    }

    #[tokio::test]
    async fn test_handle_request_tool_list() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"tool.list","id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("result").is_some());
        assert!(
            parsed
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|v| v.as_array())
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_handle_context_update_valid() {
        let server = make_server();
        let create_params =
            Some(json!({"session_id": "update-test-session", "metadata": {"key": "v1"}}));
        let create_result = server.handle_context_create(create_params).await.unwrap();
        let ctx_id = create_result
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let update_params = Some(json!({"id": ctx_id, "data": {"updated": true}}));
        let update_result = server.handle_context_update(update_params).await.unwrap();
        assert_eq!(
            update_result.get("id").and_then(|v| v.as_str()),
            Some(ctx_id.as_str())
        );
        assert!(
            update_result
                .get("version")
                .and_then(|v| v.as_u64())
                .unwrap()
                >= 1
        );
    }

    #[tokio::test]
    async fn test_handle_context_summarize_valid() {
        let server = make_server();
        let create_params = Some(json!({"session_id": "summarize-test-session"}));
        let create_result = server.handle_context_create(create_params).await.unwrap();
        let ctx_id = create_result
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let summarize_params = Some(json!({"id": ctx_id}));
        let summarize_result = server
            .handle_context_summarize(summarize_params)
            .await
            .unwrap();
        assert_eq!(
            summarize_result.get("id").and_then(|v| v.as_str()),
            Some(ctx_id.as_str())
        );
        assert!(
            summarize_result
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap()
                .contains("Context")
        );
    }

    #[tokio::test]
    async fn test_handle_execute_tool_system_health() {
        let server = make_server();
        let params = Some(json!({"tool": "system.health", "args": {}}));
        let result = server.handle_execute_tool(params).await.unwrap();
        assert_eq!(
            result.get("tool").and_then(|v| v.as_str()),
            Some("system.health")
        );
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert!(
            result
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap()
                .contains("healthy")
        );
    }

    #[tokio::test]
    async fn test_handle_execute_tool_unknown_tool() {
        let server = make_server();
        let params = Some(json!({"tool": "nonexistent.tool", "args": {}}));
        let result = server.handle_execute_tool(params).await.unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(false));
        assert!(
            result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap()
                .contains("not found")
        );
    }

    #[tokio::test]
    async fn test_handle_request_legacy_health_alias() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"health","id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("result").is_some());
        assert_eq!(
            parsed
                .get("result")
                .and_then(|r| r.get("status"))
                .and_then(|v| v.as_str()),
            Some("healthy")
        );
    }

    #[tokio::test]
    async fn test_handle_request_legacy_ping_alias() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(
            parsed
                .get("result")
                .and_then(|r| r.get("pong"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_handle_request_method_not_found() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"unknown.method","id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("error").is_some());
        assert_eq!(
            parsed
                .get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_i64()),
            Some(error_codes::METHOD_NOT_FOUND as i64)
        );
    }

    #[tokio::test]
    async fn test_handle_batch_notification_no_response() {
        let server = make_server();
        let batch = r#"[{"jsonrpc":"2.0","method":"system.ping"}]"#;
        let result = server.handle_request_or_batch(batch).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_handle_request_tool_execute() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","method":"tool.execute","params":{"tool":"system.health","args":{}},"id":1}"#;
        let result = server.handle_request_or_batch(req).await;
        assert!(result.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed.get("result").is_some());
        assert_eq!(
            parsed
                .get("result")
                .and_then(|r| r.get("success"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
