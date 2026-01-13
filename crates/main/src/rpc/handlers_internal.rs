//! Internal handler functions that can be called from multiple protocols
//!
//! These are the actual implementation functions that the protocol router
//! wires to. They're protocol-agnostic and can be called from tarpc,
//! JSON-RPC, or HTTPS.
//!
//! These handlers are wired to the actual AI router and capability registry
//! for production-ready operation.

use crate::api::ai::AiRouter;
use crate::error::PrimalError;
use crate::rpc::types::{
    HealthCheckResponse, ListProvidersResponse, QueryAiRequest, QueryAiResponse,
};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

/// Global AI router instance (lazy-initialized)
static AI_ROUTER: OnceLock<Arc<AiRouter>> = OnceLock::new();

/// Global startup time (lazy-initialized)
static STARTUP_TIME: OnceLock<Instant> = OnceLock::new();

/// Initialize the AI router (call once at startup)
pub fn init_ai_router(router: Arc<AiRouter>) {
    let _ = AI_ROUTER.set(router);
    tracing::info!("✅ AI router initialized for handlers_internal");
}

/// Initialize startup time tracking (call once at startup)
pub fn init_startup_time() {
    let _ = STARTUP_TIME.set(Instant::now());
    tracing::info!("✅ Startup time tracking initialized");
}

/// Get uptime in seconds
fn get_uptime_seconds() -> u64 {
    STARTUP_TIME
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0)
}

/// Infer available models from provider ID and name
///
/// Uses pattern matching on provider identifiers to determine likely available models
fn infer_models_from_provider(provider_id: &str, provider_name: &str) -> Vec<String> {
    let id_lower = provider_id.to_lowercase();
    let name_lower = provider_name.to_lowercase();

    // OpenAI
    if id_lower.contains("openai") || name_lower.contains("openai") {
        return vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ];
    }

    // Anthropic
    if id_lower.contains("anthropic") || name_lower.contains("claude") {
        return vec![
            "claude-3-opus".to_string(),
            "claude-3-sonnet".to_string(),
            "claude-3-haiku".to_string(),
        ];
    }

    // Google
    if id_lower.contains("google") || id_lower.contains("gemini") || name_lower.contains("gemini") {
        return vec!["gemini-pro".to_string(), "gemini-ultra".to_string()];
    }

    // Ollama (local)
    if id_lower.contains("ollama") || name_lower.contains("ollama") {
        return vec![
            "llama2".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
        ];
    }

    // Local models
    if id_lower.contains("local") || name_lower.contains("local") {
        return vec!["local-model".to_string()];
    }

    // Groq
    if id_lower.contains("groq") {
        return vec!["mixtral-8x7b".to_string(), "llama2-70b".to_string()];
    }

    // Generic fallback
    vec!["model-available".to_string()]
}

/// Get AI router reference
fn get_ai_router() -> Option<&'static Arc<AiRouter>> {
    AI_ROUTER.get()
}

/// Handle AI query (internal, protocol-agnostic)
///
/// This is now wired to the actual AI router for production use.
pub async fn handle_query_ai_internal(
    request: QueryAiRequest,
) -> Result<QueryAiResponse, PrimalError> {
    let start = Instant::now();
    tracing::debug!("Handling query_ai: prompt={}", request.prompt);

    // Try to use actual AI router if available
    if let Some(router) = get_ai_router() {
        tracing::debug!("Using real AI router for query");

        // Build request for AI router
        use crate::api::ai::types::TextGenerationRequest;
        let ai_request = TextGenerationRequest {
            prompt: request.prompt.clone(),
            system: None,
            max_tokens: request.max_tokens.map_or(1024, |v| v as u32),
            temperature: request.temperature.unwrap_or(0.7),
            model: request.model.clone(),
            constraints: vec![],
            params: std::collections::HashMap::new(),
        };

        // Execute through AI router
        match router.generate_text(ai_request, None).await {
            Ok(response) => {
                return Ok(QueryAiResponse {
                    response: response.text,
                    provider: response.provider_id,
                    model: response.model,
                    tokens_used: response.usage.map(|u| u.total_tokens as usize),
                    latency_ms: start.elapsed().as_millis() as u64,
                    success: true,
                });
            }
            Err(e) => {
                tracing::warn!("AI router query failed: {}", e);
                return Err(PrimalError::OperationFailed(format!(
                    "AI query failed: {}",
                    e
                )));
            }
        }
    }

    // Graceful degradation: Return informational response if router not configured
    tracing::debug!("AI router not configured - returning informational response");
    Ok(QueryAiResponse {
        response: format!(
            "Query received: '{}'. AI router not configured. Configure providers in config file or via environment variables.",
            request.prompt
        ),
        provider: request
            .provider
            .unwrap_or_else(|| "unconfigured".to_string()),
        model: request.model.unwrap_or_else(|| "unconfigured".to_string()),
        tokens_used: None,
        latency_ms: start.elapsed().as_millis() as u64,
        success: false,
    })
}

