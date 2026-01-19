//! AI Tools for Squirrel MCP
//!
//! This crate provides AI provider integrations and routing capabilities.

// Allow deprecated items during error type migration to universal-error crate
#![allow(deprecated)]
// TODO(docs): Systematically add documentation to all public items (enum variants, struct fields)
// Currently 324 items need docs. This is tracked as part of Week 8 completion.
// Priority: Document high-traffic APIs first, then complete rest incrementally.
#![allow(missing_docs)]
// TODO: Fix all items_after_test_module warnings by moving implementations before test modules
// This is a structural issue affecting ~140 locations in this crate. Allowing temporarily while
// we prioritize more critical issues (error handling, hardcoding elimination).
// Will be fixed in systematic refactoring pass.
#![allow(clippy::items_after_test_module)]

// Capability-based AI client (TRUE PRIMAL!)
// Delegates AI HTTP calls to Songbird (network specialist)
// NO reqwest, NO ring! Pure Rust via Unix sockets!
pub mod capability_ai;

pub mod common;
pub mod config;
pub mod error;
pub mod router;

// Re-export commonly used types
pub use common::{
    create_provider_client, AIClient, ChatMessage, ChatRequest, ChatResponse, MessageRole,
    ModelParameters, RateLimiterConfig,
};
pub use config::{AIToolsConfig, ProviderConfig};
pub use error::{Error, Result};

/// Multi-model dispatch system for seamless AI model integration
pub mod dispatch {
    use super::*;
    use crate::common::capability::AITask;
    use crate::common::ChatResponseStream;
    use crate::router::RoutingStrategy;
    use crate::router::{AIRouter, RequestContext, RouterConfig, RoutingHint};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    /// Multi-model dispatcher that can route requests to different models
    /// in the same workflow, whether API-based or local
    pub struct MultiModelDispatcher {
        /// The AI router for intelligent model selection
        router: AIRouter,

        /// Configuration for different model types
        config: DispatcherConfig,
    }

    /// Configuration for the multi-model dispatcher
    #[derive(Debug, Clone)]
    pub struct DispatcherConfig {
        /// Router configuration
        pub router_config: RouterConfig,

        /// API keys for cloud providers
        pub api_keys: HashMap<String, String>,

        /// Default models for different task types
        pub default_models: HashMap<String, String>,

        /// Whether to prefer local models for sensitive data
        pub prefer_local_for_sensitive: bool,

        /// Whether to prefer API models for complex tasks
        pub prefer_api_for_complex: bool,
    }

    impl Default for DispatcherConfig {
        fn default() -> Self {
            let mut default_models = HashMap::new();
            default_models.insert("text_generation".to_string(), "gpt-3.5-turbo".to_string());
            default_models.insert("code_generation".to_string(), "gpt-4".to_string());
            default_models.insert("local_text_generation".to_string(), "llama3-8b".to_string());

            Self {
                router_config: RouterConfig::default(),
                api_keys: HashMap::new(),
                default_models,
                prefer_local_for_sensitive: true,
                prefer_api_for_complex: false,
            }
        }
    }

    impl MultiModelDispatcher {
        /// Create a new multi-model dispatcher
        pub async fn new(config: DispatcherConfig) -> Result<Self> {
            let router = AIRouter::new(config.router_config.clone());

            // Register cloud providers based on available API keys
            #[cfg(feature = "openai")]
            if let Some(api_key) = config.api_keys.get("openai") {
                let client = crate::openai::OpenAIClient::new(api_key.clone())?;
                router.register_provider("openai", Arc::new(client))?;
            }

            #[cfg(feature = "anthropic")]
            if let Some(api_key) = config.api_keys.get("anthropic") {
                let client = crate::anthropic::AnthropicClient::new(api_key.clone());
                router.register_provider("anthropic", Arc::new(client))?;
            }

            #[cfg(feature = "gemini")]
            if let Some(api_key) = config.api_keys.get("gemini") {
                let client = crate::gemini::GeminiClient::new(api_key.clone());
                router.register_provider("gemini", Arc::new(client))?;
            }

            Ok(Self { router, config })
        }

