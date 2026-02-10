// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
            jsonrpc: "2.0".to_string(),
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
                        provider: ai_response.provider_id,
                        model: ai_response.model,
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
    pub(crate) async fn handle_announce_capabilities(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: AnnounceCapabilitiesRequest = self.parse_params(params)?;

        info!(
            "📢 announce_capabilities - {} capabilities",
            request.capabilities.len()
        );

        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: format!("Acknowledged {} capabilities", request.capabilities.len()),
            announced_at: chrono::Utc::now().to_rfc3339(),
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
    pub(crate) async fn handle_discover_capabilities(&self) -> Result<Value, JsonRpcError> {
        use crate::optimization::zero_copy::intern;

        debug!("🔍 discover_capabilities request");

        // Use interned strings for zero-copy efficiency — these are called frequently
        // during peer discovery and socket scanning
        let mut capabilities: Vec<String> = [
            "ai.query",
            "ai.complete",
            "ai.chat",
            "ai.list_providers",
            "tool.execute",
            "system.health",
            "system.metrics",
            "system.ping",
            "discovery.peers",
            "capability.announce",
            "capability.discover",
        ]
        .iter()
        .map(|name| intern::get_or_intern(name).to_string())
        .collect();

        if let Some(router) = &self.ai_router {
            let provider_count = router.provider_count().await;
            if provider_count > 0 {
                capabilities.push(intern::get_or_intern("ai.inference").to_string());
                capabilities.push(intern::get_or_intern("ai.text_generation").to_string());
            }
        }

        capabilities.dedup();

        let response = serde_json::json!({
            "primal": "squirrel",
            "capabilities": capabilities,
            "version": env!("CARGO_PKG_VERSION"),
            "metadata": {
                "transport": "unix_socket",
                "protocol": "jsonrpc_2.0",
                "service_name": self.service_name,
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
                        let socket_str = provider.socket.display().to_string();
                        if seen_sockets.insert(socket_str.clone()) {
                            peer_list.push(serde_json::json!({
                                "id": provider.id,
                                "socket": socket_str,
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
    // Tool domain
    // -----------------------------------------------------------------------

    /// Handle `tool.execute` / `execute_tool` method
    pub(crate) async fn handle_execute_tool(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("🔧 execute_tool request");

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

        info!("🔧 Executing tool: {tool_name}");

        let executor = crate::tool::ToolExecutor::new();
        let args_str = serde_json::to_string(&args).unwrap_or_default();

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
}
