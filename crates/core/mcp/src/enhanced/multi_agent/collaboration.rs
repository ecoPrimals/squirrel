//! Collaboration Engine
//!
//! Manages multi-agent collaboration sessions and strategies.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::error::types::MCPError;
use super::{
    MultiAgentConfig, CollaborationType, CollaborationSession, CollaborationState,
    CollaborationStrategy, CollaborationResult, CollaborationConfig, AgentInfo,
};

/// Collaboration engine
#[derive(Debug)]
pub struct CollaborationEngine {
    /// Active collaborations
    active_collaborations: Arc<RwLock<HashMap<String, Arc<CollaborationSession>>>>,
    /// Collaboration strategies
    strategies: HashMap<CollaborationType, Box<dyn CollaborationStrategy>>,
    /// Agent registry reference
    agent_registry: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// Configuration
    config: Arc<MultiAgentConfig>,
}

impl CollaborationEngine {
    /// Create a new collaboration engine
    pub fn new(config: Arc<MultiAgentConfig>) -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(CollaborationType::Sequential, Box::new(SequentialCollaborationStrategy) as Box<dyn CollaborationStrategy>);
        strategies.insert(CollaborationType::Parallel, Box::new(ParallelCollaborationStrategy) as Box<dyn CollaborationStrategy>);
        
        Self {
            active_collaborations: Arc::new(RwLock::new(HashMap::new())),
            strategies,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Start a collaboration session
    pub async fn start_collaboration(
        &self,
        collaboration_type: CollaborationType,
        participants: Vec<String>,
        data: serde_json::Value,
    ) -> Result<String, MCPError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = CollaborationSession {
            id: session_id.clone(),
            collaboration_type: collaboration_type.clone(),
            participants: participants.clone(),
            state: CollaborationState::Pending,
            data,
            results: Vec::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            config: CollaborationConfig::default(),
        };
        
        let mut collaborations = self.active_collaborations.write().await;
        collaborations.insert(session_id.clone(), Arc::new(session));
        
        info!("Started {} collaboration {} with {} participants", 
              match collaboration_type {
                  CollaborationType::Sequential => "sequential",
                  CollaborationType::Parallel => "parallel",
                  CollaborationType::Hierarchical => "hierarchical",
                  CollaborationType::PeerToPeer => "peer-to-peer",
              },
              session_id, participants.len());
        
        Ok(session_id)
    }
    
    /// Execute collaboration
    pub async fn execute_collaboration(
        &self,
        session_id: &str,
    ) -> Result<CollaborationResult, MCPError> {
        let collaborations = self.active_collaborations.read().await;
        
        let session = collaborations.get(session_id)
            .ok_or_else(|| MCPError::NotFound(format!("Collaboration not found: {}", session_id)))?;
        
        // Get strategy
        let strategy = self.strategies.get(&session.collaboration_type)
            .ok_or_else(|| MCPError::Internal(
                format!("No strategy found for collaboration type: {:?}", session.collaboration_type)
            ))?;
        
        // Execute strategy
        debug!("Executing {:?} collaboration strategy for session {}", 
               session.collaboration_type, session_id);
        
        let result = strategy.execute(session, &self.agent_registry).await?;
        
        Ok(result)
    }
    
    /// Get collaboration status
    pub async fn get_collaboration_status(&self, session_id: &str) -> Result<CollaborationState, MCPError> {
        let collaborations = self.active_collaborations.read().await;
        
        if let Some(session) = collaborations.get(session_id) {
            Ok(session.state.clone())
        } else {
            Err(MCPError::NotFound(format!("Collaboration not found: {}", session_id)))
        }
    }
    
    /// End collaboration
    pub async fn end_collaboration(&self, session_id: &str) -> Result<(), MCPError> {
        let mut collaborations = self.active_collaborations.write().await;
        collaborations.remove(session_id);
        
        info!("Ended collaboration session {}", session_id);
        Ok(())
    }
}

/// Sequential collaboration strategy
#[derive(Debug)]
pub struct SequentialCollaborationStrategy;