        /// Process a request with automatic model selection
        pub async fn process_request(
            &self,
            request: ChatRequest,
            task: AITask,
        ) -> Result<ChatResponse> {
            let context = self.create_request_context(task).await;
            self.router.process_request(request, context).await
        }

        /// Process a streaming request with automatic model selection
        pub async fn process_stream_request(
            &self,
            request: ChatRequest,
            task: AITask,
        ) -> Result<ChatResponseStream> {
            let context = self.create_request_context(task).await;
            self.router.process_stream_request(request, context).await
        }

        /// Process a request with explicit model preference
        pub async fn process_with_model_preference(
            &self,
            request: ChatRequest,
            task: AITask,
            preferred_provider: Option<String>,
            preferred_model: Option<String>,
        ) -> Result<ChatResponse> {
            let mut context = self.create_request_context(task).await;

            // Set routing hint for model preference
            context.routing_hint = Some(RoutingHint {
                preferred_provider,
                preferred_model,
                allow_remote: Some(true),
                max_latency_ms: None,
                max_cost_tier: None,
                priority: None,
            });

            self.router.process_request(request, context).await
        }

        /// Create a request context with intelligent routing hints
        async fn create_request_context(&self, task: AITask) -> RequestContext {
            let mut routing_hint = None;

            // Apply intelligent routing based on configuration and task characteristics
            if self.config.prefer_local_for_sensitive
                && task.security_requirements.contains_sensitive_data
            {
                routing_hint = Some(RoutingHint {
                    preferred_provider: Some("local".to_string()),
                    preferred_model: self
                        .config
                        .default_models
                        .get("local_text_generation")
                        .cloned(),
                    allow_remote: Some(false),
                    max_latency_ms: None,
                    max_cost_tier: Some(crate::common::capability::CostTier::Free),
                    priority: Some(90),
                });
            } else if self.config.prefer_api_for_complex && task.complexity_score.unwrap_or(50) > 80
            {
                routing_hint = Some(RoutingHint {
                    preferred_provider: Some("openai".to_string()),
                    preferred_model: Some("gpt-4".to_string()),
                    allow_remote: Some(true),
                    max_latency_ms: Some(10000), // 10 seconds for complex tasks
                    max_cost_tier: Some(crate::common::capability::CostTier::High),
                    priority: Some(95),
                });
            }

            RequestContext {
                request_id: Uuid::new_v4(),
                session_id: None,
                user_id: None,
                routing_hint,
                task,
                timestamp: std::time::Instant::now(),
            }
        }

        /// Get available models from all providers
        pub async fn list_all_available_models(&self) -> Result<HashMap<String, Vec<String>>> {
            let mut all_models = HashMap::new();

            for provider_id in self.router.registry().list_providers() {
                if let Some(provider) = self.router.registry().get_provider(&provider_id) {
                    match provider.list_models().await {
                        Ok(models) => {
                            all_models.insert(provider_id, models);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to list models for provider {}: {}",
                                provider_id,
                                e
                            );
                        }
                    }
                }
            }

            Ok(all_models)
        }

