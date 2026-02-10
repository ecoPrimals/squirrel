// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Orchestration Engine Implementation
//!
//! This module contains the orchestration engine for managing service execution workflows.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

use crate::error::Result;
use super::types::{
    AIService, OrchestrationState, OrchestrationStatus, OrchestrationStrategy,
    SequentialOrchestrationStrategy, ParallelOrchestrationStrategy
};
use super::super::events::EventBroadcaster;

/// Orchestration engine
#[derive(Debug)]
pub struct OrchestrationEngine {
    /// Active orchestrations
    active_orchestrations: Arc<RwLock<HashMap<String, OrchestrationState>>>,
    /// Orchestration strategies
    strategies: HashMap<String, Box<dyn OrchestrationStrategy>>,
    /// Event broadcaster for orchestration events
    event_broadcaster: Arc<EventBroadcaster>,
}

impl OrchestrationEngine {
    /// Create a new orchestration engine
    pub fn new(event_broadcaster: Arc<EventBroadcaster>) -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(
            "sequential".to_string(), 
            Box::new(SequentialOrchestrationStrategy) as Box<dyn OrchestrationStrategy>
        );
        strategies.insert(
            "parallel".to_string(),
            Box::new(ParallelOrchestrationStrategy) as Box<dyn OrchestrationStrategy>
        );
        
        Self {
            active_orchestrations: Arc::new(RwLock::new(HashMap::new())),
            strategies,
            event_broadcaster,
        }
    }
    
    /// Start orchestration
    pub async fn start_orchestration(
        &self,
        orchestration_id: String,
        services: Vec<Arc<AIService>>,
        context: serde_json::Value,
    ) -> Result<()> {
        info!("Starting orchestration: {}", orchestration_id);
        
        let orchestration_state = OrchestrationState {
            orchestration_id: orchestration_id.clone(),
            current_step: 0,
            total_steps: services.len(),
            status: OrchestrationStatus::Running,
            start_time: chrono::Utc::now(),
            end_time: None,
            context: context.clone(),
            step_results: Vec::new(),
        };
        
        // Store orchestration state
        let mut active = self.active_orchestrations.write().await;
        active.insert(orchestration_id.clone(), orchestration_state);
        drop(active);
        
        // Find appropriate strategy
        let strategy = self.find_strategy(&services).await?;
        
        // Execute orchestration
        self.execute_orchestration_with_strategy(&orchestration_id, services, context, strategy).await
    }
    
    /// Find appropriate strategy for services
    async fn find_strategy(&self, services: &[Arc<AIService>]) -> Result<&Box<dyn OrchestrationStrategy>> {
        use crate::error::types::MCPError;
        
        // Simple strategy selection logic
        for strategy in self.strategies.values() {
            if strategy.can_handle(services) {
                return Ok(strategy);
            }
        }
        
        // Default to sequential strategy
        self.strategies.get("sequential")
            .ok_or_else(|| MCPError::InvalidArgument("No suitable orchestration strategy found".to_string()).into())
    }
    
    /// Execute orchestration with a specific strategy
    async fn execute_orchestration_with_strategy(
        &self,
        orchestration_id: &str,
        services: Vec<Arc<AIService>>,
        mut context: serde_json::Value,
        strategy: &Box<dyn OrchestrationStrategy>,
    ) -> Result<()> {
        debug!("Executing orchestration {} with strategy {}", orchestration_id, strategy.strategy_name());
        
        let mut step_results = Vec::new();
        
        // Execute each step using the strategy
        for (step_index, _service) in services.iter().enumerate() {
            match strategy.execute_step(step_index, &mut context, &services).await {
                Ok(step_result) => {
                    step_results.push(step_result);
                    self.update_orchestration_progress(orchestration_id, step_index + 1, OrchestrationStatus::Running).await?;
                }
                Err(e) => {
                    warn!("Step {} failed in orchestration {}: {}", step_index, orchestration_id, e);
                    self.update_orchestration_status(orchestration_id, OrchestrationStatus::Failed).await?;
                    return Err(e.into());
                }
            }
        }
        
        // Mark orchestration as completed
        self.update_orchestration_status(orchestration_id, OrchestrationStatus::Completed).await?;
        self.finalize_orchestration(orchestration_id, step_results).await?;
        
        info!("Orchestration {} completed successfully", orchestration_id);
        Ok(())
    }
    
    /// Update orchestration progress
    async fn update_orchestration_progress(
        &self,
        orchestration_id: &str,
        current_step: usize,
        status: OrchestrationStatus,
    ) -> Result<()> {
        let mut active = self.active_orchestrations.write().await;
        if let Some(orchestration) = active.get_mut(orchestration_id) {
            orchestration.current_step = current_step;
            orchestration.status = status;
        }
        Ok(())
    }
    
    /// Update orchestration status
    async fn update_orchestration_status(
        &self,
        orchestration_id: &str,
        status: OrchestrationStatus,
    ) -> Result<()> {
        let mut active = self.active_orchestrations.write().await;
        if let Some(orchestration) = active.get_mut(orchestration_id) {
            orchestration.status = status;
            if matches!(status, OrchestrationStatus::Completed | OrchestrationStatus::Failed | OrchestrationStatus::Cancelled) {
                orchestration.end_time = Some(chrono::Utc::now());
            }
        }
        Ok(())
    }
    
    /// Finalize orchestration
    async fn finalize_orchestration(
        &self,
        orchestration_id: &str,
        step_results: Vec<super::types::StepResult>,
    ) -> Result<()> {
        let mut active = self.active_orchestrations.write().await;
        if let Some(orchestration) = active.get_mut(orchestration_id) {
            orchestration.step_results = step_results;
        }
        Ok(())
    }
    
    /// Get orchestration status
    pub async fn get_orchestration_status(&self, orchestration_id: &str) -> Option<OrchestrationState> {
        let active = self.active_orchestrations.read().await;
        active.get(orchestration_id).cloned()
    }
    
    /// List active orchestrations
    pub async fn list_active_orchestrations(&self) -> Vec<OrchestrationState> {
        let active = self.active_orchestrations.read().await;
        active.values().cloned().collect()
    }
    
    /// Cancel orchestration
    pub async fn cancel_orchestration(&self, orchestration_id: &str) -> Result<()> {
        info!("Cancelling orchestration: {}", orchestration_id);
        self.update_orchestration_status(orchestration_id, OrchestrationStatus::Cancelled).await?;
        
        // Remove from active orchestrations
        let mut active = self.active_orchestrations.write().await;
        active.remove(orchestration_id);
        
        Ok(())
    }
} 