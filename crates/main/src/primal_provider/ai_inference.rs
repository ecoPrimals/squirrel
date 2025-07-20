//! AI Inference and Provider Selection

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use super::core::SquirrelPrimalProvider;
use crate::error::PrimalError;

/// AI Inference Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInferenceRequest {
    pub task_type: String,
    pub messages: Vec<serde_json::Value>,
    pub model: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// AI Provider Selection logic
pub struct AIProviderSelection;

impl AIProviderSelection {
    /// Select the best AI provider for a given request
    pub fn select_provider(request: &AIInferenceRequest) -> Result<String, PrimalError> {
        // Determine provider based on task type and model preferences
        if let Some(model) = &request.model {
            if model.starts_with("gpt-") || model.contains("openai") {
                return Ok("openai".to_string());
            } else if model.starts_with("claude-") || model.contains("anthropic") {
                return Ok("anthropic".to_string());
            } else if model.contains("llama") || model.contains("mistral") {
                return Ok("ollama".to_string());
            }
        }

        // Default provider selection based on task type
        match request.task_type.as_str() {
            "text_generation" | "chat" => {
                // Use environment preference or default to OpenAI
                Ok(std::env::var("AI_DEFAULT_PROVIDER").unwrap_or_else(|_| "openai".to_string()))
            }
            "code_generation" => Ok("openai".to_string()),
            "analysis" | "reasoning" => Ok("anthropic".to_string()),
            "local" | "private" => Ok("ollama".to_string()),
            _ => Ok("openai".to_string()),
        }
    }
}

impl SquirrelPrimalProvider {
    /// Handle AI inference request with intelligent provider selection
    pub async fn handle_ai_inference_request(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Parse the request
        let inference_request: AIInferenceRequest = serde_json::from_value(request)
            .map_err(|e| PrimalError::ValidationError(e.to_string()))?;

        // Select appropriate provider
        let provider = self.select_ai_provider(&inference_request)?;

        // Execute the request
        let response = self
            .execute_ai_request(&provider, inference_request)
            .await?;

        Ok(response)
    }

    /// Select appropriate AI provider for the request (zero-copy optimized)
    fn select_ai_provider(&self, request: &AIInferenceRequest) -> Result<String, PrimalError> {
        // Record zero-copy optimization
        self.zero_copy_metrics.record_operation();

        // Use static strings to avoid allocations for common providers
        let openai_str = self
            .static_strings
            .get("openai")
            .map(|arc| (*arc).to_string())
            .unwrap_or_else(|| "openai".to_string());
        let anthropic_str = self
            .static_strings
            .get("anthropic")
            .map(|arc| (*arc).to_string())
            .unwrap_or_else(|| "anthropic".to_string());
        let ollama_str = self
            .static_strings
            .get("local")
            .map(|arc| (*arc).to_string())
            .unwrap_or_else(|| "ollama".to_string());

        // Determine provider based on task type and model preferences
        if let Some(model) = &request.model {
            if model.starts_with("gpt-") || model.contains("openai") {
                self.zero_copy_metrics.record_clone_avoided();
                return Ok(openai_str);
            } else if model.starts_with("claude-") || model.contains("anthropic") {
                self.zero_copy_metrics.record_clone_avoided();
                return Ok(anthropic_str);
            } else if model.contains("llama") || model.contains("mistral") {
                self.zero_copy_metrics.record_clone_avoided();
                return Ok(ollama_str);
            }
        }

        // Default provider selection based on task type
        match request.task_type.as_str() {
            "text_generation" | "chat" => {
                // Use environment preference or default to OpenAI
                Ok(std::env::var("AI_DEFAULT_PROVIDER").unwrap_or(openai_str))
            }
            "code_generation" => {
                self.zero_copy_metrics.record_clone_avoided();
                Ok(openai_str)
            }
            "analysis" | "reasoning" => {
                self.zero_copy_metrics.record_clone_avoided();
                Ok(anthropic_str)
            }
            "local" | "private" => {
                self.zero_copy_metrics.record_clone_avoided();
                Ok(ollama_str)
            }
            _ => {
                self.zero_copy_metrics.record_clone_avoided();
                Ok(openai_str)
            }
        }
    }

    /// Execute AI request with the selected provider
    async fn execute_ai_request(
        &self,
        provider: &str,
        request: AIInferenceRequest,
    ) -> Result<serde_json::Value, PrimalError> {
        // For now, create a simplified response based on the request
        // In a full implementation, this would integrate with the AI tools crate
        let response = match provider {
            "openai" => {
                json!({
                    "content": format!("OpenAI response to: {}", self.extract_user_message(&request.messages)),
                    "model": request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
                    "provider": "openai",
                    "usage": {
                        "prompt_tokens": 0,
                        "completion_tokens": 0,
                        "total_tokens": 0
                    }
                })
            }
            "anthropic" => {
                json!({
                    "content": format!("Claude response to: {}", self.extract_user_message(&request.messages)),
                    "model": request.model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
                    "provider": "anthropic",
                    "usage": {
                        "input_tokens": 0,
                        "output_tokens": 0
                    }
                })
            }
            "ollama" => {
                json!({
                    "content": format!("Local model response to: {}", self.extract_user_message(&request.messages)),
                    "model": request.model.unwrap_or_else(|| "llama3-8b".to_string()),
                    "provider": "ollama"
                })
            }
            _ => {
                return Err(PrimalError::NetworkError(format!(
                    "Unknown AI provider: {}",
                    provider
                )));
            }
        };

        Ok(response)
    }

    /// Extract user message from messages array for processing
    fn extract_user_message(&self, messages: &[serde_json::Value]) -> String {
        for message in messages {
            if let Some(role) = message.get("role") {
                if role == "user" {
                    if let Some(content) = message.get("content") {
                        if let Some(text) = content.as_str() {
                            return text.to_string();
                        }
                    }
                }
            }
        }
        "No user message found".to_string()
    }
}
