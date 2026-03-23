// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Main AI router implementation and request dispatching logic.
//!
//! This module contains the core AIRouter that processes requests and routes them
//! to appropriate providers based on capabilities and routing strategies.

use super::optimization::ProviderSelector;
use super::types::{
    CapabilityRegistry, MCPInterface, RemoteAIRequest, RequestContext, RouterConfig, RouterStats,
    TryFlattenStreamExt,
};
use crate::Result;
use crate::common::capability::{AITask, SecurityRequirements, TaskType};
use crate::common::{AIClient, ChatRequest, ChatResponse, ChatResponseStream};
use crate::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Central router for AI requests
pub struct AIRouter {
    /// Capability registry
    registry: Arc<CapabilityRegistry>,

    /// Router configuration
    config: RouterConfig,

    /// MCP client for remote communication (optional)
    mcp_client: Option<Arc<dyn MCPInterface>>,

    /// Provider selector for routing strategies
    selector: ProviderSelector,

    /// Router statistics
    stats: Arc<std::sync::RwLock<RouterStats>>,
}

impl AIRouter {
    /// Create a new AI router
    pub fn new(config: RouterConfig) -> Self {
        Self {
            registry: Arc::new(CapabilityRegistry::new()),
            config,
            mcp_client: None,
            selector: ProviderSelector::new(),
            stats: Arc::new(std::sync::RwLock::new(RouterStats::default())),
        }
    }

    /// Set the MCP client for remote communication
    pub fn with_mcp(mut self, mcp_client: Arc<dyn MCPInterface>) -> Self {
        self.mcp_client = Some(mcp_client);
        self
    }

    /// Get the capability registry
    pub fn registry(&self) -> Arc<CapabilityRegistry> {
        self.registry.clone()
    }

    /// Register a provider with the router
    pub fn register_provider(
        &self,
        provider_id: impl Into<String>,
        client: Arc<dyn AIClient>,
    ) -> Result<()> {
        let id = provider_id.into();
        debug!("Registering AI provider: {}", id);
        self.registry.register_provider(id, client)
    }

