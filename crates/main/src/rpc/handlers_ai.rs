// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI domain JSON-RPC handlers — `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::{
    ListProvidersResponse, ProviderInfo, QueryAiRequest, QueryAiResponse, SignalPlanResponse,
    SignalPlanStep, SignalToolDef,
};
use serde_json::Value;
use std::time::Instant;
use tracing::info;

impl JsonRpcServer {
    /// Handle `ai.query` / `query_ai` method.
    ///
    /// When `mode` = `"signal_plan"`, decomposes the prompt intent into
    /// a structured sequence of atomic signal calls using the tool schema.
    pub(crate) async fn handle_query_ai(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: QueryAiRequest = self.parse_params(params)?;

        if request.mode.as_deref() == Some("signal_plan") {
            return self.handle_signal_plan(request).await;
        }

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

    /// Decompose an intent into a structured signal execution plan.
    ///
    /// Formats the available signal tools into a function-calling prompt,
    /// sends to the AI provider, and parses the response into a sequence
    /// of `SignalPlanStep` entries that can be executed via `signal.dispatch`.
    async fn handle_signal_plan(&self, request: QueryAiRequest) -> Result<Value, JsonRpcError> {
        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "AI router not configured for signal planning".to_string(),
            data: None,
        })?;

        let tools = self.resolve_signal_tools(&request)?;
        if tools.is_empty() {
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "No signal tools available. Provide tools or tool_schema.".to_string(),
                data: None,
            });
        }

        let tools_description = tools
            .iter()
            .map(|t| {
                format!(
                    "- {name} (tier: {tier}): {desc}",
                    name = t.name,
                    tier = t.tier,
                    desc = t.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = format!(
            "You are an ecosystem orchestration planner. Given the user's intent, \
             decompose it into a sequence of atomic signal calls.\n\n\
             Available signals:\n{tools_description}\n\n\
             Respond ONLY with a JSON array of steps, each with: \
             {{\"tier\": \"...\", \"signal\": \"...\", \"params\": {{}}, \"reason\": \"...\"}}\n\
             No markdown fences, no explanation outside the JSON."
        );

        let full_prompt = match &request.context {
            Some(ctx) => format!(
                "Context: {ctx}\n\nIntent: {prompt}",
                prompt = request.prompt
            ),
            None => request.prompt.clone(),
        };

        use crate::api::ai::types::TextGenerationRequest;
        let ai_request = TextGenerationRequest {
            prompt: full_prompt,
            system: Some(system_prompt),
            max_tokens: request.max_tokens.map_or(2048, |v| v as u32),
            temperature: request.temperature.unwrap_or(0.3),
            model: request.model,
            constraints: vec![],
            params: std::collections::HashMap::new(),
        };

        let start = Instant::now();
        let ai_response =
            router
                .generate_text(ai_request, None)
                .await
                .map_err(|e| JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("AI router error during signal planning: {e}"),
                    data: None,
                })?;

        let plan = Self::parse_signal_plan(&ai_response.text)?;

        let response = SignalPlanResponse {
            plan,
            reasoning: ai_response.text,
            provider: ai_response.provider_id,
            model: ai_response.model,
            latency_ms: start.elapsed().as_millis() as u64,
            success: true,
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }

    /// Resolve signal tools from request (inline tools > tool_schema file).
    fn resolve_signal_tools(
        &self,
        request: &QueryAiRequest,
    ) -> Result<Vec<SignalToolDef>, JsonRpcError> {
        if let Some(ref tools) = request.tools {
            return Ok(tools.clone());
        }

        if let Some(ref schema_ref) = request.tool_schema {
            let path = std::path::Path::new(schema_ref);
            if path.exists() {
                let content = std::fs::read_to_string(path).map_err(|e| JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("Failed to read tool_schema: {e}"),
                    data: None,
                })?;
                return Self::parse_signal_tools_toml(&content);
            }
        }

        Ok(Vec::new())
    }

    /// Parse signal_tools.toml into SignalToolDef entries.
    fn parse_signal_tools_toml(content: &str) -> Result<Vec<SignalToolDef>, JsonRpcError> {
        let parsed: toml::Value = toml::from_str(content).map_err(|e| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Failed to parse signal_tools.toml: {e}"),
            data: None,
        })?;

        let tools_arr = parsed
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "signal_tools.toml missing [[tools]] array".to_string(),
                data: None,
            })?;

        let mut defs = Vec::with_capacity(tools_arr.len());
        for tool in tools_arr {
            let name = tool
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tier = tool
                .get("tier")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();
            let description = tool
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            let parameters = tool
                .get("parameters")
                .and_then(|p| serde_json::to_value(p).ok());
            defs.push(SignalToolDef {
                name,
                tier,
                description,
                parameters,
            });
        }

        Ok(defs)
    }

    /// Parse AI response text into structured signal plan steps.
    fn parse_signal_plan(response_text: &str) -> Result<Vec<SignalPlanStep>, JsonRpcError> {
        let trimmed = response_text.trim();
        // Strip markdown code fences if present
        let json_str = if trimmed.starts_with("```") {
            trimmed
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
        } else {
            trimmed
        };

        serde_json::from_str(json_str).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to parse signal plan from AI response: {e}. Raw: {json_str}"),
            data: None,
        })
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