/// List available providers (internal, protocol-agnostic)
///
/// Now wired to actual AI router for live provider list.
pub async fn handle_list_providers_internal() -> Result<ListProvidersResponse, PrimalError> {
    tracing::debug!("Handling list_providers");

    // Try to use actual AI router if available
    if let Some(router) = get_ai_router() {
        let providers = router.list_providers().await;

        let provider_infos: Vec<crate::rpc::types::ProviderInfo> = providers
            .into_iter()
            .map(|p| {
                // Infer available models from provider ID/name
                let models = infer_models_from_provider(&p.provider_id, &p.provider_name);

                crate::rpc::types::ProviderInfo {
                    id: p.provider_id,
                    name: p.provider_name,
                    models,
                    capabilities: p.capabilities,
                    online: p.is_available,
                    avg_latency_ms: Some(p.avg_latency_ms),
                    cost_tier: if p.is_local {
                        "free".to_string()
                    } else {
                        "paid".to_string()
                    },
                }
            })
            .collect();

        let total = provider_infos.len();

        return Ok(ListProvidersResponse {
            providers: provider_infos,
            total,
        });
    }

    // Graceful degradation: Return minimal response if router not configured
    tracing::debug!("AI router not configured - returning minimal provider list");
    Ok(ListProvidersResponse {
        providers: vec![crate::rpc::types::ProviderInfo {
            id: "unconfigured".to_string(),
            name: "Unconfigured".to_string(),
            models: vec!["Configure AI providers to see models".to_string()],
            capabilities: vec!["text-generation".to_string()],
            online: false,
            avg_latency_ms: None,
            cost_tier: "free".to_string(),
        }],
        total: 1,
    })
}

/// Health check (internal, protocol-agnostic)
pub async fn handle_health_check_internal() -> Result<HealthCheckResponse, PrimalError> {
    tracing::debug!("Handling health_check");

    let (active_providers, requests_processed, avg_response_time) =
        if let Some(router) = get_ai_router() {
            let providers = router.list_providers().await;
            let active = providers.iter().filter(|p| p.is_available).count();

            // Calculate aggregate metrics from providers
            let total_requests: u64 = 0; // Router doesn't track this yet, would need metrics collector
            let avg_latency = if !providers.is_empty() {
                let sum: u64 = providers.iter().map(|p| p.avg_latency_ms).sum();
                Some((sum as f64) / (providers.len() as f64))
            } else {
                None
            };

            (active, total_requests, avg_latency)
        } else {
            (0, 0, None)
        };

    Ok(HealthCheckResponse {
        status: if active_providers > 0 {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        active_providers,
        requests_processed,
        avg_response_time_ms: avg_response_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_ai_internal() {
        let request = QueryAiRequest {
            prompt: "test".to_string(),
            provider: Some("test".to_string()),
            model: Some("test".to_string()),
            priority: None,
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        let result = handle_query_ai_internal(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_providers_internal() {
        let result = handle_list_providers_internal().await;
        assert!(result.is_ok());
        assert!(!result.unwrap().providers.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_internal() {
        let result = handle_health_check_internal().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        // Status is "degraded" when no providers are configured
        assert_eq!(response.status, "degraded");
        assert_eq!(response.active_providers, 0);
    }
}
