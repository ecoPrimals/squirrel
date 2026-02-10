// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive cleanup system for advanced resource management.
//!
//! This module provides a sophisticated cleanup system for tool resources
//! with support for dependency tracking, cascading cleanup, and multiple
//! cleanup strategies. It's designed to handle complex resource management
//! scenarios in the MCP tool system.
//!
//! ## Key Features
//!
//! - **Resource Dependency Tracking**: Maintain parent-child relationships between resources
//! - **Cascading Cleanup**: Automatically clean up dependent resources
//! - **Multiple Cleanup Strategies**: Normal, forced, cascading, and custom strategies
//! - **Cleanup History**: Track cleanup attempts and success rates
//! - **Resource Leak Detection**: Identify and report resource leaks
//! - **Concurrent Cleanup Protection**: Prevent duplicate cleanup operations
//!
//! ## Architecture
//!
//! The comprehensive cleanup system is organized into several modules:
//!
//! - `types`: Core data structures and enums
//! - `strategies`: Cleanup strategy implementations
//! - `execution`: Core cleanup execution logic
//! - `ComprehensiveCleanupHook`: Main interface implementing `ToolLifecycleHook`
//!
//! ## Usage
//!
//! ```rust
//! use mcp::tool::cleanup::comprehensive::ComprehensiveCleanupHook;
//! use mcp::tool::cleanup::BasicResourceManager;
//! use std::sync::Arc;
//!
//! // Create a comprehensive cleanup hook
//! let resource_manager = Arc::new(BasicResourceManager::new());
//! let hook = ComprehensiveCleanupHook::new()
//!     .with_resource_manager(resource_manager)
//!     .with_cleanup_timeout(30000);
//!
//! // Register resources and dependencies
//! let parent_id = hook.register_resource(
//!     "tool-id",
//!     ResourceType::Memory,
//!     "parent-resource",
//!     1024,
//!     HashMap::new()
//! ).await;
//!
//! // Use with tool lifecycle management
//! // The hook will automatically perform cleanup during tool lifecycle events
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::time::{self, Duration};
use tracing::{debug, info, warn};

use crate::tool::cleanup::{ResourceLimits, ResourceManager, ResourceUsage, BasicResourceManager};
use crate::tool::management::types::{Tool, ToolError, ToolLifecycleHook};

// Module organization
pub mod types;
pub mod strategies;
pub mod execution;

// Re-export key types for convenience
pub use types::{
    ResourceType, ResourceId, ResourceDependency, ResourceAllocation,
    CleanupRecord, CleanupMethod, CleanupStrategy, ResourceOperations,
};
pub use strategies::{StrategyExecutor, ResourceLimitEnforcer, StrategySelector};
pub use execution::CleanupExecutor;

/// Comprehensive cleanup hook implementation that integrates all cleanup functionality
#[derive(Debug)]
pub struct ComprehensiveCleanupHook {
    /// Core cleanup executor
    executor: Arc<CleanupExecutor>,
    
    /// Strategy selector for choosing appropriate cleanup strategies
    strategy_selector: strategies::StrategySelector,
    
    /// Resource limit enforcer
    limit_enforcer: ResourceLimitEnforcer,
    
    /// Overall cleanup timeout in milliseconds
    cleanup_timeout_ms: u64,
}

impl ComprehensiveCleanupHook {
    /// Creates a new comprehensive cleanup hook with default configuration
    pub fn new() -> Self {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = Arc::new(CleanupExecutor::new(resource_manager.clone()));
        
        Self {
            executor,
            strategy_selector: strategies::StrategySelector::new(),
            limit_enforcer: ResourceLimitEnforcer::new(resource_manager),
            cleanup_timeout_ms: 30000, // 30 seconds default
        }
    }
    
