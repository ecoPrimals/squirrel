// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core types and configurations for the AI router system.
//!
//! This module defines all the fundamental types used throughout the router
//! infrastructure, including configurations, routing strategies, and request contexts.

use crate::AiClientImpl;
use crate::Result;
use crate::common::capability::{AICapabilities, AITask};
use crate::common::{AIClient, ChatRequest, ChatResponse, ChatResponseStream};
use crate::error::Error;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Node identifier type for distributed routing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Default provider to use when no suitable provider is found
    pub default_provider: Option<String>,

    /// Whether to allow forwarding to remote nodes
    pub allow_remote_routing: bool,

    /// Routing strategy to use
    pub routing_strategy: RoutingStrategy,

    /// Timeout for request routing in milliseconds
    pub routing_timeout_ms: u64,

    /// Maximum number of routing attempts
    pub max_routing_attempts: u32,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            allow_remote_routing: true,
            routing_strategy: RoutingStrategy::BestFit,
            routing_timeout_ms: 30000,
            max_routing_attempts: 3,
        }
    }
}

/// Available routing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RoutingStrategy {
    /// Select the first provider that can handle the task
    FirstMatch,

    /// Select the provider with the highest priority
    HighestPriority,

    /// Select the provider with the lowest latency
    LowestLatency,

    /// Select the provider with the lowest cost
    LowestCost,

    /// Select the provider that best matches the task requirements
    BestFit,

    /// Round-robin among suitable providers
    RoundRobin,

    /// Random selection among suitable providers
    Random,
}

/// Routing hint to guide the router's decision
#[derive(Debug, Clone)]
pub struct RoutingHint {
    /// Specific provider to use
    pub preferred_provider: Option<String>,

    /// Specific model to use
    pub preferred_model: Option<String>,

    /// Whether to allow forwarding to remote nodes
    pub allow_remote: Option<bool>,

    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,

    /// Cost tier limit
    pub max_cost_tier: Option<crate::common::capability::CostTier>,

    /// Task priority (0-100)
    pub priority: Option<u8>,
}

/// Request context with routing information
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request ID
    pub request_id: uuid::Uuid,

    /// Session ID for maintaining conversation state
    pub session_id: Option<uuid::Uuid>,

    /// User ID
    pub user_id: Option<String>,

    /// Routing hint
    pub routing_hint: Option<RoutingHint>,

    /// Task description
    pub task: AITask,

    /// Timestamp of the request
    pub timestamp: Instant,
}

impl RequestContext {
    /// Create a new request context
    #[must_use]
    pub fn new(task: AITask) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            session_id: None,
            user_id: None,
            routing_hint: None,
            task,
            timestamp: Instant::now(),
        }
    }

    /// Set the session ID
    #[must_use]
    pub const fn with_session_id(mut self, session_id: uuid::Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the routing hint
    #[must_use]
    pub fn with_routing_hint(mut self, hint: RoutingHint) -> Self {
        self.routing_hint = Some(hint);
        self
    }
}

/// Capability registry for AI providers
#[derive(Debug)]
pub struct CapabilityRegistry {
    /// Local providers and their capabilities
    pub(crate) local_providers: RwLock<HashMap<String, Arc<AiClientImpl>>>,

    /// Remote node capabilities
    pub(crate) remote_capabilities: RwLock<HashMap<NodeId, HashMap<String, AICapabilities>>>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[expect(
    clippy::significant_drop_tightening,
    reason = "std::sync::PoisonError from RwLock uses a significant Drop; only on error paths"
)]
impl CapabilityRegistry {
    /// Create a new capability registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            local_providers: RwLock::new(HashMap::new()),
            remote_capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Register a local provider
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] when the registry lock is poisoned.
    pub fn register_provider(
        &self,
        provider_id: impl Into<String>,
        client: Arc<AiClientImpl>,
    ) -> Result<()> {
        let mut providers = match self.local_providers.write() {
            Ok(guard) => guard,
            Err(e) => {
                let err = Error::Configuration(format!(
                    "Failed to acquire provider lock for registration: {e}"
                ));
                tracing::error!("Provider registration failed: {}", err);
                return Err(err);
            }
        };
        providers.insert(provider_id.into(), client);
        Ok(())
    }

