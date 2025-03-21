//! Tool cleanup procedures for MCP
//!
//! This module contains implementations for resource tracking, cleanup, 
//! and error recovery mechanisms for the MCP tool management system.

// Declare submodules
pub mod recovery;
pub mod resource_tracking;
pub mod adaptive_resource;

use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::tool::{Tool, ToolError, ToolLifecycleHook};

// Re-export recovery components
pub use recovery::{RecoveryHook, RecoveryStrategy};

// Re-export key types for ease of use
pub use resource_tracking::{ResourceTracker, ResourceStatus, ResourceRecord, ResourceEvent, ResourceType};
pub use adaptive_resource::{
    AdaptiveResourceManager, ResourcePattern,
    AdaptiveResourceLimits,
};

/// Resource usage for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// File handles
    pub file_handles: Vec<u32>,
    /// Network connections
    pub network_connections: Vec<u32>,
}

/// Resource limits for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Maximum number of file handles
    pub max_file_handles: usize,
    /// Maximum number of network connections
    pub max_network_connections: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 100, // 100 MB
            max_cpu_time_ms: 60 * 1000,         // 60 seconds
            max_file_handles: 100,
            max_network_connections: 20,
        }
    }
}

impl ResourceLimits {
    /// Creates a new resource limits with custom memory limit
    pub fn with_max_memory(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Creates a new resource limits with custom CPU time limit
    pub fn with_max_cpu_time(mut self, ms: u64) -> Self {
        self.max_cpu_time_ms = ms;
        self
    }

    /// Creates a new resource limits with custom file handle limit
    pub fn with_max_file_handles(mut self, count: usize) -> Self {
        self.max_file_handles = count;
        self
    }

    /// Creates a new resource limits with custom network connection limit
    pub fn with_max_network_connections(mut self, count: usize) -> Self {
        self.max_network_connections = count;
        self
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, Default)]
pub struct ResourceUsageMetrics {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Number of open file handles
    pub file_handles: u32,
    /// Number of network connections
    pub network_connections: u32,
}

impl ResourceUsageMetrics {
    /// Creates a new empty resource usage metrics
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Updates resource usage metrics
    pub fn update(&mut self, memory: Option<u64>, cpu: Option<u64>, files: Option<u32>, connections: Option<u32>) {
        if let Some(memory) = memory {
            self.memory_bytes = memory;
        }
        
        if let Some(cpu) = cpu {
            self.cpu_time_ms = cpu;
        }
        
        if let Some(files) = files {
            self.file_handles = files;
        }
        
        if let Some(connections) = connections {
            self.network_connections = connections;
        }
    }
    
    /// Resets resource usage metrics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Resource tracking record for a tool
#[derive(Debug)]
struct ResourceTrackingRecord {
    /// Current resource usage
    usage: ResourceUsageMetrics,
    /// Resource limits
    limits: ResourceLimits,
    /// Last updated timestamp
    last_updated: DateTime<Utc>,
    /// File handles map (handle ID -> description)
    file_handles: HashMap<u32, String>,
    /// Network connections map (connection ID -> endpoint)
    network_connections: HashMap<u32, String>,
}

impl ResourceTrackingRecord {
    /// Creates a new resource tracking record
    fn new(limits: ResourceLimits) -> Self {
        Self {
            usage: ResourceUsageMetrics::default(),
            limits,
            last_updated: Utc::now(),
            file_handles: HashMap::new(),
            network_connections: HashMap::new(),
        }
    }
    
