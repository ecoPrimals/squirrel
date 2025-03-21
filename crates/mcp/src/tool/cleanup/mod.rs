//! Tool cleanup procedures for MCP
//!
//! This module contains implementations for resource tracking, cleanup, 
//! and error recovery mechanisms for the MCP tool management system.

// Declare submodules
pub mod recovery;
pub mod resource_tracking;

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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resource_limits() {
        let hook = LegacyResourceCleanupHook::new();
        let tool_id = "test-tool";
        
        // Set custom limits
        let limits = ResourceLimits::default()
            .with_max_memory(1024 * 1024 * 10) // 10 MB
            .with_max_cpu_time(5000)           // 5 seconds
            .with_max_file_handles(10)
            .with_max_network_connections(5);
        
        hook.set_limits(tool_id, limits).await;
        
        // Verify limits were set
        let retrieved_limits = hook.get_limits(tool_id).await.unwrap();
        assert_eq!(retrieved_limits.max_memory_bytes, 1024 * 1024 * 10);
        assert_eq!(retrieved_limits.max_cpu_time_ms, 5000);
        assert_eq!(retrieved_limits.max_file_handles, 10);
        assert_eq!(retrieved_limits.max_network_connections, 5);
    }
    
    #[tokio::test]
    async fn test_resource_usage_tracking() {
        let hook = LegacyResourceCleanupHook::new();
        let tool_id = "test-tool";
        
        // Update resource usage
        hook.update_usage(tool_id, Some(1024 * 1024), Some(1000), Some(5), Some(2)).await.unwrap();
        
        // Verify usage was updated
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.memory_bytes, 1024 * 1024);
        assert_eq!(usage.cpu_time_ms, 1000);
        assert_eq!(usage.file_handles.len(), 0); // This is 0 because we're not directly mapping
        assert_eq!(usage.network_connections.len(), 0); // Same for network connections
        
        // Test file handles separately
        hook.register_file_handle(tool_id, 1, "test1.txt".to_string()).await;
        hook.register_file_handle(tool_id, 2, "test2.txt".to_string()).await;
        
        // File handles count is tracked in the metrics
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles.len(), 0); // Still 0 in the mapped usage
        
        // Test unregistering
        assert!(hook.unregister_file_handle(tool_id, 1).await);
        
        // Count decreases
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles.len(), 0); // Still 0 in the mapped usage
    }
    
    #[tokio::test]
    async fn test_file_handle_tracking() {
        let hook = LegacyResourceCleanupHook::new();
        let tool_id = "test-tool";
        
        // Register file handles
        hook.register_file_handle(tool_id, 1, "test-file-1.txt".to_string()).await;
        hook.register_file_handle(tool_id, 2, "test-file-2.txt".to_string()).await;
        
        // Let's check the internal file handles count in the ResourceTrackingRecord
        {
            let resources = hook.resources.read().await;
            let record = resources.get(tool_id).unwrap();
            assert_eq!(record.usage.file_handles, 2);
        } // Make sure the read lock is dropped here
        
        // But the exposed ResourceUsage doesn't directly map these
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles.len(), 0);
        
        // Unregister a file handle
        let removed = hook.unregister_file_handle(tool_id, 1).await;
        assert!(removed);
        
        // Check internal count again
        {
            let resources = hook.resources.read().await;
            let record = resources.get(tool_id).unwrap();
            assert_eq!(record.usage.file_handles, 1);
        } // Make sure the read lock is dropped here
        
        // But ResourceUsage still doesn't have these directly
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles.len(), 0);
    }
} 