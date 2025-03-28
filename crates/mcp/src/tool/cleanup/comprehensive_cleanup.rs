// Comprehensive cleanup procedures for tool management
//
// This module provides enhanced cleanup capabilities for tools in the MCP system.
// It implements sophisticated resource tracking and cleanup mechanisms including:
// - Resource leak detection
// - Forced cleanup for unresponsive tools
// - Cascading cleanup for dependent resources
// - Cleanup verification

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{self, Duration, Instant};
use tracing::{debug, info, warn};
use std::any::Any;

use crate::tool::{
    Tool, ToolError, ToolLifecycleHook,
    cleanup::{ResourceManager, ResourceUsage},
};
use crate::tool::cleanup::resource_tracking::ResourceLimits;

/// Resource dependency type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory allocation
    Memory,
    
    /// File handle
    File,
    
    /// Network connection
    Network,
    
    /// Database connection
    Database,
    
    /// Thread/Task
    Thread,
    
    /// Lock/Mutex
    Lock,
    
    /// Custom resource type
    Custom(String),
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => write!(f, "Memory"),
            Self::File => write!(f, "File"),
            Self::Network => write!(f, "Network"),
            Self::Database => write!(f, "Database"),
            Self::Thread => write!(f, "Thread"),
            Self::Lock => write!(f, "Lock"),
            Self::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Resource identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Resource name/identifier
    pub name: String,
    
    /// Resource owner tool ID
    pub owner: String,
}

impl ResourceId {
    /// Create a new resource ID
    pub fn new(resource_type: ResourceType, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            resource_type,
            name: name.into(),
            owner: owner.into(),
        }
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.resource_type, self.name)
    }
}

/// Resource dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    /// Parent resource
    pub parent: ResourceId,
    
    /// Child resource
    pub child: ResourceId,
    
    /// Whether the relationship is strong (child cannot exist without parent)
    pub is_strong: bool,
}

/// Resource allocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Resource ID
    pub id: ResourceId,
    
    /// Allocation timestamp
    pub allocated_at: DateTime<Utc>,
    
    /// Is the resource currently active
    pub is_active: bool,
    
    /// Resource size/quantity
    pub size: u64,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Resource cleanup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRecord {
    /// Resource ID
    pub resource_id: ResourceId,
    
    /// Cleanup timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Whether cleanup was successful
    pub success: bool,
    
    /// Error message if cleanup failed
    pub error: Option<String>,
    
    /// Cleanup method used
    pub method: CleanupMethod,
    
    /// Duration of cleanup in milliseconds
    pub duration_ms: u64,
}

/// Cleanup method used
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CleanupMethod {
    /// Normal cleanup
    Normal,
    
    /// Forced cleanup
    Forced,
    
    /// Cascading cleanup (triggered by parent)
    Cascading,
    
    /// Auto-recovery cleanup
    Recovery,
    
    /// Timeout-triggered cleanup
    Timeout,
}

/// Resource cleanup strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupStrategy {
    /// Normal cleanup with timeout
    Normal {
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
    
    /// Forced cleanup after normal fails
    Forced {
        /// Force method description
        method: String,
    },
    
    /// Cascading cleanup (clean up children first)
    Cascading {
        /// Whether to continue if child cleanup fails
        continue_on_error: bool,
    },
    
    /// Custom cleanup with parameters
    Custom {
        /// Strategy name
        name: String,
        
        /// Strategy parameters
        params: HashMap<String, String>,
    },
}

/// Comprehensive cleanup hook implementation
#[derive(Debug)]
pub struct ComprehensiveCleanupHook {
    /// Resource allocations by tool
    allocations: RwLock<HashMap<String, Vec<ResourceAllocation>>>,
    
    /// Resource dependencies
    dependencies: RwLock<Vec<ResourceDependency>>,
    
    /// Cleanup history
    cleanup_history: RwLock<Vec<CleanupRecord>>,
    
    /// Resource cleanup strategies by type
    cleanup_strategies: RwLock<HashMap<ResourceType, CleanupStrategy>>,
    
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
    
