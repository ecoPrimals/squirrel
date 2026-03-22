// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! tarpc RPC Server Implementation
//!
//! High-performance binary RPC server using tarpc framework.
//! Delegates to JsonRpcServer handlers for consistency with JSON-RPC.
//! Provides the same functionality as JSON-RPC but with:
//! - Type safety
//! - Binary serialization
//! - Lower latency
//! - Smaller payloads
//! - Cascading cancellation
//! - Deadline propagation

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use super::tarpc_service::{
    AnnounceCapabilitiesParams, AnnounceCapabilitiesResult, CapabilityDiscoverResult,
    ContextCreateParams, ContextCreateResult, ContextSummarizeParams, ContextSummarizeResult,
    ContextUpdateParams, ContextUpdateResult, DiscoveryPeersResult, HealthCheckResult,
    LifecycleRegisterResult, LifecycleStatusResult, ListProvidersResult, PeerInfo, PingResult,
    ProviderInfo, QueryAiParams, QueryAiResult, SquirrelRpc, SystemMetricsResult,
    ToolExecuteResult, ToolListEntry, ToolListResult, ToolSource,
};
use anyhow::Result;
use futures::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tarpc::{context, server, server::Channel};
use tracing::{info, warn};
use universal_patterns::transport::UniversalTransport;

/// tarpc RPC Server
///
/// Implements the SquirrelRpc service trait by delegating to JsonRpcServer handlers.
#[derive(Clone)]
pub struct TarpcRpcServer {
    /// JSON-RPC server for delegation
    jsonrpc: Arc<super::JsonRpcServer>,
}

impl TarpcRpcServer {
    /// Create tarpc server that delegates to the given JSON-RPC server
    #[must_use]
    pub const fn from_jsonrpc(jsonrpc: Arc<super::JsonRpcServer>) -> Self {
        Self { jsonrpc }
    }

    /// Handle a single tarpc connection
    ///
    /// This method sets up a tarpc server channel for a single client connection
    /// over Universal Transport (Unix socket, TCP, or Named pipe).
    pub async fn handle_connection(self, transport: UniversalTransport) -> Result<()> {
        info!("🔌 tarpc: New connection accepted");

        // Bridge UniversalTransport (AsyncRead + AsyncWrite) to Stream + Sink
        use super::tarpc_transport::TarpcTransportAdapter;
        let transport_adapter = TarpcTransportAdapter::new(transport);

        // Create tarpc transport using tokio-serde for serialization
        use tokio_serde::formats::Bincode;
        let transport = tokio_serde::Framed::new(transport_adapter, Bincode::default());

        // Create tarpc server channel
        let server = server::BaseChannel::with_defaults(transport);

        // Serve requests using this server's trait implementation
        server
            .execute(self.serve())
            .for_each(|response| async move {
                tokio::spawn(response);
            })
            .await;

        info!("🔌 tarpc: Connection closed");
        Ok(())
    }

    /// Convert JSON-RPC result to tarpc type, or return error result
    fn json_to_query_result(v: &serde_json::Value) -> QueryAiResult {
        QueryAiResult {
            response: v
                .get("response")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            provider: Arc::from(v.get("provider").and_then(|x| x.as_str()).unwrap_or("none")),
            model: Arc::from(v.get("model").and_then(|x| x.as_str()).unwrap_or("none")),
            tokens_used: v
                .get("tokens_used")
                .and_then(serde_json::Value::as_u64)
                .map(|u| u as usize),
            latency_ms: v
                .get("latency_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn json_to_list_providers_result(v: &serde_json::Value) -> ListProvidersResult {
        let providers: Vec<ProviderInfo> = v
            .get("providers")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|p| {
                Some(ProviderInfo {
                    id: Arc::from(p.get("id")?.as_str()?),
                    name: Arc::from(p.get("name")?.as_str()?),
                    models: p
                        .get("models")?
                        .as_array()?
                        .iter()
                        .filter_map(|m| m.as_str().map(Arc::from))
                        .collect(),
                    capabilities: p
                        .get("capabilities")?
                        .as_array()?
                        .iter()
                        .filter_map(|c| c.as_str().map(Arc::from))
                        .collect(),
                    online: p.get("online")?.as_bool()?,
                    avg_latency_ms: p.get("avg_latency_ms")?.as_f64(),
                    cost_tier: Arc::from(p.get("cost_tier")?.as_str()?),
                })
            })
            .collect();
        ListProvidersResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            providers,
        }
    }

