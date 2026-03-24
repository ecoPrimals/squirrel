// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
// Allow deprecated items during error type migration to universal-error crate
#![expect(
    deprecated,
    reason = "Error type migration to universal-error crate in progress"
)]

//! AI Tools for Squirrel MCP
//!
//! This crate provides AI provider integrations and routing capabilities.

pub(crate) mod float_helpers;

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

#[cfg(any(feature = "openai", feature = "anthropic", feature = "gemini"))]
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
    use super::{ChatRequest, ChatResponse, Result};
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
        ///
        /// # Errors
        ///
        /// Currently infallible at construction; reserved for future validation failures.
        pub fn new(config: DispatcherConfig) -> Result<Self> {
            let router = AIRouter::new(config.router_config.clone());

            // Vendor-specific direct HTTP modules are optional; production routes via IPC
            // (`ipc_routed_providers` / `neural_http`). Register OpenRouter/HuggingFace/etc.
            // manually when using the provider plugin registry.

            Ok(Self { router, config })
        }

        /// Process a request with automatic model selection
        ///
        /// # Errors
        ///
        /// Propagates errors from the underlying [`AIRouter::process_request`].
        pub async fn process_request(
            &self,
            request: ChatRequest,
            task: AITask,
        ) -> Result<ChatResponse> {
            let context = self.create_request_context(task);
            self.router.process_request(request, context).await
        }

        /// Process a streaming request with automatic model selection
        ///
        /// # Errors
        ///
        /// Propagates errors from the underlying [`AIRouter::process_stream_request`].
        pub async fn process_stream_request(
            &self,
            request: ChatRequest,
            task: AITask,
        ) -> Result<ChatResponseStream> {
            let context = self.create_request_context(task);
            self.router.process_stream_request(request, context).await
        }

        /// Process a request with explicit model preference
        ///
        /// # Errors
        ///
        /// Propagates errors from the underlying [`AIRouter::process_request`].
        pub async fn process_with_model_preference(
            &self,
            request: ChatRequest,
            task: AITask,
            preferred_provider: Option<String>,
            preferred_model: Option<String>,
        ) -> Result<ChatResponse> {
            let mut context = self.create_request_context(task);

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
        fn create_request_context(&self, task: AITask) -> RequestContext {
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
        ///
        /// # Errors
        ///
        /// Returns `Ok` with models grouped by provider. Per-provider failures are logged and
        /// skipped.
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
        #[must_use]
        pub const fn router(&self) -> &AIRouter {
            &self.router
        }
    }

    /// Builder for creating a multi-model dispatcher with fluent API
    pub struct DispatcherBuilder {
        config: DispatcherConfig,
    }

    impl DispatcherBuilder {
        /// Create a new dispatcher builder
        #[must_use]
        pub fn new() -> Self {
            Self {
                config: DispatcherConfig::default(),
            }
        }

        /// Add an API key for a provider
        #[must_use]
        pub fn with_api_key(
            mut self,
            provider: impl Into<String>,
            api_key: impl Into<String>,
        ) -> Self {
            self.config.api_keys.insert(provider.into(), api_key.into());
            self
        }

        /// Set routing strategy
        #[must_use]
        #[expect(
            clippy::missing_const_for_fn,
            reason = "Mutates DispatcherConfig (HashMap/strategy); not const"
        )]
        pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
            self.config.router_config.routing_strategy = strategy;
            self
        }

        /// Prefer local models for sensitive data
        #[must_use]
        #[expect(
            clippy::missing_const_for_fn,
            reason = "Mutates DispatcherConfig; not const"
        )]
        pub fn prefer_local_for_sensitive(mut self, prefer: bool) -> Self {
            self.config.prefer_local_for_sensitive = prefer;
            self
        }

        /// Prefer API models for complex tasks
        #[must_use]
        #[expect(
            clippy::missing_const_for_fn,
            reason = "Mutates DispatcherConfig; not const"
        )]
        pub fn prefer_api_for_complex(mut self, prefer: bool) -> Self {
            self.config.prefer_api_for_complex = prefer;
            self
        }

        /// Build the dispatcher
        ///
        /// # Errors
        ///
        /// Propagates construction errors from [`MultiModelDispatcher::new`].
        pub fn build(self) -> Result<MultiModelDispatcher> {
            MultiModelDispatcher::new(self.config)
        }
    }

    impl Default for DispatcherBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Client factory functions for easy instantiation.
///
/// Each vendor client delegates HTTP via the ecosystem IPC proxy
/// (`neural_api.proxy_http`) — no direct `reqwest`/`ring` dependency.
pub mod clients {
    #[cfg(any(feature = "openai", feature = "anthropic", feature = "gemini"))]
    use std::sync::Arc;

    #[cfg(any(feature = "openai", feature = "anthropic", feature = "gemini"))]
    use crate::{AIClient, Result, ipc_routed_providers};

    /// Create an OpenAI-compatible client routed through the ecosystem IPC HTTP proxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the IPC-routed vendor client fails to initialize.
    #[cfg(feature = "openai")]
    pub fn openai(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        ipc_routed_providers::IpcRoutedVendorClient::try_new(
            api_key,
            ipc_routed_providers::VendorKind::OpenAI,
        )
    }