    /// Checks if resource usage exceeds limits
    fn exceeds_limits(&self) -> bool {
        self.usage.memory_bytes > self.limits.max_memory_bytes as u64
            || self.usage.cpu_time_ms > self.limits.max_cpu_time_ms
            || self.usage.file_handles > self.limits.max_file_handles as u32
            || self.usage.network_connections > self.limits.max_network_connections as u32
    }
}

/// Trait for resource cleanup hooks
#[async_trait]
pub trait ResourceCleanupHook: Send + Sync + std::fmt::Debug {
    /// Cleans up resources for a tool
    async fn cleanup_resources(&self, tool_id: &str) -> Result<(), ToolError>;
}

/// Basic resource cleanup hook implementation
#[derive(Debug, Default)]
pub struct BasicResourceCleanupHook;

#[async_trait]
impl ResourceCleanupHook for BasicResourceCleanupHook {
    /// Cleans up resources for a tool (basic implementation)
    async fn cleanup_resources(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Basic resource cleanup for tool {}", tool_id);
        // Basic implementation doesn't track resources specifically
        Ok(())
    }
}

/// Hook for resource cleanup
#[derive(Debug)]
pub struct LegacyResourceCleanupHook {
    /// Resource records by tool ID
    resources: RwLock<HashMap<String, ResourceTrackingRecord>>,
}

impl Default for LegacyResourceCleanupHook {
    fn default() -> Self {
        Self::new()
    }
}

impl LegacyResourceCleanupHook {
    /// Creates a new resource cleanup hook
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
        }
    }
    
    /// Sets resource limits for a tool
    pub async fn set_limits(&self, tool_id: &str, limits: ResourceLimits) {
        let mut resources = self.resources.write().await;
        
        if let Some(record) = resources.get_mut(tool_id) {
            record.limits = limits;
        } else {
            resources.insert(tool_id.to_string(), ResourceTrackingRecord::new(limits));
        }
    }
    
    /// Updates resource usage for a tool
    pub async fn update_usage(
        &self, 
        tool_id: &str, 
        memory: Option<u64>, 
        cpu: Option<u64>, 
        files: Option<u32>, 
        connections: Option<u32>
    ) -> Result<(), ToolError> {
        let mut resources = self.resources.write().await;
        
        let record = resources
            .entry(tool_id.to_string())
            .or_insert_with(|| ResourceTrackingRecord::new(ResourceLimits::default()));
        
        if let Some(memory) = memory {
            record.usage.memory_bytes = memory;
        }
        
        if let Some(cpu) = cpu {
            record.usage.cpu_time_ms = cpu;
        }
        
        if let Some(files) = files {
            record.usage.file_handles = files;
        }
        
        if let Some(connections) = connections {
            record.usage.network_connections = connections;
        }
        
        record.last_updated = Utc::now();
        
        // Check if resource limits are exceeded
        if record.exceeds_limits() {
            // Only log a warning - actual cleanup happens in the lifecycle hooks
            warn!("Tool '{}' exceeds resource limits", tool_id);
        }
        
        Ok(())
    }
    
    /// Gets resource usage for a tool
    pub async fn get_usage(&self, tool_id: &str) -> Option<ResourceUsage> {
        let resources = self.resources.read().await;
        resources.get(tool_id).map(|record| ResourceUsage {
            memory_bytes: record.usage.memory_bytes as usize,
            cpu_time_ms: record.usage.cpu_time_ms,
            file_handles: Vec::new(), // Not directly mappable
            network_connections: Vec::new(), // Not directly mappable
        })
    }
    
    /// Gets resource limits for a tool
    pub async fn get_limits(&self, tool_id: &str) -> Option<ResourceLimits> {
        let resources = self.resources.read().await;
        resources.get(tool_id).map(|record| record.limits.clone())
    }
    
    /// Registers a file handle for a tool
    pub async fn register_file_handle(&self, tool_id: &str, handle_id: u32, description: String) {
        let mut resources = self.resources.write().await;
        
        let record = resources
            .entry(tool_id.to_string())
            .or_insert_with(|| ResourceTrackingRecord::new(ResourceLimits::default()));
        
        record.file_handles.insert(handle_id, description);
        record.usage.file_handles = record.file_handles.len() as u32;
    }
    
