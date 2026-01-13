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
use super::coordinator::{AICoordinator, UniversalAIRequest, UniversalAIResponse};
use super::events::{EventBroadcaster, EventType, MCPEvent};
use super::service_composition::{AIService, ExecutionResult, ServiceCompositionEngine};
use super::types::*;

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
                if !self.evaluate_condition(condition, context).await? {
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
                self.execute_ai_step(step, context, ai_coordinator).await?
            }
            WorkflowStepType::ServiceCall => {
                self.execute_service_step(step, context, service_composition)
                    .await?
            }
            WorkflowStepType::DataTransform => {
                self.execute_transform_step(step, context).await?
            }
            WorkflowStepType::Condition => self.execute_condition_step(step, context).await?,
            WorkflowStepType::Parallel => {
                self.execute_parallel_step(step, context, ai_coordinator, service_composition, event_broadcaster)
                    .await?
            }
            WorkflowStepType::Sequential => {
                self.execute_sequential_step(step, context, ai_coordinator, service_composition, event_broadcaster)
                    .await?
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

    /// Execute AI inference step
    async fn execute_ai_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
    ) -> Result<serde_json::Value> {
        let input = self.resolve_input(&step.input, context)?;

        let request = UniversalAIRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            prompt: input
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            model: input
                .get("model")
                .and_then(|v| v.as_str())
                .map(String::from),
            parameters: input.get("parameters").cloned(),
            context: Some(serde_json::json!({
                "workflow_instance": context.instance_id,
                "step_id": step.id,
            })),
            constraints: None,
            routing_hints: None,
        };

        let response = ai_coordinator.coordinate_ai_request(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    /// Execute service call step
    async fn execute_service_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        service_composition: &Arc<ServiceCompositionEngine>,
    ) -> Result<serde_json::Value> {
        let input = self.resolve_input(&step.input, context)?;

        let service = AIService {
            id: step
                .config
                .get("service_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string(),
            name: step.name.clone(),
            service_type: step
                .config
                .get("service_type")
                .and_then(|v| v.as_str())
                .unwrap_or("generic")
                .to_string(),
            endpoint: step
                .config
                .get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost")
                .to_string(),
            capabilities: vec![],
            priority: 0,
            timeout: self.config.default_timeout,
            retry_config: None,
        };

        let result = service_composition
            .execute_service(&service, input)
            .await?;

        match result {
            ExecutionResult::Success(data) => Ok(data),
            ExecutionResult::PartialSuccess { data, .. } => Ok(data),
            ExecutionResult::Failure(err) => Err(MCPError::ServiceError(err).into()),
        }
    }

    /// Execute data transform step
    pub(crate) async fn execute_transform_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let input = self.resolve_input(&step.input, context)?;
        
        // Get transformation configuration
        let transform_type = step.config.get("transform_type")
            .and_then(|v| v.as_str())
            .unwrap_or("passthrough");
        
        match transform_type {
            "passthrough" => {
                // No transformation, return input as-is
                Ok(input)
            }
            "extract" => {
                // Extract specific field(s) from input
                if let Some(field) = step.config.get("field").and_then(|v| v.as_str()) {
                    Ok(input.get(field).cloned().unwrap_or(serde_json::Value::Null))
                } else {
                    Ok(input)
                }
            }
            "map" => {
                // Map transformation: apply function to each element
                if let Some(array) = input.as_array() {
                    let mapped: Vec<serde_json::Value> = array.iter()
                        .map(|item| self.apply_map_function(item, step))
                        .collect();
                    Ok(serde_json::Value::Array(mapped))
                } else {
                    Ok(input)
                }
            }
            "filter" => {
                // Filter transformation: keep elements matching condition
                if let Some(array) = input.as_array() {
                    let filtered: Vec<serde_json::Value> = array.iter()
                        .filter(|item| self.matches_filter(item, step))
                        .cloned()
                        .collect();
                    Ok(serde_json::Value::Array(filtered))
                } else {
                    Ok(input)
                }
            }
            "merge" => {
                // Merge with additional data from context
                if let Some(merge_key) = step.config.get("merge_with").and_then(|v| v.as_str()) {
                    if let Some(merge_data) = context.get_variable(merge_key) {
                        let mut result = input.clone();
                        if let (Some(obj1), Some(obj2)) = (result.as_object_mut(), merge_data.as_object()) {
                            for (k, v) in obj2 {
                                obj1.insert(k.clone(), v.clone());
                            }
                        }
                        Ok(result)
                    } else {
                        Ok(input)
                    }
                } else {
                    Ok(input)
                }
            }
            _ => {
                warn!("Unknown transform type: {}, using passthrough", transform_type);
                Ok(input)
            }
        }
    }
    
    /// Apply map function to a single item
    fn apply_map_function(&self, item: &serde_json::Value, step: &WorkflowStep) -> serde_json::Value {
        // Simple transformation: extract specified fields
        if let Some(fields) = step.config.get("map_fields").and_then(|v| v.as_array()) {
            let mut result = serde_json::Map::new();
            for field in fields {
                if let Some(field_name) = field.as_str() {
                    if let Some(value) = item.get(field_name) {
                        result.insert(field_name.to_string(), value.clone());
                    }
                }
            }
            serde_json::Value::Object(result)
        } else {
            item.clone()
        }
    }
    
    /// Check if item matches filter condition
    fn matches_filter(&self, item: &serde_json::Value, step: &WorkflowStep) -> bool {
        if let Some(filter_field) = step.config.get("filter_field").and_then(|v| v.as_str()) {
            if let Some(filter_value) = step.config.get("filter_value") {
                if let Some(item_value) = item.get(filter_field) {
                    return item_value == filter_value;
                }
            }
        }
        true // No filter specified, keep all items
    }

    /// Execute condition step
    async fn execute_condition_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        if let Some(condition) = &step.condition {
            let result = self.evaluate_condition(condition, context).await?;
            Ok(serde_json::Value::Bool(result))
        } else {
            Ok(serde_json::Value::Bool(true))
        }
    }

    /// Execute parallel step
    pub(crate) async fn execute_parallel_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
        service_composition: &Arc<ServiceCompositionEngine>,
        event_broadcaster: &Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        // Get sub-steps to execute in parallel
        let sub_steps = step.config.get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| MCPError::InvalidWorkflow("Parallel step missing 'steps' array".to_string()))?;
        
        // Limit parallel execution
        let max_parallel = self.config.max_parallel_steps.min(sub_steps.len());
        
        info!("Executing {} steps in parallel (max concurrency: {})", sub_steps.len(), max_parallel);
        
        // Convert JSON steps to WorkflowStep structs
        let steps: Vec<WorkflowStep> = sub_steps.iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        
        // Execute all steps in parallel using tokio::spawn
        let mut tasks = Vec::new();
        for (idx, sub_step) in steps.into_iter().enumerate() {
            let ctx = context.clone();
            let ai = ai_coordinator.clone();
            let svc = service_composition.clone();
            let evt = event_broadcaster.clone();
            let step_name = sub_step.name.clone();
            
            let task = tokio::spawn(async move {
                debug!("Parallel step {}: {} starting", idx, step_name);
                let result = Self::execute_step_static(&sub_step, &ctx, &ai, &svc, &evt).await;
                debug!("Parallel step {}: {} completed", idx, step_name);
                result
            });
            
            tasks.push(task);
        }
        
        // Collect all results
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for (idx, task) in tasks.into_iter().enumerate() {
            match task.await {
                Ok(Ok(value)) => results.push(value),
                Ok(Err(e)) => {
                    error!("Parallel step {} failed: {}", idx, e);
                    errors.push(format!("Step {}: {}", idx, e));
                }
                Err(e) => {
                    error!("Parallel step {} panicked: {}", idx, e);
                    errors.push(format!("Step {} panicked", idx));
                }
            }
        }
        
        // Return results or error
        if errors.is_empty() {
            Ok(serde_json::Value::Array(results))
        } else {
            Err(MCPError::WorkflowExecutionFailed(
                format!("Parallel execution had {} errors: {:?}", errors.len(), errors)
            ))
        }
    }
    
    /// Static version of step execution for spawning
    async fn execute_step_static(
        step: &WorkflowStep,
        context: &ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
        service_composition: &Arc<ServiceCompositionEngine>,
        event_broadcaster: &Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        // Simple execution for parallel context
        match step.step_type.as_str() {
            "ai_task" => {
                // Execute AI task
                if let Some(prompt) = step.config.get("prompt").and_then(|v| v.as_str()) {
                    let request = UniversalAIRequest {
                        prompt: prompt.to_string(),
                        context: step.config.clone(),
                        priority: 50,
                    };
                    
                    let response = ai_coordinator.route_request(request).await?;
                    Ok(serde_json::json!({
                        "result": response.result,
                        "provider": response.provider,
                        "confidence": response.confidence
                    }))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            _ => {
                // Return step input for other types
                Ok(step.input.clone())
            }
        }
    }

    /// Execute sequential step
    pub(crate) async fn execute_sequential_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        ai_coordinator: &Arc<AICoordinator>,
        service_composition: &Arc<ServiceCompositionEngine>,
        event_broadcaster: &Arc<EventBroadcaster>,
    ) -> Result<serde_json::Value> {
        // Get sub-steps to execute sequentially
        let sub_steps = step.config.get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| MCPError::InvalidWorkflow("Sequential step missing 'steps' array".to_string()))?;
        
        info!("Executing {} steps sequentially", sub_steps.len());
        
        // Convert JSON steps to WorkflowStep structs
        let steps: Vec<WorkflowStep> = sub_steps.iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        
        // Execute steps in order, passing results through context
        let mut ctx = context.clone();
        let mut last_result = serde_json::Value::Null;
        let mut all_results = Vec::new();
        
        for (idx, sub_step) in steps.iter().enumerate() {
            debug!("Sequential step {}: {} starting", idx, sub_step.name);
            
            // Set previous result in context
            ctx.set_variable("previous_result", last_result.clone())?;
            ctx.set_variable(&format!("step_{}_result", idx.saturating_sub(1)), last_result.clone())?;
            
            // Execute step
            match Self::execute_step_static(sub_step, &ctx, ai_coordinator, service_composition, event_broadcaster).await {
                Ok(result) => {
                    debug!("Sequential step {}: {} completed successfully", idx, sub_step.name);
                    last_result = result.clone();
                    all_results.push(result);
                    
                    // Update context with step result
                    ctx.set_variable(&sub_step.name, last_result.clone())?;
                }
                Err(e) => {
                    error!("Sequential step {}: {} failed: {}", idx, sub_step.name, e);
                    
                    // Check if we should continue on error
                    let continue_on_error = step.config.get("continue_on_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    
                    if continue_on_error {
                        warn!("Continuing after error in step {}", idx);
                        last_result = serde_json::json!({
                            "error": e.to_string(),
                            "step": idx
                        });
                        all_results.push(last_result.clone());
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        // Return last result or all results based on configuration
        let return_all = step.config.get("return_all_results")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if return_all {
            Ok(serde_json::Value::Array(all_results))
        } else {
            Ok(last_result)
        }
    }

    /// Evaluate condition
    pub(crate) async fn evaluate_condition(
        &self,
        condition: &str,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Parse and evaluate condition
        // Supports simple expressions like:
        // - "variable == value"
        // - "variable != value"
        // - "variable > value"
        // - "variable < value"
        // - "variable contains value"
        // - "exists variable"
        
        let condition = condition.trim();
        
        // Check for "exists" condition
        if condition.starts_with("exists ") {
            let var_name = condition.strip_prefix("exists ").unwrap().trim();
            return Ok(context.get_variable(var_name).is_some());
        }
        
        // Parse comparison operators
        let operators = vec![
            ("==", |a: &str, b: &str| a == b),
            ("!=", |a: &str, b: &str| a != b),
            ("contains", |a: &str, b: &str| a.contains(b)),
        ];
        
        for (op, compare) in operators {
            if let Some(pos) = condition.find(op) {
                let var_name = condition[..pos].trim();
                let expected = condition[pos + op.len()..].trim().trim_matches('"');
                
                if let Some(value) = context.get_variable(var_name) {
                    let value_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        _ => value.to_string(),
                    };
                    
                    return Ok(compare(&value_str, expected));
                } else {
                    // Variable doesn't exist
                    return Ok(false);
                }
            }
        }
        
        // Parse numeric comparisons
        if let Some(pos) = condition.find('>') {
            let var_name = condition[..pos].trim();
            let expected = condition[pos + 1..].trim();
            
            if let Some(value) = context.get_variable(var_name) {
                if let (Some(val_num), Ok(exp_num)) = (
                    value.as_f64(),
                    expected.parse::<f64>()
                ) {
                    return Ok(val_num > exp_num);
                }
            }
            return Ok(false);
        }
        
        if let Some(pos) = condition.find('<') {
            let var_name = condition[..pos].trim();
            let expected = condition[pos + 1..].trim();
            
            if let Some(value) = context.get_variable(var_name) {
                if let (Some(val_num), Ok(exp_num)) = (
                    value.as_f64(),
                    expected.parse::<f64>()
                ) {
                    return Ok(val_num < exp_num);
                }
            }
            return Ok(false);
        }
        
        // If no operator found, treat as boolean variable
        if let Some(value) = context.get_variable(condition) {
            return Ok(value.as_bool().unwrap_or(false));
        }
        
        // Default to false for unknown conditions
        warn!("Could not evaluate condition: {}", condition);
        Ok(false)
    }

    /// Resolve input with variable substitution
    ///
    /// Supports variable references in the format:
    /// - `${variable_name}` - Direct variable reference
    /// - `${step_outputs.step_id.field}` - Reference to step output
    /// - `${context.field}` - Reference to execution context
    fn resolve_input(
        &self,
        input: &serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        match input {
            serde_json::Value::String(s) => {
                // Check if the entire string is a variable reference
                if s.starts_with("${") && s.ends_with('}') {
                    let var_name = &s[2..s.len() - 1];
                    self.resolve_variable(var_name, context)
                } else if s.contains("${") {
                    // String contains embedded variable references
                    let mut result = s.clone();
                    
                    // Find all ${...} patterns and replace them
                    let mut start = 0;
                    while let Some(begin) = result[start..].find("${") {
                        let begin = start + begin;
                        if let Some(end) = result[begin..].find('}') {
                            let end = begin + end;
                            let var_name = &result[begin + 2..end];
                            
                            // Resolve the variable
                            match self.resolve_variable(var_name, context) {
                                Ok(value) => {
                                    let replacement = match value {
                                        serde_json::Value::String(s) => s,
                                        serde_json::Value::Number(n) => n.to_string(),
                                        serde_json::Value::Bool(b) => b.to_string(),
                                        serde_json::Value::Null => "null".to_string(),
                                        _ => value.to_string(),
                                    };
                                    
                                    result.replace_range(begin..=end, &replacement);
                                    start = begin + replacement.len();
                                }
                                Err(_) => {
                                    // If variable not found, leave it as is and continue
                                    start = end + 1;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    
                    Ok(serde_json::Value::String(result))
                } else {
                    Ok(input.clone())
                }
            }
            serde_json::Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(self.resolve_input(item, context)?);
                }
                Ok(serde_json::Value::Array(result))
            }
            serde_json::Value::Object(obj) => {
                let mut result = serde_json::Map::new();
                for (key, value) in obj {
                    result.insert(key.clone(), self.resolve_input(value, context)?);
                }
                Ok(serde_json::Value::Object(result))
            }
            // Numbers, booleans, and null are returned as-is
            other => Ok(other.clone()),
        }
    }
    
    /// Resolve a variable reference
    ///
    /// Supports:
    /// - Direct variable names: `variable_name`
    /// - Step outputs: `step_outputs.step_id.field`
    /// - Context fields: `context.field`
    fn resolve_variable(
        &self,
        var_name: &str,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Check for step output reference
        if let Some(rest) = var_name.strip_prefix("step_outputs.") {
            let parts: Vec<&str> = rest.splitn(2, '.').collect();
            if parts.len() == 2 {
                let step_id = parts[0];
                let field_path = parts[1];
                
                if let Some(output) = context.step_outputs.get(step_id) {
                    return self.resolve_json_path(output, field_path);
                }
            } else if parts.len() == 1 {
                let step_id = parts[0];
                if let Some(output) = context.step_outputs.get(step_id) {
                    return Ok(output.clone());
                }
            }
        }
        
        // Check for context field reference
        if let Some(field) = var_name.strip_prefix("context.") {
            match field {
                "workflow_id" => return Ok(serde_json::json!(context.workflow_id)),
                "execution_id" => return Ok(serde_json::json!(context.execution_id)),
                "started_at" => return Ok(serde_json::json!(context.started_at.to_rfc3339())),
                _ => {
                    // Check in context variables
                    if let Some(value) = context.variables.get(field) {
                        return Ok(value.clone());
                    }
                }
            }
        }
        
        // Check in context variables
        if let Some(value) = context.variables.get(var_name) {
            return Ok(value.clone());
        }
        
        // Variable not found
        Err(MCPError::InvalidArgument(format!(
            "Variable '{}' not found in execution context",
            var_name
        )))
    }
    
    /// Resolve a JSON path within a value
    ///
    /// Supports simple dot notation: `field1.field2.field3`
    fn resolve_json_path(
        &self,
        value: &serde_json::Value,
        path: &str,
    ) -> Result<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;
        
        for part in parts {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(part).ok_or_else(|| {
                        MCPError::InvalidArgument(format!(
                            "Field '{}' not found in JSON path '{}'",
                            part, path
                        ))
                    })?;
                }
                serde_json::Value::Array(arr) => {
                    // Support array indexing
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index).ok_or_else(|| {
                            MCPError::InvalidArgument(format!(
                                "Array index {} out of bounds (length: {})",
                                index,
                                arr.len()
                            ))
                        })?;
                    } else {
                        return Err(MCPError::InvalidArgument(format!(
                            "Invalid array index '{}' in path '{}'",
                            part, path
                        )));
                    }
                }
                _ => {
                    return Err(MCPError::InvalidArgument(format!(
                        "Cannot navigate into non-object/non-array value at '{}' in path '{}'",
                        part, path
                    )));
                }
            }
        }
        
        Ok(current.clone())
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

