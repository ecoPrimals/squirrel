// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow Execution Engine
//!
//! Executes workflow steps, manages execution flow, handles retries and error recovery.
//! Provides parallel execution, conditional branching, and step orchestration.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::error::{Result, types::MCPError};
use super::coordinator::{AICoordinator};
use super::events::{EventBroadcaster, EventType, MCPEvent};
use super::service_composition::{ServiceCompositionEngine};
use super::types::*;

mod execution {
    pub mod handlers;
    pub mod resolver;
    pub mod condition;
}
use execution::{handlers, resolver, condition};

#[cfg(test)]
mod execution_tests;

#[cfg(test)]
mod quick_tests {
    use super::*;
    
    #[test]
    fn test_execution_engine_creation() {
        let engine = WorkflowExecutionEngine::new(ExecutionEngineConfig::default());
        // Just verify it was created successfully
        assert!(engine.active_executions.try_read().is_ok());
    }
}

/// Workflow execution engine
///
/// Executes workflow steps, manages execution flow, handles retries and error recovery.
/// Provides parallel execution, conditional branching, and step orchestration.
#[derive(Debug)]
pub struct WorkflowExecutionEngine {
    /// Execution configuration
    config: ExecutionEngineConfig,
    
    /// Active executions
    active_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    
    /// Execution history
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

/// Execution engine configuration
#[derive(Debug, Clone)]
pub struct ExecutionEngineConfig {
    /// Maximum parallel steps
    pub max_parallel_steps: usize,
    
    /// Default step timeout
    pub default_timeout: Duration,
    
    /// Enable execution history
    pub enable_history: bool,
    
    /// Maximum history entries
    pub max_history_entries: usize,
}

impl Default for ExecutionEngineConfig {
    fn default() -> Self {
        Self {
            max_parallel_steps: 10,
            default_timeout: Duration::from_secs(300),
            enable_history: true,
            max_history_entries: 1000,
        }
    }
}

/// Execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Workflow instance ID
    pub instance_id: String,
    
    /// Current step
    pub current_step: Option<String>,
    
    /// Completed steps
    pub completed_steps: Vec<String>,
    
    /// Failed steps
    pub failed_steps: Vec<String>,
    
    /// Step results
    pub step_results: HashMap<String, serde_json::Value>,
    
    /// Execution start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// Variables
    pub variables: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&serde_json::Value> {
        self.variables.get(name)
    }
    
    /// Set a variable value
    pub fn set_variable(&mut self, name: &str, value: serde_json::Value) -> Result<()> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
}

/// Execution record for history
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    /// Instance ID
    pub instance_id: String,
    
    /// Workflow ID
    pub workflow_id: String,
    
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Status
    pub status: WorkflowStatus,
    
    /// Error message if failed
    pub error: Option<String>,
}

impl WorkflowExecutionEngine {
    /// Create new execution engine
    pub fn new(config: ExecutionEngineConfig) -> Self {
        Self {
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute a workflow instance
    #[instrument(skip(self, workflow, ai_coordinator, service_composition, event_broadcaster))]
    pub async fn execute_workflow(
        &self,
        instance: Arc<WorkflowInstance>,
        workflow: Arc<WorkflowDefinition>,
        ai_coordinator: Arc<AICoordinator>,
        service_composition: Arc<ServiceCompositionEngine>,
        event_broadcaster: Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        info!("Executing workflow: {}", workflow.id);

        // Create execution context
        let mut context = ExecutionContext {
            instance_id: instance.id.clone(),
            current_step: None,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
            step_results: HashMap::new(),
            start_time: chrono::Utc::now(),
            variables: instance.parameters.clone(),
        };

        // Store active execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(instance.id.clone(), context.clone());
        }

        // Emit start event
        event_broadcaster
            .broadcast(MCPEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: EventType::WorkflowStarted,
                timestamp: chrono::Utc::now(),
                source: "workflow_execution_engine".to_string(),
                data: serde_json::json!({
                    "workflow_id": workflow.id,
                    "instance_id": instance.id,
                }),
                correlation_id: Some(instance.id.clone()),
            })
            .await;

        // Execute steps
        let result = self
            .execute_steps(
                &workflow.steps,
                &mut context,
                &ai_coordinator,
                &service_composition,
                &event_broadcaster,
            )
            .await;

        // Record execution
        if self.config.enable_history {
            let record = ExecutionRecord {
                instance_id: instance.id.clone(),
                workflow_id: workflow.id.clone(),
                start_time: context.start_time,
                end_time: Some(chrono::Utc::now()),
                status: if result.is_ok() {
                    WorkflowStatus::Completed
                } else {
                    WorkflowStatus::Failed
                },
                error: result.as_ref().err().map(|e| e.to_string()),
            };

            let mut history = self.execution_history.write().await;
            history.push(record);

            // Limit history size
            if history.len() > self.config.max_history_entries {
                history.remove(0);
            }
        }

        // Remove from active executions
        {
            let mut active = self.active_executions.write().await;
            active.remove(&instance.id);
        }

        // Emit completion event
        let event_type = if result.is_ok() {
            EventType::WorkflowCompleted
        } else {
            EventType::WorkflowFailed
        };

        event_broadcaster
            .broadcast(MCPEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type,
                timestamp: chrono::Utc::now(),
                source: "workflow_execution_engine".to_string(),
                data: serde_json::json!({
                    "workflow_id": workflow.id,
                    "instance_id": instance.id,
                    "result": result.is_ok(),
                }),
                correlation_id: Some(instance.id.clone()),
            })
            .await;

