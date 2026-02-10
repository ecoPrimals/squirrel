// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Dependency Manager Implementation
//!
//! This module contains the dependency management functionality for services.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::Result;
use super::types::{
    AIService, DependencyGraph, DependencyResolver, DependencyValidationResult,
    ResolvedDependency, ServiceDependency, DependencyNode, DependencyNodeStatus
};

/// Dependency manager
#[derive(Debug)]
pub struct DependencyManager {
    /// Dependency graph for services
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// Dependency resolvers
    resolvers: Vec<Box<dyn DependencyResolver>>,
    /// Validation cache
    validation_cache: Arc<RwLock<HashMap<String, DependencyValidationResult>>>,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new() -> Self {
        Self {
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::default())),
            resolvers: vec![],
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a dependency resolver
    pub fn add_resolver(&mut self, resolver: Box<dyn DependencyResolver>) {
        self.resolvers.push(resolver);
    }
    
    /// Register a service in the dependency graph
    pub async fn register_service(&self, service: &AIService) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;
        
        // Create dependency node
        let node = DependencyNode {
            service_id: service.id.clone(),
            service_name: service.name.clone(),
            version: service.config.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
            status: DependencyNodeStatus::Available,
            metadata: service.metadata.clone(),
        };
        
        // Add node to graph
        graph.nodes.insert(service.id.clone(), node);
        
        // Add dependencies
        graph.dependencies.insert(service.id.clone(), service.dependencies.clone());
        
        // Update reverse dependencies
        for dependency in &service.dependencies {
            let dependents = graph.dependents.entry(dependency.service_id.clone()).or_default();
            if !dependents.contains(&service.id) {
                dependents.push(service.id.clone());
            }
        }
        
