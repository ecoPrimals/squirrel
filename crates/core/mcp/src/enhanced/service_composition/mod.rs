//! Service Composition Engine
//!
//! This module provides the main service composition functionality using the types
//! defined in the types module.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, error, warn, debug, instrument};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use super::coordinator::{UniversalAIRequest, UniversalAIResponse, AICoordinator};
use super::events::{EventBroadcaster, MCPEvent, EventType};

pub mod types;
pub use types::*;

#[cfg(test)]
mod tests;

/// AI Service Composition Engine
/// 
/// Orchestrates complex AI workflows by composing multiple AI services,
/// managing dependencies, and coordinating execution across providers.
#[derive(Debug)]
pub struct ServiceCompositionEngine {
    /// Configuration
    config: Arc<ServiceCompositionConfig>,
    
    /// Service registry
    service_registry: Arc<RwLock<HashMap<String, Arc<AIService>>>>,
    
    /// Service discovery engine
    service_discovery: Arc<ServiceDiscovery>,
    
    /// Dependency manager
    dependency_manager: Arc<DependencyManager>,
    
    /// Orchestration engine
    orchestration_engine: Arc<OrchestrationEngine>,
    
    /// Health monitor
    health_monitor: Arc<ServiceHealthMonitor>,
    
    /// Event broadcaster
    event_broadcaster: Arc<EventBroadcaster>,
    
    /// Active compositions
    active_compositions: Arc<RwLock<HashMap<String, Arc<Composition>>>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<ServiceCompositionMetrics>>,
    
    /// AI coordinator for routing
    ai_coordinator: Arc<AICoordinator>,
}

/// Service discovery engine
#[derive(Debug)]
pub struct ServiceDiscovery {
    /// TODO: Implement service discovery
}

/// Dependency manager
#[derive(Debug)]
pub struct DependencyManager {
    /// TODO: Implement dependency management
}

/// Orchestration engine
#[derive(Debug)]
pub struct OrchestrationEngine {
    /// TODO: Implement orchestration
}

/// Service health monitor
#[derive(Debug)]
pub struct ServiceHealthMonitor {
    /// TODO: Implement health monitoring
}