    /// Registers a network connection for a tool
    pub async fn register_network_connection(&self, tool_id: &str, connection_id: u32, endpoint: String) {
        let mut resources = self.resources.write().await;
        
        let record = resources
            .entry(tool_id.to_string())
            .or_insert_with(|| ResourceTrackingRecord::new(ResourceLimits::default()));
        
        record.network_connections.insert(connection_id, endpoint);
        record.usage.network_connections = record.network_connections.len() as u32;
    }
    
    /// Unregisters a file handle for a tool
    pub async fn unregister_file_handle(&self, tool_id: &str, handle_id: u32) -> bool {
        let mut resources = self.resources.write().await;
        
        if let Some(record) = resources.get_mut(tool_id) {
            let removed = record.file_handles.remove(&handle_id).is_some();
            record.usage.file_handles = record.file_handles.len() as u32;
            removed
        } else {
            false
        }
    }
    
    /// Unregisters a network connection for a tool
    pub async fn unregister_network_connection(&self, tool_id: &str, connection_id: u32) -> bool {
        let mut resources = self.resources.write().await;
        
        if let Some(record) = resources.get_mut(tool_id) {
            let removed = record.network_connections.remove(&connection_id).is_some();
            record.usage.network_connections = record.network_connections.len() as u32;
            removed
        } else {
            false
        }
    }
    
    /// Performs resource cleanup for a tool
    #[instrument(skip(self))]
    async fn cleanup_resources(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut resources = self.resources.write().await;
        
        if let Some(record) = resources.get_mut(tool_id) {
            // Close all file handles
            for (handle_id, description) in record.file_handles.drain() {
                info!("Closing file handle {} ({}) for tool {}", handle_id, description, tool_id);
                // In a real implementation, we would actually close the file handle here
            }
            
            // Close all network connections
            for (connection_id, endpoint) in record.network_connections.drain() {
                info!("Closing network connection {} to {} for tool {}", connection_id, endpoint, tool_id);
                // In a real implementation, we would actually close the network connection here
            }
            
            // Reset usage stats
            record.usage.memory_bytes = 0;
            record.usage.cpu_time_ms = 0;
            record.usage.file_handles = 0;
            record.usage.network_connections = 0;
            
            Ok(())
        } else {
            // Tool not found, but that's not an error for cleanup
            Ok(())
        }
    }
    
    /// Removes all resource records for a tool
    async fn remove_resources(&self, tool_id: &str) {
        let mut resources = self.resources.write().await;
        resources.remove(tool_id);
    }
}

#[async_trait]
impl ToolLifecycleHook for LegacyResourceCleanupHook {
    /// Called when a tool is registered
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        // Set default resource limits based on security level
        let limits = match tool.security_level {
            0..=3 => ResourceLimits::default(), // Low security gets default limits
            4..=7 => ResourceLimits::default()  // Medium security gets default limits
                .with_max_memory(1024 * 1024 * 50) // 50 MB
                .with_max_network_connections(10),
            _ => ResourceLimits::default()     // High security gets restricted limits
                .with_max_memory(1024 * 1024 * 20) // 20 MB
                .with_max_cpu_time(30 * 1000)      // 30 seconds
                .with_max_file_handles(50)
                .with_max_network_connections(5),
        };
        
        self.set_limits(&tool.id, limits).await;
        Ok(())
    }
    
    /// Called when a tool is unregistered
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up resources first
        self.cleanup_resources(tool_id).await?;
        
        // Remove resource tracking
        self.remove_resources(tool_id).await;
        
        Ok(())
    }
    
    /// Called when a tool is activated
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Nothing special to do on activation
        Ok(())
    }
    
    /// Called when a tool is deactivated
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up resources when a tool is deactivated
        self.cleanup_resources(tool_id).await
    }
    
    /// Called when a tool encounters an error
    async fn on_error(&self, tool_id: &str, _error: &ToolError) -> Result<(), ToolError> {
        // Clean up resources when a tool encounters an error
        self.cleanup_resources(tool_id).await
    }
}

/// Enhanced resource cleanup hook that uses the resource tracking implementation
#[derive(Debug)]
pub struct EnhancedResourceCleanupHook {
    /// Resource tracker for monitoring tool resource usage
    tracker: Arc<ResourceTracker>,
}