        /// Get the router for advanced usage
        pub fn router(&self) -> &AIRouter {
            &self.router
        }
    }

    /// Builder for creating a multi-model dispatcher with fluent API
    pub struct DispatcherBuilder {
        config: DispatcherConfig,
    }

    impl DispatcherBuilder {
        /// Create a new dispatcher builder
        pub fn new() -> Self {
            Self {
                config: DispatcherConfig::default(),
            }
        }

        /// Add an API key for a provider
        pub fn with_api_key(
            mut self,
            provider: impl Into<String>,
            api_key: impl Into<String>,
        ) -> Self {
            self.config.api_keys.insert(provider.into(), api_key.into());
            self
        }

        /// Set routing strategy
        pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
            self.config.router_config.routing_strategy = strategy;
            self
        }

        /// Prefer local models for sensitive data
        pub fn prefer_local_for_sensitive(mut self, prefer: bool) -> Self {
            self.config.prefer_local_for_sensitive = prefer;
            self
        }

        /// Prefer API models for complex tasks
        pub fn prefer_api_for_complex(mut self, prefer: bool) -> Self {
            self.config.prefer_api_for_complex = prefer;
            self
        }

        /// Build the dispatcher
        pub async fn build(self) -> Result<MultiModelDispatcher> {
            MultiModelDispatcher::new(self.config).await
        }
    }

    impl Default for DispatcherBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Client factory functions for easy instantiation
pub mod clients {
    use super::*;
    use std::sync::Arc;

    /// Create a new OpenAI client
    #[cfg(feature = "openai")]
    pub fn openai(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        Ok(Arc::new(crate::openai::OpenAIClient::new(api_key)?))
    }

    /// Create a new Anthropic client
    #[cfg(feature = "anthropic")]
    pub fn anthropic(api_key: impl Into<String>) -> Arc<dyn AIClient> {
        Arc::new(crate::anthropic::AnthropicClient::new(api_key))
    }

    /// Create a new Gemini client
    #[cfg(feature = "gemini")]
    pub fn gemini(api_key: impl Into<String>) -> Arc<dyn AIClient> {
        let client = crate::gemini::GeminiClient::new(api_key);
        Arc::new(client)
    }
}

/// Convenience functions for common workflows
pub mod workflows {
    use super::*;
    use crate::common::capability::{AITask, SecurityLevel, SecurityRequirements, TaskType};
    use crate::dispatch::{DispatcherBuilder, MultiModelDispatcher};
    use crate::router::RoutingStrategy;

    /// Create a dispatcher optimized for development workflows
    pub async fn create_dev_dispatcher() -> Result<MultiModelDispatcher> {
        DispatcherBuilder::new()
            .prefer_local_for_sensitive(true)
            .prefer_api_for_complex(false)
            .with_routing_strategy(RoutingStrategy::BestFit)
            .build()
            .await
    }

    /// Create a dispatcher optimized for production workflows
    pub async fn create_production_dispatcher(
        openai_key: Option<String>,
        anthropic_key: Option<String>,
    ) -> Result<MultiModelDispatcher> {
        let mut builder = DispatcherBuilder::new()
            .prefer_local_for_sensitive(true)
            .prefer_api_for_complex(true)
            .with_routing_strategy(RoutingStrategy::BestFit);

        if let Some(key) = openai_key {
            builder = builder.with_api_key("openai", key);
        }

        if let Some(key) = anthropic_key {
            builder = builder.with_api_key("anthropic", key);
        }

        builder.build().await
    }

    /// Process a simple text generation task with automatic model selection
    pub async fn generate_text(
        dispatcher: &MultiModelDispatcher,
        prompt: impl Into<String>,
        sensitive_data: bool,
    ) -> Result<String> {
        let request = ChatRequest {
            model: None,
            messages: vec![crate::common::ChatMessage {
                role: crate::common::MessageRole::User,
                content: Some(prompt.into()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            parameters: Some(crate::common::ModelParameters {
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

        let task = AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements {
                contains_sensitive_data: sensitive_data,
                requires_encryption: sensitive_data,
                requires_audit_logging: sensitive_data,
                security_level: if sensitive_data {
                    SecurityLevel::High
                } else {
                    SecurityLevel::Medium
                },
                geo_restrictions: None,
            },
            complexity_score: Some(30), // Simple text generation
            priority: 50,
        };

        let response = dispatcher.process_request(request, task).await?;

        Ok(response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.content)
            .unwrap_or_default())
    }
}
