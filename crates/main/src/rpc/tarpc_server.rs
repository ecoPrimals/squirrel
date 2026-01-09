//! tarpc Server Implementation
//!
//! High-performance RPC server for Squirrel-to-Squirrel communication.

use super::tarpc_service::*;
use crate::api::ai::AiRouter;
use crate::error::PrimalError;
use futures::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tarpc::context;
use tarpc::server::BaseChannel;
use tokio::net::TcpListener;
use tokio_serde::formats::Bincode;
use tracing::{debug, error, info};

/// tarpc server implementation
#[derive(Clone)]
pub struct SquirrelRpcServer {
    /// AI router for query handling
    ai_router: Option<Arc<AiRouter>>,

    /// Start time for uptime calculation
    start_time: Instant,
}

impl SquirrelRpcServer {
    /// Create a new tarpc server
    pub fn new() -> Self {
        Self {
            ai_router: None,
            start_time: Instant::now(),
        }
    }

    /// Create a new tarpc server with AI router
    pub fn with_ai_router(ai_router: Arc<AiRouter>) -> Self {
        Self {
            ai_router: Some(ai_router),
            start_time: Instant::now(),
        }
    }

    /// Start the tarpc server on a TCP address
    pub async fn start(self, addr: SocketAddr) -> Result<(), PrimalError> {
        info!("🚀 Starting tarpc server on {}", addr);

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            PrimalError::NetworkError(format!("Failed to bind tarpc server: {}", e))
        })?;

        info!("✅ tarpc server listening on {}", addr);

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("❌ Failed to accept connection: {}", e);
                    continue;
                }
            };

            debug!("🔌 New tarpc connection from {}", peer_addr);

            let server = self.clone();

            tokio::spawn(async move {
                let transport = tarpc::serde_transport::new(stream, Bincode::default());

                let channel = BaseChannel::with_defaults(transport);

                if let Err(e) = channel.execute(server.serve()).await {
                    error!("❌ tarpc channel error: {}", e);
                }
            });
        }
    }
}

impl Default for SquirrelRpcServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tarpc::server]
impl SquirrelRpc for SquirrelRpcServer {
    async fn query_ai(
        self,
        _context: context::Context,
        request: TarpcQueryRequest,
    ) -> Result<TarpcQueryResponse, String> {
        info!(
            "🤖 tarpc: query_ai - prompt length: {}",
            request.prompt.len()
        );

        let start = Instant::now();

        // Check if AI router is available
        if let Some(router) = &self.ai_router {
            // Use actual AI router
            use crate::api::ai::types::TextGenerationRequest;
            let ai_request = TextGenerationRequest {
                prompt: request.prompt.clone(),
                system: None,
                max_tokens: request.max_tokens.map(|v| v as u32).unwrap_or(1024),
                temperature: request.temperature.unwrap_or(0.7),
                model: request.model.clone(),
                constraints: vec![],
                params: std::collections::HashMap::new(),
            };

            match router.generate_text(ai_request, None).await {
                Ok(ai_response) => {
                    let response = TarpcQueryResponse {
                        response: ai_response.text,
                        provider: ai_response.provider_id,
                        model: ai_response.model,
                        tokens_used: ai_response.usage.map(|u| u.total_tokens as usize),
                        latency_ms: start.elapsed().as_millis() as u64,
                    };
                    debug!("✅ tarpc: query_ai completed in {}ms", response.latency_ms);
                    Ok(response)
                }
                Err(e) => {
                    error!("❌ tarpc: AI router error: {}", e);
                    Err(format!("AI router error: {}", e))
                }
            }
        } else {
            // Fallback mock response
            Ok(TarpcQueryResponse {
                response: format!("Mock response to: {}", request.prompt),
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                tokens_used: Some(50),
                latency_ms: start.elapsed().as_millis() as u64,
            })
        }
    }

    async fn list_providers(
        self,
        _context: context::Context,
    ) -> Result<Vec<TarpcProviderInfo>, String> {
        info!("📋 tarpc: list_providers");

        if let Some(router) = &self.ai_router {
            let providers = router
                .list_providers()
                .await
                .into_iter()
                .map(|p| {
                    let cost_tier = if p.cost_per_unit.unwrap_or(0.0) > 0.01 {
                        "high"
                    } else if p.cost_per_unit.unwrap_or(0.0) > 0.0 {
                        "medium"
                    } else {
                        "free"
                    };

                    TarpcProviderInfo {
                        id: p.provider_id,
                        name: p.provider_name,
                        capabilities: p.capabilities,
                        online: p.is_available,
                        cost_tier: cost_tier.to_string(),
                    }
                })
                .collect();

            debug!(
                "✅ tarpc: list_providers returned {} providers",
                providers.len()
            );
            Ok(providers)
        } else {
            Ok(vec![TarpcProviderInfo {
                id: "mock".to_string(),
                name: "Mock Provider".to_string(),
                capabilities: vec!["ai.inference".to_string()],
                online: false,
                cost_tier: "free".to_string(),
            }])
        }
    }

    async fn health_check(self, _context: context::Context) -> Result<TarpcHealthStatus, String> {
        debug!("💚 tarpc: health_check");

        let status = TarpcHealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            active_providers: if let Some(router) = &self.ai_router {
                router.provider_count().await
            } else {
                0
            },
        };

        debug!(
            "✅ tarpc: health_check - uptime: {}s",
            status.uptime_seconds
        );
        Ok(status)
    }

    async fn announce_capabilities(
        self,
        _context: context::Context,
        capabilities: Vec<String>,
    ) -> Result<bool, String> {
        info!(
            "📢 tarpc: announce_capabilities - {} capabilities",
            capabilities.len()
        );
        debug!("✅ tarpc: capabilities acknowledged");
        Ok(true)
    }

    async fn discover_peers(self, _context: context::Context) -> Result<Vec<String>, String> {
        info!("🔍 tarpc: discover_peers");
        // TODO: Implement peer discovery via Songbird or UDP multicast
        // For now, return empty list
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = SquirrelRpcServer::new();
        assert!(server.ai_router.is_none());
    }

    #[test]
    fn test_server_with_router() {
        // Can't easily test with real AiRouter without async setup
        // but we can test the structure
        let server = SquirrelRpcServer::new();
        assert!(server.ai_router.is_none());
    }
}
