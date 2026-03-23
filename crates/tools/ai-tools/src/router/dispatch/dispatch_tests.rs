// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::common::capability::{
    AICapabilities, AITask, CostTier, ModelType, RoutingPreferences, SecurityRequirements, TaskType,
};
use crate::common::types::{ChatChoice, ChatRequest, ChatResponseChunk, MessageRole};
use crate::common::{AIClient, MockAIClient};
use crate::router::types::{
    MCPInterface, NodeId, RemoteAIRequest, RemoteAIResponse, RemoteAIResponseStream, RouterConfig,
    RoutingHint, RoutingStrategy,
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

    async fn chat_stream(&self, request: ChatRequest) -> Result<crate::common::ChatResponseStream> {
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
            provider_id: request.provider_id,
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
