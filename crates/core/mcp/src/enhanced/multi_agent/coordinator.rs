// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Multi-Agent Coordinator Implementation
//!
//! Core coordinator logic for managing multi-agent systems.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex, mpsc, broadcast};
use tracing::{info, warn, debug, instrument};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use super::{
    Agent, AgentConfig, AgentState, AgentMessage, AgentStatistics, AgentType, AgentMetadata,
    AgentEvent, AgentEventType, MultiAgentConfig, MultiAgentMetrics, CollaborationType,
    ConversationManager, CollaborationEngine, WorkflowOrchestrator,
};

/// Multi-agent coordination system
#[derive(Debug)]
pub struct MultiAgentCoordinator {
    /// Configuration
    config: Arc<MultiAgentConfig>,
    
    /// Active agents
    active_agents: Arc<RwLock<HashMap<String, Arc<Agent>>>>,
    
    /// Agent registry
    agent_registry: Arc<RwLock<HashMap<AgentType, Vec<String>>>>,
    
    /// Conversation manager
    conversation_manager: Arc<ConversationManager>,
    
    /// Collaboration engine
    collaboration_engine: Arc<CollaborationEngine>,
    
    /// Workflow orchestrator
    workflow_orchestrator: Arc<WorkflowOrchestrator>,
    
    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<AgentEvent>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<MultiAgentMetrics>>,
    
    /// Cleanup task handle
    cleanup_task: Arc<Mutex<Option<futures::future::AbortHandle>>>,
}

impl MultiAgentCoordinator {
    /// Create a new multi-agent coordinator
    pub fn new(config: MultiAgentConfig) -> Result<Self> {
        let config = Arc::new(config);
        let (event_tx, _) = broadcast::channel(1000);
        
        Ok(Self {
            config: config.clone(),
            active_agents: Arc::new(RwLock::new(HashMap::new())),
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            conversation_manager: Arc::new(ConversationManager::new(config.clone())),
            collaboration_engine: Arc::new(CollaborationEngine::new(config.clone())),
            workflow_orchestrator: Arc::new(WorkflowOrchestrator::new(config.clone())),
            event_broadcaster: Arc::new(event_tx),
            metrics: Arc::new(Mutex::new(MultiAgentMetrics::default())),
            cleanup_task: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Register an agent
    #[instrument(skip(self, agent_config))]
    pub async fn register_agent(&self, agent_config: AgentConfig) -> Result<String> {
        let agent_id = Uuid::new_v4().to_string();
        info!("Registering agent: {} ({})", agent_config.name, agent_id);
        
        // Create message channel
        let (message_tx, message_rx) = mpsc::channel(1000);
        
        // Create agent
        let agent = Agent {
            agent_id: agent_id.clone(),
            agent_type: AgentType::Custom(agent_config.name.clone()),
            capabilities: vec![],
            state: Arc::new(RwLock::new(AgentState::Idle)),
            config: agent_config,
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
        };
        
        // Add to active agents
        let mut active = self.active_agents.write().await;
        active.insert(agent_id.clone(), Arc::new(agent));
        
        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.total_agents += 1;
        metrics.active_agents += 1;
        
        // Broadcast agent started event
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AgentEventType::Started,
            agent_id: agent_id.clone(),
            data: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        if let Err(e) = self.event_broadcaster.send(event) {
            warn!("Failed to broadcast agent started event: {}", e);
        }
        
        Ok(agent_id)
    }
    
    /// Unregister an agent
    #[instrument(skip(self))]
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        info!("Unregistering agent: {}", agent_id);
        
        // Remove from active agents
        let mut active = self.active_agents.write().await;
        if let Some(_agent) = active.remove(agent_id) {
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.active_agents = metrics.active_agents.saturating_sub(1);
            
            // Broadcast agent stopped event
            let event = AgentEvent {
                id: Uuid::new_v4().to_string(),
                event_type: AgentEventType::Stopped,
                agent_id: agent_id.to_string(),
                data: serde_json::Value::Null,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            
            if let Err(e) = self.event_broadcaster.send(event) {
                warn!("Failed to broadcast agent stopped event: {}", e);
            }
            
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Agent not found: {}", agent_id)))
        }
    }
    
    /// Send message to agent
    #[instrument(skip(self, message))]
    pub async fn send_message(&self, agent_id: &str, message: AgentMessage) -> Result<()> {
        debug!("Sending message to agent: {}", agent_id);
        
        let active = self.active_agents.read().await;
        if let Some(agent) = active.get(agent_id) {
            agent.message_tx.send(message).await
                .map_err(|e| MCPError::Internal(format!("Failed to send message: {}", e)))?;
            
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.total_messages += 1;
            
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Agent not found: {}", agent_id)))
        }
    }
    
    /// Get agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<Option<AgentState>> {
        let active = self.active_agents.read().await;
        if let Some(agent) = active.get(agent_id) {
            let state = agent.state.read().await;
            Ok(Some(state.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// List active agents
    pub async fn list_agents(&self) -> Result<Vec<String>> {
        let active = self.active_agents.read().await;
        Ok(active.keys().cloned().collect())
    }
    
    /// Start collaboration session
    #[instrument(skip(self, participants))]
    pub async fn start_collaboration(
        &self,
        session_type: CollaborationType,
        participants: Vec<String>,
    ) -> Result<String> {
        use super::helpers::session_type_name;
        info!("Starting collaboration session: {} with {} participants", session_type_name(&session_type), participants.len());
        
        // Delegate to collaboration engine
        let session_id = self.collaboration_engine
            .start_collaboration(
                session_type.clone(),
                participants.clone(),
                serde_json::json!({}), // Empty initial data
            )
            .await?;
        
        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.total_collaborations += 1;
        metrics.active_collaborations += 1;
        
        Ok(session_id)
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> MultiAgentMetrics {
        self.metrics.lock().await.clone()
    }
    
    /// Subscribe to agent events
    pub fn subscribe_events(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_broadcaster.subscribe()
    }
}
