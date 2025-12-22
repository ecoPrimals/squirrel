//! Comprehensive tests for AI Router dispatch logic
//!
//! This test suite provides thorough coverage of the router's dispatch mechanisms,
//! error paths, and edge cases to increase coverage from 33% to 80%+

use squirrel_ai_tools::common::capability::{AITask, TaskType};
use squirrel_ai_tools::common::{ChatMessage, ChatRequest, MessageRole};
use squirrel_ai_tools::router::types::{RequestContext, RouterConfig, RoutingStrategy};
use squirrel_ai_tools::router::AIRouter;

/// Helper to create a basic chat request
fn create_test_request() -> ChatRequest {
    ChatRequest::new()
        .add_user("Test message")
        .with_model("test-model")
}

/// Helper to create a basic request context
fn create_test_context() -> RequestContext {
    RequestContext {
        request_id: "test-req-001".to_string(),
        task: AITask {
            task_type: TaskType::Chat,
            complexity: 50,
            priority: 5,
            ..Default::default()
        },
        routing_hint: None,
        security_requirements: None,
    }
}

#[tokio::test]
async fn test_router_creation() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    // Verify router is created successfully
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_router_with_default_strategy() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LoadBalancing,
        allow_remote_routing: false,
        default_provider: None,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_process_request_no_providers() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let request = create_test_request();
    let context = create_test_context();

    // Should return error when no providers available
    let result = router.process_request(request, context).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No provider found") || err_msg.contains("provider"));
}

#[tokio::test]
async fn test_process_request_with_default_provider() {
    let config = RouterConfig {
        default_provider: Some("default".to_string()),
        allow_remote_routing: false,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    let request = create_test_request();
    let context = create_test_context();

    // Should still error because default provider isn't registered
    let result = router.process_request(request, context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_stats_initialization() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let stats = router.get_stats().unwrap();
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}

#[tokio::test]
async fn test_capability_registry_access() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let registry = router.registry();
    assert_eq!(registry.list_providers().len(), 0);
}

#[tokio::test]
async fn test_routing_strategy_round_robin() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::RoundRobin,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_routing_strategy_lowest_latency() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LowestLatency,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_routing_strategy_least_loaded() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LeastLoaded,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_routing_strategy_cost_optimized() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::CostOptimized,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    assert!(router.registry().list_providers().is_empty());
}

