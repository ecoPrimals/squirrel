// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Service Composition Engine
//!
//! This module provides a modular service composition system with clear separation of concerns.
//! 
//! ## Architecture
//! 
//! The service composition system is organized into focused modules:
//! 
//! - **`types`** - All type definitions organized by domain
//! - **`discovery`** - Service discovery and registry management
//! - **`dependency`** - Dependency resolution and validation
//! - **`orchestration`** - Workflow orchestration strategies
//! - **`health`** - Service health monitoring
//! - **`execution`** - Composition execution engine
//! 
//! ## Usage
//! 
//! ```rust
//! use super::service_composition::{ServiceCompositionEngine, ServiceCompositionConfig};
//! use super::events::EventBroadcaster;
//! use super::coordinator::AICoordinator;
//! use std::sync::Arc;
//! 
//! let config = ServiceCompositionConfig::default();
//! let event_broadcaster = Arc::new(EventBroadcaster::new());
//! let ai_coordinator = Arc::new(AICoordinator::new());
//! 
//! let engine = ServiceCompositionEngine::new(config, event_broadcaster, ai_coordinator);
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, error, warn, debug, instrument};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use super::coordinator::{UniversalAIRequest, UniversalAIResponse, AICoordinator};
use super::events::{EventBroadcaster, MCPEvent, EventType};

// Module organization
pub mod types;
pub mod discovery;
pub mod dependency;
pub mod orchestration;
pub mod health;
pub mod execution;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use types::*;
pub use discovery::ServiceDiscovery;
pub use dependency::{DependencyManager, ServiceRegistryResolver};
pub use orchestration::OrchestrationEngine;
pub use health::{ServiceHealthMonitor, HealthSummary};
pub use execution::CompositionExecutor;

/// AI Service Composition Engine
/// 
/// The main orchestrator that coordinates complex AI workflows by composing multiple AI services,
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
    
    /// Execution engine
    execution_engine: Arc<CompositionExecutor>,
    
    /// Event broadcaster
    event_broadcaster: Arc<EventBroadcaster>,
    
    /// Active compositions
    active_compositions: Arc<RwLock<HashMap<String, Arc<Composition>>>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<ServiceCompositionMetrics>>,
    
    /// AI coordinator for routing
    ai_coordinator: Arc<AICoordinator>,
}

