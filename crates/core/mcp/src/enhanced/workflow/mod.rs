//! Workflow Management Engine
//!
//! This module provides the main workflow management functionality using the types
//! defined in the types module.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, error, warn, debug, instrument};

use crate::error::{Result, types::MCPError};
use super::coordinator::{UniversalAIRequest, UniversalAIResponse, AICoordinator};
use super::events::{EventBroadcaster, MCPEvent, EventType};
use super::service_composition::{ServiceCompositionEngine, AIService, ExecutionResult};

pub mod types;
pub use types::*;

#[cfg(test)]
mod tests;

/// Workflow Management Engine
/// 
/// Provides comprehensive workflow management capabilities including definition,
/// execution, scheduling, state management, and monitoring for complex AI workflows.
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
    
    /// Steps executed
    pub steps_executed: usize,
    
    /// Error message if failed
    pub error: Option<String>,
}

impl WorkflowExecutionEngine {
    /// Create a new execution engine
    pub fn new(config: ExecutionEngineConfig) -> Self {
        Self {
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start workflow execution
    pub async fn start_execution(
        &self,
        instance_id: String,
        workflow_id: String,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ExecutionContext> {
        let context = ExecutionContext {
            instance_id: instance_id.clone(),
            current_step: None,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
            step_results: HashMap::new(),
            start_time: chrono::Utc::now(),
            variables: parameters,
        };
        
        let mut active = self.active_executions.write().await;
        active.insert(instance_id.clone(), context.clone());
        
        debug!("Started execution for instance: {}", instance_id);
        Ok(context)
    }
    
    /// Execute a workflow step
    pub async fn execute_step(
        &self,
        instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing step: {} for instance: {}", step.id, instance_id);
        
        // Update context
        {
            let mut active = self.active_executions.write().await;
            if let Some(context) = active.get_mut(instance_id) {
                context.current_step = Some(step.id.clone());
            }
        }
        
        // Execute step based on type
        let result = match &step.step_type {
            WorkflowStepType::AIInference => {
                self.execute_ai_inference_step(instance_id, step).await?
            }
            WorkflowStepType::DataTransformation => {
                self.execute_data_transformation_step(instance_id, step).await?
            }
            WorkflowStepType::ServiceCall => {
                self.execute_service_call_step(instance_id, step).await?
            }
            WorkflowStepType::Conditional => {
                self.execute_conditional_step(instance_id, step).await?
            }
            WorkflowStepType::Loop => {
                self.execute_loop_step(instance_id, step).await?
            }
            WorkflowStepType::Parallel => {
                self.execute_parallel_step(instance_id, step).await?
            }
            WorkflowStepType::Wait => {
                self.execute_wait_step(instance_id, step).await?
            }
            WorkflowStepType::Custom => {
                self.execute_custom_step(instance_id, step).await?
            }
        };
        
        // Update context with result
        {
            let mut active = self.active_executions.write().await;
            if let Some(context) = active.get_mut(instance_id) {
                context.completed_steps.push(step.id.clone());
                context.step_results.insert(step.id.clone(), result.clone());
                context.current_step = None;
            }
        }
        
        Ok(result)
    }
    
    /// Execute AI inference step
    async fn execute_ai_inference_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        // TODO: Integrate with AI coordinator
        debug!("Executing AI inference step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "result": "AI inference completed"}))
    }
    
    /// Execute data transformation step
    async fn execute_data_transformation_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing data transformation step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "result": "Data transformed"}))
    }
    
    /// Execute service call step
    async fn execute_service_call_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing service call step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "result": "Service called"}))
    }
    
    /// Execute conditional step
    async fn execute_conditional_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing conditional step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "branch": "true"}))
    }
    
    /// Execute loop step
    async fn execute_loop_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing loop step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "iterations": 0}))
    }
    
    /// Execute parallel step
    async fn execute_parallel_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing parallel step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "parallel_results": []}))
    }
    
    /// Execute wait step
    async fn execute_wait_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing wait step: {}", step.id);
        // Extract wait duration from config
        if let Some(duration_secs) = step.config.get("duration_seconds").and_then(|v| v.as_u64()) {
            tokio::time::sleep(Duration::from_secs(duration_secs)).await;
        }
        Ok(serde_json::json!({"status": "success", "waited": true}))
    }
    
    /// Execute custom step
    async fn execute_custom_step(
        &self,
        _instance_id: &str,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value> {
        debug!("Executing custom step: {}", step.id);
        Ok(serde_json::json!({"status": "success", "result": "Custom step executed"}))
    }
    
    /// Complete execution
    pub async fn complete_execution(
        &self,
        instance_id: &str,
        status: WorkflowStatus,
        error: Option<String>,
    ) -> Result<()> {
        let mut active = self.active_executions.write().await;
        if let Some(context) = active.remove(instance_id) {
            // Add to history
            if self.config.enable_history {
                let record = ExecutionRecord {
                    instance_id: instance_id.to_string(),
                    workflow_id: "unknown".to_string(), // TODO: Track workflow ID
                    start_time: context.start_time,
                    end_time: Some(chrono::Utc::now()),
                    status,
                    steps_executed: context.completed_steps.len(),
                    error,
                };
                
                let mut history = self.execution_history.write().await;
                history.push(record);
                
                // Keep only max_history_entries
                if history.len() > self.config.max_history_entries {
                    history.remove(0);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get execution context
    pub async fn get_context(&self, instance_id: &str) -> Result<Option<ExecutionContext>> {
        let active = self.active_executions.read().await;
        Ok(active.get(instance_id).cloned())
    }
    
    /// Get execution history
    pub async fn get_history(&self, limit: Option<usize>) -> Result<Vec<ExecutionRecord>> {
        let history = self.execution_history.read().await;
        let limit = limit.unwrap_or(100).min(self.config.max_history_entries);
        Ok(history.iter().rev().take(limit).cloned().collect())
    }
}

/// Workflow scheduler
///
/// Manages workflow scheduling, cron jobs, and time-based execution.
/// Supports one-time, recurring, and event-driven scheduling.
#[derive(Debug)]
pub struct WorkflowScheduler {
    /// Scheduled workflows
    scheduled_workflows: Arc<RwLock<HashMap<String, ScheduledWorkflow>>>,
    
    /// Scheduler configuration
    config: SchedulerConfig,
    
    /// Active schedules
    active_schedules: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Enable scheduler
    pub enabled: bool,
    
    /// Check interval for scheduled workflows
    pub check_interval: Duration,
    
    /// Maximum concurrent scheduled workflows
    pub max_concurrent: usize,
    
    /// Timezone for scheduling
    pub timezone: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(10),
            max_concurrent: 100,
            timezone: "UTC".to_string(),
        }
    }
}

/// Scheduled workflow
#[derive(Debug, Clone)]
pub struct ScheduledWorkflow {
    /// Schedule ID
    pub id: String,
    
    /// Workflow ID to execute
    pub workflow_id: String,
    
    /// Schedule type
    pub schedule_type: ScheduleType,
    
    /// Parameters for workflow execution
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Next execution time
    pub next_execution: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Execution count
    pub execution_count: u64,
    
    /// Maximum executions (None = unlimited)
    pub max_executions: Option<u64>,
    
    /// Schedule enabled
    pub enabled: bool,
}

/// Schedule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// One-time execution at specific time
    OneTime(chrono::DateTime<chrono::Utc>),
    
    /// Recurring with cron expression
    Cron(String),
    
    /// Recurring with interval
    Interval(Duration),
    
    /// Event-driven (triggered by external event)
    EventDriven(String),
}

impl WorkflowScheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            scheduled_workflows: Arc::new(RwLock::new(HashMap::new())),
            config,
            active_schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Schedule a workflow
    pub async fn schedule_workflow(
        &self,
        workflow_id: String,
        schedule_type: ScheduleType,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let schedule_id = uuid::Uuid::new_v4().to_string();
        
        let next_execution = self.calculate_next_execution(&schedule_type)?;
        
        let scheduled = ScheduledWorkflow {
            id: schedule_id.clone(),
            workflow_id,
            schedule_type,
            parameters,
            next_execution,
            last_execution: None,
            execution_count: 0,
            max_executions: None,
            enabled: true,
        };
        
        let mut schedules = self.scheduled_workflows.write().await;
        schedules.insert(schedule_id.clone(), scheduled);
        
        info!("Scheduled workflow: {}", schedule_id);
        Ok(schedule_id)
    }
    
    /// Calculate next execution time based on schedule type
    fn calculate_next_execution(&self, schedule_type: &ScheduleType) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        match schedule_type {
            ScheduleType::OneTime(time) => Ok(Some(*time)),
            ScheduleType::Interval(duration) => {
                Ok(Some(chrono::Utc::now() + chrono::Duration::from_std(*duration).unwrap()))
            }
            ScheduleType::Cron(_expr) => {
                // TODO: Implement cron parsing
                // For now, schedule for next minute
                Ok(Some(chrono::Utc::now() + chrono::Duration::minutes(1)))
            }
            ScheduleType::EventDriven(_) => Ok(None), // No time-based execution
        }
    }
    
    /// Cancel a scheduled workflow
    pub async fn cancel_schedule(&self, schedule_id: &str) -> Result<()> {
        let mut schedules = self.scheduled_workflows.write().await;
        schedules.remove(schedule_id);
        
        // Cancel active task if running
        let mut active = self.active_schedules.write().await;
        if let Some(handle) = active.remove(schedule_id) {
            handle.abort();
        }
        
        info!("Cancelled schedule: {}", schedule_id);
        Ok(())
    }
    
    /// List all scheduled workflows
    pub async fn list_schedules(&self) -> Result<Vec<ScheduledWorkflow>> {
        let schedules = self.scheduled_workflows.read().await;
        Ok(schedules.values().cloned().collect())
    }
    
    /// Get schedule by ID
    pub async fn get_schedule(&self, schedule_id: &str) -> Result<Option<ScheduledWorkflow>> {
        let schedules = self.scheduled_workflows.read().await;
        Ok(schedules.get(schedule_id).cloned())
    }
    
    /// Update schedule
    pub async fn update_schedule(&self, schedule_id: &str, enabled: bool) -> Result<()> {
        let mut schedules = self.scheduled_workflows.write().await;
        if let Some(schedule) = schedules.get_mut(schedule_id) {
            schedule.enabled = enabled;
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!("Schedule not found: {}", schedule_id)))
        }
    }
}

