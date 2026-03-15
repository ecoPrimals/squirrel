// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! AI Router Implementation
//!
//! This module implements the AI routing infrastructure that directs AI requests
//! to the most appropriate provider based on capabilities and preferences.
//!
//! The router system consists of several key components:
//!
//! - **Types**: Core data structures, configurations, and routing strategies
//! - **Dispatch**: Main router implementation and request processing logic
//! - **Optimization**: Provider selection algorithms and scoring strategies
//! - **MCP Adapter**: Integration with the Machine Context Protocol for remote routing
//!
//! ## Architecture Overview
//!
//! ```mermaid
//! ---
//! title: AI Router Architecture
//! ---
//! graph TD
//!     A[Chat Request] --> B[AI Router]
//!     B --> C[Capability Registry]
//!     C --> D[Provider Selection]
//!     D --> E[Routing Strategy]
//!     E --> F[Local Provider]
//!     E --> G[Remote Provider]
//!     F --> H[Response]
//!     G --> I[MCP Interface]
//!     I --> H
//!     
//!     subgraph "Router Components"
//!         B
//!         C
//!         D
//!         E
//!     end
//!     
//!     subgraph "Provider Types"
//!         F
//!         G
//!         I
//!     end
//! ```
//!
//! ## Usage Example
//!
//! ```ignore
//! use ai_tools::router::{AIRouter, RouterConfig, RoutingStrategy, RequestContext};
//! use ai_tools::common::capability::AITask;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create router with configuration
//! let config = RouterConfig {
//!     routing_strategy: RoutingStrategy::BestFit,
//!     allow_remote_routing: true,
//!     ..Default::default()
//! };
//! let router = AIRouter::new(config);
//!
//! // Register providers
//! router.register_provider("gpt-4", gpt4_client)?;
//! router.register_provider("claude", claude_client)?;
//!
//! // Create request context
//! let task = AITask::text_generation();
//! let context = RequestContext::new(task)
//!     .with_user_id("user123")
//!     .with_routing_hint(routing_hint);
//!
//! // Process request
//! let response = router.process_request(chat_request, context).await?;
//! # Ok(())
//! # }
//! ```

// Re-export the MCP adapter from the existing module
mod mcp_adapter;
pub use mcp_adapter::{MCPAdapter, MCPAdapterConfig};

// New organized modules
pub mod dispatch;
pub mod optimization;
pub mod types;

// Re-export key types and traits from types module
pub use types::{
    task_matches_capabilities, CapabilityRegistry, MCPInterface, NodeId, RemoteAIRequest,
    RemoteAIResponse, RemoteAIResponseStream, RequestContext, RouterConfig, RouterStats,
    RoutingHint, RoutingStrategy, TryFlattenStreamExt,
};

// Re-export main router from dispatch module
pub use dispatch::AIRouter;

// Re-export optimization utilities
pub use optimization::{OptimizationUtils, ProviderScorer, ProviderSelector};

// Backward compatibility re-exports
pub use dispatch::AIRouter as Router;
pub use types::RouterConfig as Config;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::capability::{AITask, SecurityRequirements, TaskType};

    #[test]
    fn test_module_organization() {
        // Test that we can create core types
        let config = RouterConfig::default();
        let router = AIRouter::new(config);

        assert_eq!(router.get_provider_count(), 0);
        assert!(router.is_remote_routing_enabled());
        assert_eq!(router.get_routing_strategy(), RoutingStrategy::BestFit);
    }

    #[test]
    fn test_request_context_creation() {
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

        let context = RequestContext::new(task.clone())
            .with_user_id("test_user")
            .with_session_id(uuid::Uuid::new_v4());

        assert_eq!(context.task, task);
        assert_eq!(context.user_id, Some("test_user".to_string()));
        assert!(context.session_id.is_some());
        assert!(context.routing_hint.is_none());
    }

    #[test]
    fn test_routing_strategies() {
        // Test that all routing strategies are available
        let strategies = vec![
            RoutingStrategy::FirstMatch,
            RoutingStrategy::HighestPriority,
            RoutingStrategy::LowestLatency,
            RoutingStrategy::LowestCost,
            RoutingStrategy::BestFit,
            RoutingStrategy::RoundRobin,
        ];

        for strategy in strategies {
            let config = RouterConfig {
                routing_strategy: strategy,
                ..Default::default()
            };
            let router = AIRouter::new(config);
            assert_eq!(router.get_routing_strategy(), strategy);
        }
    }

    #[test]
    fn test_provider_selector_creation() {
        let selector = ProviderSelector::new();
        let scorer = ProviderScorer::new();

        // Just verify they can be created
        assert!(std::ptr::eq(&selector, &selector));
        assert!(std::ptr::eq(&scorer, &scorer));
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that old aliases still work
        let config = Config::default();
        let router = Router::new(config);

        assert_eq!(router.get_provider_count(), 0);
    }

    #[test]
    fn test_capability_registry() {
        let registry = CapabilityRegistry::new();

        assert!(registry.list_providers().is_empty());
        assert!(registry.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_router_stats() {
        let stats = RouterStats::default();

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.average_latency_ms, 0.0);
        assert!(stats.provider_usage.is_empty());
    }

    #[test]
    fn test_node_id() {
        let node_id = NodeId("test_node".to_string());
        let node_id_clone = node_id.clone();

        assert_eq!(node_id, node_id_clone);
        assert_eq!(node_id.0, "test_node");
    }

    #[test]
    fn test_routing_hint() {
        let hint = RoutingHint {
            preferred_provider: Some("gpt-4".to_string()),
            preferred_model: Some("gpt-4-turbo".to_string()),
            allow_remote: Some(false),
            max_latency_ms: Some(1000),
            max_cost_tier: Some(crate::common::capability::CostTier::High),
            priority: Some(90),
        };

        assert_eq!(hint.preferred_provider, Some("gpt-4".to_string()));
        assert_eq!(hint.preferred_model, Some("gpt-4-turbo".to_string()));
        assert_eq!(hint.allow_remote, Some(false));
        assert_eq!(hint.max_latency_ms, Some(1000));
        assert_eq!(hint.priority, Some(90));
    }
}
