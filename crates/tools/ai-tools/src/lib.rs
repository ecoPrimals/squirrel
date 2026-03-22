// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! AI Tools for Squirrel MCP
//!
//! This crate provides AI provider integrations and routing capabilities.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
// Allow deprecated items during error type migration to universal-error crate
#![allow(deprecated)]
// NOTE: items_after_test_module warnings - move implementations before test modules when refactoring
// This is a structural issue affecting ~140 locations in this crate. Allowing temporarily while
// we prioritize more critical issues (error handling, hardcoding elimination).
// Will be fixed in systematic refactoring pass.
#![allow(
    unused_imports,
    clippy::items_after_test_module,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unused_async,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::use_self,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions,
    clippy::redundant_else,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::significant_drop_tightening,
    clippy::option_if_let_else,
    clippy::single_match_else,
    clippy::manual_string_new,
    clippy::or_fun_call,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    clippy::struct_excessive_bools,
    clippy::match_same_arms,
    clippy::cast_precision_loss,
    clippy::wildcard_imports,
    clippy::unnecessary_wraps,
    clippy::cast_lossless,
    clippy::unused_self,
    clippy::too_many_lines,
    clippy::redundant_clone,
    clippy::suboptimal_flops,
    clippy::too_long_first_doc_paragraph,
    clippy::useless_let_if_seq,
    clippy::unnecessary_literal_bound,
    clippy::ignored_unit_patterns,
    clippy::assigning_clones,
    clippy::branches_sharing_code,
    clippy::cloned_instead_of_copied,
    clippy::unreadable_literal,
    clippy::needless_raw_string_hashes
)]

// Capability-based AI client (TRUE PRIMAL!)
// Delegates AI HTTP calls via capability discovery (network specialist)
// NO reqwest, NO ring! Pure Rust via Unix sockets!
pub mod capability_ai;

// Capability-based HTTP client (TRUE PRIMAL - agnostic network delegation!)
// Discovers http.client capability - could be Songbird, could be ANY network primal!
pub mod capability_http;

// Neural API HTTP client (TRUE PRIMAL via Ecosystem Routing!)
// Uses squirrel's own IPC client (primal autonomy - no shared IPC crates!)
// NO reqwest, NO ring! 100% Pure Rust via ecosystem routing!
pub mod neural_http;

mod ipc_routed_providers;

pub mod common;
pub mod config;
pub mod error;
pub mod router;

// Re-export commonly used types
pub use common::{
    AIClient, ChatMessage, ChatRequest, ChatResponse, MessageRole, ModelParameters,
    RateLimiterConfig, create_provider_client,
};
pub use config::{AIToolsConfig, ProviderConfig};
pub use error::{Error, Result};

/// Multi-model dispatch system for seamless AI model integration
pub mod dispatch {
    use super::*;
    use crate::common::ChatResponseStream;
    use crate::common::capability::AITask;
    use crate::router::RoutingStrategy;
    use crate::router::{AIRouter, RequestContext, RouterConfig, RoutingHint};
    use std::collections::HashMap;

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

            // Vendor-specific direct HTTP modules are optional; production routes via IPC
            // (`ipc_routed_providers` / `neural_http`). Register OpenRouter/HuggingFace/etc.
            // manually when using the provider plugin registry.

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

    /// Create an OpenAI-compatible client that sends traffic through the ecosystem IPC HTTP proxy.
    #[cfg(feature = "openai")]
    pub fn openai(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        ipc_routed_providers::IpcRoutedVendorClient::try_new(
            api_key,
            ipc_routed_providers::VendorKind::OpenAI,
        )
    }

    /// Anthropic Messages API via IPC-delegated HTTP (`neural_api.proxy_http`).
    #[cfg(feature = "anthropic")]
    pub fn anthropic(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        ipc_routed_providers::IpcRoutedVendorClient::try_new(
            api_key,
            ipc_routed_providers::VendorKind::Anthropic,
        )
    }

    /// Google Gemini `generateContent` via IPC-delegated HTTP.
    #[cfg(feature = "gemini")]
    pub fn gemini(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        ipc_routed_providers::IpcRoutedVendorClient::try_new(
            api_key,
            ipc_routed_providers::VendorKind::Gemini,
        )
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