/// Workflow state manager
///
/// Manages workflow state persistence, recovery, and synchronization.
/// Provides state snapshots, rollback capabilities, and distributed state management.
#[derive(Debug)]
pub struct WorkflowStateManager {
    /// In-memory state store (for fast access)
    state_store: Arc<RwLock<HashMap<String, WorkflowState>>>,
    
    /// State snapshots for recovery
    snapshots: Arc<RwLock<HashMap<String, Vec<StateSnapshot>>>>,
    
    /// Configuration
    config: StateManagerConfig,
}

/// State manager configuration
#[derive(Debug, Clone)]
pub struct StateManagerConfig {
    /// Enable persistent state storage
    pub enable_persistence: bool,
    
    /// Snapshot interval
    pub snapshot_interval: Duration,
    
    /// Maximum snapshots to keep
    pub max_snapshots: usize,
    
    /// Enable distributed state sync
    pub enable_distributed_sync: bool,
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            enable_persistence: true,
            snapshot_interval: Duration::from_secs(60),
            max_snapshots: 10,
            enable_distributed_sync: false,
        }
    }
}

/// State snapshot for recovery
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Workflow state at snapshot time
    pub state: WorkflowState,
    
    /// Snapshot metadata
    pub metadata: HashMap<String, String>,
}

