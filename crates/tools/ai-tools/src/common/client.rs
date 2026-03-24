// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI client trait and related functionality
//!
//! This module defines the core `AIClient` trait that provides a unified interface
//! for interacting with various AI providers (cloud APIs, local servers, etc.).

use async_trait::async_trait;

use crate::common::capability::{AICapabilities, AITask, TaskType};
use crate::common::types::{ChatRequest, ChatResponse, ChatResponseStream};
use crate::float_helpers;

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
        if let Some(model_type) = &task.required_model_type
            && !capabilities.supports_model_type(model_type)
        {
            return false;
        }

        // Check context size requirements
        if let Some(required_size) = task.min_context_size
            && capabilities.max_context_size < required_size
        {
            return false;
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
            TaskType::TextGeneration
            | TaskType::ChatCompletion
            | TaskType::Other
            | TaskType::Custom(_) => 0.01,
            TaskType::CodeGeneration | TaskType::DataAnalysis => 0.02,
            TaskType::Translation => 0.015,
            TaskType::Summarization => 0.008,
            TaskType::QuestionAnswering => 0.012,
            TaskType::FunctionCalling | TaskType::FunctionExecution => 0.03,
            TaskType::ImageGeneration => 0.2,
            TaskType::ImageAnalysis | TaskType::ImageUnderstanding => 0.15,
            TaskType::AudioTranscription => 0.05,
            TaskType::AudioGeneration | TaskType::SpeechSynthesis => 0.08,
            TaskType::TextEmbedding | TaskType::Embedding => 0.001,
            TaskType::Classification => 0.005,
            TaskType::Sentiment => 0.003,
            TaskType::NamedEntityRecognition => 0.004,
        };

        // Adjust based on estimated token usage
        let token_multiplier = task.min_context_size.map_or(1.0f64, |min_tokens| {
            float_helpers::usize_to_f64_lossy(min_tokens) / 1000.0
        });

        base_cost * token_multiplier
    }

    /// Get health score for this client
    async fn health_score(&self) -> f64 {
        // Simple health check - providers can override with more sophisticated logic
        if self.is_available().await { 1.0 } else { 0.0 }
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
    ///
    /// # Errors
    ///
    /// Returns `Err` with a human-readable message when validation fails.
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
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        float_helpers::u64_ratio(self.successful_requests, self.total_requests)
    }

    /// Calculate failure rate
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        float_helpers::u64_ratio(self.failed_requests, self.total_requests)
    }

    /// Calculate average cost per request
    #[must_use]
    pub fn avg_cost_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_cost / float_helpers::u64_to_f64_lossy(self.total_requests)
        }
    }

    /// Calculate average tokens per request
    #[must_use]
    pub fn avg_tokens_per_request(&self) -> f64 {
        float_helpers::u64_ratio(self.total_tokens, self.total_requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ClientMetrics tests ---
    #[test]
    fn test_client_metrics_default() {
        let metrics = ClientMetrics::default();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 0);
        assert!((metrics.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.total_tokens, 0);
        assert!((metrics.total_cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_success_rate_empty() {
        let metrics = ClientMetrics::default();
        assert!((metrics.success_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_success_rate() {
        let metrics = ClientMetrics {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            avg_response_time_ms: 150.0,
            total_tokens: 10000,
            total_cost: 1.5,
        };
        assert!((metrics.success_rate() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_failure_rate_empty() {
        let metrics = ClientMetrics::default();
        assert!((metrics.failure_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_failure_rate() {
        let metrics = ClientMetrics {
            total_requests: 100,
            successful_requests: 80,
            failed_requests: 20,
            avg_response_time_ms: 200.0,
            total_tokens: 5000,
            total_cost: 0.75,
        };
        assert!((metrics.failure_rate() - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_avg_cost_per_request_empty() {
        let metrics = ClientMetrics::default();
        assert!((metrics.avg_cost_per_request() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_avg_cost_per_request() {
        let metrics = ClientMetrics {
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            avg_response_time_ms: 100.0,
            total_tokens: 1000,
            total_cost: 5.0,
        };
        assert!((metrics.avg_cost_per_request() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_avg_tokens_per_request_empty() {
        let metrics = ClientMetrics::default();
        assert!((metrics.avg_tokens_per_request() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_avg_tokens_per_request() {
        let metrics = ClientMetrics {
            total_requests: 5,
            successful_requests: 5,
            failed_requests: 0,
            avg_response_time_ms: 50.0,
            total_tokens: 2500,
            total_cost: 0.25,
        };
        assert!((metrics.avg_tokens_per_request() - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_metrics_all_failed() {
        let metrics = ClientMetrics {
            total_requests: 10,
            successful_requests: 0,
            failed_requests: 10,
            avg_response_time_ms: 5000.0,
            total_tokens: 0,
            total_cost: 0.0,
        };
        assert!((metrics.success_rate() - 0.0).abs() < f64::EPSILON);
        assert!((metrics.failure_rate() - 1.0).abs() < f64::EPSILON);
    }

    // --- HealthCheckResult tests ---
    #[test]
    fn test_health_check_result_available() {
        let result = HealthCheckResult {
            available: true,
            response_time_ms: 42,
            error: None,
            metadata: std::collections::HashMap::new(),
        };
        assert!(result.available);
        assert_eq!(result.response_time_ms, 42);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_health_check_result_unavailable() {
        let result = HealthCheckResult {
            available: false,
            response_time_ms: 0,
            error: Some("Connection refused".to_string()),
            metadata: std::collections::HashMap::new(),
        };
        assert!(!result.available);
        assert_eq!(result.error.as_deref(), Some("Connection refused"));
    }

    #[test]
    fn test_health_check_result_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("region".to_string(), "us-east".to_string());

        let result = HealthCheckResult {
            available: true,
            response_time_ms: 100,
            error: None,
            metadata,
        };
        assert_eq!(result.metadata.len(), 2);
        assert_eq!(
            result.metadata.get("version").expect("should succeed"),
            "1.0.0"
        );
    }

    #[test]
    fn test_health_check_result_clone() {
        let result = HealthCheckResult {
            available: true,
            response_time_ms: 50,
            error: None,
            metadata: std::collections::HashMap::new(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.available, result.available);
        assert_eq!(cloned.response_time_ms, result.response_time_ms);
    }
}
