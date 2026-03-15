// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Orchestration-related types for the service composition system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use super::service::AIService;

/// Orchestration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationState {
    /// Orchestration ID
    pub orchestration_id: String,
    
    /// Current step
    pub current_step: usize,
    
    /// Total steps
    pub total_steps: usize,
    
    /// Status
    pub status: OrchestrationStatus,
    
    /// Start time
    pub start_time: DateTime<Utc>,
    
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    
    /// Context data
    pub context: serde_json::Value,
    
    /// Step results
    pub step_results: Vec<StepResult>,
}

/// Orchestration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrchestrationStatus {
    /// Pending
    Pending,
    
    /// Running
    Running,
    
    /// Completed
    Completed,
    
    /// Failed
    Failed,
    
    /// Paused
    Paused,
    
    /// Cancelled
    Cancelled,
}

/// Step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step index
    pub step_index: usize,
    
    /// Step name
    pub step_name: String,
    
    /// Result data
    pub result: serde_json::Value,
    
    /// Execution time
    pub execution_time: Duration,
    
    /// Success flag
    pub success: bool,
    
    /// Error message if any
    pub error: Option<String>,
}

/// Orchestration strategy trait
#[async_trait::async_trait]
pub trait OrchestrationStrategy: Send + Sync + std::fmt::Debug {
    /// Execute orchestration step
    async fn execute_step(
        &self,
        step_index: usize,
        context: &mut serde_json::Value,
        services: &[Arc<AIService>],
    ) -> Result<StepResult, crate::error::types::MCPError>;
    
    /// Get strategy name
    fn strategy_name(&self) -> &str;
    
    /// Check if strategy can handle the given services
    fn can_handle(&self, services: &[Arc<AIService>]) -> bool;
}

/// Sequential orchestration strategy
#[derive(Debug)]
pub struct SequentialOrchestrationStrategy;

#[async_trait::async_trait]
impl OrchestrationStrategy for SequentialOrchestrationStrategy {
    async fn execute_step(
        &self,
        step_index: usize,
        context: &mut serde_json::Value,
        services: &[Arc<AIService>],
    ) -> Result<StepResult, crate::error::types::MCPError> {
        use crate::error::types::MCPError;
        
        if step_index >= services.len() {
            return Err(MCPError::InvalidArgument(format!(
                "Step index {} out of bounds for {} services",
                step_index,
                services.len()
            )));
        }
        
        let service = &services[step_index];
        let start_time = std::time::Instant::now();
        
        // Simulate service execution
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let execution_time = start_time.elapsed();
        
        Ok(StepResult {
            step_index,
            step_name: service.name.clone(),
            result: serde_json::json!({
                "service_id": service.id,
                "status": "completed",
                "context": context
            }),
            execution_time,
            success: true,
            error: None,
        })
    }
    
    fn strategy_name(&self) -> &str {
        "sequential"
    }
    
    fn can_handle(&self, _services: &[Arc<AIService>]) -> bool {
        true // Sequential strategy can handle any services
    }
}

/// Parallel orchestration strategy
#[derive(Debug)]
pub struct ParallelOrchestrationStrategy;

#[async_trait::async_trait]
impl OrchestrationStrategy for ParallelOrchestrationStrategy {
    async fn execute_step(
        &self,
        step_index: usize,
        context: &mut serde_json::Value,
        services: &[Arc<AIService>],
    ) -> Result<StepResult, crate::error::types::MCPError> {
        use crate::error::types::MCPError;
        
        if step_index >= services.len() {
            return Err(MCPError::InvalidArgument(format!(
                "Step index {} out of bounds for {} services",
                step_index,
                services.len()
            )));
        }
        
        let service = &services[step_index];
        let start_time = std::time::Instant::now();
        
        // Simulate parallel service execution
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let execution_time = start_time.elapsed();
        
        Ok(StepResult {
            step_index,
            step_name: service.name.clone(),
            result: serde_json::json!({
                "service_id": service.id,
                "status": "completed_parallel",
                "context": context
            }),
            execution_time,
            success: true,
            error: None,
        })
    }
    
    fn strategy_name(&self) -> &str {
        "parallel"
    }
    
    fn can_handle(&self, services: &[Arc<AIService>]) -> bool {
        // Parallel strategy can handle services that support concurrent execution
        services.len() > 1
    }
} 