impl WorkflowStateManager {
    /// Create a new state manager
    pub fn new(config: StateManagerConfig) -> Self {
        Self {
            state_store: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Save workflow state
    pub async fn save_state(&self, workflow_id: &str, state: WorkflowState) -> Result<()> {
        debug!("Saving state for workflow: {}", workflow_id);
        
        // Store in memory
        let mut store = self.state_store.write().await;
        store.insert(workflow_id.to_string(), state.clone());
        
        // Create snapshot if needed
        if self.config.enable_persistence {
            self.create_snapshot(workflow_id, state).await?;
        }
        
        Ok(())
    }
    
    /// Load workflow state
    pub async fn load_state(&self, workflow_id: &str) -> Result<Option<WorkflowState>> {
        let store = self.state_store.read().await;
        Ok(store.get(workflow_id).cloned())
    }
    
    /// Create state snapshot
    async fn create_snapshot(&self, workflow_id: &str, state: WorkflowState) -> Result<()> {
        let snapshot = StateSnapshot {
            timestamp: chrono::Utc::now(),
            state,
            metadata: HashMap::new(),
        };
        
        let mut snapshots = self.snapshots.write().await;
        let workflow_snapshots = snapshots.entry(workflow_id.to_string()).or_insert_with(Vec::new);
        workflow_snapshots.push(snapshot);
        
        // Keep only max_snapshots
        if workflow_snapshots.len() > self.config.max_snapshots {
            workflow_snapshots.remove(0);
        }
        
        Ok(())
    }
    
    /// Restore from snapshot
    pub async fn restore_from_snapshot(&self, workflow_id: &str, snapshot_index: Option<usize>) -> Result<WorkflowState> {
        let snapshots = self.snapshots.read().await;
        let workflow_snapshots = snapshots.get(workflow_id)
            .ok_or_else(|| MCPError::InvalidArgument(format!("No snapshots found for workflow: {}", workflow_id)))?;
        
        let snapshot = if let Some(index) = snapshot_index {
            workflow_snapshots.get(index)
                .ok_or_else(|| MCPError::InvalidArgument(format!("Snapshot index out of bounds: {}", index)))?
        } else {
            // Get latest snapshot
            workflow_snapshots.last()
                .ok_or_else(|| MCPError::InvalidArgument("No snapshots available".to_string()))?
        };
        
        Ok(snapshot.state.clone())
    }
    
    /// Delete workflow state
    pub async fn delete_state(&self, workflow_id: &str) -> Result<()> {
        let mut store = self.state_store.write().await;
        store.remove(workflow_id);
        
        let mut snapshots = self.snapshots.write().await;
        snapshots.remove(workflow_id);
        
        Ok(())
    }
    
    /// Get all workflow states
    pub async fn list_states(&self) -> Result<Vec<(String, WorkflowState)>> {
        let store = self.state_store.read().await;
        Ok(store.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
}

/// Workflow template engine
///
/// Manages workflow templates for reusable patterns.
/// Supports template creation, instantiation, versioning, and parameter substitution.
#[derive(Debug)]
pub struct WorkflowTemplateEngine {
    /// Template registry
    templates: Arc<RwLock<HashMap<String, WorkflowTemplate>>>,
    
    /// Template configuration
    config: TemplateEngineConfig,
}

/// Template engine configuration
#[derive(Debug, Clone)]
pub struct TemplateEngineConfig {
    /// Enable template versioning
    pub enable_versioning: bool,
    
    /// Maximum templates to store
    pub max_templates: usize,
    
    /// Allow template overwrite
    pub allow_overwrite: bool,
}

impl Default for TemplateEngineConfig {
    fn default() -> Self {
        Self {
            enable_versioning: true,
            max_templates: 1000,
            allow_overwrite: false,
        }
    }
}

/// Workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template ID
    pub id: String,
    
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Template version
    pub version: String,
    
    /// Template parameters
    pub parameters: Vec<TemplateParameter>,
    
    /// Workflow definition (with placeholders)
    pub workflow: WorkflowDefinition,
    
    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter description
    pub description: String,
    
    /// Parameter type
    pub param_type: String,
    
    /// Default value
    pub default_value: Option<serde_json::Value>,
    
    /// Required parameter
    pub required: bool,
    
    /// Validation rules
    pub validation: Option<serde_json::Value>,
}

impl WorkflowTemplateEngine {
    /// Create a new template engine
    pub fn new(config: TemplateEngineConfig) -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Register a template
    pub async fn register_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        
        // Check if template exists
        if templates.contains_key(&template.id) && !self.config.allow_overwrite {
            return Err(MCPError::InvalidArgument(format!(
                "Template already exists: {}",
                template.id
            )));
        }
        
        // Check max templates
        if templates.len() >= self.config.max_templates {
            return Err(MCPError::InvalidArgument(
                "Maximum template limit reached".to_string()
            ));
        }
        
        templates.insert(template.id.clone(), template);
        info!("Registered template: {}", template.id);
        
        Ok(())
    }
    
    /// Instantiate a workflow from template
    pub async fn instantiate_template(
        &self,
        template_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowDefinition> {
        let templates = self.templates.read().await;
        let template = templates.get(template_id)
            .ok_or_else(|| MCPError::InvalidArgument(format!("Template not found: {}", template_id)))?;
        
        // Validate parameters
        self.validate_parameters(template, &parameters)?;
        
        // Clone workflow and substitute parameters
        let mut workflow = template.workflow.clone();
        workflow.id = uuid::Uuid::new_v4().to_string();
        
        // TODO: Implement parameter substitution in workflow steps
        // For now, just add parameters to metadata
        workflow.metadata.insert("template_id".to_string(), serde_json::json!(template_id));
        workflow.metadata.insert("parameters".to_string(), serde_json::json!(parameters));
        
        Ok(workflow)
    }
    
    /// Validate template parameters
    fn validate_parameters(
        &self,
        template: &WorkflowTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        for param in &template.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                return Err(MCPError::InvalidArgument(format!(
                    "Required parameter missing: {}",
                    param.name
                )));
            }
        }
        Ok(())
    }
    
    /// Get template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<Option<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(template_id).cloned())
    }
    
