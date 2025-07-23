//! Composition Execution Engine
//!
//! This module contains the execution methods for different composition types.

use std::sync::Arc;
use tracing::{debug, warn};

use crate::error::Result;
use super::types::{Composition, CompositionType, ExecutionResult, ExecutionStatus};
use super::super::coordinator::{UniversalAIRequest, UniversalAIResponse, AICoordinator};

/// Composition execution engine
#[derive(Debug)]
pub struct CompositionExecutor {
    ai_coordinator: Arc<AICoordinator>,
}

impl CompositionExecutor {
    /// Create a new composition executor
    pub fn new(ai_coordinator: Arc<AICoordinator>) -> Self {
        Self { ai_coordinator }
    }
    
    /// Execute a composition based on its type
    pub async fn execute_composition(
        &self,
        composition: &Composition,
        request: UniversalAIRequest,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        let result = match composition.composition_type {
            CompositionType::Sequential => {
                self.execute_sequential_composition(composition, request).await
            }
            CompositionType::Parallel => {
                self.execute_parallel_composition(composition, request).await
            }
            CompositionType::Conditional => {
                self.execute_conditional_composition(composition, request).await
            }
            CompositionType::Pipeline => {
                self.execute_pipeline_composition(composition, request).await
            }
            CompositionType::Custom(_) => {
                self.execute_custom_composition(composition, request).await
            }
        };
        
        let execution_time = start_time.elapsed();
        
        // Create execution result
        let execution_result = match result {
            Ok(data) => ExecutionResult {
                execution_id: uuid::Uuid::new_v4().to_string(),
                composition_id: composition.id.clone(),
                status: ExecutionStatus::Success,
                data,
                execution_time,
                error: None,
                metadata: std::collections::HashMap::new(),
                started_at: chrono::Utc::now() - chrono::Duration::from_std(execution_time).unwrap_or_default(),
                completed_at: Some(chrono::Utc::now()),
            },
            Err(e) => ExecutionResult {
                execution_id: uuid::Uuid::new_v4().to_string(),
                composition_id: composition.id.clone(),
                status: ExecutionStatus::Failed,
                data: serde_json::Value::Null,
                execution_time,
                error: Some(e.to_string()),
                metadata: std::collections::HashMap::new(),
                started_at: chrono::Utc::now() - chrono::Duration::from_std(execution_time).unwrap_or_default(),
                completed_at: Some(chrono::Utc::now()),
            }
        };
        
        Ok(execution_result)
    }
    
    /// Execute sequential composition
    async fn execute_sequential_composition(
        &self,
        composition: &Composition,
        mut request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing sequential composition: {}", composition.id);
        
        // Execute services in sequence, updating the request in place
        for service_id in &composition.services {
            debug!("Executing service: {}", service_id);
            
            // Update request ID for this service
            request.id = uuid::Uuid::new_v4().to_string();
            
            // Execute service via AI coordinator
            let response = self.ai_coordinator.execute_request(request.clone()).await?;
            
            // Create new request with response data for next iteration
            request = UniversalAIRequest {
                id: uuid::Uuid::new_v4().to_string(),
                model: request.model, // Keep the same model
                messages: response.messages,
                parameters: response.parameters,
            };
        }
        
        Ok(serde_json::to_value(request.parameters)?)
    }

    /// Execute parallel composition
    async fn execute_parallel_composition(
        &self,
        composition: &Composition,
        request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing parallel composition: {}", composition.id);
        
        let mut tasks = Vec::new();
        
        // Create tasks for parallel execution
        for service_id in &composition.services {
            let service_request = UniversalAIRequest {
                id: uuid::Uuid::new_v4().to_string(),
                model: request.model.clone(),
                messages: request.messages.clone(),
                parameters: request.parameters.clone(),
            };
            
            let coordinator = Arc::clone(&self.ai_coordinator);
            let service_name = service_id.clone();
            
            let task = tokio::spawn(async move {
                debug!("Executing parallel service: {}", service_name);
                coordinator.execute_request(service_request).await
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(response)) => results.push(response),
                Ok(Err(e)) => {
                    warn!("Parallel service execution failed: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    warn!("Task join failed: {}", e);
                    return Err(crate::error::types::MCPError::Internal(format!("Task join failed: {}", e)).into());
                }
            }
        }
        
        // Combine results
        let combined_result = serde_json::json!({
            "parallel_results": results,
            "execution_count": results.len()
        });
        
        Ok(combined_result)
    }

    /// Execute conditional composition
    async fn execute_conditional_composition(
        &self,
        composition: &Composition,
        request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing conditional composition: {}", composition.id);
        
        // Simple condition evaluation based on request parameters
        let condition_key = "condition";
        let condition_value = request.parameters.get(condition_key)
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        // Select service based on condition
        let selected_service_id = if condition_value && !composition.services.is_empty() {
            &composition.services[0] // Use first service if condition is true
        } else if composition.services.len() > 1 {
            &composition.services[1] // Use second service if available
        } else if !composition.services.is_empty() {
            &composition.services[0] // Fall back to first service
        } else {
            return Err(crate::error::types::MCPError::InvalidArgument("No services available in composition".to_string()).into());
        };
        
        debug!("Selected service: {} based on condition: {}", selected_service_id, condition_value);
        
        // Execute selected service
        let service_request = UniversalAIRequest {
            id: uuid::Uuid::new_v4().to_string(),
            model: request.model,
            messages: request.messages,
            parameters: request.parameters,
        };
        
        let response = self.ai_coordinator.execute_request(service_request).await?;
        Ok(serde_json::to_value(response)?)
    }

    /// Execute pipeline composition
    async fn execute_pipeline_composition(
        &self,
        composition: &Composition,
        request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing pipeline composition: {}", composition.id);
        
        let mut current_request = request;
        
        // Execute services in pipeline (each service gets the output of the previous)
        for (index, service_id) in composition.services.iter().enumerate() {
            debug!("Pipeline step {}: executing service: {}", index, service_id);
            
            let service_request = UniversalAIRequest {
                id: uuid::Uuid::new_v4().to_string(),
                model: current_request.model.clone(),
                messages: current_request.messages.clone(),
                parameters: current_request.parameters.clone(),
            };
            
            let response = self.ai_coordinator.execute_request(service_request).await?;
            
            // Update request with response for next step
            current_request = UniversalAIRequest {
                id: uuid::Uuid::new_v4().to_string(),
                model: current_request.model,
                messages: vec![super::super::coordinator::Message {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                }],
                parameters: response.parameters.clone(),
            };
        }
        
        Ok(serde_json::to_value(current_request.parameters)?)
    }

    /// Execute custom composition
    async fn execute_custom_composition(
        &self,
        composition: &Composition,
        request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing custom composition: {}", composition.id);
        
        // For custom compositions, we'll use a simple fallback to sequential execution
        warn!("Custom composition types are not fully implemented, falling back to sequential execution");
        self.execute_sequential_composition(composition, request).await
    }
} 