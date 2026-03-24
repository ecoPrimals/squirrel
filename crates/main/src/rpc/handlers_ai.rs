// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI domain JSON-RPC handlers — `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::{ListProvidersResponse, ProviderInfo, QueryAiRequest, QueryAiResponse};
use serde_json::Value;
use std::time::Instant;
use tracing::info;

impl JsonRpcServer {
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
}