        info!("Registered service {} in dependency graph", service.id);
        Ok(())
    }
    
    /// Validate service dependencies
    pub async fn validate_dependencies(&self, service_id: &str) -> Result<DependencyValidationResult> {
        use chrono::Utc;
        
        // Check cache first
        {
            let cache = self.validation_cache.read().await;
            if let Some(cached_result) = cache.get(service_id) {
                // Return cached result if it's recent (within 5 minutes)
                let cache_age = Utc::now() - cached_result.validated_at;
                if cache_age.num_minutes() < 5 {
                    return Ok(cached_result.clone());
                }
            }
        }
        
        let graph = self.dependency_graph.read().await;
        let dependencies = graph.dependencies.get(service_id).cloned().unwrap_or_default();
        
        let mut result = DependencyValidationResult {
            service_id: service_id.to_string(),
            is_valid: true,
            missing_dependencies: Vec::new(),
            circular_dependencies: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            validated_at: Utc::now(),
        };
        
        // Check for missing dependencies
        for dependency in &dependencies {
            if !graph.nodes.contains_key(&dependency.service_id) {
                result.missing_dependencies.push(dependency.service_id.clone());
                result.is_valid = false;
            }
        }
        
        // Check for circular dependencies (simplified check)
        if let Some(circular_deps) = self.detect_circular_dependencies(&graph, service_id) {
            result.circular_dependencies.push(circular_deps);
            result.is_valid = false;
        }
        
        // Cache the result
        {
            let mut cache = self.validation_cache.write().await;
            cache.insert(service_id.to_string(), result.clone());
        }
        
        Ok(result)
    }
    
    /// Resolve dependencies for a service
    pub async fn resolve_dependencies(&self, service_id: &str) -> Result<Vec<ResolvedDependency>> {
        let graph = self.dependency_graph.read().await;
        let dependencies = graph.dependencies.get(service_id).cloned().unwrap_or_default();
        
        let mut resolved_dependencies = Vec::new();
        
        for resolver in &self.resolvers {
            match resolver.resolve_dependencies(service_id, &dependencies).await {
                Ok(mut resolved) => {
                    resolved_dependencies.append(&mut resolved);
                }
                Err(e) => {
                    warn!("Resolver {} failed to resolve dependencies for {}: {}", 
                         resolver.resolver_name(), service_id, e);
                }
            }
        }
        
        Ok(resolved_dependencies)
    }
    
    /// Get dependency graph for a service
    pub async fn get_dependency_graph(&self, service_id: &str) -> Result<HashMap<String, Vec<String>>> {
        let graph = self.dependency_graph.read().await;
        let dependencies = graph.dependencies.get(service_id).cloned().unwrap_or_default();
        
        let mut dep_map = HashMap::new();
        for dependency in dependencies {
            let dependents = graph.dependents.get(&dependency.service_id).cloned().unwrap_or_default();
            dep_map.insert(dependency.service_id, dependents);
        }
        
        Ok(dep_map)
    }
    
    /// Get services that depend on a given service
    pub async fn get_dependents(&self, service_id: &str) -> Vec<String> {
        let graph = self.dependency_graph.read().await;
        graph.dependents.get(service_id).cloned().unwrap_or_default()
    }
    
    /// Update service status in dependency management
    pub async fn update_service_status(&self, service_id: &str, status: DependencyNodeStatus) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;
        if let Some(node) = graph.nodes.get_mut(service_id) {
            node.status = status;
        }
        Ok(())
    }
    
    /// Detect circular dependencies (simplified implementation)
    fn detect_circular_dependencies(&self, graph: &DependencyGraph, service_id: &str) -> Option<Vec<String>> {
        let mut visited = HashMap::new();
        let mut path = Vec::new();
        
        if self.has_circular_dependency(graph, service_id, &mut visited, &mut path) {
            Some(path)
        } else {
            None
        }
    }
    
    /// Helper method for circular dependency detection
    fn has_circular_dependency(
        &self,
        graph: &DependencyGraph,
        service_id: &str,
        visited: &mut HashMap<String, bool>,
        path: &mut Vec<String>,
    ) -> bool {
        if let Some(&in_path) = visited.get(service_id) {
            return in_path;
        }
        
        visited.insert(service_id.to_string(), true);
        path.push(service_id.to_string());
        
        if let Some(dependencies) = graph.dependencies.get(service_id) {
            for dependency in dependencies {
                if self.has_circular_dependency(graph, &dependency.service_id, visited, path) {
                    return true;
                }
            }
        }
        
        visited.insert(service_id.to_string(), false);
        path.pop();
        false
    }
}

/// Service registry dependency resolver
#[derive(Debug)]
pub struct ServiceRegistryResolver {
    service_registry: Arc<RwLock<HashMap<String, Arc<AIService>>>>,
}

impl ServiceRegistryResolver {
    pub fn new(service_registry: Arc<RwLock<HashMap<String, Arc<AIService>>>>) -> Self {
        Self { service_registry }
    }
}

#[async_trait::async_trait]
impl DependencyResolver for ServiceRegistryResolver {
    async fn resolve_dependencies(
        &self,
        _service_id: &str,
        dependencies: &[ServiceDependency],
    ) -> Result<Vec<ResolvedDependency>, crate::error::types::MCPError> {
        let registry = self.service_registry.read().await;
        let mut resolved = Vec::new();
        
        for dependency in dependencies {
            if let Some(service) = registry.get(&dependency.service_id) {
                resolved.push(ResolvedDependency {
                    dependency: dependency.clone(),
                    resolved_service_id: service.id.clone(),
                    endpoint: Some(service.config.endpoint.clone()),
                    metadata: HashMap::new(),
                    resolved_at: chrono::Utc::now(),
                });
            }
        }
        
        Ok(resolved)
    }
    
    async fn check_dependency_availability(
        &self,
        dependency: &ServiceDependency,
    ) -> Result<bool, crate::error::types::MCPError> {
        let registry = self.service_registry.read().await;
        Ok(registry.contains_key(&dependency.service_id))
    }
    
    fn resolver_name(&self) -> &str {
        "service_registry"
    }
} 