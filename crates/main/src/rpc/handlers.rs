//! JSON-RPC API Handlers
//!
//! This module implements the actual business logic for each JSON-RPC method.
//! Handlers delegate to Squirrel's core functionality.

use super::types::*;
use crate::error::PrimalError;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// RPC method handlers
pub struct RpcHandlers {
    /// Start time for uptime calculation
    start_time: Instant,

    /// Request counter
    request_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl RpcHandlers {
    /// Create new RPC handlers
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Handle query_ai method
    pub async fn handle_query_ai(
        &self,
        request: QueryAiRequest,
    ) -> Result<QueryAiResponse, PrimalError> {
        info!("🤖 RPC: query_ai - prompt length: {}", request.prompt.len());

        self.increment_counter();

        let start = Instant::now();

        // TODO: Integrate with actual AI router
        // For now, return a mock response to establish the pipeline
        let response = QueryAiResponse {
            response: format!(
                "AI response to: '{}' (provider: {:?})",
                request.prompt,
                request.provider.as_deref().unwrap_or("auto")
            ),
            provider: request.provider.unwrap_or_else(|| "mock".to_string()),
            model: request.model.unwrap_or_else(|| "mock-model-v1".to_string()),
            tokens_used: Some(50),
            latency_ms: start.elapsed().as_millis() as u64,
            success: true,
        };

        debug!("✅ RPC: query_ai completed in {}ms", response.latency_ms);
        Ok(response)
    }

    /// Handle list_providers method
    pub async fn handle_list_providers(
        &self,
        request: ListProvidersRequest,
    ) -> Result<ListProvidersResponse, PrimalError> {
        info!(
            "📋 RPC: list_providers - capability filter: {:?}",
            request.capability
        );

        self.increment_counter();

        // TODO: Integrate with actual provider registry
        // For now, return mock providers
        let providers = vec![
            ProviderInfo {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                capabilities: vec!["ai.inference".to_string(), "ai.chat".to_string()],
                online: true,
                avg_latency_ms: Some(250),
                cost_tier: "high".to_string(),
            },
            ProviderInfo {
                id: "ollama".to_string(),
                name: "Ollama (Local)".to_string(),
                models: vec!["llama2".to_string(), "mistral".to_string()],
                capabilities: vec!["ai.inference".to_string(), "ai.local".to_string()],
                online: true,
                avg_latency_ms: Some(500),
                cost_tier: "free".to_string(),
            },
        ];

        let response = ListProvidersResponse {
            total: providers.len(),
            providers,
        };

        debug!(
            "✅ RPC: list_providers returned {} providers",
            response.total
        );
        Ok(response)
    }

    /// Handle announce_capabilities method
    pub async fn handle_announce_capabilities(
        &self,
        request: AnnounceCapabilitiesRequest,
    ) -> Result<AnnounceCapabilitiesResponse, PrimalError> {
        info!(
            "📢 RPC: announce_capabilities - {} capabilities",
            request.capabilities.len()
        );

        self.increment_counter();

        // TODO: Integrate with actual capability registry
        // For now, acknowledge the announcement
        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: format!(
                "Announced {} capabilities successfully",
                request.capabilities.len()
            ),
            announced_at: chrono::Utc::now().to_rfc3339(),
        };

        debug!("✅ RPC: announce_capabilities acknowledged");
        Ok(response)
    }

    /// Handle health_check method
    pub async fn handle_health_check(
        &self,
        _request: HealthCheckRequest,
    ) -> Result<HealthCheckResponse, PrimalError> {
        debug!("💚 RPC: health_check");

        let uptime = self.start_time.elapsed().as_secs();
        let requests = self
            .request_counter
            .load(std::sync::atomic::Ordering::Relaxed);

        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            active_providers: 2, // TODO: Get from actual registry
            requests_processed: requests,
            avg_response_time_ms: Some(150.0), // TODO: Calculate from actual metrics
        };

        debug!(
            "✅ RPC: health_check - uptime: {}s, requests: {}",
            uptime, requests
        );
        Ok(response)
    }

    /// Increment request counter
    fn increment_counter(&self) {
        self.request_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for RpcHandlers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_query_ai() {
        let handlers = RpcHandlers::new();

        let request = QueryAiRequest {
            prompt: "Test prompt".to_string(),
            provider: Some("openai".to_string()),
            model: None,
            priority: Some(50),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
        };

        let response = handlers.handle_query_ai(request).await.unwrap();

        assert!(response.success);
        assert!(!response.response.is_empty());
        assert_eq!(response.provider, "openai");
    }

    #[tokio::test]
    async fn test_handle_list_providers() {
        let handlers = RpcHandlers::new();

        let request = ListProvidersRequest {
            capability: None,
            include_offline: Some(false),
        };

        let response = handlers.handle_list_providers(request).await.unwrap();

        assert!(response.total > 0);
        assert!(!response.providers.is_empty());
    }

    #[tokio::test]
    async fn test_handle_health_check() {
        let handlers = RpcHandlers::new();

        let request = HealthCheckRequest {};
        let response = handlers.handle_health_check(request).await.unwrap();

        assert_eq!(response.status, "healthy");
        assert!(!response.version.is_empty());
        assert!(response.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_handle_announce_capabilities() {
        let handlers = RpcHandlers::new();

        let request = AnnounceCapabilitiesRequest {
            capabilities: vec!["ai.inference".to_string(), "ai.chat".to_string()],
            sub_federations: None,
            genetic_families: None,
        };

        let response = handlers
            .handle_announce_capabilities(request)
            .await
            .unwrap();

        assert!(response.success);
        assert!(!response.message.is_empty());
    }
}