    /// Set a custom resource manager
    pub fn with_resource_manager(self, manager: impl ResourceManager + 'static) -> Self {
        let resource_manager = Arc::new(manager);
        let executor = Arc::new(CleanupExecutor::new(resource_manager.clone()));
        
        Self {
            executor,
            strategy_selector: self.strategy_selector,
            limit_enforcer: ResourceLimitEnforcer::new(resource_manager),
            cleanup_timeout_ms: self.cleanup_timeout_ms,
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
        self.executor
            .register_resource(tool_id, resource_type, name, size, metadata)
            .await
    }
    
    /// Register a resource dependency
    pub async fn register_dependency(
        &self,
        parent: &ResourceId,
        child: &ResourceId,
        is_strong: bool,
    ) {
        self.executor
            .register_dependency(parent, child, is_strong)
            .await
    }
    
    /// Set a resource as inactive
    pub async fn deactivate_resource(&self, resource_id: &ResourceId) -> Result<(), ToolError> {
        self.executor.deactivate_resource(resource_id).await
    }
    
    /// Get all active resources for a tool
    pub async fn get_active_resources(&self, tool_id: &str) -> Vec<ResourceAllocation> {
        self.executor.get_active_resources(tool_id).await
    }
    
    /// Get child resources for a resource
    pub async fn get_child_resources(&self, parent_id: &ResourceId) -> Vec<ResourceId> {
        self.executor.get_child_resources(parent_id).await
    }
    
    /// Get parent resources for a resource
    pub async fn get_parent_resources(&self, child_id: &ResourceId) -> Vec<ResourceId> {
        self.executor.get_parent_resources(child_id).await
    }
    
    /// Check if a tool has a specific resource type
    pub async fn has_resource_type(&self, tool_id: &str, resource_type: &ResourceType) -> bool {
        self.executor
            .get_resource_count_by_type(tool_id, resource_type)
            .await
            > 0
    }
    
    /// Get the strategy for a resource type
    pub fn get_cleanup_strategy(&self, resource_type: &ResourceType) -> CleanupStrategy {
        self.strategy_selector.get_default_strategy(resource_type)
    }
    
    /// Set the strategy for a resource type
    pub fn set_cleanup_strategy(
        &mut self,
        resource_type: ResourceType,
        strategy: CleanupStrategy,
    ) {
        self.strategy_selector
            .set_default_strategy(resource_type, strategy);
    }
    
    /// Perform cleanup for a specific resource
    pub async fn cleanup_resource(
        &self,
        resource_id: &ResourceId,
        method: CleanupMethod,
    ) -> Result<(), ToolError> {
        // Select appropriate strategy based on method and resource type
        let strategy = self.strategy_selector
            .select_strategy(&resource_id.resource_type, &method);
        
        self.executor
            .cleanup_resource(resource_id, method, &strategy)
            .await
    }
    
    /// Perform cleanup for all resources of a tool
    pub async fn cleanup_tool_resources(
        &self,
        tool_id: &str,
        method: CleanupMethod,
    ) -> Result<(), ToolError> {
        // Use a default strategy for tool-level cleanup
        let strategy = match method {
            CleanupMethod::Forced => CleanupStrategy::Forced {
                method: "tool_cleanup".to_string(),
            },
            CleanupMethod::Cascading => CleanupStrategy::Cascading {
                continue_on_error: true,
            },
            _ => CleanupStrategy::Normal { timeout_ms: 10000 },
        };
        
        self.executor
            .cleanup_tool_resources(tool_id, method, &strategy)
            .await
    }
    
    /// Get all cleanup records for a tool
    pub async fn get_cleanup_history(&self, tool_id: &str) -> Vec<CleanupRecord> {
        self.executor.get_cleanup_history(tool_id).await
    }
    
    /// Get cleanup success rate for a tool
    pub async fn get_cleanup_success_rate(&self, tool_id: &str) -> f64 {
        self.executor.get_cleanup_success_rate(tool_id).await
    }
    
    /// Check for resource leaks
    pub async fn check_for_leaks(&self, tool_id: &str) -> Vec<ResourceId> {
        self.executor.check_for_leaks(tool_id).await
    }
    
    /// Handle resource limits exceeded
    pub async fn handle_resource_limits_exceeded(
        &self,
        resource_id: &ResourceId,
        usage: &ResourceUsage,
        limits: &ResourceLimits,
    ) -> Result<(), ToolError> {
        self.limit_enforcer
            .handle_limits_exceeded(resource_id, usage, limits)
            .await
    }
    
    /// Get resource statistics for a tool
    pub async fn get_resource_stats(&self, tool_id: &str) -> ResourceStats {
        let total_resources = self.executor.get_active_resources(tool_id).await.len();
        let total_usage = self.executor.get_total_resource_usage(tool_id).await;
        let success_rate = self.executor.get_cleanup_success_rate(tool_id).await;
        let leak_count = self.executor.check_for_leaks(tool_id).await.len();
        
        ResourceStats {
            total_resources,
            total_usage,
            success_rate,
            leak_count,
        }
    }
    
    /// Force cleanup with timeout
    pub async fn force_cleanup_with_timeout(
        &self,
        tool_id: &str,
        timeout_ms: u64,
    ) -> Result<(), ToolError> {
        self.executor
            .force_cleanup_with_timeout(tool_id, timeout_ms)
            .await
    }
}

impl Default for ComprehensiveCleanupHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolLifecycleHook for ComprehensiveCleanupHook {
    async fn on_register(&self, _tool: &Tool) -> Result<(), ToolError> {
        // Nothing to do on registration
        Ok(())
    }
    
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up all resources when tool is unregistered
        self.cleanup_tool_resources(tool_id, CleanupMethod::Normal).await
    }
    
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Nothing specific to do on activation
        Ok(())
    }
    
    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Nothing specific to do on deactivation
        Ok(())
    }
    
    async fn on_error(&self, tool_id: &str, _error: &ToolError) -> Result<(), ToolError> {
        // On error, check for resource leaks but don't clean up yet
        let leaks = self.check_for_leaks(tool_id).await;
        
        if !leaks.is_empty() {
            warn!(
                "Detected {} potential resource leaks for tool {} after error",
                leaks.len(),
                tool_id
            );
        }
        
        Ok(())
    }
    
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up all resources
        self.cleanup_tool_resources(tool_id, CleanupMethod::Normal).await
    }
    
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up all resources with a timeout
        let cleanup_future = self.cleanup_tool_resources(tool_id, CleanupMethod::Normal);
        
        match time::timeout(Duration::from_millis(self.cleanup_timeout_ms), cleanup_future).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred, try forced cleanup
                warn!(
                    "Normal cleanup timed out for tool {}, attempting forced cleanup",
                    tool_id
                );
                self.cleanup_tool_resources(tool_id, CleanupMethod::Forced).await
            }
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Resource statistics for a tool
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Total number of active resources
    pub total_resources: usize,
    
    /// Total resource usage (sum of all resource sizes)
    pub total_usage: u64,
    
    /// Cleanup success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Number of detected resource leaks
    pub leak_count: usize,
}