#[async_trait::async_trait]
impl CollaborationStrategy for SequentialCollaborationStrategy {
    async fn execute(
        &self,
        session: &CollaborationSession,
        agent_registry: &Arc<RwLock<HashMap<String, AgentInfo>>>,
    ) -> Result<CollaborationResult, MCPError> {
        info!("Executing sequential collaboration for session {}", session.id);
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut context = session.context.clone();
        
        // Execute each agent's task sequentially, passing results forward
        for agent_id in &session.agents {
            let registry = agent_registry.read().await;
            let agent_info = registry.get(agent_id).ok_or_else(|| {
                MCPError::NotFound(format!("Agent {} not found", agent_id))
            })?;
            
            debug!("Sequential execution: Agent {} executing", agent_id);
            
            // Execute agent task with accumulated context
            match Self::execute_agent_task(agent_info, &context).await {
                Ok(result) => {
                    // Add result to context for next agent
                    if let serde_json::Value::Object(mut ctx) = context {
                        ctx.insert(
                            format!("agent_{}_result", agent_id),
                            serde_json::json!({
                                "agent_id": agent_id,
                                "output": result.clone(),
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            })
                        );
                        context = serde_json::Value::Object(ctx);
                    }
                    results.push(result);
                }
                Err(e) => {
                    errors.push(format!("Agent {} failed: {}", agent_id, e));
                    // In sequential mode, stop on first error
                    break;
                }
            }
        }
        
        Ok(CollaborationResult {
            session_id: session.id.clone(),
            success: errors.is_empty(),
            results,
            errors,
            metadata: serde_json::json!({
                "strategy": "sequential",
                "agents_executed": results.len(),
                "total_agents": session.agents.len()
            }).as_object().unwrap().clone(),
        })
    }
    
    fn name(&self) -> &str {
        "sequential"
    }
}

impl SequentialCollaborationStrategy {
    async fn execute_agent_task(
        agent_info: &AgentInfo,
        context: &serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        // Simulate agent task execution
        // In real implementation, this would dispatch to actual agent
        debug!("Executing task for agent: {}", agent_info.agent_id);
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(serde_json::json!({
            "agent_id": agent_info.agent_id,
            "status": "completed",
            "result": format!("Agent {} processed context", agent_info.agent_id),
            "context_size": context.to_string().len()
        }))
    }
}

/// Parallel collaboration strategy
#[derive(Debug)]
pub struct ParallelCollaborationStrategy;

#[async_trait::async_trait]
impl CollaborationStrategy for ParallelCollaborationStrategy {
    async fn execute(
        &self,
        session: &CollaborationSession,
        agent_registry: &Arc<RwLock<HashMap<String, AgentInfo>>>,
    ) -> Result<CollaborationResult, MCPError> {
        info!("Executing parallel collaboration for session {}", session.id);
        
        let mut handles = Vec::new();
        
        // Launch all agent tasks in parallel
        for agent_id in &session.agents {
            let registry_clone = agent_registry.clone();
            let agent_id_clone = agent_id.clone();
            let context_clone = session.context.clone();
            
            let handle = tokio::spawn(async move {
                let registry = registry_clone.read().await;
                let agent_info = registry.get(&agent_id_clone).ok_or_else(|| {
                    MCPError::NotFound(format!("Agent {} not found", agent_id_clone))
                })?;
                
                debug!("Parallel execution: Agent {} executing", agent_id_clone);
                
                Self::execute_agent_task(agent_info, &context_clone).await
            });
            
            handles.push((agent_id.clone(), handle));
        }
        
        // Wait for all tasks to complete
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for (agent_id, handle) in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    results.push(result);
                }
                Ok(Err(e)) => {
                    errors.push(format!("Agent {} failed: {}", agent_id, e));
                }
                Err(e) => {
                    errors.push(format!("Agent {} task panicked: {}", agent_id, e));
                }
            }
        }
        
        Ok(CollaborationResult {
            session_id: session.id.clone(),
            success: errors.is_empty(),
            results,
            errors,
            metadata: serde_json::json!({
                "strategy": "parallel",
                "agents_succeeded": results.len(),
                "agents_failed": errors.len(),
                "total_agents": session.agents.len()
            }).as_object().unwrap().clone(),
        })
    }
    
    fn name(&self) -> &str {
        "parallel"
    }
}

impl ParallelCollaborationStrategy {
    async fn execute_agent_task(
        agent_info: &AgentInfo,
        context: &serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        // Simulate agent task execution
        // In real implementation, this would dispatch to actual agent
        debug!("Executing task for agent: {}", agent_info.agent_id);
        
        // Simulate some work with varying durations
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        
        Ok(serde_json::json!({
            "agent_id": agent_info.agent_id,
            "status": "completed",
            "result": format!("Agent {} processed context in parallel", agent_info.agent_id),
            "context_size": context.to_string().len()
        }))
    }
}