impl ServiceCompositionEngine {
    /// Create a new service composition engine
    pub fn new(
        config: ServiceCompositionConfig,
        event_broadcaster: Arc<EventBroadcaster>,
        ai_coordinator: Arc<AICoordinator>,
    ) -> Self {
        let config = Arc::new(config);
        
        Self {
            config: config.clone(),
            service_registry: Arc::new(RwLock::new(HashMap::new())),
            service_discovery: Arc::new(ServiceDiscovery {}),
            dependency_manager: Arc::new(DependencyManager {}),
            orchestration_engine: Arc::new(OrchestrationEngine {}),
            health_monitor: Arc::new(ServiceHealthMonitor {}),
            event_broadcaster,
            active_compositions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(ServiceCompositionMetrics::default())),
            ai_coordinator,
        }
    }
    
    /// Register a service
    #[instrument(skip(self, service))]
    pub async fn register_service(&self, service: AIService) -> Result<()> {
        info!("Registering service: {}", service.name);
        
        let mut registry = self.service_registry.write().await;
        registry.insert(service.id.clone(), Arc::new(service));
        
        Ok(())
    }
    
    /// Compose services
    #[instrument(skip(self, services))]
    pub async fn compose_services(
        &self,
        composition_name: &str,
        services: Vec<String>,
        workflow: CompositionWorkflow,
    ) -> Result<Arc<Composition>> {
        info!("Composing services: {}", composition_name);
        
        let composition = Composition {
            id: Uuid::new_v4().to_string(),
            name: composition_name.to_string(),
            description: format!("Composition of {} services", services.len()),
            services,
            workflow,
            config: CompositionConfig::default(),
            state: CompositionState::Pending,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        let composition = Arc::new(composition);
        
        // Add to active compositions
        let mut active = self.active_compositions.write().await;
        active.insert(composition.id.clone(), composition.clone());
        
        // TODO: Implement actual composition execution
        warn!("Service composition execution not yet implemented");
        
        Ok(composition)
    }
    
    /// Execute a composition
    #[instrument(skip(self, request))]
    pub async fn execute_composition(
        &self,
        composition_id: &str,
        request: UniversalAIRequest,
    ) -> Result<ExecutionResult> {
        info!("Executing composition: {}", composition_id);
        
        // Get composition
        let active = self.active_compositions.read().await;
        let composition = active.get(composition_id)
            .ok_or_else(|| MCPError::InvalidArgument(format!("Composition not found: {}", composition_id)))?
            .clone();
        drop(active);
        
        let start_time = std::time::Instant::now();
        
        // Execute composition based on its type
        let result = match composition.composition_type {
            CompositionType::Sequential => {
                self.execute_sequential_composition(&composition, request).await
            }
            CompositionType::Parallel => {
                self.execute_parallel_composition(&composition, request).await
            }
            CompositionType::Conditional => {
                self.execute_conditional_composition(&composition, request).await
            }
            CompositionType::Pipeline => {
                self.execute_pipeline_composition(&composition, request).await
            }
            CompositionType::Custom(_) => {
                self.execute_custom_composition(&composition, request).await
            }
        };
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(data) => {
                info!("Composition {} completed successfully in {:?}", composition_id, execution_time);
                Ok(ExecutionResult {
                    id: Uuid::new_v4().to_string(),
                    status: ExecutionStatus::Success,
                    data,
                    metadata: HashMap::new(),
                    execution_time,
                    error: None,
                })
            }
            Err(error) => {
                error!("Composition {} failed: {}", composition_id, error);
                Ok(ExecutionResult {
                    id: Uuid::new_v4().to_string(),
                    status: ExecutionStatus::Failed,
                    data: serde_json::Value::Null,
                    metadata: HashMap::new(),
                    execution_time,
                    error: Some(error.to_string()),
                })
            }
        }
    }
    
    /// Get composition status
    pub async fn get_composition_status(&self, composition_id: &str) -> Result<Option<Arc<Composition>>> {
        let active = self.active_compositions.read().await;
        Ok(active.get(composition_id).cloned())
    }
    
    /// Cancel a composition
    #[instrument(skip(self))]
    pub async fn cancel_composition(&self, composition_id: &str) -> Result<()> {
        info!("Cancelling composition: {}", composition_id);
        
        // Remove from active compositions
        {
            let mut active = self.active_compositions.write().await;
            if active.remove(composition_id).is_none() {
                return Err(MCPError::InvalidArgument(format!("Composition not found: {}", composition_id)));
            }
        }
        
        // Publish cancellation event
        let event = MCPEvent {
            id: Uuid::new_v4().to_string(),
            event_type: EventType::ServiceCompositionCancelled,
            source: super::events::EventSource::ServiceComposition,
            data: serde_json::json!({
                "composition_id": composition_id,
                "cancelled_at": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        if let Err(e) = self.event_broadcaster.broadcast_event(event).await {
            warn!("Failed to broadcast composition cancellation event: {}", e);
        }
        
        info!("Composition {} cancelled successfully", composition_id);
        Ok(())
    }
    
    /// Get service health
    pub async fn get_service_health(&self, service_id: &str) -> Result<Option<ServiceHealth>> {
        let registry = self.service_registry.read().await;
        if let Some(service) = registry.get(service_id) {
            let health = service.health.read().await;
            Ok(Some(health.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// List available services
    pub async fn list_services(&self) -> Result<Vec<Arc<AIService>>> {
        let registry = self.service_registry.read().await;
        Ok(registry.values().cloned().collect())
    }
    
    /// List active compositions
    pub async fn list_active_compositions(&self) -> Result<Vec<Arc<Composition>>> {
        let active = self.active_compositions.read().await;
        Ok(active.values().cloned().collect())
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> Result<ServiceCompositionMetrics> {
        let metrics = self.metrics.lock().await;
        Ok(metrics.clone())
    }
    
    /// Discover services
    #[instrument(skip(self))]
    pub async fn discover_services(&self) -> Result<Vec<Arc<AIService>>> {
        info!("Discovering services");
        
        // TODO: Implement service discovery
        warn!("Service discovery not yet implemented");
        
        Ok(vec![])
    }
    
    /// Validate service dependencies
    pub async fn validate_dependencies(&self, service_id: &str) -> Result<bool> {
        let registry = self.service_registry.read().await;
        if let Some(service) = registry.get(service_id) {
            // TODO: Implement dependency validation
            warn!("Dependency validation not yet implemented");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Execute sequential composition
    async fn execute_sequential_composition(
        &self,
        composition: &Composition,
        mut request: UniversalAIRequest,
    ) -> Result<serde_json::Value> {
        debug!("Executing sequential composition: {}", composition.id);
        
        // Execute services in sequence, updating the request in place
        for service in &composition.services {
            debug!("Executing service: {}", service.name);
            
            // Update request model for this service (reuse existing request structure)
            request.id = Uuid::new_v4().to_string();
            request.model = service.model.clone(); // TODO: Use Arc<str> to avoid clone
            
            // Execute service via AI coordinator (takes ownership temporarily)
            let response = self.ai_coordinator.execute_request(request).await?;
            
            // Create new request with response data for next iteration
            request = UniversalAIRequest {
                id: Uuid::new_v4().to_string(),
                model: service.model.clone(), // Will be updated in next iteration
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
        
        let mut handles = vec![];
        
        // Share immutable data across parallel tasks to avoid cloning
        let shared_messages = Arc::new(request.messages);
        let shared_parameters = Arc::new(request.parameters);
        let ai_coordinator = &self.ai_coordinator; // Use reference instead of cloning
        
        // Execute all services in parallel
        for service in &composition.services {
            let service_request = UniversalAIRequest {
                id: Uuid::new_v4().to_string(),
                model: service.model.clone(), // TODO: Use Arc<str> to avoid clone
                messages: (*shared_messages).clone(), // Dereference Arc to get Vec, then clone only once per task
                parameters: (*shared_parameters).clone(), // Dereference Arc to get HashMap, then clone only once per task
            };
            
            let handle = tokio::spawn({
                let ai_coordinator = ai_coordinator.clone(); // Clone only the Arc pointer, not the entire coordinator
                async move {
                    ai_coordinator.execute_request(service_request).await
                }
            });
            handles.push(handle);
        }
        
        // Collect results
        let mut results = HashMap::new();
        for (index, handle) in handles.into_iter().enumerate() {
            let response = handle.await.map_err(|e| MCPError::Internal(e.to_string()))??;
            results.insert(format!("service_{}", index), serde_json::to_value(response)?);
        }
        
        Ok(serde_json::to_value(results)?)
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
        let selected_service = if condition_value && !composition.services.is_empty() {
            &composition.services[0] // Use first service if condition is true
        } else if composition.services.len() > 1 {
            &composition.services[1] // Use second service if available
        } else if !composition.services.is_empty() {
            &composition.services[0] // Fall back to first service
        } else {
            return Err(MCPError::InvalidArgument("No services available in composition".to_string()));
        };
        
        debug!("Selected service: {} based on condition: {}", selected_service.name, condition_value);
        
        // Execute selected service
        let service_request = UniversalAIRequest {
            id: Uuid::new_v4().to_string(),
            model: selected_service.model.clone(),
            messages: request.messages.clone(),
            parameters: request.parameters.clone(),
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
        for (index, service) in composition.services.iter().enumerate() {
            debug!("Pipeline step {}: executing service: {}", index, service.name);
            
            let service_request = UniversalAIRequest {
                id: Uuid::new_v4().to_string(),
                model: service.model.clone(),
                messages: current_request.messages.clone(),
                parameters: current_request.parameters.clone(),
            };
            
            let response = self.ai_coordinator.execute_request(service_request).await?;
            
            // Update request with response for next step
            current_request = UniversalAIRequest {
                id: Uuid::new_v4().to_string(),
                model: current_request.model,
                messages: vec![super::coordinator::Message {
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

impl Default for ServiceCompositionMetrics {
    fn default() -> Self {
        Self {
            total_compositions: 0,
            active_compositions: 0,
            completed_compositions: 0,
            failed_compositions: 0,
            avg_execution_time: Duration::from_secs(0),
            success_rate: 0.0,
            service_availability: HashMap::new(),
        }
    }
}

impl Default for ServiceCompositionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_compositions: 50,
            default_timeout: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30), // 30 seconds
            metrics_interval: Duration::from_secs(60), // 1 minute
            service_discovery: ServiceDiscoveryConfig::default(),
        }
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            strategy: DiscoveryStrategy::Static,
            interval: Duration::from_secs(60), // 1 minute
            timeout: Duration::from_secs(10), // 10 seconds
            endpoints: vec![],
        }
    }
}

impl Default for CompositionConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(600), // 10 minutes
            resources: ResourceLimits::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu: 4.0,
            max_memory: 8 * 1024 * 1024 * 1024, // 8GB
            max_storage: 100 * 1024 * 1024 * 1024, // 100GB
            max_network: 1024 * 1024 * 1024, // 1GB
            custom: HashMap::new(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            logging_enabled: true,
            tracing_enabled: true,
            alerts: vec![],
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_required: true,
            authorization: vec![],
            encryption: EncryptionConfig::default(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES256".to_string(),
            key_management: "local".to_string(),
        }
    }
} 