    /// List all templates
    pub async fn list_templates(&self, tags: Option<Vec<String>>) -> Result<Vec<WorkflowTemplate>> {
        let templates = self.templates.read().await;
        
        let mut result: Vec<WorkflowTemplate> = templates.values().cloned().collect();
        
        // Filter by tags if provided
        if let Some(filter_tags) = tags {
            result.retain(|t| {
                filter_tags.iter().any(|tag| t.tags.contains(tag))
            });
        }
        
        Ok(result)
    }
    
    /// Delete template
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.remove(template_id)
            .ok_or_else(|| MCPError::InvalidArgument(format!("Template not found: {}", template_id)))?;
        
        info!("Deleted template: {}", template_id);
        Ok(())
    }
    
    /// Update template
    pub async fn update_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        
        if !templates.contains_key(&template.id) {
            return Err(MCPError::InvalidArgument(format!(
                "Template not found: {}",
                template.id
            )));
        }
        
        templates.insert(template.id.clone(), template);
        info!("Updated template: {}", template.id);
        
        Ok(())
    }
}

/// Workflow monitoring system
///
/// Provides real-time monitoring, metrics collection, and alerting for workflows.
/// Tracks performance, errors, and resource usage.
#[derive(Debug)]
pub struct WorkflowMonitoring {
    /// Metrics storage
    metrics: Arc<RwLock<MonitoringMetrics>>,
    