    /// Unregister a local provider
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] when the registry lock is poisoned.
    pub fn unregister_provider(&self, provider_id: &str) -> Result<()> {
        let mut providers = match self.local_providers.write() {
            Ok(guard) => guard,
            Err(e) => {
                let err = Error::Configuration(format!(
                    "Failed to acquire provider lock for unregistration: {e}"
                ));
                tracing::error!("Provider unregistration failed: {}", err);
                return Err(err);
            }
        };
        providers.remove(provider_id);
        Ok(())
    }

    /// Register remote node capabilities
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] when the registry lock is poisoned.
    pub fn register_remote_capabilities(
        &self,
        node_id: NodeId,
        capabilities: HashMap<String, AICapabilities>,
    ) -> Result<()> {
        let mut remote_caps = match self.remote_capabilities.write() {
            Ok(guard) => guard,
            Err(e) => {
                let err = Error::Configuration(format!(
                    "Failed to acquire remote capabilities lock: {e}"
                ));
                tracing::error!("Remote capabilities registration failed: {}", err);
                return Err(err);
            }
        };
        remote_caps.insert(node_id, capabilities);
        Ok(())
    }

    /// Get a local provider by ID
    pub fn get_provider(&self, provider_id: &str) -> Option<Arc<AiClientImpl>> {
        let providers = match self.local_providers.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire provider read lock: {}", e);
                return None;
            }
        };
        providers.get(provider_id).cloned()
    }

    /// Find providers that can handle a specific task
    pub fn find_providers_for_task(&self, task: &AITask) -> Vec<(String, Arc<AiClientImpl>)> {
        let providers = match self.local_providers.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!(
                    "Failed to acquire provider read lock for task search: {}",
                    e
                );
                return Vec::new();
            }
        };

        let mut matches = Vec::new();

        for (id, provider) in providers.iter() {
            if provider.can_handle_task(task) {
                matches.push((id.clone(), provider.clone()));
            }
        }

        matches
    }

    /// Find remote nodes that can handle a specific task
    pub fn find_remote_nodes_for_task(&self, task: &AITask) -> Vec<(NodeId, String)> {
        let remote_caps = match self.remote_capabilities.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire remote capabilities read lock: {}", e);
                return Vec::new();
            }
        };

        let mut matches = Vec::new();

        for (node_id, providers) in remote_caps.iter() {
            for (provider_id, capabilities) in providers {
                if task_matches_capabilities(task, capabilities) {
                    matches.push((node_id.clone(), provider_id.clone()));
                }
            }
        }

        matches
    }

    /// List all registered providers
    pub fn list_providers(&self) -> Vec<String> {
        let providers = match self.local_providers.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire provider read lock for listing: {}", e);
                return Vec::new();
            }
        };
        providers.keys().cloned().collect()
    }
}

/// Interface for MCP communication
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait MCPInterface: Send + Sync + 'static {
    /// Send a request to a remote node
    async fn send_request(
        &self,
        node_id: &NodeId,
        request: RemoteAIRequest,
    ) -> Result<RemoteAIResponse>;

    /// Stream a response from a remote node
    async fn stream_request(
        &self,
        node_id: &NodeId,
        request: RemoteAIRequest,
    ) -> Result<RemoteAIResponseStream>;

    /// Discover AI capabilities in the network
    async fn discover_capabilities(
        &self,
    ) -> Result<HashMap<NodeId, HashMap<String, AICapabilities>>>;
}

/// Placeholder MCP type when no remote client is configured (`mcp_client` is [`None`]).
/// Methods are never invoked in that case; they return errors if called.
#[derive(Debug, Copy, Clone, Default)]
pub struct NoMcpInterface;

impl MCPInterface for NoMcpInterface {
    async fn send_request(
        &self,
        _node_id: &NodeId,
        _request: RemoteAIRequest,
    ) -> Result<RemoteAIResponse> {
        Err(Error::Configuration(
            "MCP client not configured (NoMcpInterface)".to_string(),
        ))
    }

    async fn stream_request(
        &self,
        _node_id: &NodeId,
        _request: RemoteAIRequest,
    ) -> Result<RemoteAIResponseStream> {
        Err(Error::Configuration(
            "MCP client not configured (NoMcpInterface)".to_string(),
        ))
    }

    async fn discover_capabilities(
        &self,
    ) -> Result<HashMap<NodeId, HashMap<String, AICapabilities>>> {
        Ok(HashMap::new())
    }
}

/// Remote AI request
#[derive(Debug, Clone)]
pub struct RemoteAIRequest {
    /// Request ID
    pub request_id: uuid::Uuid,

    /// Session ID
    pub session_id: Option<uuid::Uuid>,

    /// Provider ID to route to
    pub provider_id: String,

    /// Chat request
    pub chat_request: ChatRequest,

    /// Task description
    pub task: AITask,
}

/// Remote AI response
#[derive(Debug, Clone)]
pub struct RemoteAIResponse {
    /// Response ID
    pub response_id: uuid::Uuid,

    /// Request ID this is responding to
    pub request_id: uuid::Uuid,

    /// Provider ID that fulfilled the request
    pub provider_id: String,

    /// Chat response
    pub chat_response: ChatResponse,
}

