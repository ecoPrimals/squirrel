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
    pub(crate) fn json_to_query_result(v: &serde_json::Value) -> QueryAiResult {
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

    pub(crate) fn json_to_list_providers_result(v: &serde_json::Value) -> ListProvidersResult {
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

    pub(crate) fn json_to_announce_result(v: &serde_json::Value) -> AnnounceCapabilitiesResult {
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

    pub(crate) fn json_to_health_result(v: &serde_json::Value) -> HealthCheckResult {
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

    pub(crate) fn json_to_ping_result(v: &serde_json::Value) -> PingResult {
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

    pub(crate) fn json_to_discovery_peers_result(v: &serde_json::Value) -> DiscoveryPeersResult {
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

    pub(crate) fn json_to_tool_execute_result(v: &serde_json::Value) -> ToolExecuteResult {
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

    pub(crate) fn json_to_tool_list_result(v: &serde_json::Value) -> ToolListResult {
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

    pub(crate) fn json_to_capability_discover_result(
        v: &serde_json::Value,
    ) -> CapabilityDiscoverResult {
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

    pub(crate) fn json_to_system_metrics_result(v: &serde_json::Value) -> SystemMetricsResult {
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
    use crate::rpc::tarpc_service::{
        AnnounceCapabilitiesParams, ContextCreateParams, ContextSummarizeParams,
        ContextUpdateParams, QueryAiParams, ToolSource,
    };
    use serde_json::json;
    use std::sync::Arc;

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

    #[test]
    fn json_to_query_result_defaults_and_fields() {
        let empty = json!({});
        let r = TarpcRpcServer::json_to_query_result(&empty);
        assert_eq!(r.response, "");
        assert_eq!(r.provider.as_ref(), "none");
        assert_eq!(r.model.as_ref(), "none");
        assert!(r.tokens_used.is_none());
        assert_eq!(r.latency_ms, 0);
        assert!(!r.success);

        let full = json!({
            "response": "hi",
            "provider": "p",
            "model": "m",
            "tokens_used": 42,
            "latency_ms": 7,
            "success": true
        });
        let r2 = TarpcRpcServer::json_to_query_result(&full);
        assert_eq!(r2.response, "hi");
        assert_eq!(r2.provider.as_ref(), "p");
        assert_eq!(r2.model.as_ref(), "m");
        assert_eq!(r2.tokens_used, Some(42));
        assert_eq!(r2.latency_ms, 7);
        assert!(r2.success);
    }

    #[test]
    fn json_to_list_providers_result_parses_and_skips_invalid() {
        let v = json!({
            "total": 2,
            "providers": [
                {
                    "id": "a",
                    "name": "A",
                    "models": ["m1"],
                    "capabilities": ["c1"],
                    "online": true,
                    "avg_latency_ms": 12.5,
                    "cost_tier": "low"
                },
                { "id": "broken" }
            ]
        });
        let r = TarpcRpcServer::json_to_list_providers_result(&v);
        assert_eq!(r.total, 2);
        assert_eq!(r.providers.len(), 1);
        assert_eq!(r.providers[0].id.as_ref(), "a");
        assert_eq!(r.providers[0].models[0].as_ref(), "m1");
    }

    #[test]
    fn json_to_announce_and_health_and_ping() {
        let ann = json!({
            "success": true,
            "message": "ok",
            "announced_at": "t0",
            "tools_registered": 3
        });
        let a = TarpcRpcServer::json_to_announce_result(&ann);
        assert!(a.success);
        assert_eq!(a.message, "ok");
        assert_eq!(a.announced_at, "t0");
        assert_eq!(a.tools_registered, 3);

        let health = json!({
            "status": "degraded",
            "version": "v9",
            "uptime_seconds": 100,
            "active_providers": 2,
            "requests_processed": 50,
            "avg_response_time_ms": 1.25
        });
        let h = TarpcRpcServer::json_to_health_result(&health);
        assert_eq!(h.status, "degraded");
        assert_eq!(h.version, "v9");
        assert_eq!(h.uptime_seconds, 100);
        assert_eq!(h.active_providers, 2);
        assert_eq!(h.requests_processed, 50);
        assert_eq!(h.avg_response_time_ms, Some(1.25));

        let ping = json!({
            "pong": false,
            "timestamp": "ts",
            "version": "pv"
        });
        let p = TarpcRpcServer::json_to_ping_result(&ping);
        assert!(!p.pong);
        assert_eq!(p.timestamp, "ts");
        assert_eq!(p.version, "pv");
    }

    #[test]
    fn json_to_discovery_peers_result() {
        let v = json!({
            "total": 1,
            "peers": [{
                "id": "peer1",
                "socket": "/tmp/s",
                "capabilities": ["a"],
                "discovered_via": "mdns"
            }],
            "discovery_method": "registry"
        });
        let r = TarpcRpcServer::json_to_discovery_peers_result(&v);
        assert_eq!(r.total, 1);
        assert_eq!(r.peers.len(), 1);
        assert_eq!(r.peers[0].id, "peer1");
        assert_eq!(r.discovery_method, "registry");

        let missing_caps = json!({
            "peers": [{ "id": "x", "socket": "y" }]
        });
        let r2 = TarpcRpcServer::json_to_discovery_peers_result(&missing_caps);
        assert!(r2.peers.is_empty());
    }

    #[test]
    fn json_to_tool_execute_and_list() {
        let ex = json!({
            "tool": "t1",
            "success": true,
            "output": "out",
            "error": null,
            "timestamp": "now"
        });
        let e = TarpcRpcServer::json_to_tool_execute_result(&ex);
        assert_eq!(e.tool, "t1");
        assert!(e.success);
        assert_eq!(e.output, "out");
        assert!(e.error.is_none());

        let list = json!({
            "total": 2,
            "tools": [
                {
                    "name": "n",
                    "description": "d",
                    "domain": "dom",
                    "source": { "Remote": { "primal": "bird" } },
                    "input_schema": { "type": "object" }
                },
                {
                    "name": "b",
                    "description": "bd",
                    "domain": "bd",
                    "source": { "primal": "inline" }
                }
            ]
        });
        let t = TarpcRpcServer::json_to_tool_list_result(&list);
        assert_eq!(t.total, 2);
        assert_eq!(t.tools.len(), 2);
        match &t.tools[0].source {
            ToolSource::Remote { primal } => assert_eq!(primal, "bird"),
            ToolSource::Builtin => panic!("expected Remote"),
        }
        match &t.tools[1].source {
            ToolSource::Remote { primal } => assert_eq!(primal, "inline"),
            ToolSource::Builtin => panic!("expected Remote from primal key"),
        }
    }

    #[test]
    fn json_to_capability_discover_and_system_metrics() {
        let cap = json!({
            "primal": "sq",
            "capabilities": ["x"],
            "version": "1",
            "metadata": { "k": "v" }
        });
        let c = TarpcRpcServer::json_to_capability_discover_result(&cap);
        assert_eq!(c.primal, "sq");
        assert_eq!(c.capabilities, vec!["x"]);
        assert_eq!(c.version, "1");
        assert!(c.metadata.contains_key("k"));

        let m = json!({
            "requests_handled": 9,
            "errors": 1,
            "uptime_seconds": 3,
            "avg_response_time_ms": 4.0,
            "success_rate": 0.9
        });
        let sm = TarpcRpcServer::json_to_system_metrics_result(&m);
        assert_eq!(sm.requests_handled, 9);
        assert_eq!(sm.errors, 1);
        assert_eq!(sm.uptime_seconds, 3);
        assert_eq!(sm.avg_response_time_ms, Some(4.0));
        assert!((sm.success_rate - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn json_to_edge_defaults_and_tool_list_builtin() {
        let ex = json!({
            "tool": "t",
            "success": false,
            "output": "",
            "error": "boom",
            "timestamp": "t1"
        });
        let e = TarpcRpcServer::json_to_tool_execute_result(&ex);
        assert_eq!(e.error.as_deref(), Some("boom"));

        let list = json!({
            "total": 1,
            "tools": [{
                "name": "builtin.tool",
                "description": "d",
                "domain": "d0",
                "input_schema": null
            }]
        });
        let t = TarpcRpcServer::json_to_tool_list_result(&list);
        assert_eq!(t.tools.len(), 1);
        assert!(matches!(t.tools[0].source, ToolSource::Builtin));

        let peers = json!({
            "total": 1,
            "peers": [{
                "id": "p",
                "socket": "/s",
                "capabilities": ["c"],
                "discovered_via": ""
            }]
        });
        let d = TarpcRpcServer::json_to_discovery_peers_result(&peers);
        assert_eq!(d.peers[0].discovered_via, "");

        let cap = json!({
            "primal": "x",
            "capabilities": [],
            "version": ""
        });
        let c = TarpcRpcServer::json_to_capability_discover_result(&cap);
        assert!(c.metadata.is_empty());

        let m = json!({});
        let sm = TarpcRpcServer::json_to_system_metrics_result(&m);
        assert_eq!(sm.requests_handled, 0);
        assert!((sm.success_rate - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn tarpc_delegates_ai_aliases_system_metrics_status_and_tools() {
        let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new(
            "/tmp/tarpc-delegate.sock".to_string(),
        ));
        let server = TarpcRpcServer::from_jsonrpc(jsonrpc);

        let q = QueryAiParams {
            prompt: "p".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
        };
        let r = server
            .clone()
            .ai_complete(context::current(), q.clone())
            .await;
        assert!(!r.success);
        let r2 = server.clone().ai_chat(context::current(), q).await;
        assert!(!r2.success);

        let m = server.clone().system_metrics(context::current()).await;
        assert!(m.success_rate <= 1.0);

        let h = server.clone().system_status(context::current()).await;
        assert_eq!(h.status, "healthy");

        let disc = server.clone().capability_discover(context::current()).await;
        assert_eq!(disc.primal, "squirrel");
        assert!(!disc.capabilities.is_empty());

        let peers = server.clone().discovery_peers(context::current()).await;
        assert_eq!(peers.peers.len(), peers.total);

        let tools = server.clone().tool_list(context::current()).await;
        assert!(tools.total > 0);

        let exec = server
            .clone()
            .tool_execute(
                context::current(),
                "system.health".to_string(),
                std::collections::HashMap::new(),
            )
            .await;
        assert!(exec.success);

        let ann = server
            .clone()
            .capability_announce(
                context::current(),
                AnnounceCapabilitiesParams {
                    capabilities: vec!["ai.query".to_string()],
                    primal: Some("p".to_string()),
                    socket_path: None,
                    tools: None,
                    sub_federations: None,
                    genetic_families: None,
                },
            )
            .await;
        assert!(ann.success);

        let life = server.clone().lifecycle_register(context::current()).await;
        assert!(life.success);

        let st = server.lifecycle_status(context::current()).await;
        assert_eq!(st.status, "healthy");
    }

    #[tokio::test]
    async fn tarpc_context_roundtrip() {
        let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new(
            "/tmp/tarpc-ctx.sock".to_string(),
        ));
        let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
        let ctx = context::current();

        let created = server
            .clone()
            .context_create(
                ctx,
                ContextCreateParams {
                    session_id: Some("sess-tarpc-1".to_string()),
                    metadata: Some(json!({ "k": "v" })),
                },
            )
            .await;
        assert!(!created.id.is_empty());

        let updated = server
            .clone()
            .context_update(
                context::current(),
                ContextUpdateParams {
                    id: created.id.clone(),
                    data: json!({ "x": 1 }),
                },
            )
            .await;
        assert_eq!(updated.id, created.id);
        assert!(updated.version >= 1);

        let sum = server
            .context_summarize(
                context::current(),
                ContextSummarizeParams { id: created.id },
            )
            .await;
        assert!(!sum.summary.is_empty());
    }
}
