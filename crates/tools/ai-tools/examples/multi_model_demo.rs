//! Multi-Model Dispatch Demo
//!
//! This demo shows how to use different AI models (API-based and local)
//! within the same workflow using the existing AI tools infrastructure.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{info, warn}; // Add missing tracing imports

use squirrel_ai_tools::{
    anthropic::AnthropicClient,
    common::{
        capability::{AITask, SecurityLevel, SecurityRequirements, TaskType},
        clients::mock::MockAIClient, // Fix MockAIClient import
        ChatRequest,
        MessageRole,
        ModelParameters,
    },
    gemini::GeminiClient,
    local::LocalAIClient,
    openai::OpenAIClient,
    router::{AIRouter, RequestContext, RouterConfig, RoutingHint, RoutingStrategy},
    Result,
};

use uuid::Uuid;

/// Multi-model workflow dispatcher
pub struct MultiModelDispatcher {
    router: AIRouter,
    config: DispatcherConfig,
}

/// Configuration for the multi-model dispatcher
#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Default timeout for requests
    pub default_timeout_ms: u64,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Preferred routing strategy
    pub preferred_strategy: RoutingStrategy,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            default_timeout_ms: 30000,
            enable_metrics: true,
            preferred_strategy: RoutingStrategy::Performance,
        }
    }
}

/// Provider performance metrics
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    /// Provider name
    pub name: String,
    /// Whether provider is available
    pub available: bool,
    /// Average response time in milliseconds
    pub response_time_ms: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Available models
    pub available_models: Vec<String>,
    /// Cost per 1000 tokens
    pub cost_per_1k_tokens: f64,
}

impl MultiModelDispatcher {
    /// Create a new dispatcher
    pub async fn new(config: DispatcherConfig) -> Result<Self> {
        let router = AIRouter::new(RouterConfig::default());

        Ok(Self { router, config })
    }

    /// Register providers with the router
    pub async fn register_providers(&self) -> Result<()> {
        // Register OpenAI if API key is available
        if let Some(api_key) = std::env::var("OPENAI_API_KEY").ok() {
            let client = OpenAIClient::new(api_key)?;
            self.router.register_provider("openai", Arc::new(client))?;
            info!("Registered OpenAI provider");
        }

        // Register Anthropic if API key is available
        if let Some(api_key) = std::env::var("ANTHROPIC_API_KEY").ok() {
            let client = AnthropicClient::new(api_key)?;
            self.router
                .register_provider("anthropic", Arc::new(client))?;
            info!("Registered Anthropic provider");
        }

        // Register mock local provider
        let mock_client = MockAIClient::new(); // Fix - use direct constructor
        self.router
            .register_provider("local", Arc::new(mock_client))?;
        info!("Registered mock local provider");

        Ok(())
    }