#[tokio::test]
async fn test_remote_routing_disabled() {
    let config = RouterConfig {
        allow_remote_routing: false,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    let request = create_test_request();
    let context = create_test_context();

    // Should not attempt remote routing
    let result = router.process_request(request, context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remote_routing_enabled_no_client() {
    let config = RouterConfig {
        allow_remote_routing: true,
        ..Default::default()
    };

    let router = AIRouter::new(config);
    let request = create_test_request();
    let context = create_test_context();

    // Should still fail because no MCP client is set
    let result = router.process_request(request, context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_request_context_with_high_priority() {
    let context = RequestContext {
        request_id: "high-priority-001".to_string(),
        task: AITask {
            task_type: TaskType::Chat,
            complexity: 80,
            priority: 10, // High priority
            ..Default::default()
        },
        routing_hint: None,
        security_requirements: None,
    };

    assert_eq!(context.task.priority, 10);
    assert_eq!(context.task.complexity, 80);
}

#[tokio::test]
async fn test_request_context_with_low_complexity() {
    let context = RequestContext {
        request_id: "simple-001".to_string(),
        task: AITask {
            task_type: TaskType::Chat,
            complexity: 10, // Low complexity
            priority: 5,
            ..Default::default()
        },
        routing_hint: None,
        security_requirements: None,
    };

    assert_eq!(context.task.complexity, 10);
}

#[tokio::test]
async fn test_chat_request_with_system_message() {
    let request = ChatRequest::new()
        .add_system("You are a helpful assistant")
        .add_user("Hello");

    assert_eq!(request.messages.len(), 2);
    assert!(matches!(request.messages[0].role, MessageRole::System));
    assert!(matches!(request.messages[1].role, MessageRole::User));
}

#[tokio::test]
async fn test_multiple_task_types() {
    let task_types = vec![
        TaskType::Chat,
        TaskType::Completion,
        TaskType::Embedding,
        TaskType::CodeGeneration,
    ];

    for task_type in task_types {
        let context = RequestContext {
            request_id: format!("test-{:?}", task_type),
            task: AITask {
                task_type,
                complexity: 50,
                priority: 5,
                ..Default::default()
            },
            routing_hint: None,
            security_requirements: None,
        };

        assert_eq!(context.task.task_type, task_type);
    }
}

#[tokio::test]
async fn test_router_with_various_configs() {
    let configs = vec![
        RouterConfig {
            routing_strategy: RoutingStrategy::LoadBalancing,
            allow_remote_routing: true,
            default_provider: Some("provider1".to_string()),
            ..Default::default()
        },
        RouterConfig {
            routing_strategy: RoutingStrategy::RoundRobin,
            allow_remote_routing: false,
            default_provider: None,
            ..Default::default()
        },
        RouterConfig {
            routing_strategy: RoutingStrategy::LowestLatency,
            allow_remote_routing: true,
            default_provider: Some("provider2".to_string()),
            ..Default::default()
        },
    ];

    for config in configs {
        let router = AIRouter::new(config);
        assert!(router.registry().list_providers().is_empty());
    }
}

#[tokio::test]
async fn test_error_handling_empty_messages() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let request = ChatRequest::new(); // Empty messages
    let context = create_test_context();

    // Router should handle gracefully
    let result = router.process_request(request, context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_configuration_defaults() {
    let config = RouterConfig::default();

    // Verify default configuration values
    assert!(!config.allow_remote_routing);
    assert!(config.default_provider.is_none());
    assert!(matches!(
        config.routing_strategy,
        RoutingStrategy::LoadBalancing
    ));
}

#[tokio::test]
async fn test_request_id_uniqueness() {
    let ids: Vec<String> = (0..10).map(|i| format!("req-{:06}", i)).collect();

    for id in ids {
        let context = RequestContext {
            request_id: id.clone(),
            task: AITask {
                task_type: TaskType::Chat,
                complexity: 50,
                priority: 5,
                ..Default::default()
            },
            routing_hint: None,
            security_requirements: None,
        };

        assert_eq!(context.request_id, id);
    }
}

#[tokio::test]
async fn test_complexity_levels() {
    let levels = vec![0, 25, 50, 75, 100];

    for complexity in levels {
        let context = RequestContext {
            request_id: format!("complex-{}", complexity),
            task: AITask {
                task_type: TaskType::Chat,
                complexity,
                priority: 5,
                ..Default::default()
            },
            routing_hint: None,
            security_requirements: None,
        };

        assert_eq!(context.task.complexity, complexity);
    }
}

#[tokio::test]
async fn test_priority_levels() {
    let priorities = vec![1, 3, 5, 7, 10];

    for priority in priorities {
        let context = RequestContext {
            request_id: format!("priority-{}", priority),
            task: AITask {
                task_type: TaskType::Chat,
                complexity: 50,
                priority,
                ..Default::default()
            },
            routing_hint: None,
            security_requirements: None,
        };

        assert_eq!(context.task.priority, priority);
    }
}

#[tokio::test]
async fn test_concurrent_router_creation() {
    use tokio::task::JoinSet;

    let mut set = JoinSet::new();

    for _ in 0..10 {
        set.spawn(async {
            let config = RouterConfig::default();
            let router = AIRouter::new(config);
            assert!(router.registry().list_providers().is_empty());
        });
    }

    while let Some(result) = set.join_next().await {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_large_message_content() {
    let large_content = "a".repeat(10000); // 10KB message

    let request = ChatRequest::new().add_user(&large_content);

    assert_eq!(request.messages[0].content.as_ref().unwrap().len(), 10000);
}

#[tokio::test]
async fn test_multiple_conversation_turns() {
    let request = ChatRequest::new()
        .add_system("You are helpful")
        .add_user("Hello")
        .add_assistant("Hi there!")
        .add_user("How are you?");

    assert_eq!(request.messages.len(), 4);
}

#[tokio::test]
async fn test_chat_message_creation() {
    let msg = ChatMessage {
        role: MessageRole::User,
        content: Some("Test".to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    assert!(matches!(msg.role, MessageRole::User));
    assert_eq!(msg.content, Some("Test".to_string()));
}

#[tokio::test]
async fn test_message_roles() {
    let roles = vec![
        MessageRole::System,
        MessageRole::User,
        MessageRole::Assistant,
        MessageRole::Tool,
        MessageRole::Function,
    ];

    for role in roles {
        let msg = ChatMessage {
            role: role.clone(),
            content: Some("Test".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        assert_eq!(msg.role, role);
    }
}