    fn json_to_announce_result(v: &serde_json::Value) -> AnnounceCapabilitiesResult {
        AnnounceCapabilitiesResult {
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            message: v
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            announced_at: v
                .get("announced_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            tools_registered: v
                .get("tools_registered")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
        }
    }

    fn json_to_health_result(v: &serde_json::Value) -> HealthCheckResult {
        HealthCheckResult {
            status: v
                .get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            uptime_seconds: v
                .get("uptime_seconds")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            active_providers: v
                .get("active_providers")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            requests_processed: v
                .get("requests_processed")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            avg_response_time_ms: v
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64),
        }
    }

    fn json_to_ping_result(v: &serde_json::Value) -> PingResult {
        PingResult {
            pong: v
                .get("pong")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            timestamp: v
                .get("timestamp")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    fn json_to_discovery_peers_result(v: &serde_json::Value) -> DiscoveryPeersResult {
        let peers: Vec<PeerInfo> = v
            .get("peers")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|p| {
                Some(PeerInfo {
                    id: p.get("id")?.as_str()?.to_string(),
                    socket: p.get("socket")?.as_str()?.to_string(),
                    capabilities: p
                        .get("capabilities")?
                        .as_array()?
                        .iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect(),
                    discovered_via: p.get("discovered_via")?.as_str().unwrap_or("").to_string(),
                })
            })
            .collect();
        DiscoveryPeersResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            peers,
            discovery_method: v
                .get("discovery_method")
                .and_then(|x| x.as_str())
                .unwrap_or("socket_scan")
                .to_string(),
        }
    }

    fn json_to_tool_execute_result(v: &serde_json::Value) -> ToolExecuteResult {
        ToolExecuteResult {
            tool: v
                .get("tool")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            output: v
                .get("output")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            error: v.get("error").and_then(|x| x.as_str()).map(String::from),
            timestamp: v
                .get("timestamp")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    fn json_to_tool_list_result(v: &serde_json::Value) -> ToolListResult {
        let tools: Vec<ToolListEntry> = v
            .get("tools")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|t| {
                let source = match t.get("source") {
                    Some(s) => {
                        // Serde serializes Remote as {"Remote": {"primal": "x"}}
                        let primal = s
                            .get("Remote")
                            .and_then(|r| r.get("primal"))
                            .and_then(|p| p.as_str())
                            .or_else(|| s.get("primal").and_then(|p| p.as_str()));
                        match primal {
                            Some(p) => ToolSource::Remote {
                                primal: p.to_string(),
                            },
                            None => ToolSource::Builtin,
                        }
                    }
                    None => ToolSource::Builtin,
                };
                Some(ToolListEntry {
                    name: t.get("name")?.as_str()?.to_string(),
                    description: t.get("description")?.as_str()?.to_string(),
                    domain: t.get("domain")?.as_str()?.to_string(),
                    source,
                    input_schema: t.get("input_schema").cloned(),
                })
            })
            .collect();
        ToolListResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            tools,
        }
    }

    fn json_to_capability_discover_result(v: &serde_json::Value) -> CapabilityDiscoverResult {
        let metadata: HashMap<String, String> = v
            .get("metadata")
            .and_then(|x| x.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
            .unwrap_or_default();
        CapabilityDiscoverResult {
            primal: v
                .get("primal")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            capabilities: v
                .get("capabilities")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            metadata,
        }
    }

    fn json_to_system_metrics_result(v: &serde_json::Value) -> SystemMetricsResult {
        SystemMetricsResult {
            requests_handled: v
                .get("requests_handled")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            errors: v
                .get("errors")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            uptime_seconds: v
                .get("uptime_seconds")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            avg_response_time_ms: v
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64),
            success_rate: v
                .get("success_rate")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0),
        }
    }
}

