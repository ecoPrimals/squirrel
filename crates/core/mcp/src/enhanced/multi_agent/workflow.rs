// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow Orchestration
//!
//! Manages workflow definitions and execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::error::types::MCPError;
use super::{
    MultiAgentConfig, Agent, WorkflowDefinition, WorkflowExecution, WorkflowExecutionState,
    WorkflowExecutionEngine, WorkflowDependencyResolver, WorkflowResourceManager,
    WorkflowStepResult, StepExecutor,
};

/// Workflow orchestrator
#[derive(Debug)]
pub struct WorkflowOrchestrator {
    /// Active workflows
    active_workflows: Arc<RwLock<HashMap<String, Arc<WorkflowExecution>>>>,
    /// Workflow definitions
    workflow_definitions: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    /// Execution engine
    execution_engine: Arc<WorkflowExecutionEngine>,
    /// Agent coordinator reference
    agent_coordinator: Arc<RwLock<HashMap<String, Arc<Agent>>>>,
    /// Configuration
    config: Arc<MultiAgentConfig>,
}

impl WorkflowOrchestrator {
    /// Create a new workflow orchestrator
    pub fn new(config: Arc<MultiAgentConfig>) -> Self {
        let execution_engine = WorkflowExecutionEngine::new();
        
        Self {
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            workflow_definitions: Arc::new(RwLock::new(HashMap::new())),
            execution_engine: Arc::new(execution_engine),
            agent_coordinator: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Register workflow definition
    pub async fn register_workflow(&self, workflow: WorkflowDefinition) -> Result<(), MCPError> {
        let mut definitions = self.workflow_definitions.write().await;
        definitions.insert(workflow.id.clone(), workflow.clone());
        
        info!("Registered workflow definition: {} ({})", workflow.name, workflow.id);
        Ok(())
    }
    
    /// Execute workflow
    pub async fn execute_workflow(&self, workflow_id: &str, context: serde_json::Value) -> Result<String, MCPError> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // Get workflow definition
        let definitions = self.workflow_definitions.read().await;
        let workflow_def = definitions.get(workflow_id)
            .ok_or_else(|| MCPError::NotFound(
                format!("Workflow definition {} not found", workflow_id)
            ))?
            .clone();
        drop(definitions);
        
        // Create execution
        let execution = WorkflowExecution {
            id: execution_id.clone(),
            workflow_id: workflow_id.to_string(),
            current_step: 0,
            state: WorkflowExecutionState::Executing,
            step_results: HashMap::new(),
            context: context.clone(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            assigned_agents: HashMap::new(),
        };
        
        // Store execution
        let mut workflows = self.active_workflows.write().await;
        workflows.insert(execution_id.clone(), Arc::new(execution));
        drop(workflows);
        
        info!("Started workflow execution: {} for workflow {}", execution_id, workflow_id);
        
        // Spawn workflow execution in background
        let workflow_clone = workflow_def.clone();
        let execution_id_clone = execution_id.clone();
        let active_workflows_clone = self.active_workflows.clone();
        let execution_engine_clone = self.execution_engine.clone();
        let agent_coordinator_clone = self.agent_coordinator.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::execute_workflow_steps(
                execution_id_clone,
                workflow_clone,
                context,
                active_workflows_clone,
                execution_engine_clone,
                agent_coordinator_clone,
            ).await {
                tracing::error!("Workflow execution failed: {}", e);
            }
        });
        
        Ok(execution_id)
    }
    
    /// Get workflow execution status
    pub async fn get_execution_status(&self, execution_id: &str) -> Result<WorkflowExecutionState, MCPError> {
        let workflows = self.active_workflows.read().await;
        
        if let Some(execution) = workflows.get(execution_id) {
            Ok(execution.state.clone())
        } else {
            Err(MCPError::NotFound(format!("Workflow execution not found: {}", execution_id)))
        }
    }
    
    /// Cancel workflow execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), MCPError> {
        let mut workflows = self.active_workflows.write().await;
        