impl Default for EnhancedResourceCleanupHook {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedResourceCleanupHook {
    /// Creates a new enhanced resource cleanup hook
    pub fn new() -> Self {
        Self {
            tracker: Arc::new(ResourceTracker::new(1000)), // Keep 1000 history records
        }
    }
    
    /// Gets the resource tracker
    pub fn get_tracker(&self) -> &ResourceTracker {
        &self.tracker
    }
    
    /// Sets resource limits based on tool's security level
    #[instrument(skip(self))]
    pub async fn set_security_based_limits(&self, tool: &Tool) -> Result<(), ToolError> {
        // Define limits based on security level (0-10)
        // Lower security level = stricter limits
        let security_factor = (tool.security_level as f64) / 10.0;
        
        let base_memory = 100 * 1024 * 1024; // 100 MB
        let memory = (base_memory as f64 * security_factor) as usize;
        
        let base_cpu = 60_000; // 60 seconds
        let cpu = (base_cpu as f64 * security_factor) as u64;
        
        let base_files = 100;
        let files = (base_files as f64 * security_factor) as usize;
        
        let base_connections = 20;
        let connections = (base_connections as f64 * security_factor) as usize;
        
        let limits = ResourceLimits {
            max_memory_bytes: memory,
            max_cpu_time_ms: cpu,
            max_file_handles: files,
            max_network_connections: connections,
        };
        
        self.tracker.set_limits(&tool.id, limits).await
    }
    
    /// Checks if an error is resource-related
    fn is_resource_related_error(&self, error: &ToolError) -> bool {
        match error {
            ToolError::ExecutionFailed(msg) => 
                msg.contains("memory") || 
                msg.contains("resource") || 
                msg.contains("timeout"),
            _ => false,
        }
    }
}

#[async_trait]
impl ToolLifecycleHook for EnhancedResourceCleanupHook {
    /// Called when a tool is registered
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        // Initialize resource tracking
        if let Err(err) = self.tracker.initialize_tool(&tool.id).await {
            warn!("Failed to initialize resource tracking for tool {}: {}", tool.id, err);
        }
        
        // Set security-based limits based on tool's security level
        if let Err(err) = self.set_security_based_limits(tool).await {
            warn!("Failed to set security-based limits for tool {}: {}", tool.id, err);
        }
        
        Ok(())
    }
    
    /// Called when a tool is unregistered
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up resource tracking
        if let Err(err) = self.tracker.cleanup_tool(tool_id).await {
            warn!("Failed to clean up resource tracking for tool {}: {}", tool_id, err);
        }
        
