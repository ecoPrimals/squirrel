//! Multi-Agent Coordination System
//!
//! This module provides advanced multi-agent coordination capabilities for
//! distributed AI processing, agent collaboration patterns, and collective
//! intelligence workflows.
//!
//! This module has been refactored to use a modular structure. The main implementation
//! is now in the `multi_agent` module, with types organized separately.

// Re-export everything from the multi_agent module for backward compatibility
pub use super::multi_agent::*;

// Additional backward compatibility exports
pub use super::multi_agent::types::*;
pub use super::multi_agent::MultiAgentCoordinator;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_multi_agent_coordinator_creation() {
        let config = MultiAgentConfig::default();
        let coordinator = MultiAgentCoordinator::new(config).unwrap();
        
        // Test basic functionality
        let metrics = coordinator.get_metrics().await.unwrap();
        assert_eq!(metrics.total_agents, 0);
        assert_eq!(metrics.active_agents, 0);
    }
    
    #[tokio::test]
    async fn test_agent_registration() {
        let config = MultiAgentConfig::default();
        let coordinator = MultiAgentCoordinator::new(config).unwrap();
        
        let agent_config = AgentConfig {
            name: "test-agent".to_string(),
            description: "A test agent".to_string(),
            timeout: Duration::from_secs(30),
            retry: RetryConfig::default(),
            resources: ResourceLimits::default(),
            behavior: BehaviorConfig::default(),
            metadata: HashMap::new(),
        };
        
        let agent_id = coordinator.register_agent(agent_config).await.unwrap();
        assert!(!agent_id.is_empty());
        
        // Check metrics
        let metrics = coordinator.get_metrics().await.unwrap();
        assert_eq!(metrics.total_agents, 1);
        assert_eq!(metrics.active_agents, 1);
        
        // Check agent list
        let agents = coordinator.list_agents().await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0], agent_id);
    }
    
    #[tokio::test]
    async fn test_agent_unregistration() {
        let config = MultiAgentConfig::default();
        let coordinator = MultiAgentCoordinator::new(config).unwrap();
        
        let agent_config = AgentConfig {
            name: "test-agent".to_string(),
            description: "A test agent".to_string(),
            timeout: Duration::from_secs(30),
            retry: RetryConfig::default(),
            resources: ResourceLimits::default(),
            behavior: BehaviorConfig::default(),
            metadata: HashMap::new(),
        };
        
        let agent_id = coordinator.register_agent(agent_config).await.unwrap();
        coordinator.unregister_agent(&agent_id).await.unwrap();
        
        // Check metrics
        let metrics = coordinator.get_metrics().await.unwrap();
        assert_eq!(metrics.total_agents, 1); // Total doesn't decrease
        assert_eq!(metrics.active_agents, 0); // Active decreases
        
        // Check agent list
        let agents = coordinator.list_agents().await.unwrap();
        assert_eq!(agents.len(), 0);
    }
    
    #[tokio::test]
    async fn test_agent_message_creation() {
        let message = AgentMessage {
            id: "msg-123".to_string(),
            sender: "agent-1".to_string(),
            receiver: "agent-2".to_string(),
            message_type: MessageType::Request,
            content: serde_json::json!({
                "action": "process",
                "data": "test data"
            }),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            priority: MessagePriority::Normal,
        };
        
        assert_eq!(message.id, "msg-123");
        assert_eq!(message.sender, "agent-1");
        assert_eq!(message.receiver, "agent-2");
        assert_eq!(message.message_type, MessageType::Request);
        assert_eq!(message.priority, MessagePriority::Normal);
    }
    
    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: "A test agent".to_string(),
            timeout: Duration::from_secs(30),
            retry: RetryConfig::default(),
            resources: ResourceLimits::default(),
            behavior: BehaviorConfig::default(),
            metadata: HashMap::new(),
        };
        
        let agent = Agent::new(
            "agent-123".to_string(),
            AgentType::Processor,
            config,
        );
        
        assert_eq!(agent.agent_id, "agent-123");
        assert_eq!(agent.agent_type, AgentType::Processor);
        
        let state = agent.get_state().await;
        assert_eq!(state, AgentState::Idle);
        
        let stats = agent.get_statistics().await;
        assert_eq!(stats.messages_processed, 0);
        assert_eq!(stats.collaborations_participated, 0);
    }
    
    #[tokio::test]
    async fn test_agent_state_update() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: "A test agent".to_string(),
            timeout: Duration::from_secs(30),
            retry: RetryConfig::default(),
            resources: ResourceLimits::default(),
            behavior: BehaviorConfig::default(),
            metadata: HashMap::new(),
        };
        
        let agent = Agent::new(
            "agent-123".to_string(),
            AgentType::Processor,
            config,
        );
        
        agent.update_state(AgentState::Processing).await;
        let state = agent.get_state().await;
        assert_eq!(state, AgentState::Processing);
        
        agent.update_state(AgentState::Collaborating).await;
        let state = agent.get_state().await;
        assert_eq!(state, AgentState::Collaborating);
    }
    
    #[tokio::test]
    async fn test_agent_capability_creation() {
        let capability = AgentCapability {
            name: "text-processing".to_string(),
            description: "Process text data".to_string(),
            parameters: HashMap::from([
                ("max_length".to_string(), serde_json::json!(1000)),
                ("language".to_string(), serde_json::json!("en")),
            ]),
            requirements: vec!["memory".to_string(), "cpu".to_string()],
            performance: Some(CapabilityPerformance {
                avg_execution_time: Duration::from_millis(100),
                success_rate: 0.95,
                throughput: 50.0,
                quality_score: 0.9,
            }),
        };
        
        assert_eq!(capability.name, "text-processing");
        assert_eq!(capability.description, "Process text data");
        assert_eq!(capability.requirements.len(), 2);
        assert!(capability.performance.is_some());
        
        let perf = capability.performance.unwrap();
        assert_eq!(perf.avg_execution_time, Duration::from_millis(100));
        assert_eq!(perf.success_rate, 0.95);
        assert_eq!(perf.throughput, 50.0);
        assert_eq!(perf.quality_score, 0.9);
    }
    
    #[tokio::test]
    async fn test_collaboration_session_creation() {
        let session = CollaborationSession {
            id: "session-123".to_string(),
            session_type: CollaborationType::Parallel,
            participants: vec!["agent-1".to_string(), "agent-2".to_string()],
            state: CollaborationState::Pending,
            config: CollaborationConfig {
                timeout: Duration::from_secs(300),
                sync_strategy: SyncStrategy::Eventual,
                aggregation_strategy: AggregationStrategy::Average,
                quality_threshold: 0.8,
            },
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            completed_at: None,
        };
        
        assert_eq!(session.id, "session-123");
        assert_eq!(session.session_type, CollaborationType::Parallel);
        assert_eq!(session.participants.len(), 2);
        assert_eq!(session.state, CollaborationState::Pending);
        assert!(session.completed_at.is_none());
    }
    
    #[tokio::test]
    async fn test_workflow_definition_creation() {
        let workflow = WorkflowDefinition {
            id: "workflow-123".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "step-1".to_string(),
                    name: "Initial Processing".to_string(),
                    step_type: WorkflowStepType::SingleAgent,
                    agent_assignments: vec!["agent-1".to_string()],
                    dependencies: vec![],
                    config: serde_json::json!({
                        "process_type": "initial"
                    }),
                    timeout: Duration::from_secs(60),
                },
                WorkflowStep {
                    id: "step-2".to_string(),
                    name: "Collaborative Analysis".to_string(),
                    step_type: WorkflowStepType::Collaboration,
                    agent_assignments: vec!["agent-1".to_string(), "agent-2".to_string()],
                    dependencies: vec!["step-1".to_string()],
                    config: serde_json::json!({
                        "collaboration_type": "analysis"
                    }),
                    timeout: Duration::from_secs(120),
                },
            ],
            config: WorkflowConfig {
                execution_strategy: ExecutionStrategy::Sequential,
                timeout: Duration::from_secs(300),
                retry: RetryConfig::default(),
                error_handling: ErrorHandlingConfig::default(),
                resources: ResourceLimits::default(),
            },
            metadata: HashMap::new(),
        };
        
        assert_eq!(workflow.id, "workflow-123");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].name, "Initial Processing");
        assert_eq!(workflow.steps[1].name, "Collaborative Analysis");
        assert_eq!(workflow.steps[1].dependencies.len(), 1);
        assert_eq!(workflow.steps[1].dependencies[0], "step-1");
    }
} 