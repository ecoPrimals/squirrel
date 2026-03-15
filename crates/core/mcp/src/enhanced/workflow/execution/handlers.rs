// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Step Execution Handlers
//!
//! Handlers for executing different types of workflow steps:
//! - AI inference steps
//! - Service call steps
//! - Data transform steps
//! - Condition steps
//! - Parallel steps
//! - Sequential steps

use std::sync::Arc;
use tracing::{debug, error, info, warn};
use crate::error::{Result, types::MCPError};
use crate::enhanced::coordinator::{AICoordinator, UniversalAIRequest};
use crate::enhanced::events::{EventBroadcaster};
use crate::enhanced::service_composition::{AIService, ExecutionResult, ServiceCompositionEngine};
use crate::enhanced::workflow::types::{ExecutionContext, WorkflowStep};
use super::resolver::resolve_input;
use super::condition::evaluate_condition;

/// Execute AI inference step
pub async fn execute_ai_step(
    step: &WorkflowStep,
    context: &ExecutionContext,
    ai_coordinator: &Arc<AICoordinator>,
) -> Result<serde_json::Value> {
    let input = resolve_input(&step.input, context)?;

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
pub async fn execute_service_step(
    step: &WorkflowStep,
    context: &ExecutionContext,
    service_composition: &Arc<ServiceCompositionEngine>,
    default_timeout: std::time::Duration,
) -> Result<serde_json::Value> {
    let input = resolve_input(&step.input, context)?;

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
        timeout: default_timeout,
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
pub async fn execute_transform_step(
    step: &WorkflowStep,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let input = resolve_input(&step.input, context)?;
    
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
                    .map(|item| apply_map_function(item, step))
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
                    .filter(|item| matches_filter(item, step))
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
fn apply_map_function(item: &serde_json::Value, step: &WorkflowStep) -> serde_json::Value {
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
fn matches_filter(item: &serde_json::Value, step: &WorkflowStep) -> bool {
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
pub async fn execute_condition_step(
    step: &WorkflowStep,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    if let Some(condition) = &step.condition {
        let result = evaluate_condition(condition, context).await?;
        Ok(serde_json::Value::Bool(result))
    } else {
        Ok(serde_json::Value::Bool(true))
    }
}

/// Execute parallel step
pub async fn execute_parallel_step(
    step: &WorkflowStep,
    context: &ExecutionContext,
    ai_coordinator: &Arc<AICoordinator>,
    service_composition: &Arc<ServiceCompositionEngine>,
    event_broadcaster: &Arc<EventBroadcaster>,
    max_parallel_steps: usize,
) -> Result<serde_json::Value> {
    // Get sub-steps to execute in parallel
    let sub_steps = step.config.get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MCPError::InvalidWorkflow("Parallel step missing 'steps' array".to_string()))?;
    
    // Limit parallel execution
    let max_parallel = max_parallel_steps.min(sub_steps.len());
    
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
            let result = execute_step_static(&sub_step, &ctx, &ai, &svc, &evt).await;
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
    use crate::enhanced::workflow::types::WorkflowStepType;
    
    // Simple execution for parallel context
    match &step.step_type {
        WorkflowStepType::AIInference => {
            // Execute AI task
            if let Some(prompt) = step.config.get("prompt").and_then(|v| v.as_str()) {
                let request = UniversalAIRequest {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    prompt: prompt.to_string(),
                    model: None,
                    parameters: step.config.clone(),
                    context: None,
                    constraints: None,
                    routing_hints: None,
                };
                
                let response = ai_coordinator.coordinate_ai_request(request).await?;
                Ok(serde_json::json!({
                    "result": response,
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
pub async fn execute_sequential_step(
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
        match execute_step_static(sub_step, &ctx, ai_coordinator, service_composition, event_broadcaster).await {
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
