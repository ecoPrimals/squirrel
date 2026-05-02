// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! tarpc trait implementation — dispatches `SquirrelRpc` methods to JSON-RPC handlers.
//!
//! Extracted from `tarpc_server.rs` for module size management (same pattern as
//! `jsonrpc_dispatch.rs`). Each trait method converts typed params → JSON,
//! delegates to the corresponding `JsonRpcServer` handler, then maps JSON → typed result.

use super::tarpc_server::TarpcRpcServer;
use super::tarpc_service::{
    AnnounceCapabilitiesParams, AnnounceCapabilitiesResult, BtspNegotiateParams,
    BtspNegotiateResult, CapabilityDiscoverResult, ContextCreateParams, ContextCreateResult,
    ContextSummarizeParams, ContextSummarizeResult, ContextUpdateParams, ContextUpdateResult,
    DiscoveryPeersResult, HealthCheckResult, LifecycleRegisterResult, LifecycleStatusResult,
    ListProvidersResult, ProviderDeregisterResult, ProviderListResult, ProviderRegisterParams,
    ProviderRegisterResult, QueryAiParams, QueryAiResult, SquirrelRpc, SystemMetricsResult,
    ToolExecuteResult, ToolListResult,
};
use std::collections::HashMap;
use tarpc::context;
use tracing::warn;

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
                provider: std::sync::Arc::from("error"),
                model: std::sync::Arc::from("none"),
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
                tier: super::types::HealthTier::Alive,
                alive: false,
                ready: false,
                healthy: false,
                status: "error".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: 0,
                active_providers: 0,
                requests_processed: 0,
                avg_response_time_ms: None,
            },
        }
    }

    async fn system_ping(self, _ctx: context::Context) -> super::tarpc_service::PingResult {
        match self.jsonrpc.handle_ping().await {
            Ok(v) => Self::json_to_ping_result(&v),
            Err(_) => super::tarpc_service::PingResult {
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

    async fn provider_register(
        self,
        _ctx: context::Context,
        params: ProviderRegisterParams,
    ) -> ProviderRegisterResult {
        let json_params = serde_json::json!({
            "provider_id": params.provider_id,
            "socket": params.socket,
            "endpoint": params.endpoint,
            "capabilities": params.capabilities,
            "version": params.version.unwrap_or_default(),
            "domain": params.domain.unwrap_or_default(),
            "priority": params.priority.unwrap_or(50),
        });
        match self
            .jsonrpc
            .handle_provider_register(Some(json_params))
            .await
        {
            Ok(v) => ProviderRegisterResult {
                success: v
                    .get("success")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                message: v
                    .get("message")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            Err(e) => ProviderRegisterResult {
                success: false,
                message: e.message,
            },
        }
    }

    async fn provider_list(self, _ctx: context::Context) -> ProviderListResult {
        match self.jsonrpc.handle_provider_list().await {
            Ok(v) => ProviderListResult {
                providers: v
                    .get("providers")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default(),
                count: v
                    .get("count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as usize,
            },
            Err(_) => ProviderListResult {
                providers: vec![],
                count: 0,
            },
        }
    }

    async fn provider_deregister(
        self,
        _ctx: context::Context,
        provider_id: String,
    ) -> ProviderDeregisterResult {
        let json_params = serde_json::json!({ "provider_id": provider_id });
        match self
            .jsonrpc
            .handle_provider_deregister(Some(json_params))
            .await
        {
            Ok(v) => ProviderDeregisterResult {
                success: v
                    .get("success")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                message: v
                    .get("message")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            Err(e) => ProviderDeregisterResult {
                success: false,
                message: e.message,
            },
        }
    }

    async fn btsp_negotiate(
        self,
        _ctx: context::Context,
        params: BtspNegotiateParams,
    ) -> BtspNegotiateResult {
        let json_params = serde_json::json!({
            "session_id": params.session_id,
            "preferred_cipher": params.preferred_cipher,
            "bond_type": params.bond_type,
        });
        match self.jsonrpc.handle_btsp_negotiate(Some(json_params)).await {
            Ok(v) => BtspNegotiateResult {
                cipher: v
                    .get("cipher")
                    .and_then(|x| x.as_str())
                    .unwrap_or("null")
                    .to_string(),
                server_nonce: v
                    .get("server_nonce")
                    .and_then(|x| x.as_str())
                    .map(String::from),
            },
            Err(_) => BtspNegotiateResult {
                cipher: "null".to_string(),
                server_nonce: None,
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
