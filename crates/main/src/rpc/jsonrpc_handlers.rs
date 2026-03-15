// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 Method Handlers
//!
//! Individual handler functions for each JSON-RPC method supported by
//! the Squirrel server. Handlers are grouped by domain:
//!
//! - **AI**: `ai.query`, `ai.list_providers`
//! - **Capability**: `capability.announce`, `capability.discover`
//! - **System**: `system.health`, `system.metrics`, `system.ping`
//! - **Discovery**: `discovery.peers`
//! - **Tool**: `tool.execute`

use super::jsonrpc_server::{error_codes, JsonRpcError, JsonRpcServer};
use super::types::*;
use serde_json::Value;
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

        info!("🤖 query_ai - prompt length: {}", request.prompt.len());

        // Delegate to AI router; return error if no router configured
        if let Some(router) = &self.ai_router {
            use crate::api::ai::types::TextGenerationRequest;

            let ai_request = TextGenerationRequest {
                prompt: request.prompt,
                system: None,
                max_tokens: request.max_tokens.map(|v| v as u32).unwrap_or(1024),
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
                        provider: ai_response.provider_id.to_string(),
                        model: ai_response.model.to_string(),
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
        info!("📋 list_providers");

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

    /// Handle `capability.announce` / `announce_capabilities` method
    ///
    /// When `primal` and `socket_path` are provided, the announcement is
    /// stored for routing: subsequent `tool.execute` calls for the announced
    /// tool names are forwarded to the remote primal's socket.
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
            // neuralSpring pattern: use `capabilities` as tool routing fallback
            // when `tools` is empty. This allows primals that only send capabilities
            // (not explicit tool names) to still be routed correctly.
            let tools = match request.tools.clone() {
                Some(t) if !t.is_empty() => t,
                _ => request.capabilities.clone(),
            };
            tools_registered = tools.len();

            let announced = super::types::AnnouncedPrimal {
                primal: primal.clone(),
                socket_path: socket_path.clone(),
                capabilities: request.capabilities.clone(),
                tools: tools.clone(),
                announced_at: chrono::Utc::now(),
            };

            let mut registry = self.announced_tools.write().await;
            for tool_name in &tools {
                info!(
                    "Registered remote tool '{}' -> {} at {}",
                    tool_name, primal, socket_path
                );
                registry.insert(tool_name.clone(), announced.clone());
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

    /// Handle `capability.discover` / `discover_capabilities` method
    ///
    /// Returns Squirrel's capabilities so other primals can discover us
    /// via socket scanning. TRUE PRIMAL: uses capability namespaces,
    /// not primal names. Consumers learn WHAT we do, not WHO we are.
    ///
    /// Source of truth: `capability_registry.toml` loaded at startup.
    pub(crate) async fn handle_discover_capabilities(&self) -> Result<Value, JsonRpcError> {
        debug!("🔍 discover_capabilities request");

        let mut capabilities: Vec<&str> = self.capability_registry.method_names();

        if let Some(router) = &self.ai_router {
            if router.provider_count().await > 0 {
                if !capabilities.contains(&"ai.inference") {
                    capabilities.push("ai.inference");
                }
                if !capabilities.contains(&"ai.text_generation") {
                    capabilities.push("ai.text_generation");
                }
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
            }
        });

        Ok(response)
    }

    // -----------------------------------------------------------------------
    // System domain
    // -----------------------------------------------------------------------

    /// Handle `system.health` / `health` method
    pub(crate) async fn handle_health(&self) -> Result<Value, JsonRpcError> {
        debug!("💚 health check");

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
        debug!("📊 metrics request");

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

    /// Handle `system.ping` method (simple connectivity test)
    pub(crate) async fn handle_ping(&self) -> Result<Value, JsonRpcError> {
        debug!("🏓 ping");

        Ok(serde_json::json!({
            "pong": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    // -----------------------------------------------------------------------
    // Discovery domain
    // -----------------------------------------------------------------------

    /// Handle `discovery.peers` / `discover_peers` method
    ///
    /// Scans socket directories for running primals and probes each to
    /// discover capabilities. TRUE PRIMAL: no hardcoded peer list.
    pub(crate) async fn handle_discover_peers(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("🔍 discover_peers request");

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
    // Context domain
    // -----------------------------------------------------------------------

    /// Handle `context.create` — create a new context session
    pub(crate) async fn handle_context_create(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.create request");

        let session_id = params
            .as_ref()
            .and_then(|p| p.get("session_id"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let metadata = params
            .as_ref()
            .and_then(|p| p.get("metadata"))
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let manager = squirrel_context::ContextManager::new();

        let state = manager.get_context_state(&session_id).await.map_err(|e| {
            JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to create context: {e}"),
                data: None,
            }
        })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "metadata": metadata,
        }))
    }

    /// Handle `context.update` — update an existing context with new data
    pub(crate) async fn handle_context_update(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.update request");

        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'id' parameter".to_string(),
                data: None,
            })?;

        let data = params
            .get("data")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let manager = squirrel_context::ContextManager::new();

        let mut state = manager.get_context_state(id).await.map_err(|e| {
            JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Context not found: {e}"),
                data: None,
            }
        })?;

        state.data = data;
        state.version += 1;
        state.last_modified = std::time::SystemTime::now();

        manager
            .update_context_state(id, state.clone())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to update context: {e}"),
                data: None,
            })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Handle `context.summarize` — summarize a context session
    pub(crate) async fn handle_context_summarize(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.summarize request");

        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'id' parameter".to_string(),
                data: None,
            })?;

        let manager = squirrel_context::ContextManager::new();

        let state = manager.get_context_state(id).await.map_err(|e| {
            JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Context not found: {e}"),
                data: None,
            }
        })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "summary": format!("Context {} (v{}) with {} metadata keys",
                state.id, state.version, state.metadata.len()),
            "data": state.data,
            "synchronized": state.synchronized,
        }))
    }

    // -----------------------------------------------------------------------
    // Tool domain
    // -----------------------------------------------------------------------

    /// Handle `tool.execute` / `execute_tool` method
    ///
    /// Checks the announced-primal registry first. If the tool was registered
    /// by a remote primal via `capability.announce`, the request is forwarded
    /// to that primal's Unix socket. Otherwise, local execution proceeds.
    pub(crate) async fn handle_execute_tool(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("execute_tool request");

        let tool_params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters for execute_tool".to_string(),
            data: None,
        })?;

        let tool_name = tool_params
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'tool' parameter".to_string(),
                data: None,
            })?;

        let args = tool_params
            .get("args")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Check if tool is announced by a remote primal
        let remote = {
            let registry = self.announced_tools.read().await;
            registry.get(tool_name).map(|a| a.socket_path.clone())
        };

        if let Some(socket_path) = remote {
            info!(
                "Forwarding tool '{}' to remote primal at {}",
                tool_name, socket_path
            );
            return self
                .forward_tool_to_remote(tool_name, &args, &socket_path)
                .await;
        }

        // Local execution
        info!("Executing local tool: {tool_name}");

        let executor = crate::tool::ToolExecutor::new();
        let args_str = serde_json::to_string(&args).map_err(|e| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Invalid tool args: {e}"),
            data: None,
        })?;

        match executor.execute_tool(tool_name, &args_str).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "tool": result.tool_name,
                    "success": result.success,
                    "output": result.output,
                    "error": result.error,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                Ok(response)
            }
            Err(e) => Err(JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Tool execution failed: {e}"),
                data: Some(serde_json::json!({ "tool": tool_name })),
            }),
        }
    }

    /// Forward a `tool.execute` call to a remote primal via Unix socket.
    async fn forward_tool_to_remote(
        &self,
        tool_name: &str,
        args: &Value,
        socket_path: &str,
    ) -> Result<Value, JsonRpcError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to connect to remote primal at {socket_path}: {e}"),
                data: Some(serde_json::json!({ "tool": tool_name, "socket": socket_path })),
            })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tool.execute",
            "params": { "tool": tool_name, "args": args },
            "id": 1
        });

        let mut request_line = serde_json::to_string(&request).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })?;
        request_line.push('\n');

        let (reader, mut writer) = tokio::io::split(stream);
        writer
            .write_all(request_line.as_bytes())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to write to remote primal: {e}"),
                data: None,
            })?;
        writer.flush().await.map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush to remote primal: {e}"),
            data: None,
        })?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        buf_reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to read from remote primal: {e}"),
                data: None,
            })?;

        let response: Value =
            serde_json::from_str(response_line.trim()).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Invalid response from remote primal: {e}"),
                data: None,
            })?;

        if let Some(result) = response.get("result") {
            Ok(result.clone())
        } else if let Some(error) = response.get("error") {
            Err(JsonRpcError {
                code: error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32,
                message: error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Remote error")
                    .to_string(),
                data: error.get("data").cloned(),
            })
        } else {
            Ok(response)
        }
    }

    // -----------------------------------------------------------------------
    // Tool listing
    // -----------------------------------------------------------------------

    /// Handle `tool.list` method
    ///
    /// Returns all available tools: local built-ins + tools announced by
    /// remote primals via `capability.announce`.
    ///
    /// Tool definitions enriched with JSON Schema from capability_registry.toml
    /// (McpToolDef pattern from neuralSpring).
    pub(crate) async fn handle_list_tools(&self) -> Result<Value, JsonRpcError> {
        debug!("tool.list request");

        let executor = crate::tool::ToolExecutor::new();
        let mut entries: Vec<super::types::ToolListEntry> = executor
            .list_tools()
            .iter()
            .map(|t| {
                let schema = self
                    .capability_registry
                    .find(&t.name)
                    .and_then(|c| c.input_schema.clone());
                super::types::ToolListEntry {
                    name: t.name.to_string(),
                    description: t.description.clone(),
                    domain: t.domain.to_string(),
                    source: super::types::ToolSource::Builtin,
                    input_schema: schema,
                }
            })
            .collect();

        // Add registry-only capabilities as tools (for methods not in ToolExecutor)
        let mut seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        for cap in &self.capability_registry.capabilities {
            if seen.insert(cap.method.clone()) {
                entries.push(super::types::ToolListEntry {
                    name: cap.method.clone(),
                    description: cap.description.clone(),
                    domain: cap.domain.clone(),
                    source: super::types::ToolSource::Builtin,
                    input_schema: cap.input_schema.clone(),
                });
            }
        }

        // Add remote announced tools
        let announced = self.announced_tools.read().await;
        for (tool_name, announced_primal) in announced.iter() {
            if seen.insert(tool_name.clone()) {
                let domain = tool_name
                    .split('.')
                    .next()
                    .unwrap_or("external")
                    .to_string();
                entries.push(super::types::ToolListEntry {
                    name: tool_name.clone(),
                    description: format!("Remote tool from {}", announced_primal.primal),
                    domain,
                    source: super::types::ToolSource::Remote {
                        primal: announced_primal.primal.clone(),
                    },
                    input_schema: None,
                });
            }
        }

        let total = entries.len();
        let response = super::types::ToolListResponse {
            tools: entries,
            total,
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }
}

#[cfg(test)]
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
            .unwrap();
        assert!(caps.iter().any(|c| c.as_str() == Some("ai.query")));
        assert!(caps.iter().any(|c| c.as_str() == Some("system.health")));
    }

    #[tokio::test]
    async fn test_handle_announce_capabilities_valid() {
        let server = make_server();
        let params = Some(json!({"capabilities": ["ai.inference", "tool.execute"]}));
        let result = server.handle_announce_capabilities(params).await.unwrap();
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        assert!(result
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("2"));
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
        assert!(result
            .get("providers")
            .and_then(|v| v.as_array())
            .unwrap()
            .is_empty());
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
        assert!(result
            .unwrap_err()
            .message
            .contains("AI router not configured"));
    }
}