    /// Anthropic Messages API routed through the ecosystem IPC HTTP proxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the IPC-routed vendor client fails to initialize.
    #[cfg(feature = "anthropic")]
    pub fn anthropic(api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
        ipc_routed_providers::IpcRoutedVendorClient::try_new(
            api_key,
            ipc_routed_providers::VendorKind::Anthropic,
        )
    }

    /// Google Gemini `generateContent` routed through the ecosystem IPC HTTP proxy.
    ///
    /// # Errors
    ///
    /// Returns an error if the IPC-routed vendor client fails to initialize.
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
    use super::{ChatRequest, Result};
    use crate::common::capability::{AITask, SecurityLevel, SecurityRequirements, TaskType};
    use crate::dispatch::{DispatcherBuilder, MultiModelDispatcher};
    use crate::router::RoutingStrategy;

    /// Create a dispatcher optimized for development workflows
    ///
    /// # Errors
    ///
    /// Propagates errors from [`DispatcherBuilder::build`].
    pub fn create_dev_dispatcher() -> Result<MultiModelDispatcher> {
        DispatcherBuilder::new()
            .prefer_local_for_sensitive(true)
            .prefer_api_for_complex(false)
            .with_routing_strategy(RoutingStrategy::BestFit)
            .build()
    }

    /// Create a dispatcher optimized for production workflows
    ///
    /// # Errors
    ///
    /// Propagates errors from [`DispatcherBuilder::build`].
    pub fn create_production_dispatcher(
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

        builder.build()
    }

    /// Process a simple text generation task with automatic model selection
    ///
    /// # Errors
    ///
    /// Propagates errors from [`MultiModelDispatcher::process_request`].
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

#[cfg(test)]
mod lib_dispatch_tests {
    use std::sync::Arc;

    use crate::common::capability::{AITask, SecurityRequirements, TaskType};
    use crate::common::{ChatRequest, MockAIClient};
    use crate::dispatch::{DispatcherBuilder, DispatcherConfig};
    use crate::router::{RequestContext, RoutingStrategy};

    #[test]
    fn dispatcher_config_default_populates_models_and_flags() {
        let c = DispatcherConfig::default();
        assert!(c.default_models.contains_key("text_generation"));
        assert!(c.prefer_local_for_sensitive);
        assert!(!c.prefer_api_for_complex);
    }

    #[test]
    fn routing_strategy_serde_roundtrip() {
        let s = RoutingStrategy::LowestCost;
        let json = serde_json::to_string(&s).expect("serialize strategy");
        let back: RoutingStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s, back);
    }

    #[tokio::test]
    async fn dispatcher_builder_builds_dispatcher_with_empty_registry() {
        let d = DispatcherBuilder::default()
            .with_api_key("openai", "sk-test")
            .with_routing_strategy(RoutingStrategy::LowestLatency)
            .prefer_local_for_sensitive(false)
            .prefer_api_for_complex(true)
            .build()
            .expect("dispatcher should construct without network providers");
        assert_eq!(d.router().get_provider_count(), 0);
        assert!(
            d.list_all_available_models()
                .await
                .expect("list models should succeed with empty registry")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn dispatcher_registers_provider_and_processes_chat() {
        let d = DispatcherBuilder::new()
            .with_routing_strategy(RoutingStrategy::FirstMatch)
            .build()
            .expect("build");
        d.router()
            .register_provider("mock", Arc::new(MockAIClient::new().with_latency(0)))
            .expect("register");

        let task = AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        };
        let resp = d
            .process_request(ChatRequest::new().add_user("hello"), task)
            .await
            .expect("chat");
        assert!(!resp.choices.is_empty());
    }

    #[tokio::test]
    async fn dispatcher_stream_and_model_preference_paths() {
        let d = DispatcherBuilder::new().build().expect("build");
        d.router()
            .register_provider("mock", Arc::new(MockAIClient::new().with_latency(0)))
            .expect("register");

        let task = AITask::default();
        let _ = d
            .process_stream_request(ChatRequest::new().add_user("hi"), task.clone())
            .await
            .expect("stream");

        let _ = d
            .process_with_model_preference(
                ChatRequest::new().add_user("x"),
                task,
                Some("mock".to_string()),
                None,
            )
            .await
            .expect("preference");
    }

    #[tokio::test]
    async fn workflows_dev_and_production_dispatchers_build() {
        let dev = crate::workflows::create_dev_dispatcher().expect("dev");
        assert_eq!(
            dev.router().get_routing_strategy(),
            RoutingStrategy::BestFit
        );

        let prod = crate::workflows::create_production_dispatcher(
            Some("k1".to_string()),
            Some("k2".to_string()),
        )
        .expect("prod");
        assert!(prod.router().is_remote_routing_enabled());
    }

    #[tokio::test]
    async fn workflows_generate_text_with_registered_provider() {
        let d = DispatcherBuilder::new().build().expect("build");
        d.router()
            .register_provider("mock", Arc::new(MockAIClient::new().with_latency(0)))
            .expect("register");
        let text = crate::workflows::generate_text(&d, "hello world", false)
            .await
            .expect("text");
        assert!(!text.is_empty());
    }

    #[test]
    fn request_context_new_for_integration() {
        let t = AITask::default();
        let ctx = RequestContext::new(t.clone());
        assert_eq!(ctx.task.task_type, t.task_type);
    }
}
