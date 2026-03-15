// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Multi-Agent Coordination System
//!
//! This module provides comprehensive multi-agent coordination functionality including:
//! - Agent registration and lifecycle management
//! - Multi-agent conversations
//! - Collaboration sessions with various strategies
//! - Workflow orchestration
//!
//! ## Architecture
//!
//! The module is organized into sub-modules:
//! - `types`: Core type definitions
//! - `coordinator`: Main coordinator implementation
//! - `conversation`: Conversation and message management
//! - `collaboration`: Collaboration sessions and strategies
//! - `workflow`: Workflow orchestration
//! - `helpers`: Utility functions

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex, mpsc};

// Sub-modules
pub mod types;
pub mod coordinator;
pub mod conversation;
pub mod collaboration;
pub mod workflow;
mod helpers;

// Re-export types
pub use types::*;

// Re-export implementations
pub use coordinator::MultiAgentCoordinator;
pub use conversation::{ConversationManager, MessageDispatcher};
pub use collaboration::{CollaborationEngine, SequentialCollaborationStrategy, ParallelCollaborationStrategy};
pub use workflow::WorkflowOrchestrator;

/// Agent representation
#[derive(Debug)]
pub struct Agent {
    /// Agent ID
    pub agent_id: String,
    
    /// Agent type
    pub agent_type: AgentType,
    
    /// Agent capabilities
    pub capabilities: Vec<AgentCapability>,
    
    /// Agent state
    pub state: Arc<RwLock<AgentState>>,
    
    /// Agent configuration
    pub config: AgentConfig,
    
    /// Communication channels
    pub message_tx: mpsc::Sender<AgentMessage>,
    pub message_rx: Arc<Mutex<Option<mpsc::Receiver<AgentMessage>>>>,
    
    /// Agent metadata
    pub metadata: Arc<RwLock<AgentMetadata>>,
    
    /// Agent statistics
    pub stats: Arc<Mutex<AgentStatistics>>,
}

impl Agent {
    /// Create a new agent
    pub fn new(agent_id: String, agent_type: AgentType, config: AgentConfig) -> Self {
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        Self {
            agent_id,
            agent_type,
            capabilities: vec![],
            state: Arc::new(RwLock::new(AgentState::Idle)),
            config,
            message_tx,
            message_rx: Arc::new(Mutex::new(Some(message_rx))),
            metadata: Arc::new(RwLock::new(AgentMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: vec![],
                properties: HashMap::new(),
            })),
            stats: Arc::new(Mutex::new(AgentStatistics::default())),
        }
    }
    
    /// Get agent state
    pub async fn get_state(&self) -> AgentState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Update agent state
    pub async fn update_state(&self, new_state: AgentState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }
    
    /// Get agent statistics
    pub async fn get_statistics(&self) -> AgentStatistics {
        let stats = self.stats.lock().await;
        stats.clone()
    }
    
    /// Update statistics
    pub async fn update_statistics<F>(&self, update_fn: F) 
    where
        F: FnOnce(&mut AgentStatistics),
    {
        let mut stats = self.stats.lock().await;
        update_fn(&mut stats);
    }
}

// Default implementations

impl Default for MultiAgentMetrics {
    fn default() -> Self {
        Self {
            total_agents: 0,
            active_agents: 0,
            total_messages: 0,
            total_collaborations: 0,
            active_collaborations: 0,
            avg_response_time: Duration::from_secs(0),
            success_rate: 0.0,
            resource_utilization: 0.0,
        }
    }
}

impl Default for MultiAgentConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (agent_timeout, conversation_timeout, collaboration_timeout, workflow_timeout, metrics_interval, cleanup_interval) = 
            if let Some(cfg) = config {
                let agent = cfg.timeouts.get_custom_timeout("agent_timeout")
                    .unwrap_or_else(|| Duration::from_secs(300));
                let conv = cfg.timeouts.get_custom_timeout("agent_conversation")
                    .unwrap_or_else(|| Duration::from_secs(1800));
                let collab = cfg.timeouts.get_custom_timeout("agent_collaboration")
                    .unwrap_or_else(|| Duration::from_secs(3600));
                let wf = cfg.timeouts.get_custom_timeout("agent_workflow")
                    .unwrap_or_else(|| Duration::from_secs(7200));
                let metrics = cfg.timeouts.get_custom_timeout("agent_metrics")
                    .unwrap_or_else(|| Duration::from_secs(60));
                let cleanup = cfg.timeouts.get_custom_timeout("agent_cleanup")
                    .unwrap_or_else(|| Duration::from_secs(300));
                (agent, conv, collab, wf, metrics, cleanup)
            } else {
                (
                    Duration::from_secs(300),   // 5 minutes
                    Duration::from_secs(1800),  // 30 minutes
                    Duration::from_secs(3600),  // 1 hour
                    Duration::from_secs(7200),  // 2 hours
                    Duration::from_secs(60),    // 1 minute
                    Duration::from_secs(300),   // 5 minutes
                )
            };
        
        Self {
            max_agents: 100,
            agent_timeout,
            conversation_timeout,
            collaboration_timeout,
            workflow_timeout,
            message_buffer_size: 1000,
            metrics_interval,
            cleanup_interval,
            resources: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu: 4.0,
            max_memory: 8 * 1024 * 1024 * 1024, // 8GB
            max_concurrent_ops: 100,
        }
    }
}

impl Default for AgentStatistics {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            collaborations_participated: 0,
            errors_encountered: 0,
            avg_processing_time: Duration::from_secs(0),
            success_rate: 0.0,
            uptime: Duration::from_secs(0),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let delay = if let Some(cfg) = config {
            cfg.timeouts.get_custom_timeout("agent_retry_delay")
                .unwrap_or_else(|| Duration::from_secs(1))
        } else {
            Duration::from_secs(1)
        };
        
        Self {
            max_attempts: 3,
            delay,
            backoff_strategy: BackoffStrategy::Exponential,
        }
    }
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            strategy: ErrorHandlingStrategy::Retry,
            recovery_actions: vec![],
            notifications: vec![],
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            collaboration_preference: CollaborationPreference::Adaptive,
            communication_style: CommunicationStyle::Collaborative,
            decision_strategy: DecisionStrategy::DataDriven,
            learning_enabled: true,
            adaptation_enabled: true,
        }
    }
}

