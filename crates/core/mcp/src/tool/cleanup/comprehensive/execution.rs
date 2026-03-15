// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core cleanup execution logic for comprehensive resource management.
//!
//! This module handles the execution of cleanup operations including
//! cascading cleanup, dependency tracking, and resource state management.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::tool::cleanup::ResourceManager;
use crate::tool::management::types::ToolError;
use super::types::{
    ResourceId, ResourceType, ResourceAllocation, ResourceDependency, 
    CleanupRecord, CleanupMethod, CleanupStrategy
};
use super::strategies::StrategyExecutor;

/// Core cleanup execution engine
pub struct CleanupExecutor {
    /// Resource allocations by tool
    allocations: Arc<RwLock<HashMap<String, Vec<ResourceAllocation>>>>,
    
    /// Resource dependencies
    dependencies: Arc<RwLock<Vec<ResourceDependency>>>,
    
    /// Cleanup history
    cleanup_history: Arc<RwLock<Vec<CleanupRecord>>>,
    
    /// Active cleanup operations
    active_cleanups: Arc<Mutex<HashSet<String>>>,
    
    /// Strategy executor
    strategy_executor: StrategyExecutor,
    
    /// Cleanup timeout in milliseconds
    cleanup_timeout_ms: u64,
}

impl CleanupExecutor {
    /// Create a new cleanup executor
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(Vec::new())),
            cleanup_history: Arc::new(RwLock::new(Vec::new())),
            active_cleanups: Arc::new(Mutex::new(HashSet::new())),
            strategy_executor: StrategyExecutor::new(resource_manager),
            cleanup_timeout_ms: 30000, // 30 seconds default
        }
    }
    
    /// Set the cleanup timeout
    pub fn with_cleanup_timeout(mut self, timeout_ms: u64) -> Self {
        self.cleanup_timeout_ms = timeout_ms;
        self
    }
    
    /// Register a resource allocation
    pub async fn register_resource(
        &self,
        tool_id: &str,
        resource_type: ResourceType,
        name: impl Into<String>,
        size: u64,
        metadata: HashMap<String, String>,
    ) -> ResourceId {
        let name = name.into();
        let id = ResourceId::new(resource_type, name, tool_id);
        
        let allocation = ResourceAllocation {
            id: id.clone(),
            allocated_at: chrono::Utc::now(),
            is_active: true,
            size,
            metadata,
        };
        
        let mut allocations = self.allocations.write().await;
        let tool_allocations = allocations.entry(tool_id.to_string()).or_insert_with(Vec::new);
        tool_allocations.push(allocation);
        
        debug!("Registered resource {} for tool {}", id, tool_id);
        id
    }
    
    /// Register a resource dependency
    pub async fn register_dependency(
        &self,
        parent: &ResourceId,
        child: &ResourceId,
        is_strong: bool,
    ) {
        let dependency = ResourceDependency {
            parent: parent.clone(),
            child: child.clone(),
            is_strong,
        };
        
        let mut dependencies = self.dependencies.write().await;
        dependencies.push(dependency);
        
        debug!("Registered dependency: {} -> {} (strong: {})", parent, child, is_strong);
    }
    
    /// Set a resource as inactive
    pub async fn deactivate_resource(&self, resource_id: &ResourceId) -> Result<(), ToolError> {
        let mut allocations = self.allocations.write().await;
        
        if let Some(tool_allocations) = allocations.get_mut(&resource_id.owner) {
            for allocation in tool_allocations.iter_mut() {
                if allocation.id.name == resource_id.name
                    && allocation.id.resource_type == resource_id.resource_type
                {
                    allocation.is_active = false;
                    debug!("Deactivated resource {}", resource_id);
                    return Ok(());
                }
            }
        }
        
        Err(ToolError::ResourceError(format!(
            "Resource not found: {resource_id}"
        )))
    }
    
    /// Get all active resources for a tool
    pub async fn get_active_resources(&self, tool_id: &str) -> Vec<ResourceAllocation> {
        let allocations = self.allocations.read().await;
        
        if let Some(tool_allocations) = allocations.get(tool_id) {
            tool_allocations
                .iter()
                .filter(|a| a.is_active)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get child resources for a resource
    pub async fn get_child_resources(&self, parent_id: &ResourceId) -> Vec<ResourceId> {
        let dependencies = self.dependencies.read().await;
        
        dependencies
            .iter()
            .filter(|d| d.parent == *parent_id)
            .map(|d| d.child.clone())
            .collect()
    }
    
    /// Get parent resources for a resource
    pub async fn get_parent_resources(&self, child_id: &ResourceId) -> Vec<ResourceId> {
        let dependencies = self.dependencies.read().await;
        
        dependencies
            .iter()
            .filter(|d| d.child == *child_id)
            .map(|d| d.parent.clone())
            .collect()
    }
    
    /// Execute cleanup for a specific resource
    pub async fn cleanup_resource(
        &self,
        resource_id: &ResourceId,
        method: CleanupMethod,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        let resource_key = format!("{}:{}", resource_id.resource_type, resource_id.name);
        
        // Check if cleanup is already active for this resource
        {
            let mut active = self.active_cleanups.lock().await;
            if active.contains(&resource_key) {
                warn!("Cleanup already in progress for resource {}", resource_id);
                return Err(ToolError::ResourceError(
                    format!("Cleanup already in progress for resource {resource_id}")
                ));
            }
            active.insert(resource_key.clone());
        }
        
        // Execute cleanup with automatic cleanup state management
        let cleanup_result = self.execute_cleanup_internal(resource_id, method, strategy).await;
        
        // Always remove from active cleanups
        {
            let mut active = self.active_cleanups.lock().await;
            active.remove(&resource_key);
        }
        
        cleanup_result
    }
    
    /// Internal cleanup execution with timing and history tracking
    async fn execute_cleanup_internal(
        &self,
        resource_id: &ResourceId,
        method: CleanupMethod,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        let start = Instant::now();
        debug!("Starting cleanup of resource {:?} using method {:?}", resource_id, method);
        
        // Execute the cleanup based on method
        let result = match method {
            CleanupMethod::Cascading => {
                self.execute_cascading_cleanup(resource_id, strategy).await
            }
            _ => {
                // For non-cascading methods, use the strategy executor
                self.strategy_executor.execute_strategy(resource_id, strategy).await
            }
        };
        
        // Record the cleanup attempt
        let duration = start.elapsed().as_millis() as u64;
        let success = result.is_ok();
        let error = result.as_ref().err().map(std::string::ToString::to_string);
        
        let record = CleanupRecord {
            resource_id: resource_id.clone(),
            timestamp: chrono::Utc::now(),
            success,
            error,
            method,
            duration_ms: duration,
        };
        
        {
            let mut history = self.cleanup_history.write().await;
            history.push(record);
        }
        
        // If successful, deactivate the resource
        if success {
            self.deactivate_resource(resource_id).await?;
        }
        
        result
    }
    
    /// Execute cascading cleanup for a resource and its dependencies
    async fn execute_cascading_cleanup(
        &self,
        parent_id: &ResourceId,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        // Get all child resources
        let children = self.get_child_resources(parent_id).await;
        
        if !children.is_empty() {
            info!(
                "Performing cascading cleanup for {} child resources of {}",
                children.len(),
                parent_id
            );
            
            // Determine if we should continue on error
            let continue_on_error = match strategy {
                CleanupStrategy::Cascading { continue_on_error } => *continue_on_error,
                _ => false,
            };
            
            // Clean up each child resource
            let mut errors = Vec::new();
            
            for child in children {
                let child_key = format!("{}:{}", child.resource_type, child.name);
                
                // Check if child cleanup is already active
                {
                    let active = self.active_cleanups.lock().await;
                    if active.contains(&child_key) {
                        warn!("Cleanup already in progress for child resource {}", child);
                        continue;
                    }
                }
                
                // Mark child as being cleaned up
                {
                    let mut active = self.active_cleanups.lock().await;
                    active.insert(child_key.clone());
                }
                
                // Execute child cleanup
                let child_result = self.execute_cleanup_internal(
                    &child, 
                    CleanupMethod::Cascading, 
                    strategy
                ).await;
                
                // Remove from active cleanups
                {
                    let mut active = self.active_cleanups.lock().await;
                    active.remove(&child_key);
                }
                
                if let Err(e) = child_result {
                    warn!("Failed to clean up child resource {}: {}", child, e);
                    errors.push((child, e));
                    
                    if !continue_on_error {
                        break;
                    }
                }
            }
            
            // If any child failed and we're not continuing on error
            if !errors.is_empty() && !continue_on_error {
                let error_msg = format!(
                    "Failed to clean up {} child resources of {}",
                    errors.len(),
                    parent_id
                );
                return Err(ToolError::ResourceError(error_msg));
            }
        }
        
        // After children are cleaned up, clean up the parent
        self.strategy_executor.execute_strategy(parent_id, strategy).await
    }
    
    /// Cleanup all resources for a tool
    pub async fn cleanup_tool_resources(
        &self,
        tool_id: &str,
        method: CleanupMethod,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        let resources = self.get_active_resources(tool_id).await;
        
        if resources.is_empty() {
            debug!("No active resources to clean up for tool {}", tool_id);
            return Ok(());
        }
        
        info!(
            "Cleaning up {} active resources for tool {}",
            resources.len(),
            tool_id
        );
        
        // Track errors
        let mut errors = Vec::new();
        
        // Clean up each resource
        for resource in resources {
            if let Err(e) = self.cleanup_resource(&resource.id, method.clone(), strategy).await {
                warn!("Failed to clean up resource {}: {}", resource.id, e);
                errors.push((resource.id, e));
            }
        }
        
        // Return result based on errors
        if errors.is_empty() {
            Ok(())
        } else {
            let error_msg = format!(
                "Failed to clean up {} resources for tool {}",
                errors.len(),
                tool_id
            );
            Err(ToolError::ResourceError(error_msg))
        }
    }
    
    /// Get cleanup history for a tool
    pub async fn get_cleanup_history(&self, tool_id: &str) -> Vec<CleanupRecord> {
        let history = self.cleanup_history.read().await;
        
        history
            .iter()
            .filter(|r| r.resource_id.owner == tool_id)
            .cloned()
            .collect()
    }
    
    /// Get cleanup success rate for a tool
    pub async fn get_cleanup_success_rate(&self, tool_id: &str) -> f64 {
        let history = self.get_cleanup_history(tool_id).await;
        
        if history.is_empty() {
            return 1.0; // No history means 100% success rate
        }
        
        let successful = history.iter().filter(|r| r.success).count();
        successful as f64 / history.len() as f64
    }
    
    /// Check for resource leaks
    pub async fn check_for_leaks(&self, tool_id: &str) -> Vec<ResourceId> {
        let allocations = self.allocations.read().await;
        
        if let Some(tool_allocations) = allocations.get(tool_id) {
            tool_allocations
                .iter()
                .filter(|a| a.is_active)
                .map(|a| a.id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get resource count by type for a tool
    pub async fn get_resource_count_by_type(
        &self,
        tool_id: &str,
        resource_type: &ResourceType,
    ) -> usize {
        let allocations = self.allocations.read().await;
        
        if let Some(tool_allocations) = allocations.get(tool_id) {
            tool_allocations
                .iter()
                .filter(|a| a.is_active && a.id.resource_type == *resource_type)
                .count()
        } else {
            0
        }
    }
    
    /// Get total resource usage for a tool
    pub async fn get_total_resource_usage(&self, tool_id: &str) -> u64 {
        let allocations = self.allocations.read().await;
        
        if let Some(tool_allocations) = allocations.get(tool_id) {
            tool_allocations
                .iter()
                .filter(|a| a.is_active)
                .map(|a| a.size)
                .sum()
        } else {
            0
        }
    }
    
    /// Force cleanup with timeout
    pub async fn force_cleanup_with_timeout(
        &self,
        tool_id: &str,
        timeout_ms: u64,
    ) -> Result<(), ToolError> {
        let strategy = CleanupStrategy::Forced {
            method: "timeout_force".to_string(),
        };
        
        let cleanup_future = self.cleanup_tool_resources(tool_id, CleanupMethod::Forced, &strategy);
        
        match tokio::time::timeout(Duration::from_millis(timeout_ms), cleanup_future).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Force cleanup timed out for tool {}", tool_id);
                Err(ToolError::ResourceError(format!(
                    "Force cleanup timed out for tool {tool_id}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::cleanup::BasicResourceManager;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_cleanup_executor_creation() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = CleanupExecutor::new(resource_manager);
        
        assert_eq!(executor.cleanup_timeout_ms, 30000);
    }
    
    #[tokio::test]
    async fn test_resource_registration() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = CleanupExecutor::new(resource_manager);
        
        // Register a resource
        let resource_id = executor
            .register_resource(
                "test-tool",
                ResourceType::Memory,
                "test-memory",
                1024,
                HashMap::new(),
            )
            .await;
        
        // Check if resource exists
        let resources = executor.get_active_resources("test-tool").await;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id.name, "test-memory");
        assert_eq!(resources[0].id.resource_type, ResourceType::Memory);
        assert_eq!(resources[0].size, 1024);
        assert!(resources[0].is_active);
        
        // Deactivate the resource
        executor.deactivate_resource(&resource_id).await.unwrap();
        
        // Should now have no active resources
        let active = executor.get_active_resources("test-tool").await;
        assert_eq!(active.len(), 0);
    }
    
    #[tokio::test]
    async fn test_dependency_registration() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = CleanupExecutor::new(resource_manager);
        
        // Register parent and child resources
        let parent_id = executor
            .register_resource(
                "test-tool",
                ResourceType::Memory,
                "parent-memory",
                1024,
                HashMap::new(),
            )
            .await;
        
        let child_id = executor
            .register_resource(
                "test-tool",
                ResourceType::File,
                "child-file",
                512,
                HashMap::new(),
            )
            .await;
        
        // Register dependency
        executor.register_dependency(&parent_id, &child_id, true).await;
        
        // Check if dependency exists
        let children = executor.get_child_resources(&parent_id).await;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child_id);
        
        let parents = executor.get_parent_resources(&child_id).await;
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], parent_id);
    }
    
    #[tokio::test]
    async fn test_cleanup_success_rate() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = CleanupExecutor::new(resource_manager);
        
        // Initially should be 100% success rate (no history)
        let rate = executor.get_cleanup_success_rate("test-tool").await;
        assert_eq!(rate, 1.0);
    }
    
    #[tokio::test]
    async fn test_resource_count_by_type() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = CleanupExecutor::new(resource_manager);
        
        // Register multiple resources of different types
        executor.register_resource(
            "test-tool",
            ResourceType::Memory,
            "memory-1",
            1024,
            HashMap::new(),
        ).await;
        
        executor.register_resource(
            "test-tool",
            ResourceType::Memory,
            "memory-2",
            2048,
            HashMap::new(),
        ).await;
        
        executor.register_resource(
            "test-tool",
            ResourceType::File,
            "file-1",
            512,
            HashMap::new(),
        ).await;
        
        // Check counts
        let memory_count = executor.get_resource_count_by_type("test-tool", &ResourceType::Memory).await;
        assert_eq!(memory_count, 2);
        
        let file_count = executor.get_resource_count_by_type("test-tool", &ResourceType::File).await;
        assert_eq!(file_count, 1);
        
        let network_count = executor.get_resource_count_by_type("test-tool", &ResourceType::Network).await;
        assert_eq!(network_count, 0);
    }
} 