/// Remote AI response stream
pub struct RemoteAIResponseStream {
    /// Inner stream
    pub inner: Pin<Box<dyn Stream<Item = Result<ChatResponseStream>> + Send + Unpin>>,
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    /// Total number of requests processed.
    pub total_requests: u64,
    /// Number of successfully completed requests.
    pub successful_requests: u64,
    /// Number of failed requests.
    pub failed_requests: u64,
    /// Average request latency in milliseconds.
    pub average_latency_ms: f64,
    /// Request count per provider.
    pub provider_usage: HashMap<String, u64>,
}

impl Default for RouterStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            provider_usage: HashMap::new(),
        }
    }
}

/// Check if a task matches given capabilities
#[must_use]
pub fn task_matches_capabilities(task: &AITask, capabilities: &AICapabilities) -> bool {
    // Check basic task type support
    if !capabilities.supports_task(&task.task_type) {
        return false;
    }

    // Check model type requirements
    if let Some(ref model_type) = task.required_model_type
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

    // Check tool use support
    if task.requires_tool_use && !capabilities.supports_tool_use {
        return false;
    }

    true
}

/// Helper trait for stream flattening
#[expect(async_fn_in_trait, reason = "single concrete impl — Send guaranteed")]
pub trait TryFlattenStreamExt {
    /// Flattens a stream of results into a single result stream.
    async fn try_flatten_stream(self) -> Result<ChatResponseStream>;
}

impl TryFlattenStreamExt
    for Pin<Box<dyn Stream<Item = Result<ChatResponseStream>> + Send + Unpin>>
{
    async fn try_flatten_stream(mut self) -> Result<ChatResponseStream> {
        use futures::StreamExt;

        self.next()
            .await
            .unwrap_or_else(|| Err(Error::Runtime("Empty stream received".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::capability::{
        CostMetrics, CostTier, ModelType, PerformanceMetrics, ResourceRequirements,
        RoutingPreferences, SecurityRequirements, TaskType,
    };
    use uuid::Uuid;

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert_eq!(config.default_provider, None);
        assert!(config.allow_remote_routing);
        assert_eq!(config.routing_strategy, RoutingStrategy::BestFit);
        assert_eq!(config.routing_timeout_ms, 30000);
        assert_eq!(config.max_routing_attempts, 3);
    }

    #[test]
    fn test_request_context_builder() {
        let task = AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: Some(ModelType::ChatModel),
            min_context_size: Some(4096),
            requires_streaming: true,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: Some(75),
            priority: 80,
        };

        let context = RequestContext::new(task.clone())
            .with_session_id(Uuid::new_v4())
            .with_user_id("test_user")
            .with_routing_hint(RoutingHint {
                preferred_provider: Some("gpt-4".to_string()),
                preferred_model: Some("gpt-4-turbo".to_string()),
                allow_remote: Some(false),
                max_latency_ms: Some(1000),
                max_cost_tier: Some(CostTier::High),
                priority: Some(90),
            });

        assert_eq!(context.task, task);
        assert!(context.session_id.is_some());
        assert_eq!(context.user_id, Some("test_user".to_string()));
        assert!(context.routing_hint.is_some());
    }

    #[test]
    fn test_router_stats_default() {
        let stats = RouterStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert!((stats.average_latency_ms - 0.0).abs() < f64::EPSILON);
        assert!(stats.provider_usage.is_empty());
    }

    #[test]
    fn test_task_matches_capabilities() {
        let task = AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: Some(ModelType::ChatModel),
            min_context_size: Some(4096),
            requires_streaming: true,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: Some(75),
            priority: 80,
        };

        let capabilities = AICapabilities {
            supported_task_types: vec![TaskType::TextGeneration].into_iter().collect(),
            supported_model_types: vec![ModelType::ChatModel].into_iter().collect(),
            max_context_size: 8192,
            supports_streaming: true,
            supports_function_calling: false,
            supports_tool_use: false,
            supports_images: false,
            performance_metrics: PerformanceMetrics::default(),
            cost_metrics: CostMetrics::default(),
            resource_requirements: ResourceRequirements::default(),
            routing_preferences: RoutingPreferences::default(),
            security_requirements: SecurityRequirements::default(),
        };

        assert!(task_matches_capabilities(&task, &capabilities));

        // Test with incompatible capabilities
        let bad_capabilities = AICapabilities {
            supported_task_types: vec![TaskType::TextGeneration].into_iter().collect(),
            supported_model_types: vec![ModelType::ChatModel].into_iter().collect(),
            max_context_size: 2048,    // Too small
            supports_streaming: false, // Required but not supported
            supports_function_calling: false,
            supports_tool_use: false,
            supports_images: false,
            performance_metrics: PerformanceMetrics::default(),
            cost_metrics: CostMetrics::default(),
            resource_requirements: ResourceRequirements::default(),
            routing_preferences: RoutingPreferences::default(),
            security_requirements: SecurityRequirements::default(),
        };

        assert!(!task_matches_capabilities(&task, &bad_capabilities));
    }
}
