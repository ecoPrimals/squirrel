// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! tarpc RPC Server Implementation
//!
//! High-performance binary RPC server using tarpc framework.
//! Provides the same functionality as JSON-RPC but with:
//! - Type safety
//! - Binary serialization
//! - Lower latency
//! - Smaller payloads
//! - Cascading cancellation
//! - Deadline propagation

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use super::tarpc_service::*;
use anyhow::Result;
use futures::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tarpc::{
    context,
    server::{self, Channel},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_patterns::transport::UniversalTransport;

use super::jsonrpc_server::ServerMetrics;

/// tarpc RPC Server
///
/// Implements the SquirrelRpc service trait with actual business logic.
#[derive(Clone)]
pub struct TarpcRpcServer {
    /// Service name
    service_name: String,

    /// Server metrics (shared with JSON-RPC)
    metrics: Arc<RwLock<ServerMetrics>>,

    /// AI router (optional)
    ai_router: Option<Arc<crate::api::ai::AiRouter>>,
}

impl TarpcRpcServer {
    /// Create a new tarpc RPC server
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: None,
        }
    }

    /// Create server with AI router
    pub fn with_ai_router(service_name: String, ai_router: Arc<crate::api::ai::AiRouter>) -> Self {
        Self {
            service_name,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: Some(ai_router),
        }
    }

    /// Create server with shared metrics
    pub fn with_metrics(
        service_name: String,
        metrics: Arc<RwLock<ServerMetrics>>,
        ai_router: Option<Arc<crate::api::ai::AiRouter>>,
    ) -> Self {
        Self {
            service_name,
            metrics,
            ai_router,
        }
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
}

/// Implement the SquirrelRpc service trait
impl SquirrelRpc for TarpcRpcServer {
    async fn query_ai(self, _ctx: context::Context, params: QueryAiParams) -> QueryAiResult {
        let start = Instant::now();

        info!(
            "🤖 tarpc::query_ai - prompt length: {}",
            params.prompt.len()
        );

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;

        // If AI router available, use it; otherwise return error
        if let Some(router) = &self.ai_router {
            use crate::api::ai::types::TextGenerationRequest;

            let ai_request = TextGenerationRequest {
                prompt: params.prompt,
                system: None,
                max_tokens: params.max_tokens.map(|v| v as u32).unwrap_or(1024),
                temperature: params.temperature.unwrap_or(0.7) as f32,
                model: params.model,
                constraints: vec![],
                params: std::collections::HashMap::new(),
            };

            drop(metrics); // Release lock before async call

            match router.generate_text(ai_request, None).await {
                Ok(ai_response) => {
                    let latency_ms = start.elapsed().as_millis() as u64;
                    let mut metrics = self.metrics.write().await;
                    metrics.total_response_time_ms += latency_ms;

                    QueryAiResult {
                        response: ai_response.text,
                        provider: ai_response.provider_id,
                        model: ai_response.model,
                        tokens_used: ai_response.usage.map(|u| u.total_tokens as usize),
                        latency_ms,
                        success: true,
                    }
                }
                Err(e) => {
                    error!("AI router error: {}", e);
                    let mut metrics = self.metrics.write().await;
                    metrics.errors += 1;

                    QueryAiResult {
                        response: format!("Error: {}", e),
                        provider: "error".to_string(),
                        model: "none".to_string(),
                        tokens_used: None,
                        latency_ms: start.elapsed().as_millis() as u64,
                        success: false,
                    }
                }
            }
        } else {
            metrics.errors += 1;
            QueryAiResult {
                response: "AI router not configured".to_string(),
                provider: "none".to_string(),
                model: "none".to_string(),
                tokens_used: None,
                latency_ms: start.elapsed().as_millis() as u64,
                success: false,
            }
        }
    }

    async fn list_providers(self, _ctx: context::Context) -> ListProvidersResult {
        info!("📋 tarpc::list_providers");

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        drop(metrics);

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
                        avg_latency_ms: Some(p.avg_latency_ms as f64),
                        cost_tier,
                    }
                })
                .collect();

            ListProvidersResult {
                total: providers.len(),
                providers,
            }
        } else {
            ListProvidersResult {
                total: 0,
                providers: vec![],
            }
        }
    }

    async fn announce_capabilities(
        self,
        _ctx: context::Context,
        params: AnnounceCapabilitiesParams,
    ) -> AnnounceCapabilitiesResult {
        info!(
            "📢 tarpc::announce_capabilities - {} capabilities",
            params.capabilities.len()
        );

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        drop(metrics);

        AnnounceCapabilitiesResult {
            success: true,
            message: format!("Acknowledged {} capabilities", params.capabilities.len()),
            announced_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    async fn health(self, _ctx: context::Context) -> HealthCheckResult {
        debug!("💚 tarpc::health check");

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;

        let result = HealthCheckResult {
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

        drop(metrics);
        result
    }

    async fn ping(self, _ctx: context::Context) -> String {
        debug!("🏓 tarpc::ping");

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        drop(metrics);

        format!(
            "pong from {} ({})",
            self.service_name,
            env!("CARGO_PKG_VERSION")
        )
    }

    async fn discover_peers(self, _ctx: context::Context) -> Vec<String> {
        info!("🔍 tarpc::discover_peers");

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        drop(metrics);

        // Use actual capability discovery to find peers
        match crate::capabilities::discovery::discover_all_capabilities().await {
            Ok(capabilities_map) => {
                let mut seen = std::collections::HashSet::new();
                let mut peers = Vec::new();
                for providers in capabilities_map.values() {
                    for provider in providers {
                        if seen.insert(provider.id.clone()) {
                            peers.push(format!("{}@{}", provider.id, provider.socket.display()));
                        }
                    }
                }
                peers
            }
            Err(e) => {
                warn!("Peer discovery failed: {}", e);
                vec![]
            }
        }
    }

    async fn execute_tool(
        self,
        _ctx: context::Context,
        tool: String,
        args: std::collections::HashMap<String, String>,
    ) -> String {
        info!("🔧 tarpc::execute_tool: {}", tool);

        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        drop(metrics);

        // Delegate to ToolExecutor
        let executor = crate::tool::ToolExecutor::new();
        let args_str = serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string());

        match executor.execute_tool(&tool, &args_str).await {
            Ok(result) => {
                if result.success {
                    result.output
                } else {
                    format!(
                        "Tool '{}' failed: {}",
                        tool,
                        result.error.unwrap_or_default()
                    )
                }
            }
            Err(e) => format!("Tool '{}' execution error: {}", tool, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tarpc_server_creation() {
        let server = TarpcRpcServer::new("squirrel".to_string());
        assert_eq!(server.service_name, "squirrel");
    }

    #[tokio::test]
    async fn test_tarpc_ping() {
        let server = TarpcRpcServer::new("squirrel".to_string());
        let ctx = context::current();
        let response = server.ping(ctx).await;
        assert!(response.contains("pong"));
        assert!(response.contains("squirrel"));
    }

    #[tokio::test]
    async fn test_tarpc_health() {
        let server = TarpcRpcServer::new("squirrel".to_string());
        let ctx = context::current();
        let health = server.health(ctx).await;
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_tarpc_list_providers_no_router() {
        let server = TarpcRpcServer::new("squirrel".to_string());
        let ctx = context::current();
        let result = server.list_providers(ctx).await;
        assert_eq!(result.total, 0);
        assert!(result.providers.is_empty());
    }
}