impl ServiceCompositionEngine {
    /// Create a new service composition engine
    pub fn new(
        config: ServiceCompositionConfig,
        event_broadcaster: Arc<EventBroadcaster>,
        ai_coordinator: Arc<AICoordinator>,
    ) -> Self {
        let config = Arc::new(config);
        let service_registry = Arc::new(RwLock::new(HashMap::new()));
        
        // Initialize dependency manager
        let mut dependency_manager = DependencyManager::new();
        let registry_resolver = ServiceRegistryResolver::new(service_registry.clone());
        dependency_manager.add_resolver(Box::new(registry_resolver));
        
        // Initialize execution engine
        let execution_engine = Arc::new(CompositionExecutor::new(ai_coordinator.clone()));
        
        Self {
            config: config.clone(),
            service_registry,
            service_discovery: Arc::new(ServiceDiscovery::new()),
            dependency_manager: Arc::new(dependency_manager),
            orchestration_engine: Arc::new(OrchestrationEngine::new(event_broadcaster.clone())),
            health_monitor: {
                // Load unified config for environment-aware health monitor interval
                let unified_config = squirrel_mcp_config::unified::ConfigLoader::load()
                    .ok()
                    .and_then(|loaded| loaded.try_into_config().ok());
                
                let interval = if let Some(cfg) = unified_config {
                    cfg.timeouts.get_custom_timeout("svc_monitor_interval")
                        .unwrap_or_else(|| Duration::from_secs(30))
                } else {
                    Duration::from_secs(30)
                };
                
                Arc::new(ServiceHealthMonitor::new(interval))
            },
            execution_engine,
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
        
        // Register in service registry
        let service_arc = Arc::new(service);
        {
            let mut registry = self.service_registry.write().await;
            registry.insert(service_arc.id.clone(), service_arc.clone());
        }
        
        // Register in dependency manager
        self.dependency_manager.register_service(&service_arc).await?;
        
        Ok(())
    }
    
    /// Create a composition
    #[instrument(skip(self))]
    pub async fn create_composition(
        &self,
        composition_name: &str,
        services: Vec<String>,
        workflow: CompositionWorkflow,
    ) -> Result<Arc<Composition>> {
        info!("Creating composition: {}", composition_name);
        
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
        
        // Execute using the execution engine
        let result = self.execution_engine.execute_composition(&composition, request).await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_compositions += 1;
            if result.status == ExecutionStatus::Success {
                metrics.completed_compositions += 1;
            } else {
                metrics.failed_compositions += 1;
            }
        }
        
        // Publish completion event
        let event = MCPEvent {
            id: Uuid::new_v4().to_string(),
            event_type: if result.status == ExecutionStatus::Success {
                EventType::ServiceCompositionCompleted
            } else {
                EventType::ServiceCompositionFailed
            },
            source: super::events::EventSource::ServiceComposition,
            data: serde_json::json!({
                "composition_id": composition_id,
                "execution_id": result.execution_id,
                "status": result.status,
                "execution_time_ms": result.execution_time.as_millis()
            }),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        if let Err(e) = self.event_broadcaster.broadcast_event(event).await {
            warn!("Failed to broadcast composition completion event: {}", e);
        }
        
        Ok(result)
    }
    
    /// Cancel a composition
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
    
    /// Get health summary
    pub async fn get_health_summary(&self) -> Result<HealthSummary> {
        Ok(self.health_monitor.get_health_summary().await)
    }
    
    /// Discover services
    #[instrument(skip(self))]
    pub async fn discover_services(&self) -> Result<Vec<Arc<AIService>>> {
        info!("Discovering services");
        
        // Discover services using the service discovery engine
        let discovered_entries = self.service_discovery.discover_all_services().await?;
        
        // Load unified config for environment-aware service timeout
        let unified_config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let default_service_timeout = if let Some(cfg) = unified_config {
            cfg.timeouts.get_custom_timeout("svc_discovered_timeout")
                .unwrap_or_else(|| Duration::from_secs(30))
        } else {
            Duration::from_secs(30)
        };
        
        // Convert discovery entries to AI services
        let mut services = Vec::new();
        for entry in discovered_entries {
            let service = AIService {
                id: entry.service_id.clone(),
                name: entry.service_name.clone(),
                description: format!("Discovered service at {}:{}", entry.endpoint, entry.port),
                config: ServiceConfig {
                    service_type: ServiceType::Inference, // Default type
                    endpoint: format!("{}:{}", entry.endpoint, entry.port),
                    auth: None,
                    timeout: default_service_timeout,
                    retry: RetryConfig::default(),
                    resources: ResourceLimits::default(),
                    scaling: ScalingConfig::default(),
                    version: Some("1.0.0".to_string()),
                },
                capabilities: vec![],
                dependencies: vec![],
                health: Arc::new(RwLock::new(ServiceHealth::default())),
                metadata: entry.metadata,
                provider: self.ai_coordinator.get_default_provider().await?,
            };
            services.push(Arc::new(service));
        }
        
        info!("Discovered {} services", services.len());
        Ok(services)
    }
    
    /// Validate service dependencies
    pub async fn validate_dependencies(&self, service_id: &str) -> Result<bool> {
        let validation_result = self.dependency_manager.validate_dependencies(service_id).await?;
        
        if !validation_result.is_valid {
            warn!("Service {} has invalid dependencies:", service_id);
            
            if !validation_result.missing_dependencies.is_empty() {
                warn!("  Missing dependencies: {:?}", validation_result.missing_dependencies);
            }
            
            if !validation_result.circular_dependencies.is_empty() {
                warn!("  Circular dependencies: {:?}", validation_result.circular_dependencies);
            }
            
            for error in &validation_result.errors {
                warn!("  Error: {}", error);
            }
        }
        
        Ok(validation_result.is_valid)
    }
    
    /// Get detailed dependency validation result
    pub async fn get_dependency_validation(&self, service_id: &str) -> Result<DependencyValidationResult> {
        self.dependency_manager.validate_dependencies(service_id).await
    }
    
    /// Resolve dependencies for a service
    pub async fn resolve_service_dependencies(&self, service_id: &str) -> Result<Vec<ResolvedDependency>> {
        self.dependency_manager.resolve_dependencies(service_id).await
    }
    
    /// Get dependency graph for a service
    pub async fn get_service_dependency_graph(&self, service_id: &str) -> Result<HashMap<String, Vec<String>>> {
        self.dependency_manager.get_dependency_graph(service_id).await
    }
}