        result
    }

    /// Execute workflow steps
    async fn execute_steps(
        &self,
        steps: &[WorkflowStep],
        context: &mut ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
        service_composition: &Arc<ServiceCompositionEngine>,
        event_broadcaster: &Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        let mut last_result = serde_json::Value::Null;

        for step in steps {
            // Check if should execute based on condition
            if let Some(condition) = &step.condition {
                if !condition::evaluate_condition(condition, context).await? {
                    debug!("Skipping step {} due to condition", step.id);
                    continue;
                }
            }

            context.current_step = Some(step.id.clone());

            // Execute step
            match self
                .execute_single_step(
                    step,
                    context,
                    ai_coordinator,
                    service_composition,
                    event_broadcaster,
                )
                .await
            {
                Ok(result) => {
                    context.completed_steps.push(step.id.clone());
                    context.step_results.insert(step.id.clone(), result.clone());
                    last_result = result;
                }
                Err(e) => {
                    context.failed_steps.push(step.id.clone());
                    error!("Step {} failed: {}", step.id, e);
                    return Err(e);
                }
            }
        }

        Ok(last_result)
    }

    /// Execute a single workflow step
    async fn execute_single_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
        service_composition: &Arc<ServiceCompositionEngine>,
        event_broadcaster: &Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        info!("Executing step: {}", step.id);

        // Emit step started event
        event_broadcaster
            .broadcast(MCPEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: EventType::Custom("step_started".to_string()),
                timestamp: chrono::Utc::now(),
                source: "workflow_execution_engine".to_string(),
                data: serde_json::json!({
                    "step_id": step.id,
                    "instance_id": context.instance_id,
                }),
                correlation_id: Some(context.instance_id.clone()),
            })
            .await;

        // Execute based on step type
        let result = match &step.step_type {
            WorkflowStepType::AIInference => {
                handlers::execute_ai_step(step, context, ai_coordinator).await?
            }
            WorkflowStepType::ServiceComposition => {
                handlers::execute_service_step(step, context, service_composition, self.config.default_timeout)
                    .await?
            }
            WorkflowStepType::DataProcessing => {
                handlers::execute_transform_step(step, context).await?
            }
            WorkflowStepType::Condition => handlers::execute_condition_step(step, context).await?,
            WorkflowStepType::Parallel => {
                handlers::execute_parallel_step(step, context, ai_coordinator, service_composition, event_broadcaster, self.config.max_parallel_steps)
                    .await?
            }
            WorkflowStepType::Custom(ref custom_type) if custom_type == "sequential" => {
                handlers::execute_sequential_step(step, context, ai_coordinator, service_composition, event_broadcaster)
                    .await?
            }
            _ => {
                warn!("Unsupported step type: {:?}", step.step_type);
                return Err(MCPError::InvalidWorkflow(format!("Unsupported step type: {:?}", step.step_type)));
            }
        };

        // Emit step completed event
        event_broadcaster
            .broadcast(MCPEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event_type: EventType::Custom("step_completed".to_string()),
                timestamp: chrono::Utc::now(),
                source: "workflow_execution_engine".to_string(),
                data: serde_json::json!({
                    "step_id": step.id,
                    "instance_id": context.instance_id,
                }),
                correlation_id: Some(context.instance_id.clone()),
            })
            .await;

        Ok(result)
    }


    /// Get active executions
    pub async fn get_active_executions(&self) -> Vec<ExecutionContext> {
        let active = self.active_executions.read().await;
        active.values().cloned().collect()
    }

    /// Get execution history
    pub async fn get_execution_history(&self) -> Vec<ExecutionRecord> {
        let history = self.execution_history.read().await;
        history.clone()
    }
}