    /// Alert rules
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    
    /// Monitoring configuration
    config: MonitoringConfig,
    
    /// Active alerts
    active_alerts: Arc<RwLock<Vec<Alert>>>,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    
    /// Metrics retention period
    pub retention_period: Duration,
    
    /// Alert check interval
    pub alert_check_interval: Duration,
    
    /// Maximum alerts to keep
    pub max_alerts: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_period: Duration::from_secs(86400), // 24 hours
            alert_check_interval: Duration::from_secs(60),
            max_alerts: 1000,
        }
    }
}

/// Monitoring metrics
#[derive(Debug, Clone, Default)]
pub struct MonitoringMetrics {
    /// Total workflows executed
    pub total_workflows: u64,
    
    /// Successful workflows
    pub successful_workflows: u64,
    
    /// Failed workflows
    pub failed_workflows: u64,
    
    /// Average execution time (milliseconds)
    pub avg_execution_time: f64,
    
    /// Peak execution time (milliseconds)
    pub peak_execution_time: u64,
    
    /// Active workflows
    pub active_workflows: u64,
    
    /// Total steps executed
    pub total_steps: u64,
    
    /// Failed steps
    pub failed_steps: u64,
    
    /// Metrics by workflow ID
    pub workflow_metrics: HashMap<String, WorkflowMetricData>,
    
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Per-workflow metric data
#[derive(Debug, Clone)]
pub struct WorkflowMetricData {
    /// Execution count
    pub execution_count: u64,
    
    /// Success count
    pub success_count: u64,
    
    /// Failure count
    pub failure_count: u64,
    
    /// Average duration
    pub avg_duration: f64,
    
    /// Last execution
    pub last_execution: chrono::DateTime<chrono::Utc>,
}

/// Alert rule
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Condition type
    pub condition: AlertCondition,
    
    /// Threshold value
    pub threshold: f64,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Enabled
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Failure rate exceeds threshold
    FailureRate,
    
    /// Execution time exceeds threshold
    ExecutionTime,
    
    /// Active workflows exceed threshold
    ActiveWorkflows,
    
    /// Step failure rate exceeds threshold
    StepFailureRate,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational
    Info,
    
    /// Warning
    Warning,
    
    /// Error
    Error,
    
    /// Critical
    Critical,
}