impl ResourceStats {
    /// Check if the tool has resource issues
    pub fn has_issues(&self) -> bool {
        self.leak_count > 0 || self.success_rate < 0.95
    }
    
    /// Get a summary of resource health
    pub fn health_summary(&self) -> String {
        if self.has_issues() {
            format!(
                "Resource health: DEGRADED - {} resources, {} leaks, {:.1}% success rate",
                self.total_resources,
                self.leak_count,
                self.success_rate * 100.0
            )
        } else {
            format!(
                "Resource health: HEALTHY - {} resources, {:.1}% success rate",
                self.total_resources,
                self.success_rate * 100.0
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_comprehensive_cleanup_hook_creation() {
        let hook = ComprehensiveCleanupHook::new();
        assert_eq!(hook.cleanup_timeout_ms, 30000);
    }
    
    #[tokio::test]
    async fn test_resource_registration() {
        let hook = ComprehensiveCleanupHook::new();
        
        // Register a resource
        let resource_id = hook
            .register_resource(
                "test-tool",
                ResourceType::Memory,
                "test-memory",
                1024,
                HashMap::new(),
            )
            .await;
        
        // Check if resource exists
        let resources = hook.get_active_resources("test-tool").await;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id.name, "test-memory");
        assert_eq!(resources[0].id.resource_type, ResourceType::Memory);
        assert_eq!(resources[0].size, 1024);
        assert!(resources[0].is_active);
        
        // Deactivate the resource
        hook.deactivate_resource(&resource_id).await.unwrap();
        
        // Should now have no active resources
        let active = hook.get_active_resources("test-tool").await;
        assert_eq!(active.len(), 0);
    }
    
    #[tokio::test]
    async fn test_dependency_registration() {
        let hook = ComprehensiveCleanupHook::new();
        
        // Register parent and child resources
        let parent_id = hook
            .register_resource(
                "test-tool",
                ResourceType::Memory,
                "parent-memory",
                1024,
                HashMap::new(),
            )
            .await;
        
        let child_id = hook
            .register_resource(
                "test-tool",
                ResourceType::File,
                "child-file",
                512,
                HashMap::new(),
            )
            .await;
        
        // Register dependency
        hook.register_dependency(&parent_id, &child_id, true).await;
        
        // Check if dependency exists
        let children = hook.get_child_resources(&parent_id).await;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child_id);
        
        let parents = hook.get_parent_resources(&child_id).await;
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], parent_id);
    }
    
    #[tokio::test]
    async fn test_cleanup_strategies() {
        let mut hook = ComprehensiveCleanupHook::new();
        
        // Get default strategy
        let default_strategy = hook.get_cleanup_strategy(&ResourceType::Memory);
        match default_strategy {
            CleanupStrategy::Normal { timeout_ms } => {
                assert_eq!(timeout_ms, 5000);
            }
            _ => panic!("Expected Normal strategy"),
        }
        
        // Set custom strategy
        let custom_strategy = CleanupStrategy::Custom {
            name: "test-strategy".to_string(),
            params: HashMap::new(),
        };
        hook.set_cleanup_strategy(ResourceType::Memory, custom_strategy.clone());
        
        // Check if strategy was updated
        let updated_strategy = hook.get_cleanup_strategy(&ResourceType::Memory);
        match updated_strategy {
            CleanupStrategy::Custom { name, params: _ } => {
                assert_eq!(name, "test-strategy");
            }
            _ => panic!("Expected Custom strategy"),
        }
    }
    
    #[tokio::test]
    async fn test_resource_stats() {
        let hook = ComprehensiveCleanupHook::new();
        
        // Register some resources
        hook.register_resource(
            "test-tool",
            ResourceType::Memory,
            "memory-1",
            1024,
            HashMap::new(),
        ).await;
        
        hook.register_resource(
            "test-tool",
            ResourceType::File,
            "file-1",
            512,
            HashMap::new(),
        ).await;
        
        // Get stats
        let stats = hook.get_resource_stats("test-tool").await;
        assert_eq!(stats.total_resources, 2);
        assert_eq!(stats.total_usage, 1536);
        assert_eq!(stats.success_rate, 1.0); // No cleanup history yet
        assert_eq!(stats.leak_count, 2); // All resources are active, so they're "leaks"
        
        // Check health summary
        let summary = stats.health_summary();
        assert!(summary.contains("DEGRADED")); // Due to leaks
    }
} 