        Ok(())
    }
    
    /// Called when a tool is activated
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No specific resource tracking actions needed for activation
        Ok(())
    }
    
    /// Called when a tool is deactivated
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        // Release all resources
        if let Err(err) = self.tracker.release_all_resources(tool_id).await {
            warn!("Failed to release resources for tool {}: {}", tool_id, err);
        }
        
        Ok(())
    }
    
    /// Called when a tool encounters an error
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // Check if the error is resource-related
        if self.is_resource_related_error(error) {
            // Release some resources to recover
            if let Err(err) = self.tracker.release_all_resources(tool_id).await {
                warn!("Failed to release resources for tool {} during error recovery: {}", tool_id, err);
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl ResourceCleanupHook for EnhancedResourceCleanupHook {
    /// Cleans up resources for a tool
    async fn cleanup_resources(&self, tool_id: &str) -> Result<(), ToolError> {
        self.tracker.cleanup_tool(tool_id).await
    }
}

/// Enhanced resource manager that combines tracking and adaptive management
#[derive(Debug)]
pub struct EnhancedResourceManager {
    /// Basic resource tracker
    tracker: ResourceTracker,
    /// Adaptive resource manager
    adaptive_manager: Arc<AdaptiveResourceManager>,
}

impl EnhancedResourceManager {
    /// Creates a new enhanced resource manager
    pub fn new(max_history_size: usize) -> Self {
        Self {
            tracker: ResourceTracker::new(max_history_size),
            adaptive_manager: Arc::new(AdaptiveResourceManager::new()),
        }
    }

    /// Initializes a tool for resource management
    #[instrument(skip(self))]
    pub async fn initialize_tool(
        &self,
        tool_id: &str,
        base_limits: ResourceLimits,
        max_limits: ResourceLimits,
    ) -> Result<(), ToolError> {
        // Initialize basic tracking
        self.tracker.initialize_tool(tool_id).await?;

        // Initialize adaptive management
        self.adaptive_manager
            .initialize_tool(tool_id, base_limits, max_limits)
            .await?;

        info!("Initialized enhanced resource management for tool {}", tool_id);
        Ok(())
    }

    /// Sets resource limits for a tool
    #[instrument(skip(self))]
    pub async fn set_limits(
        &self,
        tool_id: &str,
        base_limits: ResourceLimits,
        max_limits: ResourceLimits,
    ) -> Result<(), ToolError> {
        // Set basic limits
        self.tracker.set_limits(tool_id, base_limits.clone()).await?;

        // Initialize adaptive limits
        self.adaptive_manager
            .initialize_tool(tool_id, base_limits, max_limits)
            .await?;

        Ok(())
    }

    /// Gets the current resource usage for a tool
    #[instrument(skip(self))]
    pub async fn get_usage(&self, tool_id: &str) -> Result<ResourceUsage, ToolError> {
        self.tracker.get_usage(tool_id).await
    }

    /// Gets the current resource limits for a tool
    #[instrument(skip(self))]
    pub async fn get_limits(&self, tool_id: &str) -> Result<ResourceLimits, ToolError> {
        // Get adaptive limits if available
        match self.adaptive_manager.get_current_limits(tool_id).await {
            Ok(limits) => Ok(limits),
            Err(_) => self.tracker.get_limits(tool_id).await,
        }
    }

    /// Tracks memory allocation for a tool
    #[instrument(skip(self))]
    pub async fn track_memory_allocation(
        &self,
        tool_id: &str,
        bytes: usize,
    ) -> Result<ResourceStatus, ToolError> {
        let status = self.tracker.track_memory_allocation(tool_id, bytes).await?;

        // Record usage for adaptive management
        let usage = self.tracker.get_usage(tool_id).await?;
        self.adaptive_manager
            .record_usage(tool_id, ResourceType::Memory, &usage)
            .await?;

        // Adjust limits if needed
        self.adaptive_manager.adjust_limits(tool_id).await?;

        Ok(status)
    }

    /// Tracks CPU time usage for a tool
    #[instrument(skip(self))]
    pub async fn track_cpu_time(
        &self,
        tool_id: &str,
        time_ms: u64,
    ) -> Result<ResourceStatus, ToolError> {
        let status = self.tracker.track_cpu_time(tool_id, time_ms).await?;

        // Record usage for adaptive management
        let usage = self.tracker.get_usage(tool_id).await?;
        self.adaptive_manager
            .record_usage(tool_id, ResourceType::CpuTime, &usage)
            .await?;

        // Adjust limits if needed
        self.adaptive_manager.adjust_limits(tool_id).await?;

        Ok(status)
    }

    /// Tracks file handle allocation for a tool
    #[instrument(skip(self))]
    pub async fn track_file_handle(
        &self,
        tool_id: &str,
        handle_id: u32,
    ) -> Result<ResourceStatus, ToolError> {
        let status = self.tracker.track_file_handle(tool_id, handle_id).await?;

        // Record usage for adaptive management
        let usage = self.tracker.get_usage(tool_id).await?;
        self.adaptive_manager
            .record_usage(tool_id, ResourceType::FileHandle, &usage)
            .await?;

        // Adjust limits if needed
        self.adaptive_manager.adjust_limits(tool_id).await?;

        Ok(status)
    }

    /// Tracks network connection allocation for a tool
    #[instrument(skip(self))]
    pub async fn track_network_connection(
        &self,
        tool_id: &str,
        connection_id: u32,
    ) -> Result<ResourceStatus, ToolError> {
        let status = self
            .tracker
            .track_network_connection(tool_id, connection_id)
            .await?;

        // Record usage for adaptive management
        let usage = self.tracker.get_usage(tool_id).await?;
        self.adaptive_manager
            .record_usage(tool_id, ResourceType::NetworkConnection, &usage)
            .await?;

        // Adjust limits if needed
        self.adaptive_manager.adjust_limits(tool_id).await?;

        Ok(status)
    }

    /// Releases a file handle for a tool
    #[instrument(skip(self))]
    pub async fn release_file_handle(
        &self,
        tool_id: &str,
        handle_id: u32,
    ) -> Result<(), ToolError> {
        self.tracker.release_file_handle(tool_id, handle_id).await
    }

    /// Releases a network connection for a tool
    #[instrument(skip(self))]
    pub async fn release_network_connection(
        &self,
        tool_id: &str,
        connection_id: u32,
    ) -> Result<(), ToolError> {
        self.tracker
            .release_network_connection(tool_id, connection_id)
            .await
    }

    /// Releases all resources for a tool
    #[instrument(skip(self))]
    pub async fn release_all_resources(&self, tool_id: &str) -> Result<(), ToolError> {
        self.tracker.release_all_resources(tool_id).await
    }

    /// Cleans up all resource management for a tool
    #[instrument(skip(self))]
    pub async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Cleanup basic tracking
        self.tracker.cleanup_tool(tool_id).await?;

        // Cleanup adaptive management
        self.adaptive_manager.cleanup_tool(tool_id).await?;

        info!("Cleaned up enhanced resource management for tool {}", tool_id);
        Ok(())
    }

    /// Gets resource usage history for a tool
    #[instrument(skip(self))]
    pub async fn get_history(&self, tool_id: &str) -> Vec<ResourceRecord> {
        self.tracker.get_history(tool_id).await
    }

    /// Enables resource tracking
    pub fn enable_tracking(&mut self) {
        self.tracker.enable_tracking();
    }

    /// Disables resource tracking
    pub fn disable_tracking(&mut self) {
        self.tracker.disable_tracking();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_enhanced_resource_management() {
        let manager = EnhancedResourceManager::new(1000);
        let tool_id = "test_tool";

        // Initialize with base and max limits
        let base_limits = ResourceLimits {
            max_memory_bytes: 100_000_000,
            max_cpu_time_ms: 30_000,
            max_file_handles: 50,
            max_network_connections: 10,
        };

        let max_limits = ResourceLimits {
            max_memory_bytes: 500_000_000,
            max_cpu_time_ms: 120_000,
            max_file_handles: 200,
            max_network_connections: 50,
        };

        manager
            .initialize_tool(tool_id, base_limits.clone(), max_limits)
            .await
            .unwrap();

        // Test memory allocation
        let status = manager
            .track_memory_allocation(tool_id, 50_000_000)
            .await
            .unwrap();
        assert_eq!(status, ResourceStatus::Normal);

        // Test CPU time tracking
        let status = manager.track_cpu_time(tool_id, 15_000).await.unwrap();
        assert_eq!(status, ResourceStatus::Normal);

        // Test file handle tracking
        let status = manager.track_file_handle(tool_id, 1).await.unwrap();
        assert_eq!(status, ResourceStatus::Normal);

        // Test network connection tracking
        let status = manager.track_network_connection(tool_id, 1).await.unwrap();
        assert_eq!(status, ResourceStatus::Normal);

        // Let adaptive management work
        sleep(Duration::from_millis(100)).await;

        // Check current limits
        let current_limits = manager.get_limits(tool_id).await.unwrap();
        assert!(current_limits.max_memory_bytes >= base_limits.max_memory_bytes);

        // Cleanup
        manager.cleanup_tool(tool_id).await.unwrap();
    }
} 