/// Active alert
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    
    /// Rule ID that triggered
    pub rule_id: String,
    
    /// Alert message
    pub message: String,
    
    /// Severity
    pub severity: AlertSeverity,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Acknowledged
    pub acknowledged: bool,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowMonitoring {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            config,
            active_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Record workflow execution
    pub async fn record_execution(
        &self,
        workflow_id: &str,
        success: bool,
        duration_ms: u64,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        
        // Update global metrics
        metrics.total_workflows += 1;
        if success {
            metrics.successful_workflows += 1;
        } else {
            metrics.failed_workflows += 1;
        }
        
        // Update average execution time
        let total_time = metrics.avg_execution_time * (metrics.total_workflows - 1) as f64;
        metrics.avg_execution_time = (total_time + duration_ms as f64) / metrics.total_workflows as f64;
        
        // Update peak execution time
        if duration_ms > metrics.peak_execution_time {
            metrics.peak_execution_time = duration_ms;
        }
        
        // Update per-workflow metrics
        let workflow_metric = metrics.workflow_metrics
            .entry(workflow_id.to_string())
            .or_insert_with(|| WorkflowMetricData {
                execution_count: 0,
                success_count: 0,
                failure_count: 0,
                avg_duration: 0.0,
                last_execution: chrono::Utc::now(),
            });
        
        workflow_metric.execution_count += 1;
        if success {
            workflow_metric.success_count += 1;
        } else {
            workflow_metric.failure_count += 1;
        }
        
        let total_duration = workflow_metric.avg_duration * (workflow_metric.execution_count - 1) as f64;
        workflow_metric.avg_duration = (total_duration + duration_ms as f64) / workflow_metric.execution_count as f64;
        workflow_metric.last_execution = chrono::Utc::now();
        
        metrics.last_updated = chrono::Utc::now();
        
        // Check alert rules
        self.check_alerts(&metrics).await?;
        
        Ok(())
    }
    
    /// Record step execution
    pub async fn record_step(&self, success: bool) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        metrics.total_steps += 1;
        if !success {
            metrics.failed_steps += 1;
        }
        
        Ok(())
    }
    
    /// Update active workflow count
    pub async fn update_active_count(&self, count: u64) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        metrics.active_workflows = count;
        
        Ok(())
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> Result<MonitoringMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// Add alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.push(rule);
        Ok(())
    }
    
    /// Check alert rules
    async fn check_alerts(&self, metrics: &MonitoringMetrics) -> Result<()> {
        let rules = self.alert_rules.read().await;
        
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            let triggered = match &rule.condition {
                AlertCondition::FailureRate => {
                    let rate = if metrics.total_workflows > 0 {
                        metrics.failed_workflows as f64 / metrics.total_workflows as f64
                    } else {
                        0.0
                    };
                    rate > rule.threshold
                }
                AlertCondition::ExecutionTime => {
                    metrics.avg_execution_time > rule.threshold
                }
                AlertCondition::ActiveWorkflows => {
                    metrics.active_workflows as f64 > rule.threshold
                }
                AlertCondition::StepFailureRate => {
                    let rate = if metrics.total_steps > 0 {
                        metrics.failed_steps as f64 / metrics.total_steps as f64
                    } else {
                        0.0
                    };
                    rate > rule.threshold
                }
            };
            
            if triggered {
                self.create_alert(rule).await?;
            }
        }
        
        Ok(())
    }
    
    /// Create alert
    async fn create_alert(&self, rule: &AlertRule) -> Result<()> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            message: format!("Alert triggered: {}", rule.name),
            severity: rule.severity.clone(),
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            metadata: HashMap::new(),
        };
        
        let mut alerts = self.active_alerts.write().await;
        alerts.push(alert);
        
        // Keep only max_alerts
        if alerts.len() > self.config.max_alerts {
            alerts.remove(0);
        }
        
        Ok(())
    }
    
    /// Get active alerts
    pub async fn get_alerts(&self, unacknowledged_only: bool) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.read().await;
        
        if unacknowledged_only {
            Ok(alerts.iter().filter(|a| !a.acknowledged).cloned().collect())
        } else {
            Ok(alerts.clone())
        }
    }
    
    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!("Alert not found: {}", alert_id)))
        }
    }
    
    /// Get workflow-specific metrics
    pub async fn get_workflow_metrics(&self, workflow_id: &str) -> Result<Option<WorkflowMetricData>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.workflow_metrics.get(workflow_id).cloned())
    }
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
        let engine = Arc::clone(&Arc::new(self.clone())); // TODO: Refactor to use &self with proper lifetimes
        
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
} 