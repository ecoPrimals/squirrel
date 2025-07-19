//! AI client trait and related functionality
//!
//! This module defines the core AIClient trait that provides a unified interface
//! for interacting with various AI providers (OpenAI, Anthropic, Ollama, etc.).

use async_trait::async_trait;

use crate::common::capability::{AICapabilities, AITask, ModelType, TaskType};
use crate::common::types::{ChatRequest, ChatResponse, ChatResponseStream};

/// AI client trait for unified interface across providers
#[async_trait]
pub trait AIClient: Send + Sync + std::fmt::Debug + 'static {
    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get model capabilities
    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities>;

    /// Send a chat request
    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse>;

    /// Send a streaming chat request
    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream>;

    /// List available models
    async fn list_models(&self) -> crate::Result<Vec<String>>;

    /// Check if the client is available
    async fn is_available(&self) -> bool;

    /// Get the default model name
    fn default_model(&self) -> &str;

    /// Get the client as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get capabilities of this AI client
    fn capabilities(&self) -> AICapabilities {
        AICapabilities::default()
    }

    /// Check if this client can handle a specific task
    fn can_handle_task(&self, task: &AITask) -> bool {
        let capabilities = self.capabilities();

        // Check basic task type support
        if !capabilities.supports_task(&task.task_type) {
            return false;
        }

        // Check model type requirements
        if let Some(model_type) = &task.required_model_type {
            if !capabilities.supports_model_type(model_type) {
                return false;
            }
        }

        // Check context size requirements
        if let Some(required_size) = task.min_context_size {
            if capabilities.max_context_size < required_size {
                return false;
            }
        }

        // Check streaming support
        if task.requires_streaming && !capabilities.supports_streaming {
            return false;
        }

        // Check function calling support
        if task.requires_function_calling && !capabilities.supports_function_calling {
            return false;
        }

        true
    }

    /// Get cost estimate for a task
    fn estimate_cost(&self, task: &AITask) -> f64 {
        // Base cost estimation - providers can override
        let base_cost = match task.task_type {
            TaskType::TextGeneration => 0.01,
            TaskType::CodeGeneration => 0.02,
            TaskType::Translation => 0.015,
            TaskType::Summarization => 0.008,
            TaskType::QuestionAnswering => 0.012,
            TaskType::ChatCompletion => 0.01,
            TaskType::FunctionCalling => 0.03,
            TaskType::ImageGeneration => 0.2,
            TaskType::ImageAnalysis => 0.15,
            TaskType::ImageUnderstanding => 0.15,
            TaskType::AudioTranscription => 0.05,
            TaskType::AudioGeneration => 0.08,
            TaskType::SpeechSynthesis => 0.08,
            TaskType::TextEmbedding => 0.001,
            TaskType::Embedding => 0.001,
            TaskType::Classification => 0.005,
            TaskType::Sentiment => 0.003,
            TaskType::NamedEntityRecognition => 0.004,
            TaskType::DataAnalysis => 0.02,
            TaskType::FunctionExecution => 0.03,
            TaskType::Other => 0.01,
            TaskType::Custom(_) => 0.01,
        };

        // Adjust based on estimated token usage
        let token_multiplier = if let Some(min_tokens) = task.min_context_size {
            (min_tokens as f64) / 1000.0
        } else {
            1.0
        };

        base_cost * token_multiplier
    }

    /// Get health score for this client
    async fn health_score(&self) -> f64 {
        // Simple health check - providers can override with more sophisticated logic
        if self.is_available().await {
            1.0
        } else {
            0.0
        }
    }

    /// Get priority for this client (higher = more preferred)
    fn priority(&self) -> u32 {
        // Default priority - providers can override
        100
    }

    /// Get routing preferences for this client
    fn routing_preferences(&self) -> crate::common::capability::RoutingPreferences {
        crate::common::capability::RoutingPreferences::default()
    }
}

/// Helper trait for client factory
pub trait AIClientFactory {
    /// Create a new client instance
    fn create_client(&self) -> Box<dyn AIClient>;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Check if the factory can create a client with current configuration
    fn can_create(&self) -> bool;
}

/// Client configuration trait
pub trait ClientConfig {
    /// Validate the configuration
    fn validate(&self) -> Result<(), String>;

    /// Get the provider name for this configuration
    fn provider_name(&self) -> &str;

    /// Check if the configuration is complete
    fn is_complete(&self) -> bool;
}

/// Client health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether the client is available
    pub available: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error message if unavailable
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Client metrics
#[derive(Debug, Clone)]
pub struct ClientMetrics {
    /// Total requests made
    pub total_requests: u64,
    /// Total successful requests
    pub successful_requests: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
}

impl Default for ClientMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
        }
    }
}

impl ClientMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }

    /// Calculate average cost per request
    pub fn avg_cost_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_cost / self.total_requests as f64
        }
    }

    /// Calculate average tokens per request
    pub fn avg_tokens_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_tokens as f64 / self.total_requests as f64
        }
    }
}