        if let Some(_execution) = workflows.remove(execution_id) {
            info!("Cancelled workflow execution: {}", execution_id);
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Workflow execution not found: {}", execution_id)))
        }
    }
    
    /// List workflow definitions
    pub async fn list_workflows(&self) -> Vec<String> {
        let definitions = self.workflow_definitions.read().await;
        definitions.keys().cloned().collect()
    }
    
    /// List active executions
    pub async fn list_executions(&self) -> Vec<String> {
        let workflows = self.active_workflows.read().await;
        workflows.keys().cloned().collect()
    }
    
    /// Execute workflow steps (internal implementation)
    async fn execute_workflow_steps(
        execution_id: String,
        workflow: WorkflowDefinition,
        context: serde_json::Value,
        active_workflows: Arc<RwLock<HashMap<String, Arc<WorkflowExecution>>>>,
        execution_engine: Arc<WorkflowExecutionEngine>,
        _agent_coordinator: Arc<RwLock<HashMap<String, Arc<Agent>>>>,
    ) -> Result<(), MCPError> {
        debug!("Executing workflow steps for execution {}", execution_id);
        
        // 1. Resolve dependencies
        let execution_order = execution_engine.dependency_resolver
            .resolve_execution_order(&workflow).await?;
        
        debug!("Execution order resolved: {} steps", execution_order.len());
        
        // 2. Allocate resources for workflow
        execution_engine.resource_manager
            .allocate_resources(&workflow).await?;
        
        // 3. Execute steps in resolved order
        let mut step_results = HashMap::new();
        let mut current_context = context.clone();
        
        for step_index in execution_order {
            if step_index >= workflow.steps.len() {
                tracing::warn!("Invalid step index {} in execution order", step_index);
                continue;
            }
            
            let step = &workflow.steps[step_index];
            debug!("Executing step {}: {}", step_index, step.name);
            
            // Update execution state
            Self::update_execution_state(
                &execution_id,
                &active_workflows,
                step_index,
                WorkflowExecutionState::Executing,
            ).await?;
            
            // Execute step with retry logic
            let step_result = Self::execute_step_with_retries(
                step,
                &current_context,
                &execution_engine,
                3, // max retries
            ).await;
            
            match step_result {
                Ok(result) => {
                    // Store result and update context for next step
                    step_results.insert(step_index, result.clone());
                    
                    // Merge result into context for next steps
                    if let Some(output) = result.output {
                        if let serde_json::Value::Object(mut context_map) = current_context {
                            if let serde_json::Value::Object(output_map) = output {
                                context_map.extend(output_map);
                            }
                            current_context = serde_json::Value::Object(context_map);
                        }
                    }
                    
                    debug!("Step {} completed successfully", step_index);
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {}", step_index, e);
                    
                    // Update execution to failed state
                    Self::update_execution_state(
                        &execution_id,
                        &active_workflows,
                        step_index,
                        WorkflowExecutionState::Failed,
                    ).await?;
                    
                    // Release resources
                    execution_engine.resource_manager
                        .release_resources(&workflow).await?;
                    
                    return Err(e);
                }
            }
        }
        
        // 4. Mark workflow as completed
        Self::update_execution_state(
            &execution_id,
            &active_workflows,
            workflow.steps.len(),
            WorkflowExecutionState::Completed,
        ).await?;
        
        // 5. Release resources
        execution_engine.resource_manager
            .release_resources(&workflow).await?;
        
        info!("Workflow execution {} completed successfully", execution_id);
        Ok(())
    }
    
    /// Update execution state
    async fn update_execution_state(
        execution_id: &str,
        active_workflows: &Arc<RwLock<HashMap<String, Arc<WorkflowExecution>>>>,
        current_step: usize,
        state: WorkflowExecutionState,
    ) -> Result<(), MCPError> {
        let mut workflows = active_workflows.write().await;
        
        if let Some(execution_arc) = workflows.get_mut(execution_id) {
            // Create updated execution (Arc makes this necessary)
            let mut updated = (**execution_arc).clone();
            updated.current_step = current_step;
            updated.state = state.clone();
            
            if matches!(state, WorkflowExecutionState::Completed | WorkflowExecutionState::Failed) {
                updated.completed_at = Some(chrono::Utc::now());
            }
            
            *execution_arc = Arc::new(updated);
        }
        
        Ok(())
    }
    
    /// Execute step with retry logic
    async fn execute_step_with_retries(
        step: &super::WorkflowStep,
        context: &serde_json::Value,
        execution_engine: &Arc<WorkflowExecutionEngine>,
        max_retries: usize,
    ) -> Result<WorkflowStepResult, MCPError> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                debug!("Retrying step {} (attempt {}/{})", step.name, attempt, max_retries);
                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt as u64)).await;
            }
            
            match execution_engine.execute_step(step, context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tracing::warn!("Step {} failed, will retry: {}", step.name, last_error.as_ref().expect("last_error set on line above"));
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| MCPError::InvalidArgument("Step execution failed".to_string())))
    }
}