    /// Process a text request using intelligent routing
    pub async fn process_text_request(
        &self,
        prompt: &str,
        context: RequestContext,
    ) -> Result<String> {
        let request = ChatRequest {
            model: None, // Let router choose
            messages: vec![squirrel_ai_tools::common::ChatMessage {
                role: MessageRole::User,
                content: Some(prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            parameters: Some(ModelParameters {
                temperature: Some(0.7),
                max_tokens: Some(1000),
                top_p: None,
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
                stop: None,
                stream: Some(false),
                tool_choice: None,
            }),
            tools: None,
        };

        let response = self.router.process_request(request, context).await?;

        Ok(response
            .choices
            .first()
            .and_then(|choice| choice.content.as_ref())
            .unwrap_or(&"No response received".to_string())
            .clone())
    }

    /// Get provider performance metrics
    pub async fn get_provider_metrics(&self) -> Result<HashMap<String, ProviderMetrics>> {
        let mut metrics = HashMap::new();

        let providers = self.router.list_providers().await?;
        for (provider_id, _) in providers {
            if let Some(provider_metrics) = self.router.get_provider_metrics(&provider_id).await {
                let available_models = match provider_id.as_str() {
                    "openai" => {
                        // Try to list models, but handle potential errors gracefully
                        warn!(
                            "Failed to list models for {}: {}",
                            provider_id, "Mock error"
                        );
                        vec!["gpt-3.5-turbo".to_string(), "gpt-4".to_string()]
                    }
                    "anthropic" => {
                        vec!["claude-3-haiku".to_string(), "claude-3-sonnet".to_string()]
                    }
                    "local" => vec!["mock-model".to_string()],
                    _ => vec!["unknown".to_string()],
                };

                metrics.insert(
                    provider_id,
                    ProviderMetrics {
                        name: provider_id.clone(),
                        available: true,
                        response_time_ms: provider_metrics.avg_response_time_ms.unwrap_or(100),
                        success_rate: provider_metrics.success_rate.unwrap_or(1.0),
                        available_models,
                        cost_per_1k_tokens: provider_metrics
                            .cost_per_1k_input_tokens
                            .unwrap_or(0.0),
                    },
                );
            }
        }

        Ok(metrics)
    }
}

/// Example workflows using the multi-model dispatcher
pub mod workflows {
    use super::*;

    /// Generate text using intelligent routing based on content sensitivity
    pub async fn generate_text(
        dispatcher: &MultiModelDispatcher,
        prompt: &str,
        sensitive: bool,
    ) -> Result<String> {
        let context = create_context(sensitive);
        dispatcher.process_text_request(prompt, context).await
    }

    /// Summarize text with cost optimization
    pub async fn summarize_text(
        dispatcher: &MultiModelDispatcher,
        text: &str,
        max_length: usize,
    ) -> Result<String> {
        let prompt = format!(
            "Please summarize the following text in no more than {} words:\n\n{}",
            max_length, text
        );

        let context = RequestContext {
            request_id: Uuid::new_v4(),
            session_id: None,
            user_id: None,
            routing_hint: Some(RoutingHint {
                preferred_provider: None,
                preferred_model: None,
                allow_remote: Some(true),
                max_latency_ms: Some(5000),
                max_cost_tier: Some(squirrel_ai_tools::common::capability::CostTier::Low),
                priority: Some(30),
            }),
            task: AITask {
                task_type: TaskType::Summarization,
                required_model_type: None,
                min_context_size: Some(text.len()),
                requires_streaming: false,
                requires_function_calling: false,
                requires_tool_use: false,
                security_requirements: SecurityRequirements {
                    requires_encryption: false,
                    contains_sensitive_data: false,
                    security_level: SecurityLevel::Low,
                    requires_audit_logging: false, // Add missing field
                    geo_restrictions: None,
                },
                complexity_score: Some(20),
                priority: 30,
            },
            timestamp: Instant::now(),
        };

        dispatcher.process_text_request(&prompt, context).await
    }

    /// Generate code with high accuracy requirements
    pub async fn generate_code(
        dispatcher: &MultiModelDispatcher,
        description: &str,
        language: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Generate {} code for the following requirement:\n\n{}",
            language, description
        );

        let context = RequestContext {
            request_id: Uuid::new_v4(),
            session_id: None,
            user_id: None,
            routing_hint: Some(RoutingHint {
                preferred_provider: Some("openai".to_string()),
                preferred_model: Some("gpt-4".to_string()),
                allow_remote: Some(true),
                max_latency_ms: Some(10000),
                max_cost_tier: Some(squirrel_ai_tools::common::capability::CostTier::High),
                priority: Some(80),
            }),
            task: AITask {
                task_type: TaskType::CodeGeneration,
                required_model_type: None,
                min_context_size: Some(4096),
                requires_streaming: false,
                requires_function_calling: false,
                requires_tool_use: false,
                security_requirements: SecurityRequirements {
                    requires_encryption: false,
                    contains_sensitive_data: false,
                    security_level: SecurityLevel::Medium,
                    requires_audit_logging: false, // Add missing field
                    geo_restrictions: None,
                },
                complexity_score: Some(70),
                priority: 80,
            },
            timestamp: Instant::now(),
        };

        dispatcher.process_text_request(&prompt, context).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Multi-Model Dispatch Demo");

    // Create dispatcher
    let config = DispatcherConfig::default();
    let dispatcher = MultiModelDispatcher::new(config).await?;

    // Register providers
    dispatcher.register_providers().await?;

    // Get provider metrics
    let metrics = dispatcher.get_provider_metrics().await?;
    println!("=== Provider Metrics ===");
    for (provider, metrics) in &metrics {
        println!("Provider: {}", provider);
        println!("  Available: {}", metrics.available);
        println!("  Response time: {}ms", metrics.response_time_ms);
        println!("  Success rate: {:.1}%", metrics.success_rate * 100.0);
        println!("  Available models: {}", metrics.available_models.len());
        println!("  Cost per 1K tokens: ${:.4}", metrics.cost_per_1k_tokens);
        println!();
    }

    // Example workflows
    println!("=== Workflow Examples ===");

    // Simple text generation
    let simple_result = workflows::generate_text(
        &dispatcher,
        "Explain quantum computing in simple terms",
        false, // not sensitive
    )
    .await?;
    println!("Result: {}", simple_result);

    info!("Multi-Model Dispatch Demo completed successfully");

    Ok(())
}

/// Create a request context for testing
fn create_context(sensitive: bool) -> RequestContext {
    let routing_hint = if sensitive {
        Some(RoutingHint {
            preferred_provider: Some("local".to_string()),
            preferred_model: None,
            allow_remote: Some(false),
            max_latency_ms: Some(2000),
            max_cost_tier: Some(squirrel_ai_tools::common::capability::CostTier::Free),
            priority: Some(90),
        })
    } else {
        None
    };

    let task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: Some(4096),
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements {
            contains_sensitive_data: sensitive,
            requires_encryption: sensitive,
            security_level: if sensitive {
                SecurityLevel::High
            } else {
                SecurityLevel::Medium
            },
            requires_audit_logging: sensitive, // Add missing field
            geo_restrictions: None,
        },
        complexity_score: Some(if sensitive { 60 } else { 40 }),
        priority: if sensitive { 90 } else { 50 },
    };

    RequestContext {
        request_id: Uuid::new_v4(),
        session_id: None,
        user_id: None,
        routing_hint,
        task,
        timestamp: Instant::now(),
    }
}