/// Implement the SquirrelRpc service trait by delegating to JsonRpcServer
impl SquirrelRpc for TarpcRpcServer {
    async fn ai_query(self, _ctx: context::Context, params: QueryAiParams) -> QueryAiResult {
        let json_params = serde_json::json!({
            "prompt": params.prompt,
            "model": params.model.as_deref(),
            "max_tokens": params.max_tokens,
            "temperature": params.temperature,
        });
        match self.jsonrpc.handle_query_ai(Some(json_params)).await {
            Ok(v) => Self::json_to_query_result(&v),
            Err(_) => QueryAiResult {
                response: String::new(),
                provider: Arc::from("error"),
                model: Arc::from("none"),
                tokens_used: None,
                latency_ms: 0,
                success: false,
            },
        }
    }

    async fn ai_complete(self, ctx: context::Context, params: QueryAiParams) -> QueryAiResult {
        self.ai_query(ctx, params).await
    }

    async fn ai_chat(self, ctx: context::Context, params: QueryAiParams) -> QueryAiResult {
        self.ai_query(ctx, params).await
    }

    async fn ai_list_providers(self, _ctx: context::Context) -> ListProvidersResult {
        match self.jsonrpc.handle_list_providers(None).await {
            Ok(v) => Self::json_to_list_providers_result(&v),
            Err(_) => ListProvidersResult {
                total: 0,
                providers: vec![],
            },
        }
    }

    async fn system_health(self, _ctx: context::Context) -> HealthCheckResult {
        match self.jsonrpc.handle_health().await {
            Ok(v) => Self::json_to_health_result(&v),
            Err(_) => HealthCheckResult {
                status: "error".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: 0,
                active_providers: 0,
                requests_processed: 0,
                avg_response_time_ms: None,
            },
        }
    }