// Implementation of WorkflowExecutionEngine

impl WorkflowExecutionEngine {
    pub fn new() -> Self {
        Self {
            step_executors: HashMap::new(),
            dependency_resolver: Arc::new(WorkflowDependencyResolver::new()),
            resource_manager: Arc::new(WorkflowResourceManager::new()),
        }
    }
    
    /// Execute a single workflow step
    pub async fn execute_step(
        &self,
        step: &super::WorkflowStep,
        context: &serde_json::Value,
    ) -> Result<WorkflowStepResult, MCPError> {
        debug!("Executing workflow step: {}", step.name);
        
        // Check if we have a registered executor for this step type
        if let Some(executor) = self.step_executors.get(&step.step_type) {
            return executor.execute(step, context).await;
        }
        
        // Default execution: simulate step completion
        // In a real implementation, this would dispatch to actual agent/service
        Ok(WorkflowStepResult {
            step_id: step.id.clone(),
            success: true,
            output: Some(serde_json::json!({
                "step_name": step.name,
                "executed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
            error: None,
            duration_ms: 100,
        })
    }
    
    /// Register a step executor
    pub fn register_executor(&mut self, step_type: String, executor: Arc<dyn StepExecutor>) {
        self.step_executors.insert(step_type, executor);
    }
}

impl Default for WorkflowExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowDependencyResolver {
    pub fn new() -> Self {
        Self {
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Resolve execution order based on dependencies
    pub async fn resolve_execution_order(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<Vec<usize>, MCPError> {
        // Simple topological sort implementation
        // For steps without explicit dependencies, use sequential order
        
        let mut execution_order = Vec::new();
        let mut visited = vec![false; workflow.steps.len()];
        let mut in_progress = vec![false; workflow.steps.len()];
        
        // Build dependency map
        let mut dependencies: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, step) in workflow.steps.iter().enumerate() {
            let deps: Vec<usize> = step.dependencies
                .iter()
                .filter_map(|dep_id| {
                    workflow.steps.iter().position(|s| s.id == *dep_id)
                })
                .collect();
            dependencies.insert(idx, deps);
        }
        
        // Perform topological sort using DFS
        for idx in 0..workflow.steps.len() {
            if !visited[idx] {
                self.visit_node(
                    idx,
                    &dependencies,
                    &mut visited,
                    &mut in_progress,
                    &mut execution_order,
                )?;
            }
        }
        
        execution_order.reverse(); // DFS produces reverse topological order
        Ok(execution_order)
    }
    
    fn visit_node(
        &self,
        node: usize,
        dependencies: &HashMap<usize, Vec<usize>>,
        visited: &mut Vec<bool>,
        in_progress: &mut Vec<bool>,
        execution_order: &mut Vec<usize>,
    ) -> Result<(), MCPError> {
        if in_progress[node] {
            return Err(MCPError::InvalidArgument(
                "Circular dependency detected in workflow".to_string()
            ));
        }
        
        if visited[node] {
            return Ok(());
        }
        
        in_progress[node] = true;
        
        if let Some(deps) = dependencies.get(&node) {
            for &dep in deps {
                self.visit_node(dep, dependencies, visited, in_progress, execution_order)?;
            }
        }
        
        visited[node] = true;
        in_progress[node] = false;
        execution_order.push(node);
        
        Ok(())
    }
}

impl Default for WorkflowDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowResourceManager {
    pub fn new() -> Self {
        Self {
            available_resources: Arc::new(RwLock::new(HashMap::new())),
            allocated_resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Allocate resources for workflow execution
    pub async fn allocate_resources(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<(), MCPError> {
        let mut allocated = self.allocated_resources.write().await;
        
        // Track resource allocation for this workflow
        allocated.insert(
            workflow.id.clone(),
            serde_json::json!({
                "workflow_id": workflow.id,
                "allocated_at": chrono::Utc::now().to_rfc3339(),
                "steps": workflow.steps.len()
            })
        );
        
        debug!("Allocated resources for workflow: {}", workflow.id);
        Ok(())
    }
    
    /// Release resources after workflow completion
    pub async fn release_resources(
        &self,
        workflow: &WorkflowDefinition,
    ) -> Result<(), MCPError> {
        let mut allocated = self.allocated_resources.write().await;
        
        if allocated.remove(&workflow.id).is_some() {
            debug!("Released resources for workflow: {}", workflow.id);
        }
        
        Ok(())
    }
}

impl Default for WorkflowResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

