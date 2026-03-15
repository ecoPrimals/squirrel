// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Workflow Management Engine
//!
//! This module provides comprehensive workflow management capabilities including:
//! - Workflow execution engine
//! - Workflow scheduling and cron jobs
//! - State management and persistence
//! - Template engine for reusable patterns
//! - Real-time monitoring and alerting
//!
//! The architecture is modular with clear separation of concerns:
//! - `execution`: Executes workflow steps with retry and error recovery
//! - `scheduler`: Time-based and event-driven workflow scheduling
//! - `state`: State persistence, snapshots, and recovery
//! - `templates`: Reusable workflow patterns with parameterization
//! - `monitoring`: Metrics collection and alerting
//! - `types`: Common types and definitions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, error, warn, instrument};

use crate::error::{Result, types::MCPError};
use super::coordinator::{AICoordinator};
use super::events::{EventBroadcaster, MCPEvent, EventType};
use super::service_composition::{ServiceCompositionEngine};

// Module declarations
pub mod types;
pub mod execution;
pub mod scheduler;
pub mod state;
pub mod templates;
pub mod monitoring;

// Re-export all public types for convenience
pub use types::*;
pub use execution::{WorkflowExecutionEngine, ExecutionEngineConfig, ExecutionContext, ExecutionRecord};
pub use scheduler::{WorkflowScheduler, SchedulerConfig, ScheduledWorkflow, ScheduleType};
pub use state::{WorkflowStateManager, StateManagerConfig, StateSnapshot};
pub use templates::{WorkflowTemplateEngine, TemplateEngineConfig, WorkflowTemplate, TemplateParameter};
pub use monitoring::{
    WorkflowMonitoring, MonitoringConfig, MonitoringMetrics, WorkflowMetricData,
    AlertRule, AlertCondition, AlertSeverity, Alert,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scheduler_tests;

#[cfg(test)]
mod template_tests;

/// Workflow Management Engine
/// 
/// Main orchestration engine that coordinates all workflow subsystems.
/// Provides a unified interface for workflow definition, execution, scheduling,
/// state management, templating, and monitoring.
#[derive(Debug)]
pub struct WorkflowManagementEngine {
    /// Configuration
    config: Arc<WorkflowManagementConfig>,
    
    /// Workflow registry
    workflow_registry: Arc<RwLock<HashMap<String, Arc<WorkflowDefinition>>>>,
    
    /// Execution engine
    execution_engine: Arc<WorkflowExecutionEngine>,
    
    /// Scheduler
    scheduler: Arc<WorkflowScheduler>,
    
    /// State manager
    state_manager: Arc<WorkflowStateManager>,
    
    /// Event broadcaster
    event_broadcaster: Arc<EventBroadcaster>,
    
    /// Service composition engine
    service_composition: Arc<ServiceCompositionEngine>,
    
    /// AI coordinator
    ai_coordinator: Arc<AICoordinator>,
    
    /// Active workflows
    active_workflows: Arc<RwLock<HashMap<String, Arc<WorkflowInstance>>>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<WorkflowMetrics>>,
    
    /// Template engine
    template_engine: Arc<WorkflowTemplateEngine>,
    
    /// Monitoring system
    monitoring: Arc<WorkflowMonitoring>,
}

impl WorkflowManagementEngine {
    /// Create a new workflow management engine
    pub fn new(
        config: WorkflowManagementConfig,
        event_broadcaster: Arc<EventBroadcaster>,
        service_composition: Arc<ServiceCompositionEngine>,
        ai_coordinator: Arc<AICoordinator>,
    ) -> Self {
        let config = Arc::new(config);
        
        Self {
            config: config.clone(),
            workflow_registry: Arc::new(RwLock::new(HashMap::new())),
            execution_engine: Arc::new(WorkflowExecutionEngine::new(ExecutionEngineConfig::default())),
            scheduler: Arc::new(WorkflowScheduler::new(SchedulerConfig::default())),
            state_manager: Arc::new(WorkflowStateManager::new(StateManagerConfig::default())),
            event_broadcaster,
            service_composition,
            ai_coordinator,
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(WorkflowMetrics::default())),
            template_engine: Arc::new(WorkflowTemplateEngine::new(TemplateEngineConfig::default())),
            monitoring: Arc::new(WorkflowMonitoring::new(MonitoringConfig::default())),
        }
    }
    
    /// Register a workflow definition
    #[instrument(skip(self, workflow))]
    pub async fn register_workflow(&self, workflow: WorkflowDefinition) -> Result<()> {
        info!("Registering workflow: {}", workflow.name);
        
        let mut registry = self.workflow_registry.write().await;
        registry.insert(workflow.id.clone(), Arc::new(workflow));
        
        Ok(())
    }
    
    /// Execute a workflow
    #[instrument(skip(self, parameters))]
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<Arc<WorkflowInstance>> {
        info!("Executing workflow: {}", workflow_id);
        
        // Get workflow definition
        let registry = self.workflow_registry.read().await;
        let workflow = registry.get(workflow_id)
            .ok_or_else(|| MCPError::InvalidArgument(format!("Workflow not found: {}", workflow_id)))?;
        
        // Create workflow instance
        let instance = WorkflowInstance {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: workflow_id.to_string(),
            state: WorkflowState::Pending,
            parameters,
            outputs: HashMap::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            step_states: HashMap::new(),
        };
        
        let instance = Arc::new(instance);
        
        // Add to active workflows
        let instance_id = instance.id.clone(); // Clone only the ID, not the entire instance
        {
            let mut active = self.active_workflows.write().await;
            active.insert(instance_id.clone(), Arc::clone(&instance)); // Use Arc::clone for pointer sharing
        }
        
        // Start workflow execution asynchronously
        // Use Arc references to avoid heavy cloning
        let workflow_ref = Arc::clone(&workflow);
        let instance_ref = Arc::clone(&instance);
        let engine = Arc::clone(&Arc::new(self.clone())); // FUTURE: [Refactor] Refactor to use &self with proper lifetimes
        // Tracking: Requires lifetime analysis and refactoring
        
        tokio::spawn(async move {
            if let Err(e) = (*engine).execute_workflow_steps(workflow_ref, instance_ref).await {
                error!("Workflow execution failed: {}", e);
                (*engine).handle_workflow_error(&instance_id, e).await;
            }
        });
        
        Ok(instance)
    }
    
    /// Get workflow status
    pub async fn get_workflow_status(&self, instance_id: &str) -> Result<Option<Arc<WorkflowInstance>>> {
        let active = self.active_workflows.read().await;
        Ok(active.get(instance_id).cloned())
    }
    
    /// Cancel a workflow
    #[instrument(skip(self))]
    pub async fn cancel_workflow(&self, instance_id: &str) -> Result<()> {
        info!("Cancelling workflow: {}", instance_id);
        
        // Update workflow state to Cancelled
        self.update_workflow_state(instance_id, WorkflowState::Cancelled).await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.failed_workflows += 1; // Cancelled workflows count as failed
        }
        
        // Publish cancellation event
        let event = crate::enhanced::events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::enhanced::events::EventType::WorkflowCancelled,
            source: crate::enhanced::events::EventSource::WorkflowEngine,
            data: serde_json::json!({
                "instance_id": instance_id,
                "cancelled_at": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        if let Err(e) = self.event_broadcaster.broadcast_event(event).await {
            warn!("Failed to broadcast workflow cancellation event: {}", e);
        }
        
        // Remove from active workflows
        {
            let mut active = self.active_workflows.write().await;
            active.remove(instance_id);
        }
        
        info!("Workflow {} cancelled successfully", instance_id);
        Ok(())
    }
    
    /// Get workflow metrics
    pub async fn get_metrics(&self) -> Result<WorkflowMetrics> {
        let metrics = self.metrics.lock().await;
        Ok(metrics.clone())
    }
    
    /// List active workflows
    pub async fn list_active_workflows(&self) -> Result<Vec<Arc<WorkflowInstance>>> {
        let active = self.active_workflows.read().await;
        Ok(active.values().cloned().collect())
    }
    
    /// Get workflow definition
    pub async fn get_workflow_definition(&self, workflow_id: &str) -> Result<Option<Arc<WorkflowDefinition>>> {
        let registry = self.workflow_registry.read().await;
        Ok(registry.get(workflow_id).cloned())
    }
    
    /// List workflow definitions
    pub async fn list_workflow_definitions(&self) -> Result<Vec<Arc<WorkflowDefinition>>> {
        let registry = self.workflow_registry.read().await;
        Ok(registry.values().cloned().collect())
    }

    /// Execute workflow steps sequentially or in parallel based on configuration
    #[instrument(skip(self, workflow, instance))]
    async fn execute_workflow_steps(
        &self,
        workflow: Arc<WorkflowDefinition>,
        instance: Arc<WorkflowInstance>,
    ) -> Result<()> {
        info!("Executing {} steps for workflow {}", workflow.steps.len(), instance.id);
        
        // Update instance state to Running
        self.update_workflow_state(&instance.id, WorkflowState::Running).await?;
        
        // Execute steps based on execution strategy
        match workflow.config.execution_strategy {
            ExecutionStrategy::Sequential => {
                for (step_index, step) in workflow.steps.iter().enumerate() {
                    if let Err(e) = self.execute_step(&instance.id, step_index, step, &instance.parameters).await {
                        error!("Step {} failed for workflow {}: {}", step_index, instance.id, e);
                        self.update_workflow_state(&instance.id, WorkflowState::Failed).await?;
                        return Err(e);
                    }
                }
            }
            ExecutionStrategy::Parallel => {
                let mut handles = vec![];
                for (step_index, step) in workflow.steps.iter().enumerate() {
                    let engine = self.clone();
                    let instance_id = instance.id.clone();
                    let step_clone = step.clone();
                    let parameters = instance.parameters.clone();
                    
                    let handle = tokio::spawn(async move {
                        engine.execute_step(&instance_id, step_index, &step_clone, &parameters).await
                    });
                    handles.push(handle);
                }
                
                // Wait for all steps to complete
                for (step_index, handle) in handles.into_iter().enumerate() {
                    if let Err(e) = handle.await.map_err(|e| MCPError::Internal(e.to_string()))?.map_err(|e| e) {
                        error!("Parallel step {} failed for workflow {}: {}", step_index, instance.id, e);
                        self.update_workflow_state(&instance.id, WorkflowState::Failed).await?;
                        return Err(e);
                    }
                }
            }
        }
        
        // Mark workflow as completed
        self.update_workflow_state(&instance.id, WorkflowState::Completed).await?;
        info!("Workflow {} completed successfully", instance.id);
        
        Ok(())
    }
    
    /// Execute a single workflow step
    #[instrument(skip(self, parameters))]
    async fn execute_step(
        &self,
        instance_id: &str,
        step_index: usize,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        info!("Executing step {} ({}) for workflow {}", step_index, step.name, instance_id);
        
        // Update step state to Running
        self.update_step_state(instance_id, step_index, StepState::Running).await?;
        
        // Execute step based on type
        let result = match &step.step_type {
            StepType::AIService => {
                self.execute_ai_service_step(step, parameters).await
            }
            StepType::ServiceComposition => {
                self.execute_service_composition_step(step, parameters).await
            }
            StepType::DataProcessing => {
                self.execute_data_processing_step(step, parameters).await
            }
            StepType::Condition => {
                self.execute_condition_step(step, parameters).await
            }
            StepType::Wait => {
                self.execute_wait_step(step, parameters).await
            }
            StepType::Notification => {
                self.execute_notification_step(step, parameters).await
            }
            _ => {
                warn!("Step type {:?} not implemented yet", step.step_type);
                Ok(serde_json::Value::Null)
            }
        };
        
        match result {
            Ok(output) => {
                info!("Step {} completed successfully", step_index);
                self.update_step_state(instance_id, step_index, StepState::Completed).await?;
                self.store_step_output(instance_id, step_index, output).await?;
                Ok(())
            }
            Err(e) => {
                error!("Step {} failed: {}", step_index, e);
                self.update_step_state(instance_id, step_index, StepState::Failed).await?;
                Err(e)
            }
        }
    }

    /// Execute an AI service step
    async fn execute_ai_service_step(
        &self,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing AI service step: {}", step.name);
        
        // Use the AI coordinator to execute the service
        let request = crate::enhanced::coordinator::UniversalAIRequest {
            id: uuid::Uuid::new_v4().to_string(),
            model: step.config.get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("gpt-4")
                .to_string(),
            messages: vec![crate::enhanced::coordinator::Message {
                role: "user".to_string(),
                content: step.config.get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Execute AI service step")
                    .to_string(),
            }],
            parameters: parameters.clone(),
        };
        
        let response = self.ai_coordinator.execute_request(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    /// Execute a service composition step
    async fn execute_service_composition_step(
        &self,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing service composition step: {}", step.name);
        
        // Use the service composition engine
        let composition_id = step.config.get("composition_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::InvalidArgument("Missing composition_id".to_string()))?;
        
        let request = crate::enhanced::coordinator::UniversalAIRequest {
            id: uuid::Uuid::new_v4().to_string(),
            model: "service-composition".to_string(),
            messages: vec![],
            parameters: parameters.clone(),
        };
        
        let result = self.service_composition.execute_composition(composition_id, request).await?;
        Ok(serde_json::to_value(result)?)
    }

    /// Execute a data processing step
    async fn execute_data_processing_step(
        &self,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing data processing step: {}", step.name);
        
        // Simple data transformation based on step configuration
        let input_key = step.config.get("input_key")
            .and_then(|v| v.as_str())
            .unwrap_or("input");
            
        let input_data = parameters.get(input_key)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
            
        let operation = step.config.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("identity");
            
        let result = match operation {
            "uppercase" => {
                if let Some(text) = input_data.as_str() {
                    serde_json::Value::String(text.to_uppercase())
                } else {
                    input_data
                }
            }
            "lowercase" => {
                if let Some(text) = input_data.as_str() {
                    serde_json::Value::String(text.to_lowercase())
                } else {
                    input_data
                }
            }
            "identity" | _ => input_data,
        };
        
        Ok(result)
    }

    /// Execute a condition step
    async fn execute_condition_step(
        &self,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing condition step: {}", step.name);
        
        // Simple condition evaluation
        let condition_key = step.config.get("condition_key")
            .and_then(|v| v.as_str())
            .unwrap_or("condition");
            
        let expected_value = step.config.get("expected_value")
            .cloned()
            .unwrap_or(serde_json::Value::Bool(true));
            
        let actual_value = parameters.get(condition_key)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
            
        let condition_met = actual_value == expected_value;
        
        Ok(serde_json::Value::Bool(condition_met))
    }

    /// Execute a wait step
    async fn execute_wait_step(
        &self,
        step: &WorkflowStep,
        _parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing wait step: {}", step.name);
        
        let duration = step.config.get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);
            
        tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
        
        Ok(serde_json::Value::String(format!("Waited {}ms", duration)))
    }

    /// Execute a notification step
    async fn execute_notification_step(
        &self,
        step: &WorkflowStep,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        debug!("Executing notification step: {}", step.name);
        
        let message = step.config.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification from workflow step");
            
        // Publish event through event broadcaster
        let event = crate::enhanced::events::MCPEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: crate::enhanced::events::EventType::WorkflowNotification,
            source: crate::enhanced::events::EventSource::WorkflowEngine,
            data: serde_json::json!({
                "message": message,
                "step": step.name.clone(),
                "parameters": parameters
            }),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        self.event_broadcaster.broadcast_event(event).await?;
        
        Ok(serde_json::Value::String(message.to_string()))
    }

    /// Handle workflow execution error
    async fn handle_workflow_error(&self, instance_id: &str, error: crate::error::types::MCPError) {
        error!("Workflow {} failed: {}", instance_id, error);
        
        // Update workflow state
        if let Err(e) = self.update_workflow_state(instance_id, WorkflowState::Failed).await {
            error!("Failed to update workflow state after error: {}", e);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.failed_workflows += 1;
        }
        
        // Remove from active workflows
        {
            let mut active = self.active_workflows.write().await;
            active.remove(instance_id);
        }
    }

    /// Update workflow instance state
    async fn update_workflow_state(&self, instance_id: &str, new_state: WorkflowState) -> Result<()> {
        let mut active = self.active_workflows.write().await;
        if let Some(instance) = active.get_mut(instance_id) {
            let instance = Arc::get_mut(instance).ok_or_else(|| {
                MCPError::Internal("Cannot get mutable reference to workflow instance".to_string())
            })?;
            
            instance.state = new_state.clone();
            
            // Update timestamps
            match new_state {
                WorkflowState::Running => {
                    instance.started_at = Some(chrono::Utc::now());
                }
                WorkflowState::Completed | WorkflowState::Failed | WorkflowState::Cancelled => {
                    instance.completed_at = Some(chrono::Utc::now());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Update step state
    async fn update_step_state(&self, instance_id: &str, step_index: usize, new_state: StepState) -> Result<()> {
        let mut active = self.active_workflows.write().await;
        if let Some(instance) = active.get_mut(instance_id) {
            let instance = Arc::get_mut(instance).ok_or_else(|| {
                MCPError::Internal("Cannot get mutable reference to workflow instance".to_string())
            })?;
            
            instance.step_states.insert(step_index.to_string(), new_state);
        }
        
        Ok(())
    }

    /// Store step output
    async fn store_step_output(
        &self,
        instance_id: &str,
        step_index: usize,
        output: serde_json::Value,
    ) -> Result<()> {
        let mut active = self.active_workflows.write().await;
        if let Some(instance) = active.get_mut(instance_id) {
            let instance = Arc::get_mut(instance).ok_or_else(|| {
                MCPError::Internal("Cannot get mutable reference to workflow instance".to_string())
            })?;
            
            instance.outputs.insert(format!("step_{}", step_index), output);
        }
        
        Ok(())
    }
}

impl Default for WorkflowMetrics {
    fn default() -> Self {
        Self {
            total_workflows: 0,
            active_workflows: 0,
            completed_workflows: 0,
            failed_workflows: 0,
            avg_execution_time: Duration::from_secs(0),
            success_rate: 0.0,
        }
    }
}

impl Default for WorkflowManagementConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (default_timeout, metrics_interval, cleanup_interval) = if let Some(cfg) = config {
            let timeout = cfg.timeouts.get_custom_timeout("wfmgmt_default")
                .unwrap_or_else(|| Duration::from_secs(3600));
            let metrics = cfg.timeouts.get_custom_timeout("wfmgmt_metrics")
                .unwrap_or_else(|| Duration::from_secs(60));
            let cleanup = cfg.timeouts.get_custom_timeout("wfmgmt_cleanup")
                .unwrap_or_else(|| Duration::from_secs(300));
            (timeout, metrics, cleanup)
        } else {
            (
                Duration::from_secs(3600),  // 1 hour
                Duration::from_secs(60),    // 1 minute
                Duration::from_secs(300),   // 5 minutes
            )
        };
        
        Self {
            max_concurrent_workflows: 100,
            default_timeout,
            metrics_interval,
            cleanup_interval,
            storage: StorageConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: "memory".to_string(),
            connection_string: "".to_string(),
            config: HashMap::new(),
        }
    }
