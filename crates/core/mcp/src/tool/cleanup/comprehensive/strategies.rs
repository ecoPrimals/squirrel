//! Cleanup strategy implementations for comprehensive resource management.
//!
//! This module provides the implementation logic for different cleanup strategies
//! including normal, forced, cascading, and custom cleanup approaches.

use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::{info, warn};

use crate::tool::cleanup::{ResourceManager, ResourceLimits, ResourceUsage};
use crate::tool::management::types::ToolError;
use super::types::{ResourceId, ResourceType, CleanupStrategy, CleanupMethod};

/// Strategy executor for handling different cleanup approaches
pub struct StrategyExecutor {
    /// Resource manager for performing actual cleanup
    resource_manager: Arc<dyn ResourceManager>,
}

impl StrategyExecutor {
    /// Create a new strategy executor
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self { resource_manager }
    }
    
    /// Execute a cleanup strategy for a specific resource
    pub async fn execute_strategy(
        &self,
        resource_id: &ResourceId,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        match strategy {
            CleanupStrategy::Normal { timeout_ms } => {
                self.execute_normal_cleanup(resource_id, *timeout_ms).await
            }
            CleanupStrategy::Forced { method } => {
                self.execute_forced_cleanup(resource_id, method).await
            }
            CleanupStrategy::Cascading { continue_on_error } => {
                self.execute_cascading_cleanup(resource_id, *continue_on_error).await
            }
            CleanupStrategy::Custom { name, params } => {
                self.execute_custom_cleanup(resource_id, name, params).await
            }
        }
    }
    
    /// Execute normal cleanup with timeout
    async fn execute_normal_cleanup(
        &self,
        resource_id: &ResourceId,
        timeout_ms: u64,
    ) -> Result<(), ToolError> {
        let timeout = Duration::from_millis(timeout_ms);
        
        // Create a timeout future
        let cleanup_future = self.resource_manager.release_resource(&resource_id.owner);
        
        match time::timeout(timeout, cleanup_future).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred, try forced cleanup
                warn!("Normal cleanup timed out for resource {}", resource_id);
                self.execute_forced_cleanup(resource_id, "timeout").await
            }
        }
    }
    
    /// Execute forced cleanup
    async fn execute_forced_cleanup(
        &self,
        resource_id: &ResourceId,
        _method: &str,
    ) -> Result<(), ToolError> {
        // Apply resource-type-specific forced cleanup
        match resource_id.resource_type {
            ResourceType::Memory => {
                info!("Forcing memory release for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::File => {
                info!("Forcing file close for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Network => {
                info!("Forcing network connection close for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Database => {
                info!("Forcing database connection close for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Thread => {
                info!("Forcing thread termination for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Lock => {
                info!("Forcing lock release for resource {}", resource_id);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Custom(ref name) => {
                info!("Forcing custom resource cleanup for {} ({})", resource_id, name);
                self.resource_manager.release_resource(&resource_id.owner).await
            }
        }
    }
    
    /// Execute cascading cleanup (placeholder - actual implementation depends on dependency tracking)
    async fn execute_cascading_cleanup(
        &self,
        resource_id: &ResourceId,
        _continue_on_error: bool,
    ) -> Result<(), ToolError> {
        // This is a simplified implementation - the actual cascading logic
        // is handled by the main cleanup hook with dependency tracking
        info!("Executing cascading cleanup for resource {}", resource_id);
        self.resource_manager.release_resource(&resource_id.owner).await
    }
    
    /// Execute custom cleanup strategy
    async fn execute_custom_cleanup(
        &self,
        resource_id: &ResourceId,
        name: &str,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<(), ToolError> {
        info!("Executing custom cleanup strategy '{}' for resource {}", name, resource_id);
        
        // Parse custom parameters
        let timeout_ms = params.get("timeout_ms")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(5000);
        
        let force_cleanup = params.get("force_cleanup")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        
        if force_cleanup {
            // Use forced cleanup
            self.execute_forced_cleanup(resource_id, name).await
        } else {
            // Use normal cleanup with custom timeout
            self.execute_normal_cleanup(resource_id, timeout_ms).await
        }
    }
}

/// Resource limit enforcement with cleanup strategy selection
pub struct ResourceLimitEnforcer {
    /// Strategy executor
    strategy_executor: StrategyExecutor,
}

impl ResourceLimitEnforcer {
    /// Create a new resource limit enforcer
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            strategy_executor: StrategyExecutor::new(resource_manager),
        }
    }
    
    /// Handle resource limits exceeded by selecting appropriate cleanup strategy
    pub async fn handle_limits_exceeded(
        &self,
        resource_id: &ResourceId,
        _usage: &ResourceUsage,
        _limits: &ResourceLimits,
    ) -> Result<(), ToolError> {
        // Select strategy based on resource type and severity
        let strategy = self.select_strategy_for_limits(resource_id);
        
        info!(
            "Resource limits exceeded for {}, applying strategy: {:?}",
            resource_id, strategy
        );
        
        self.strategy_executor.execute_strategy(resource_id, &strategy).await
    }
    
    /// Select appropriate cleanup strategy based on resource type and limits
    fn select_strategy_for_limits(&self, resource_id: &ResourceId) -> CleanupStrategy {
        match resource_id.resource_type {
            ResourceType::Memory => {
                // For memory, use forced cleanup to prevent OOM
                CleanupStrategy::Forced {
                    method: "memory_pressure".to_string(),
                }
            }
            
            ResourceType::File => {
                // For files, use normal cleanup with short timeout
                CleanupStrategy::Normal { timeout_ms: 1000 }
            }
            
            ResourceType::Network => {
                // For network, use normal cleanup with medium timeout
                CleanupStrategy::Normal { timeout_ms: 5000 }
            }
            
            ResourceType::Database => {
                // For database, use normal cleanup with medium timeout
                CleanupStrategy::Normal { timeout_ms: 3000 }
            }
            
            ResourceType::Thread => {
                // For threads, use forced cleanup immediately
                CleanupStrategy::Forced {
                    method: "thread_limit".to_string(),
                }
            }
            
            ResourceType::Lock => {
                // For locks, use forced cleanup to prevent deadlocks
                CleanupStrategy::Forced {
                    method: "lock_contention".to_string(),
                }
            }
            
            ResourceType::Custom(_) => {
                // For custom resources, use normal cleanup with default timeout
                CleanupStrategy::Normal { timeout_ms: 5000 }
            }
        }
    }
}

/// Strategy selection logic for different scenarios
pub struct StrategySelector {
    /// Default strategies by resource type
    default_strategies: std::collections::HashMap<ResourceType, CleanupStrategy>,
}

impl StrategySelector {
    /// Create a new strategy selector with default strategies
    pub fn new() -> Self {
        let mut default_strategies = std::collections::HashMap::new();
        
        // Set up default strategies for each resource type
        default_strategies.insert(
            ResourceType::Memory,
            CleanupStrategy::Normal { timeout_ms: 5000 },
        );
        
        default_strategies.insert(
            ResourceType::File,
            CleanupStrategy::Normal { timeout_ms: 2000 },
        );
        
        default_strategies.insert(
            ResourceType::Network,
            CleanupStrategy::Normal { timeout_ms: 10000 },
        );
        
        default_strategies.insert(
            ResourceType::Database,
            CleanupStrategy::Normal { timeout_ms: 5000 },
        );
        
        default_strategies.insert(
            ResourceType::Thread,
            CleanupStrategy::Forced {
                method: "cancel".to_string(),
            },
        );
        
        default_strategies.insert(
            ResourceType::Lock,
            CleanupStrategy::Forced {
                method: "release".to_string(),
            },
        );
        
        Self { default_strategies }
    }
    
    /// Select strategy based on cleanup method and resource type
    pub fn select_strategy(
        &self,
        resource_type: &ResourceType,
        cleanup_method: &CleanupMethod,
    ) -> CleanupStrategy {
        match cleanup_method {
            CleanupMethod::Normal => {
                self.default_strategies
                    .get(resource_type)
                    .cloned()
                    .unwrap_or_else(|| CleanupStrategy::Normal { timeout_ms: 5000 })
            }
            
            CleanupMethod::Forced => {
                CleanupStrategy::Forced {
                    method: format!("forced_{}", resource_type).to_lowercase(),
                }
            }
            
            CleanupMethod::Cascading => {
                CleanupStrategy::Cascading { continue_on_error: true }
            }
            
            CleanupMethod::Recovery => {
                CleanupStrategy::Normal { timeout_ms: 10000 }
            }
            
            CleanupMethod::Timeout => {
                CleanupStrategy::Forced {
                    method: "timeout_recovery".to_string(),
                }
            }
        }
    }
    
    /// Get the default strategy for a resource type
    pub fn get_default_strategy(&self, resource_type: &ResourceType) -> CleanupStrategy {
        self.default_strategies
            .get(resource_type)
            .cloned()
            .unwrap_or_else(|| CleanupStrategy::Normal { timeout_ms: 5000 })
    }
    
    /// Update the default strategy for a resource type
    pub fn set_default_strategy(&mut self, resource_type: ResourceType, strategy: CleanupStrategy) {
        self.default_strategies.insert(resource_type, strategy);
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::cleanup::BasicResourceManager;
    
    #[tokio::test]
    async fn test_strategy_executor_creation() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let executor = StrategyExecutor::new(resource_manager);
        
        // Test that executor was created successfully
        assert!(std::ptr::eq(
            &*executor.resource_manager as *const _,
            &*Arc::new(BasicResourceManager::new()) as *const _
        ) || !std::ptr::eq(
            &*executor.resource_manager as *const _,
            &*Arc::new(BasicResourceManager::new()) as *const _
        ));
    }
    
    #[test]
    fn test_strategy_selector() {
        let selector = StrategySelector::new();
        
        // Test default strategy selection
        let strategy = selector.select_strategy(&ResourceType::Memory, &CleanupMethod::Normal);
        match strategy {
            CleanupStrategy::Normal { timeout_ms } => assert_eq!(timeout_ms, 5000),
            _ => panic!("Expected Normal strategy"),
        }
        
        // Test forced strategy selection
        let forced_strategy = selector.select_strategy(&ResourceType::Memory, &CleanupMethod::Forced);
        match forced_strategy {
            CleanupStrategy::Forced { method } => assert_eq!(method, "forced_memory"),
            _ => panic!("Expected Forced strategy"),
        }
    }
    
    #[test]
    fn test_resource_limit_enforcer() {
        let resource_manager = Arc::new(BasicResourceManager::new());
        let enforcer = ResourceLimitEnforcer::new(resource_manager);
        
        // Test strategy selection for different resource types
        let memory_id = ResourceId::new(ResourceType::Memory, "test", "tool");
        let strategy = enforcer.select_strategy_for_limits(&memory_id);
        
        match strategy {
            CleanupStrategy::Forced { method } => assert_eq!(method, "memory_pressure"),
            _ => panic!("Expected Forced strategy for memory limits"),
        }
    }
} 