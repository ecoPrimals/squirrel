// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Main AI router implementation and request dispatching logic.
//!
//! This module contains the core `AIRouter` that processes requests and routes them
//! to appropriate providers based on capabilities and routing strategies.

use super::optimization::ProviderSelector;
use super::types::{
    CapabilityRegistry, MCPInterface, NoMcpInterface, RemoteAIRequest, RequestContext,
    RouterConfig, RouterStats, TryFlattenStreamExt,
};
use crate::Result;
use crate::common::capability::{AITask, SecurityRequirements, TaskType};
use crate::common::{AIClient, ChatRequest, ChatResponse, ChatResponseStream};
use crate::error::Error;
use crate::float_helpers;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Central router for AI requests
pub struct AIRouter<M: MCPInterface = NoMcpInterface> {
    /// Capability registry
    registry: Arc<CapabilityRegistry>,

    /// Router configuration
    config: RouterConfig,

    /// MCP client for remote communication (optional)
    mcp_client: Option<Arc<M>>,

    /// Provider selector for routing strategies
    selector: ProviderSelector,

    /// Router statistics
    stats: Arc<std::sync::RwLock<RouterStats>>,
}

impl AIRouter<NoMcpInterface> {
    /// Create a new AI router (no MCP client; use [`AIRouter::with_mcp`] to attach one).
    #[must_use]
    pub fn new(config: RouterConfig) -> Self {
        Self {
            registry: Arc::new(CapabilityRegistry::new()),
            config,
            mcp_client: None,
            selector: ProviderSelector::new(),
            stats: Arc::new(std::sync::RwLock::new(RouterStats::default())),
        }
    }
}

impl<M: MCPInterface> AIRouter<M> {
    /// Set the MCP client for remote communication
    #[must_use]
    pub fn with_mcp<OtherM: MCPInterface>(self, mcp_client: Arc<OtherM>) -> AIRouter<OtherM> {
        AIRouter {
            registry: self.registry,
            config: self.config,
            mcp_client: Some(mcp_client),
            selector: self.selector,
            stats: self.stats,
        }
    }

    /// Get the capability registry
    pub fn registry(&self) -> Arc<CapabilityRegistry> {
        self.registry.clone()
    }

    /// Register a provider with the router
    ///
    /// # Errors
    ///
    /// Propagates registry errors when the id is invalid or registration fails.
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] when no provider matches, or provider/network errors from the selected backend.
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
        let filtered_providers = Self::apply_routing_hint(providers, &context)?;

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
    ///
    /// # Errors
    ///
    /// Same error cases as [`Self::process_request`], plus streaming-specific provider errors.
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
        let filtered_providers = Self::apply_routing_hint(providers, &context)?;

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
    ///
    /// # Errors
    ///
    /// Propagates [`Error::Configuration`] from [`ProviderSelector::select_provider`] when selection fails.
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
        let latency_ms = start_time.elapsed().as_millis();
        let latency = float_helpers::u128_to_f64_lossy(latency_ms);

        let mut stats = match self.stats.write() {
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

        let n = stats.total_requests;
        let prev = n.saturating_sub(1);
        stats.average_latency_ms = stats
            .average_latency_ms
            .mul_add(float_helpers::u64_to_f64_lossy(prev), latency)
            / float_helpers::u64_to_f64_lossy(n);

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
    pub const fn get_config(&self) -> &RouterConfig {
        &self.config
    }

    /// Update router configuration
    pub fn update_config(&mut self, config: RouterConfig) {
        self.config = config;
    }

    /// Check if remote routing is enabled
    pub const fn is_remote_routing_enabled(&self) -> bool {
        self.config.allow_remote_routing
    }

    /// Get the current routing strategy
    pub const fn get_routing_strategy(&self) -> crate::router::types::RoutingStrategy {
        self.config.routing_strategy
    }

    /// Set the routing strategy
    pub const fn set_routing_strategy(&mut self, strategy: crate::router::types::RoutingStrategy) {
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
    ///
    /// # Errors
    ///
    /// Propagates registry errors when the provider id is unknown.
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
        float_helpers::u64_ratio(stats.successful_requests, stats.total_requests)
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
#[path = "dispatch_tests.rs"]
#[expect(
    clippy::field_reassign_with_default,
    reason = "Struct update after Default::default()"
)]
// Tests build `RouterConfig` field-by-field for clarity
mod tests;