    async fn system_ping(self, _ctx: context::Context) -> PingResult {
        match self.jsonrpc.handle_ping().await {
            Ok(v) => Self::json_to_ping_result(&v),
            Err(_) => PingResult {
                pong: true,
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    async fn system_metrics(self, _ctx: context::Context) -> SystemMetricsResult {
        match self.jsonrpc.handle_metrics().await {
            Ok(v) => Self::json_to_system_metrics_result(&v),
            Err(_) => SystemMetricsResult {
                requests_handled: 0,
                errors: 0,
                uptime_seconds: 0,
                avg_response_time_ms: None,
                success_rate: 1.0,
            },
        }
    }

    async fn system_status(self, ctx: context::Context) -> HealthCheckResult {
        self.system_health(ctx).await
    }

    async fn capability_discover(self, _ctx: context::Context) -> CapabilityDiscoverResult {
        match self.jsonrpc.handle_discover_capabilities().await {
            Ok(v) => Self::json_to_capability_discover_result(&v),
            Err(_) => CapabilityDiscoverResult {
                primal: "squirrel".to_string(),
                capabilities: vec![],
                version: env!("CARGO_PKG_VERSION").to_string(),
                metadata: HashMap::new(),
            },
        }
    }

    async fn capability_announce(
        self,
        _ctx: context::Context,
        params: AnnounceCapabilitiesParams,
    ) -> AnnounceCapabilitiesResult {
        let json_params = serde_json::json!({
            "capabilities": params.capabilities,
            "primal": params.primal,
            "socket_path": params.socket_path,
            "tools": params.tools,
            "sub_federations": params.sub_federations,
            "genetic_families": params.genetic_families,
        });
        match self
            .jsonrpc
            .handle_announce_capabilities(Some(json_params))
            .await
        {
            Ok(v) => Self::json_to_announce_result(&v),
            Err(_) => AnnounceCapabilitiesResult {
                success: false,
                message: "Announce failed".to_string(),
                announced_at: chrono::Utc::now().to_rfc3339(),
                tools_registered: 0,
            },
        }
    }

    async fn discovery_peers(self, _ctx: context::Context) -> DiscoveryPeersResult {
        match self.jsonrpc.handle_discover_peers(None).await {
            Ok(v) => Self::json_to_discovery_peers_result(&v),
            Err(e) => {
                warn!("discovery_peers failed: {:?}", e);
                DiscoveryPeersResult {
                    peers: vec![],
                    total: 0,
                    discovery_method: "socket_scan".to_string(),
                }
            }
        }
    }

    async fn tool_execute(
        self,
        _ctx: context::Context,
        tool: String,
        args: HashMap<String, serde_json::Value>,
    ) -> ToolExecuteResult {
        let json_params = serde_json::json!({
            "tool": tool,
            "args": args,
        });
        match self.jsonrpc.handle_execute_tool(Some(json_params)).await {
            Ok(v) => Self::json_to_tool_execute_result(&v),
            Err(e) => ToolExecuteResult {
                tool,
                success: false,
                output: String::new(),
                error: Some(e.message),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    async fn tool_list(self, _ctx: context::Context) -> ToolListResult {
        match self.jsonrpc.handle_list_tools().await {
            Ok(v) => Self::json_to_tool_list_result(&v),
            Err(_) => ToolListResult {
                tools: vec![],
                total: 0,
            },
        }
    }

    async fn context_create(
        self,
        _ctx: context::Context,
        params: ContextCreateParams,
    ) -> ContextCreateResult {
        let json_params = serde_json::json!({
            "session_id": params.session_id,
            "metadata": params
                .metadata
                .unwrap_or_else(|| serde_json::json!({})),
        });
        match self.jsonrpc.handle_context_create(Some(json_params)).await {
            Ok(v) => ContextCreateResult {
                id: v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                version: v
                    .get("version")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                created_at: v
                    .get("created_at")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                metadata: v
                    .get("metadata")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
            },
            Err(_) => ContextCreateResult {
                id: String::new(),
                version: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
                metadata: serde_json::json!({}),
            },
        }
    }

    async fn context_update(
        self,
        _ctx: context::Context,
        params: ContextUpdateParams,
    ) -> ContextUpdateResult {
        let json_params = serde_json::json!({
            "id": params.id,
            "data": params.data,
        });
        match self.jsonrpc.handle_context_update(Some(json_params)).await {
            Ok(v) => ContextUpdateResult {
                id: v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                version: v
                    .get("version")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                updated_at: v
                    .get("updated_at")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            Err(_) => ContextUpdateResult {
                id: params.id,
                version: 0,
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    async fn context_summarize(
        self,
        _ctx: context::Context,
        params: ContextSummarizeParams,
    ) -> ContextSummarizeResult {
        let json_params = serde_json::json!({ "id": params.id });
        match self
            .jsonrpc
            .handle_context_summarize(Some(json_params))
            .await
        {
            Ok(v) => ContextSummarizeResult {
                id: v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                version: v
                    .get("version")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                summary: v
                    .get("summary")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                data: v
                    .get("data")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                synchronized: v
                    .get("synchronized")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            },
            Err(_) => ContextSummarizeResult {
                id: params.id,
                version: 0,
                summary: String::new(),
                data: serde_json::json!({}),
                synchronized: false,
            },
        }
    }

    async fn lifecycle_register(self, _ctx: context::Context) -> LifecycleRegisterResult {
        match self.jsonrpc.handle_lifecycle_register().await {
            Ok(v) => LifecycleRegisterResult {
                success: v
                    .get("success")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true),
                message: v
                    .get("message")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            Err(_) => LifecycleRegisterResult {
                success: false,
                message: "Registration failed".to_string(),
            },
        }
    }

    async fn lifecycle_status(self, _ctx: context::Context) -> LifecycleStatusResult {
        match self.jsonrpc.handle_lifecycle_status().await {
            Ok(v) => LifecycleStatusResult {
                status: v
                    .get("status")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: v
                    .get("version")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                uptime_seconds: v
                    .get("uptime_seconds")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            },
            Err(_) => LifecycleStatusResult {
                status: "error".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tarpc_server_from_jsonrpc() {
        let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
        let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
        let ctx = context::current();
        let ping = server.system_ping(ctx).await;
        assert!(ping.pong);
        assert!(!ping.version.is_empty());
    }

    #[tokio::test]
    async fn test_tarpc_system_health() {
        let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
        let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
        let ctx = context::current();
        let health = server.system_health(ctx).await;
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_tarpc_ai_list_providers_no_router() {
        let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
        let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
        let ctx = context::current();
        let result = server.ai_list_providers(ctx).await;
        assert_eq!(result.total, 0);
        assert!(result.providers.is_empty());
    }
}