    /// Process a chat request with the given context
    pub async fn process_request(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse> {
        let start_time = Instant::now();
        debug!("Processing AI request: {:?}", context.request_id);

        // Find providers that can handle the task
        let providers = self.registry.find_providers_for_task(&context.task);

        if providers.is_empty() {
            // Try finding remote providers if allowed
            if self.config.allow_remote_routing
                && context
                    .routing_hint
                    .as_ref()
                    .is_none_or(|h| h.allow_remote.unwrap_or(true))
            {
                let result = self.route_to_remote(request, context).await;
                self.update_stats("remote", start_time, result.is_ok());
                return result;
            }

            // Try default provider if specified
            if let Some(default_provider) = &self.config.default_provider
                && let Some(provider) = self.registry.get_provider(default_provider)
            {
                debug!("Using default provider: {}", default_provider);
                let result = provider.chat(request).await;
                self.update_stats(default_provider, start_time, result.is_ok());
                return result;
            }

            return Err(Error::Configuration(format!(
                "No provider found that can handle task: {:?}",
                context.task.task_type
            )));
        }

        // Apply routing hint if provided
        let filtered_providers = self.apply_routing_hint(providers, &context)?;

        // Apply routing strategy
        let (provider_id, provider) = self.selector.select_provider(
            filtered_providers,
            &context,
            self.config.routing_strategy,
        )?;

        debug!("Selected provider: {}", provider_id);

        // Delegate to selected provider
        let result = provider.chat(request).await;
        self.update_stats(&provider_id, start_time, result.is_ok());

        result
    }

    /// Process a streaming chat request with the given context
    pub async fn process_stream_request(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponseStream> {
        let start_time = Instant::now();
        debug!("Processing streaming AI request: {:?}", context.request_id);

        // Find providers that can handle the task
        let providers = self.registry.find_providers_for_task(&context.task);

        if providers.is_empty() {
            // Try finding remote providers if allowed
            if self.config.allow_remote_routing
                && context
                    .routing_hint
                    .as_ref()
                    .is_none_or(|h| h.allow_remote.unwrap_or(true))
            {
                let result = self.route_stream_to_remote(request, context).await;
                self.update_stats("remote", start_time, result.is_ok());
                return result;
            }

            // Try default provider if specified
            if let Some(default_provider) = &self.config.default_provider
                && let Some(provider) = self.registry.get_provider(default_provider)
            {
                debug!("Using default provider: {}", default_provider);
                let result = provider.chat_stream(request).await;
                self.update_stats(default_provider, start_time, result.is_ok());
                return result;
            }

            return Err(Error::Configuration(format!(
                "No provider found that can handle task: {:?}",
                context.task.task_type
            )));
        }

        // Apply routing hint if provided
        let filtered_providers = self.apply_routing_hint(providers, &context)?;

        // Apply routing strategy
        let (provider_id, provider) = self.selector.select_provider(
            filtered_providers,
            &context,
            self.config.routing_strategy,
        )?;

        debug!("Selected provider for streaming: {}", provider_id);

        // Delegate to selected provider
        let result = provider.chat_stream(request).await;
        self.update_stats(&provider_id, start_time, result.is_ok());

        result
    }

    /// Apply routing hint to filter providers
    fn apply_routing_hint(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> Result<Vec<(String, Arc<dyn AIClient>)>> {
        let filtered_providers = if let Some(hint) = &context.routing_hint {
            if let Some(preferred_provider) = &hint.preferred_provider {
                providers
                    .into_iter()
                    .filter(|(id, _)| id == preferred_provider)
                    .collect()
            } else {
                providers
            }
        } else {
            providers
        };

        if filtered_providers.is_empty() {
            return Err(Error::Configuration(
                "No provider matches routing hint".to_string(),
            ));
        }

        Ok(filtered_providers)
    }

    /// Route a request to a remote node via MCP
    async fn route_to_remote(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse> {
        if let Some(mcp) = &self.mcp_client {
            debug!(
                "Looking for remote providers for task: {:?}",
                context.task.task_type
            );

            // Discover remote capabilities
            let remote_nodes = self.registry.find_remote_nodes_for_task(&context.task);

            if remote_nodes.is_empty() {
                return Err(Error::Configuration(
                    "No remote providers found for task".to_string(),
                ));
            }

            // For now, just use the first remote node
            let (node_id, provider_id) = &remote_nodes[0];

            debug!(
                "Routing request to remote node: {:?}, provider: {}",
                node_id, provider_id
            );

            // Create remote request
            let remote_request = RemoteAIRequest {
                request_id: context.request_id,
                session_id: context.session_id,
                provider_id: provider_id.clone(),
                chat_request: request,
                task: context.task,
            };

            // Send to remote node
            let response = mcp.send_request(node_id, remote_request).await?;

            Ok(response.chat_response)
        } else {
            Err(Error::Configuration(
                "MCP client not configured for remote routing".to_string(),
            ))
        }
    }

    /// Route a streaming request to a remote node via MCP
    async fn route_stream_to_remote(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponseStream> {
        if let Some(mcp) = &self.mcp_client {
            debug!(
                "Looking for remote providers for streaming task: {:?}",
                context.task.task_type
            );

            // Discover remote capabilities
            let remote_nodes = self.registry.find_remote_nodes_for_task(&context.task);

            if remote_nodes.is_empty() {
                return Err(Error::Configuration(
                    "No remote providers found for streaming task".to_string(),
                ));
            }

            // For now, just use the first remote node
            let (node_id, provider_id) = &remote_nodes[0];

            debug!(
                "Routing streaming request to remote node: {:?}, provider: {}",
                node_id, provider_id
            );

            // Create remote request
            let remote_request = RemoteAIRequest {
                request_id: context.request_id,
                session_id: context.session_id,
                provider_id: provider_id.clone(),
                chat_request: request,
                task: context.task,
            };

            // Send to remote node
            let response_stream = mcp.stream_request(node_id, remote_request).await?;

            // Extract the first stream (we expect only one for now)
            let first_stream = response_stream.inner.try_flatten_stream().await?;

            Ok(first_stream)
        } else {
            Err(Error::Configuration(
                "MCP client not configured for remote streaming".to_string(),
            ))
        }
    }

    /// Create a task for text generation
    pub fn create_text_generation_task(&self) -> AITask {
        AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        }
    }

    /// Select a provider for a given task context (public API)
    pub fn select_provider_for_task(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        self.selector
            .select_provider(providers, context, self.config.routing_strategy)
    }

    /// Update statistics for a request
    fn update_stats(&self, provider_name: &str, start_time: Instant, success: bool) {
        let stats_result = self.stats.write();

        let mut stats = match stats_result {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire stats write lock for update: {}", e);
                return; // Gracefully fail - stats are not critical for operation
            }
        };

        stats.total_requests += 1;

        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        let latency = start_time.elapsed().as_millis() as f64;
        stats.average_latency_ms = (stats.average_latency_ms * (stats.total_requests - 1) as f64
            + latency)
            / stats.total_requests as f64;

        *stats
            .provider_usage
            .entry(provider_name.to_string())
            .or_insert(0) += 1;
    }

    /// Get router statistics
    pub fn get_stats(&self) -> RouterStats {
        let stats_result = self.stats.read();

        match stats_result {
            Ok(stats) => stats.clone(),
            Err(e) => {
                tracing::error!("Failed to acquire stats read lock: {}", e);
                // Return default stats if lock fails
                RouterStats::default()
            }
        }
    }

    /// Get router configuration
    pub fn get_config(&self) -> &RouterConfig {
        &self.config
    }

    /// Update router configuration
    pub fn update_config(&mut self, config: RouterConfig) {
        self.config = config;
    }

    /// Check if remote routing is enabled
    pub fn is_remote_routing_enabled(&self) -> bool {
        self.config.allow_remote_routing
    }

    /// Get the current routing strategy
    pub fn get_routing_strategy(&self) -> crate::router::types::RoutingStrategy {
        self.config.routing_strategy
    }

    /// Set the routing strategy
    pub fn set_routing_strategy(&mut self, strategy: crate::router::types::RoutingStrategy) {
        self.config.routing_strategy = strategy;
    }

    /// Get provider count
    pub fn get_provider_count(&self) -> usize {
        self.registry.list_providers().len()
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_id: &str) -> bool {
        self.registry.get_provider(provider_id).is_some()
    }

    /// Unregister a provider
    pub fn unregister_provider(&self, provider_id: &str) -> Result<()> {
        self.registry.unregister_provider(provider_id)
    }

    /// List all registered providers
    pub fn list_providers(&self) -> Vec<String> {
        self.registry.list_providers()
    }

    /// Get success rate from statistics
    pub fn get_success_rate(&self) -> f64 {
        let stats = self.get_stats();
        if stats.total_requests == 0 {
            return 0.0;
        }
        (stats.successful_requests as f64) / (stats.total_requests as f64)
    }

    /// Get average latency
    pub fn get_average_latency(&self) -> f64 {
        self.get_stats().average_latency_ms
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = RouterStats::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::capability::{
        AICapabilities, AITask, CostTier, ModelType, RoutingPreferences, SecurityRequirements,
        TaskType,
    };
    use crate::common::types::{ChatChoice, ChatRequest, ChatResponseChunk, MessageRole};
    use crate::common::{AIClient, MockAIClient};
    use crate::router::types::{
        MCPInterface, NodeId, RemoteAIRequest, RemoteAIResponse, RemoteAIResponseStream,
        RouterConfig, RoutingHint, RoutingStrategy,
    };
    use async_trait::async_trait;
    use futures::Stream;
    use std::collections::HashMap;
    use std::pin::Pin;
    use std::sync::Arc;
    use uuid::Uuid;

    fn base_task() -> AITask {
        AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        }
    }

    fn request_context(task: AITask) -> RequestContext {
        RequestContext {
            request_id: Uuid::new_v4(),
            session_id: None,
            user_id: None,
            routing_hint: None,
            task,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Configurable test double for routing strategy coverage (mirrors `optimization` tests).
    #[derive(Debug, Clone)]
    struct TestClient {
        name: String,
        caps: AICapabilities,
        prefs: RoutingPreferences,
        default_model: String,
        chat_ok: bool,
    }

    impl TestClient {
        fn new(name: &str) -> Self {
            let mut caps = AICapabilities::new();
            caps.add_task_type(TaskType::TextGeneration);
            caps.add_model_type(ModelType::LargeLanguageModel);
            caps.max_context_size = 8192;
            caps.supports_streaming = true;
            caps.supports_function_calling = true;
            caps.supports_tool_use = true;
            caps.performance_metrics.avg_latency_ms = Some(100);
            Self {
                name: name.to_string(),
                caps,
                prefs: RoutingPreferences::default(),
                default_model: "mock-model".to_string(),
                chat_ok: true,
            }
        }

        fn with_prefs(mut self, prefs: RoutingPreferences) -> Self {
            self.prefs = prefs;
            self
        }

        fn with_caps(mut self, caps: AICapabilities) -> Self {
            self.caps = caps;
            self
        }

        fn with_chat_ok(mut self, ok: bool) -> Self {
            self.chat_ok = ok;
            self
        }
    }

    #[async_trait]
    impl AIClient for TestClient {
        async fn get_capabilities(&self, _model: &str) -> Result<AICapabilities> {
            Ok(self.caps.clone())
        }

        fn capabilities(&self) -> AICapabilities {
            self.caps.clone()
        }

        fn routing_preferences(&self) -> RoutingPreferences {
            self.prefs.clone()
        }

        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn default_model(&self) -> &str {
            &self.default_model
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        async fn list_models(&self) -> Result<Vec<String>> {
            Ok(vec![format!("{}-model", self.name)])
        }

        async fn chat(&self, _request: ChatRequest) -> Result<crate::common::ChatResponse> {
            if !self.chat_ok {
                return Err(Error::Configuration("chat failed".to_string()));
            }
            Ok(crate::common::ChatResponse {
                id: "mock-response".to_string(),
                model: self.default_model.clone(),
                choices: vec![ChatChoice {
                    index: 0,
                    role: MessageRole::Assistant,
                    content: Some("ok".to_string()),
                    finish_reason: Some("stop".to_string()),
                    tool_calls: None,
                }],
                usage: None,
            })
        }

        async fn chat_stream(
            &self,
            request: ChatRequest,
        ) -> Result<crate::common::ChatResponseStream> {
            self.chat(request).await?;
            Err(Error::Configuration(
                "stream unsupported in TestClient".to_string(),
            ))
        }
    }

    struct RecordingMcp;

    #[async_trait]
    impl MCPInterface for RecordingMcp {
        async fn send_request(
            &self,
            _node_id: &NodeId,
            request: RemoteAIRequest,
        ) -> Result<RemoteAIResponse> {
            Ok(RemoteAIResponse {
                response_id: Uuid::new_v4(),
                request_id: request.request_id,
                provider_id: request.provider_id.clone(),
                chat_response: crate::common::ChatResponse {
                    id: "remote".to_string(),
                    model: "remote-model".to_string(),
                    choices: vec![ChatChoice {
                        index: 0,
                        role: MessageRole::Assistant,
                        content: Some("remote-ok".to_string()),
                        finish_reason: Some("stop".to_string()),
                        tool_calls: None,
                    }],
                    usage: None,
                },
            })
        }

        async fn stream_request(
            &self,
            _node_id: &NodeId,
            _request: RemoteAIRequest,
        ) -> Result<RemoteAIResponseStream> {
            let chunk = ChatResponseChunk {
                id: "s".to_string(),
                model: "m".to_string(),
                choices: vec![],
            };
            let inner: crate::common::ChatResponseStream =
                Box::pin(futures::stream::once(async move { Ok(chunk) }));
            let outer: Pin<
                Box<dyn Stream<Item = Result<crate::common::ChatResponseStream>> + Send + Unpin>,
            > = Box::pin(futures::stream::iter(vec![Ok::<
                crate::common::ChatResponseStream,
                Error,
            >(inner)]));
            Ok(RemoteAIResponseStream { inner: outer })
        }

        async fn discover_capabilities(
            &self,
        ) -> Result<HashMap<NodeId, HashMap<String, AICapabilities>>> {
            Ok(HashMap::new())
        }
    }

    fn register_text_provider(router: &AIRouter, id: &str) -> Result<()> {
        router.register_provider(id, Arc::new(TestClient::new(id)))
    }

    #[tokio::test]
    async fn process_request_first_match_and_highest_priority() {
        let mut config = RouterConfig::default();
        config.routing_strategy = RoutingStrategy::FirstMatch;
        let router = AIRouter::new(config);
        register_text_provider(&router, "a").unwrap();
        register_text_provider(&router, "b").unwrap();

        let req = ChatRequest::new().add_user("hi");
        let resp = router
            .process_request(req, request_context(base_task()))
            .await
            .unwrap();
        assert_eq!(resp.choices[0].content.as_deref(), Some("ok"));

        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::HighestPriority;
        let router = AIRouter::new(cfg);
        let low = Arc::new(TestClient::new("low").with_prefs(RoutingPreferences {
            priority: 10,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        let high = Arc::new(TestClient::new("high").with_prefs(RoutingPreferences {
            priority: 99,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        router.register_provider("low", low).unwrap();
        router.register_provider("high", high).unwrap();

        let req = ChatRequest::new().add_user("x");
        let resp = router
            .process_request(req, request_context(base_task()))
            .await
            .unwrap();
        assert_eq!(resp.model, "mock-model");
        assert!(router.get_stats().provider_usage.contains_key("high"));
    }

    #[tokio::test]
    async fn process_request_lowest_latency_and_lowest_cost() {
        let mut slow_caps = AICapabilities::new();
        slow_caps.add_task_type(TaskType::TextGeneration);
        slow_caps.add_model_type(ModelType::LargeLanguageModel);
        slow_caps.max_context_size = 8192;
        slow_caps.supports_streaming = true;
        slow_caps.performance_metrics.avg_latency_ms = Some(500);

        let mut fast_caps = slow_caps.clone();
        fast_caps.performance_metrics.avg_latency_ms = Some(50);

        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::LowestLatency;
        let router = AIRouter::new(cfg);
        router
            .register_provider(
                "slow",
                Arc::new(TestClient::new("slow").with_caps(slow_caps)),
            )
            .unwrap();
        router
            .register_provider(
                "fast",
                Arc::new(TestClient::new("fast").with_caps(fast_caps)),
            )
            .unwrap();

        let req = ChatRequest::new().add_user("ping");
        let resp = router
            .process_request(req, request_context(base_task()))
            .await
            .unwrap();
        assert_eq!(resp.choices[0].content.as_deref(), Some("ok"));
        assert!(router.get_stats().provider_usage.contains_key("fast"));

        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::LowestCost;
        let router = AIRouter::new(cfg);
        let cheap = Arc::new(TestClient::new("cheap").with_prefs(RoutingPreferences {
            cost_tier: CostTier::Free,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        let pricey = Arc::new(TestClient::new("pricey").with_prefs(RoutingPreferences {
            cost_tier: CostTier::High,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        router.register_provider("cheap", cheap).unwrap();
        router.register_provider("pricey", pricey).unwrap();

        let req = ChatRequest::new().add_user("z");
        let resp = router
            .process_request(req, request_context(base_task()))
            .await
            .unwrap();
        assert!(router.get_stats().provider_usage.contains_key("cheap"));
        assert_eq!(resp.choices[0].content.as_deref(), Some("ok"));
    }

    #[tokio::test]
    async fn process_request_round_robin_and_random() {
        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::RoundRobin;
        let router = AIRouter::new(cfg);
        register_text_provider(&router, "r1").unwrap();
        register_text_provider(&router, "r2").unwrap();

        let ctx = request_context(base_task());
        let req = || ChatRequest::new().add_user("rr");
        router.process_request(req(), ctx.clone()).await.unwrap();
        router.process_request(req(), ctx.clone()).await.unwrap();
        assert!(router.get_stats().total_requests >= 2);

        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::Random;
        let router = AIRouter::new(cfg);
        register_text_provider(&router, "x").unwrap();
        register_text_provider(&router, "y").unwrap();
        router
            .process_request(
                ChatRequest::new().add_user("r"),
                request_context(base_task()),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn routing_hint_preferred_provider_and_mismatch_error() {
        let router = AIRouter::new(RouterConfig::default());
        register_text_provider(&router, "alpha").unwrap();
        register_text_provider(&router, "beta").unwrap();

        let mut ctx = request_context(base_task());
        ctx.routing_hint = Some(RoutingHint {
            preferred_provider: Some("beta".to_string()),
            preferred_model: None,
            allow_remote: Some(true),
            max_latency_ms: None,
            max_cost_tier: None,
            priority: None,
        });
        let resp = router
            .process_request(ChatRequest::new().add_user("u"), ctx)
            .await
            .unwrap();
        assert_eq!(resp.choices[0].content.as_deref(), Some("ok"));

        let mut ctx = request_context(base_task());
        ctx.routing_hint = Some(RoutingHint {
            preferred_provider: Some("nope".to_string()),
            preferred_model: None,
            allow_remote: Some(true),
            max_latency_ms: None,
            max_cost_tier: None,
            priority: None,
        });
        let err = router
            .process_request(ChatRequest::new().add_user("u"), ctx)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("routing hint"));
    }

    #[tokio::test]
    async fn no_provider_uses_default_when_configured() {
        let mut cfg = RouterConfig::default();
        cfg.default_provider = Some("fallback".to_string());
        cfg.allow_remote_routing = false;
        let router = AIRouter::new(cfg);
        register_text_provider(&router, "fallback").unwrap();

        let mut task = base_task();
        task.task_type = TaskType::ImageGeneration;
        let resp = router
            .process_request(ChatRequest::new().add_user("pic"), request_context(task))
            .await
            .unwrap();
        assert_eq!(resp.choices[0].content.as_deref(), Some("ok"));
    }

    #[tokio::test]
    async fn no_provider_config_error() {
        let cfg = RouterConfig {
            allow_remote_routing: false,
            default_provider: None,
            ..RouterConfig::default()
        };
        let router = AIRouter::new(cfg);
        register_text_provider(&router, "only").unwrap();

        let mut task = base_task();
        task.task_type = TaskType::ImageGeneration;
        let err = router
            .process_request(ChatRequest::new().add_user("x"), request_context(task))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("No provider found"));
    }

    #[tokio::test]
    async fn remote_routing_via_mcp() {
        let mut cfg = RouterConfig::default();
        cfg.allow_remote_routing = true;
        let router = AIRouter::new(cfg).with_mcp(Arc::new(RecordingMcp));

        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.max_context_size = 8192;
        caps.supports_streaming = true;

        let node = NodeId("node-1".to_string());
        let mut map = HashMap::new();
        map.insert("remote-p".to_string(), caps);
        router
            .registry()
            .register_remote_capabilities(node, map)
            .unwrap();

        let resp = router
            .process_request(
                ChatRequest::new().add_user("remote"),
                request_context(base_task()),
            )
            .await
            .unwrap();
        assert_eq!(resp.choices[0].content.as_deref(), Some("remote-ok"));
        assert!(router.get_stats().provider_usage.contains_key("remote"));
    }

    #[tokio::test]
    async fn remote_routing_errors_without_mcp_or_nodes() {
        let cfg = RouterConfig::default();
        let router = AIRouter::new(cfg).with_mcp(Arc::new(RecordingMcp));
        let err = router
            .process_request(
                ChatRequest::new().add_user("x"),
                request_context(base_task()),
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("remote"));

        let router = AIRouter::new(RouterConfig::default());
        let mut task = base_task();
        task.task_type = TaskType::ImageGeneration;
        let err = router
            .process_request(ChatRequest::new().add_user("x"), request_context(task))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("MCP client not configured"));
    }

    #[tokio::test]
    async fn remote_blocked_by_routing_hint() {
        let cfg = RouterConfig::default();
        let router = AIRouter::new(cfg).with_mcp(Arc::new(RecordingMcp));

        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.max_context_size = 8192;
        caps.supports_streaming = true;
        let node = NodeId("n".to_string());
        let mut map = HashMap::new();
        map.insert("p".to_string(), caps);
        router
            .registry()
            .register_remote_capabilities(node, map)
            .unwrap();

        let mut ctx = request_context(base_task());
        ctx.routing_hint = Some(RoutingHint {
            preferred_provider: None,
            preferred_model: None,
            allow_remote: Some(false),
            max_latency_ms: None,
            max_cost_tier: None,
            priority: None,
        });

        let err = router
            .process_request(ChatRequest::new().add_user("x"), ctx)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("MCP client not configured")
                || err.to_string().contains("No provider")
        );
    }

    #[tokio::test]
    async fn process_stream_uses_registered_provider() {
        let router = AIRouter::new(RouterConfig::default());
        router
            .register_provider("m", Arc::new(MockAIClient::new().with_latency(0)))
            .unwrap();

        let _stream = router
            .process_stream_request(
                ChatRequest::new().add_user("hello"),
                request_context(base_task()),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn process_stream_remote_path() {
        let router = AIRouter::new(RouterConfig::default()).with_mcp(Arc::new(RecordingMcp));
        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.max_context_size = 8192;
        caps.supports_streaming = true;
        let node = NodeId("node-2".to_string());
        let mut map = HashMap::new();
        map.insert("rp".to_string(), caps);
        router
            .registry()
            .register_remote_capabilities(node, map)
            .unwrap();

        let _ = router
            .process_stream_request(
                ChatRequest::new().add_user("s"),
                request_context(base_task()),
            )
            .await
            .unwrap();
    }

    #[test]
    fn select_provider_for_task_exposes_selector() {
        let router = AIRouter::new(RouterConfig::default());
        let p = Arc::new(TestClient::new("one")) as Arc<dyn AIClient>;
        let out = router
            .select_provider_for_task(
                vec![("one".to_string(), p)],
                &RequestContext::new(base_task()),
            )
            .unwrap();
        assert_eq!(out.0, "one");
    }

    #[tokio::test]
    async fn stats_success_failure_and_reset() {
        let mut cfg = RouterConfig::default();
        cfg.routing_strategy = RoutingStrategy::FirstMatch;
        let mut router = AIRouter::new(cfg);
        router
            .register_provider("ok", Arc::new(TestClient::new("ok")))
            .unwrap();

        // First request succeeds (only "ok" is registered for TextGeneration)
        router
            .process_request(
                ChatRequest::new().add_user("a"),
                request_context(base_task()),
            )
            .await
            .unwrap();

        // Second request uses ImageGeneration (no provider matches), falls through
        // to default_provider "bad" which fails
        router
            .register_provider("bad", Arc::new(TestClient::new("bad").with_chat_ok(false)))
            .unwrap();
        let mut task = base_task();
        task.task_type = TaskType::ImageGeneration;
        let mut cfg = RouterConfig::default();
        cfg.default_provider = Some("bad".to_string());
        cfg.allow_remote_routing = false;
        router.update_config(cfg);
        let _ = router
            .process_request(ChatRequest::new().add_user("b"), request_context(task))
            .await;

        let s = router.get_stats();
        assert!(s.total_requests >= 2);
        assert!(s.failed_requests >= 1);

        router.reset_stats();
        assert_eq!(router.get_stats().total_requests, 0);
        assert!((router.get_success_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unregister_and_list_providers() {
        let router = AIRouter::new(RouterConfig::default());
        router
            .register_provider("z", Arc::new(TestClient::new("z")))
            .unwrap();
        assert!(router.has_provider("z"));
        router.unregister_provider("z").unwrap();
        assert!(!router.has_provider("z"));
    }

    #[test]
    fn test_router_creation() {
        let config = RouterConfig::default();
        let router = AIRouter::new(config);

        assert_eq!(router.get_provider_count(), 0);
        assert!(!router.has_provider("nonexistent"));
        assert!(router.is_remote_routing_enabled());
        assert_eq!(router.get_routing_strategy(), RoutingStrategy::BestFit);
    }

    #[test]
    fn test_text_generation_task() {
        let config = RouterConfig::default();
        let router = AIRouter::new(config);
        let task = router.create_text_generation_task();

        assert_eq!(task.task_type, TaskType::TextGeneration);
        assert!(!task.requires_streaming);
        assert!(!task.requires_function_calling);
        assert!(!task.requires_tool_use);
        assert_eq!(task.priority, 50);
    }

    #[test]
    fn test_config_updates() {
        let config = RouterConfig {
            routing_strategy: RoutingStrategy::FirstMatch,
            allow_remote_routing: false,
            ..Default::default()
        };

        let mut router = AIRouter::new(config);

        assert_eq!(router.get_routing_strategy(), RoutingStrategy::FirstMatch);
        assert!(!router.is_remote_routing_enabled());

        router.set_routing_strategy(RoutingStrategy::RoundRobin);
        assert_eq!(router.get_routing_strategy(), RoutingStrategy::RoundRobin);

        let new_config = RouterConfig {
            routing_strategy: RoutingStrategy::BestFit,
            allow_remote_routing: true,
            ..Default::default()
        };

        router.update_config(new_config);
        assert_eq!(router.get_routing_strategy(), RoutingStrategy::BestFit);
        assert!(router.is_remote_routing_enabled());
    }

    #[test]
    fn test_statistics() {
        let config = RouterConfig::default();
        let router = AIRouter::new(config);

        let stats = router.get_stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert!((stats.average_latency_ms - 0.0).abs() < f64::EPSILON);

        assert!((router.get_success_rate() - 0.0).abs() < f64::EPSILON);
        assert!((router.get_average_latency() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_management() {
        let config = RouterConfig::default();
        let router = AIRouter::new(config);

        assert_eq!(router.get_provider_count(), 0);
        assert!(router.list_providers().is_empty());
        assert!(!router.has_provider("test"));
    }
}