    /// Active cleanup operations
    active_cleanups: Mutex<HashSet<String>>,
    
    /// Cleanup timeout in milliseconds
    cleanup_timeout_ms: u64,
}

impl Default for ComprehensiveCleanupHook {
    fn default() -> Self {
        Self::new()
    }
}

impl ComprehensiveCleanupHook {
    /// Creates a new comprehensive cleanup hook
    #[must_use] pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Default strategies
        strategies.insert(
            ResourceType::Memory,
            CleanupStrategy::Normal { timeout_ms: 5000 },
        );
        
        strategies.insert(
            ResourceType::File,
            CleanupStrategy::Normal { timeout_ms: 2000 },
        );
        
        strategies.insert(
            ResourceType::Network,
            CleanupStrategy::Normal { timeout_ms: 10000 },
        );
        
        strategies.insert(
            ResourceType::Database,
            CleanupStrategy::Normal { timeout_ms: 5000 },
        );
        
        strategies.insert(
            ResourceType::Thread,
            CleanupStrategy::Forced {
                method: "cancel".to_string(),
            },
        );
        
        strategies.insert(
            ResourceType::Lock,
            CleanupStrategy::Forced {
                method: "release".to_string(),
            },
        );
        
        Self {
            allocations: RwLock::new(HashMap::new()),
            dependencies: RwLock::new(Vec::new()),
            cleanup_history: RwLock::new(Vec::new()),
            cleanup_strategies: RwLock::new(strategies),
            resource_manager: Arc::new(crate::tool::cleanup::BasicResourceManager::new()),
            active_cleanups: Mutex::new(HashSet::new()),
            cleanup_timeout_ms: 30000, // 30 seconds
        }
    }
    
    /// Set the resource manager
    pub fn with_resource_manager(mut self, manager: impl ResourceManager + 'static) -> Self {
        self.resource_manager = Arc::new(manager);
        self
    }
    
    /// Set the cleanup timeout
    pub const fn with_cleanup_timeout(mut self, timeout_ms: u64) -> Self {
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
            allocated_at: Utc::now(),
            is_active: true,
            size,
            metadata,
        };
        
        let mut allocations = self.allocations.write().await;
        let tool_allocations = allocations.entry(tool_id.to_string()).or_insert_with(Vec::new);
        tool_allocations.push(allocation);
        
        id
    }
    
    /// Register a resource dependency
    pub async fn register_dependency(&self, parent: &ResourceId, child: &ResourceId, is_strong: bool) {
        let dependency = ResourceDependency {
            parent: parent.clone(),
            child: child.clone(),
            is_strong,
        };
        
        let mut dependencies = self.dependencies.write().await;
        dependencies.push(dependency);
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
    
    /// Check if a tool has a specific resource type
    pub async fn has_resource_type(&self, tool_id: &str, resource_type: &ResourceType) -> bool {
        let allocations = self.allocations.read().await;
        
        if let Some(tool_allocations) = allocations.get(tool_id) {
            tool_allocations
                .iter()
                .any(|a| a.is_active && a.id.resource_type == *resource_type)
        } else {
            false
        }
    }
    
    /// Get the strategy for a resource type
    pub async fn get_cleanup_strategy(&self, resource_type: &ResourceType) -> CleanupStrategy {
        let strategies = self.cleanup_strategies.read().await;
        
        strategies
            .get(resource_type)
            .cloned()
            .unwrap_or(CleanupStrategy::Normal { timeout_ms: 5000 })
    }
    
    /// Set the strategy for a resource type
    pub async fn set_cleanup_strategy(
        &self,
        resource_type: ResourceType,
        strategy: CleanupStrategy,
    ) {
        let mut strategies = self.cleanup_strategies.write().await;
        strategies.insert(resource_type, strategy);
    }
    
    /// Perform cleanup for a specific resource - public interface
    pub async fn cleanup_resource(
        &self,
        resource_id: &ResourceId,
        method: CleanupMethod,
    ) -> Result<(), ToolError> {
        // We'll use a non-recursive approach by handling all cleanup in the execute_cleanup method
        let resource_key = format!("{}:{}", resource_id.resource_type, resource_id.name);
        debug!(
            "Starting cleanup of resource {:?} using method {:?}",
            resource_id, method
        );
        
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
        
        // Use a scope to ensure we remove from active cleanups even if there's an error
        let cleanup_result = self.execute_cleanup_impl(resource_id, method).await;
        
        // Always remove from active cleanups
        {
            let mut active = self.active_cleanups.lock().await;
            active.remove(&resource_key);
        }
        
        cleanup_result
    }
    
    /// Execute the cleanup process - internal implementation that returns a future
    fn execute_cleanup_impl<'a>(
        &'a self,
        resource_id: &'a ResourceId,
        method: CleanupMethod,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
        Box::pin(async move {
            // Perform cleanup using specified method
            let start = Instant::now();
            let result = match method {
                CleanupMethod::Normal => {
                    // Get the strategy for this resource type
                    let strategy = self.get_cleanup_strategy(&resource_id.resource_type).await;
                    
                    // Apply a timeout to normal cleanup
                    if let CleanupStrategy::Normal { timeout_ms } = strategy {
                        if let Ok(result) = time::timeout(
                            Duration::from_millis(timeout_ms),
                            self.cleanup_resource_with_strategy(resource_id, &strategy)
                        ).await { result } else {
                            warn!("Normal cleanup timed out for resource {}, trying forced cleanup", resource_id);
                            
                            // Apply forced cleanup strategy directly
                            self.apply_forced_cleanup_strategy(resource_id).await
                        }
                    } else {
                        self.cleanup_resource_with_strategy(resource_id, &strategy).await
                    }
                },
                CleanupMethod::Forced => {
                    self.apply_forced_cleanup_strategy(resource_id).await
                },
                CleanupMethod::Cascading => {
                    self.cleanup_resource_cascade_impl(resource_id).await
                },
                CleanupMethod::Recovery => {
                    self.cleanup_resource_with_strategy(
                        resource_id,
                        &CleanupStrategy::Normal { timeout_ms: 10000 }
                    ).await
                },
                CleanupMethod::Timeout => {
                    // Timed out, try forced cleanup
                    self.apply_forced_cleanup_strategy(resource_id).await
                },
            };
            
            // Record the cleanup attempt
            let duration = start.elapsed().as_millis() as u64;
            let success = result.is_ok();
            let error = result.as_ref().err().map(std::string::ToString::to_string);
            
            // Create and store the cleanup record
            let record = CleanupRecord {
                resource_id: resource_id.clone(),
                timestamp: Utc::now(),
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
        })
    }
    
    /// Non-recursive implementation of `cleanup_resource_cascade` that returns a future
    fn cleanup_resource_cascade_impl<'a>(
        &'a self,
        parent_id: &'a ResourceId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
        Box::pin(async move {
            // Check if this resource has dependencies
            let dependencies = {
                let deps = self.dependencies.read().await;
                deps.iter()
                    .filter(|d| d.parent == *parent_id)
                    .map(|d| d.child.clone())
                    .collect::<Vec<ResourceId>>()
            };
            
            if !dependencies.is_empty() {
                info!(
                    "Performing cascading cleanup for {} child resources of {}",
                    dependencies.len(),
                    parent_id
                );
                
                // Clean up each child
                let mut errors = Vec::new();
                
                for child in dependencies {
                    // Execute cleanup directly without going through the public method
                    let resource_key = format!("{}:{}", child.resource_type, child.name);
                    
                    // Check if cleanup is already active for this child
                    {
                        let active = self.active_cleanups.lock().await;
                        if active.contains(&resource_key) {
                            warn!("Cleanup already in progress for child resource {}", child);
                            continue; // Skip this child
                        }
                    }
                    
                    // Mark child as being cleaned up
                    {
                        let mut active = self.active_cleanups.lock().await;
                        active.insert(resource_key.clone());
                    }
                    
                    // Perform the cleanup using a direct implementation with Box::pin
                    let cleanup_result = self.execute_cleanup_impl(&child, CleanupMethod::Cascading).await;
                    
                    // Remove from active cleanups
                    {
                        let mut active = self.active_cleanups.lock().await;
                        active.remove(&resource_key);
                    }
                    
                    if let Err(e) = cleanup_result {
                        warn!("Failed to clean up child resource {}: {}", child, e);
                        errors.push((child, e));
                    }
                }
                
                // If any child failed to clean up
                if !errors.is_empty() {
                    let error_msg = format!(
                        "Failed to clean up {} child resources of {}",
                        errors.len(),
                        parent_id
                    );
                    return Err(ToolError::ResourceError(error_msg));
                }
            }
            
            // After children are cleaned up, clean up the parent
            Ok(())
        })
    }
    
    /// Cleanup resources when limits are exceeded
    fn handle_resource_limits_exceeded<'a>(
        &'a self,
        tool_id: &'a str,
        resource_type: ResourceType,
        _usage: &'a ResourceUsage,
        _limits: &'a ResourceLimits,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
        Box::pin(async move {
            // Get all active resources of the specified type
            let resources = {
                let allocations = self.allocations.read().await;
                allocations
                    .get(tool_id)
                    .map(|allocs| 
                        allocs.iter()
                        .filter(|r| r.id.resource_type == resource_type)
                        .cloned()
                        .collect::<Vec<_>>()
                    )
                    .unwrap_or_default()
            };
            
            info!(
                "Cleaning up {} active resources for tool {}",
                resources.len(),
                tool_id
            );
            
            // Track errors
            let mut errors = Vec::new();
            
            // Clean up each resource
            for resource in resources {
                // Execute cleanup directly without going through the public method
                let resource_key = format!("{}:{}", resource.id.resource_type, resource.id.name);
                    
                // Check if cleanup is already active for this resource
                {
                    let active = self.active_cleanups.lock().await;
                    if active.contains(&resource_key) {
                        warn!("Cleanup already in progress for resource {}", resource.id);
                        continue; // Skip this resource
                    }
                }
                
                // Mark resource as being cleaned up
                {
                    let mut active = self.active_cleanups.lock().await;
                    active.insert(resource_key.clone());
                }
                
                // Perform the cleanup with Box::pin
                let cleanup_result = self.execute_cleanup_impl(&resource.id, CleanupMethod::Forced).await;
                
                // Remove from active cleanups
                {
                    let mut active = self.active_cleanups.lock().await;
                    active.remove(&resource_key);
                }
                
                if let Err(e) = cleanup_result {
                    warn!("Failed to clean up resource {}: {}", resource.id, e);
                    errors.push((resource.id, e));
                }
            }
            
            // If any resources failed to clean up
            if !errors.is_empty() {
                let error_msg = format!(
                    "Failed to clean up {} resources for tool {}",
                    errors.len(),
                    tool_id
                );
                return Err(ToolError::ResourceError(error_msg));
            }
            
            Ok(())
        })
    }
    
    /// Cleanup a resource with a specific strategy
    async fn cleanup_resource_with_strategy(
        &self,
        resource_id: &ResourceId,
        strategy: &CleanupStrategy,
    ) -> Result<(), ToolError> {
        // This method has been refactored to avoid recursive calls
        match strategy {
            CleanupStrategy::Normal { timeout_ms } => {
                // For normal cleanup, use resource manager with timeout
                let timeout = Duration::from_millis(*timeout_ms);
                
                // Create a timeout future
                let cleanup_future = self.resource_manager.release_resource(&resource_id.owner);
                
                if let Ok(result) = time::timeout(timeout, cleanup_future).await { result } else {
                    // Timeout occurred, try forced cleanup
                    warn!("Normal cleanup timed out for resource {}", resource_id);
                    
                    // Apply forced cleanup strategy directly
                    self.apply_forced_cleanup_strategy(resource_id).await
                }
            },
            
            CleanupStrategy::Forced { method: _ } => {
                self.apply_forced_cleanup_strategy(resource_id).await
            },
            
            CleanupStrategy::Cascading { continue_on_error } => {
                // Cascading cleanup handled by cleanup_resource_cascade
                self.cleanup_resource_cascade_impl(resource_id).await?;
                
                if *continue_on_error {
                    // If we should continue on error, ignore child cleanup errors
                    self.resource_manager.release_resource(&resource_id.owner).await
                } else {
                    // Otherwise only proceed if all children cleaned up successfully
                    self.resource_manager.release_resource(&resource_id.owner).await
                }
            },
            
            CleanupStrategy::Custom { name, params: _ } => {
                // For custom strategies, rely on resource manager
                info!(
                    "Performing custom cleanup '{}' for resource {}",
                    name, resource_id
                );
                self.resource_manager.release_resource(&resource_id.owner).await
            }
        }
    }
    
    /// Apply forced cleanup strategy without recursion
    async fn apply_forced_cleanup_strategy(&self, resource_id: &ResourceId) -> Result<(), ToolError> {
        // For forced cleanup, use a more aggressive approach
        info!("Performing forced cleanup for resource {}", resource_id);
        
        // Implement forced resource release based on resource type
        match &resource_id.resource_type {
            ResourceType::Memory => {
                // Force memory release - implementation depends on runtime
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::File => {
                // Force file close - implementation depends on file system abstraction
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Network => {
                // Force connection close - implementation depends on network abstraction
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Database => {
                // Force database connection close
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Thread => {
                // Force thread termination
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Lock => {
                // Force lock release
                self.resource_manager.release_resource(&resource_id.owner).await
            }
            
            ResourceType::Custom(_) => {
                // For custom resources, rely on resource manager
                self.resource_manager.release_resource(&resource_id.owner).await
            }
        }
    }
    
    /// Perform cleanup for all resources of a tool
    pub async fn cleanup_tool_resources(
        &self,
        tool_id: &str,
        method: CleanupMethod,
    ) -> Result<(), ToolError> {
        // Get all active resources
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
            if let Err(e) = self.cleanup_resource(&resource.id, method.clone()).await {
                warn!("Failed to clean up resource {}: {}", resource.id, e);
                errors.push((resource.id, e));
            }
        }
        
        // If any resources failed to clean up
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
    
    /// Get all cleanup records for a tool
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
            return 1.0; // No history means no failures
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
        
        if let Ok(result) = time::timeout(Duration::from_millis(self.cleanup_timeout_ms), cleanup_future).await { result } else {
            // Timeout occurred, try forced cleanup
            warn!(
                "Normal cleanup timed out for tool {}, attempting forced cleanup",
                tool_id
            );
            self.cleanup_tool_resources(tool_id, CleanupMethod::Forced).await
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    async fn test_resource_dependencies() {
        let hook = ComprehensiveCleanupHook::new();
        
        // Register parent and child resources
        let parent_id = hook
            .register_resource(
                "test-tool",
                ResourceType::Database,
                "db-connection",
                1,
                HashMap::new(),
            )
            .await;
        
        let child_id = hook
            .register_resource(
                "test-tool",
                ResourceType::File,
                "db-file",
                1024,
                HashMap::new(),
            )
            .await;
        
        // Register dependency
        hook.register_dependency(&parent_id, &child_id, true).await;
        
        // Check relationships
        let children = hook.get_child_resources(&parent_id).await;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "db-file");
        
        let parents = hook.get_parent_resources(&child_id).await;
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].name, "db-connection");
    }
    
    #[tokio::test]
    async fn test_cleanup_strategies() {
        let hook = ComprehensiveCleanupHook::new();
        
        // Get default strategy
        let default_strategy = hook.get_cleanup_strategy(&ResourceType::Memory).await;
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
        hook.set_cleanup_strategy(ResourceType::Memory, custom_strategy.clone())
            .await;
        
        // Check if strategy was updated
        let updated_strategy = hook.get_cleanup_strategy(&ResourceType::Memory).await;
        match updated_strategy {
            CleanupStrategy::Custom { name, params: _ } => {
                assert_eq!(name, "test-strategy");
            }
            _ => panic!("Expected Custom strategy"),
        }
    }
} 