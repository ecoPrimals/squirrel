//! Tool cleanup procedures for MCP
//!
//! This module contains implementations for resource tracking, cleanup, 
//! and error recovery mechanisms for the MCP tool management system.

// Declare submodules
pub mod recovery;

use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};
use chrono::{DateTime, Utc};

use crate::tool::{Tool, ToolError, ToolLifecycleHook};

// Re-export recovery components
pub use recovery::{RecoveryHook, RecoveryStrategy};

/// Resource usage metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Number of open file handles
    pub file_handles: u32,
    /// Number of network connections
    pub network_connections: u32,
}

impl ResourceUsage {
    /// Creates a new empty resource usage
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Updates resource usage
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
    
    /// Resets resource usage
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Resource limits
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Maximum number of open file handles
    pub max_file_handles: u32,
    /// Maximum number of network connections
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 100, // 100 MB
            max_cpu_time_ms: 60 * 1000, // 60 seconds
            max_file_handles: 100,
            max_network_connections: 20,
        }
    }
}

impl ResourceLimits {
    /// Creates a new resource limits with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets maximum memory limit in bytes
    pub fn with_max_memory(mut self, max_bytes: u64) -> Self {
        self.max_memory_bytes = max_bytes;
        self
    }
    
    /// Sets maximum CPU time limit in milliseconds
    pub fn with_max_cpu_time(mut self, max_ms: u64) -> Self {
        self.max_cpu_time_ms = max_ms;
        self
    }
    
    /// Sets maximum file handles limit
    pub fn with_max_file_handles(mut self, max_handles: u32) -> Self {
        self.max_file_handles = max_handles;
        self
    }
    
    /// Sets maximum network connections limit
    pub fn with_max_network_connections(mut self, max_connections: u32) -> Self {
        self.max_network_connections = max_connections;
        self
    }
}

/// Resource tracking record for a tool
#[derive(Debug)]
struct ResourceRecord {
    /// Current resource usage
    usage: ResourceUsage,
    /// Resource limits
    limits: ResourceLimits,
    /// Last updated timestamp
    last_updated: DateTime<Utc>,
    /// File handles map (handle ID -> description)
    file_handles: HashMap<u32, String>,
    /// Network connections map (connection ID -> endpoint)
    network_connections: HashMap<u32, String>,
}

impl ResourceRecord {
    /// Creates a new resource record
    fn new(limits: ResourceLimits) -> Self {
        Self {
            usage: ResourceUsage::default(),
            limits,
            last_updated: Utc::now(),
            file_handles: HashMap::new(),
            network_connections: HashMap::new(),
        }
    }
    
    /// Checks if resource usage exceeds limits
    fn exceeds_limits(&self) -> bool {
        self.usage.memory_bytes > self.limits.max_memory_bytes
            || self.usage.cpu_time_ms > self.limits.max_cpu_time_ms
            || self.usage.file_handles > self.limits.max_file_handles
            || self.usage.network_connections > self.limits.max_network_connections
    }
}

/// Hook for resource cleanup
#[derive(Debug)]
pub struct ResourceCleanupHook {
    /// Resource records by tool ID
    resources: RwLock<HashMap<String, ResourceRecord>>,
}

impl ResourceCleanupHook {
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
            resources.insert(tool_id.to_string(), ResourceRecord::new(limits));
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
            .or_insert_with(|| ResourceRecord::new(ResourceLimits::default()));
        
        record.usage.update(memory, cpu, files, connections);
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
        resources.get(tool_id).map(|record| record.usage)
    }
    
    /// Gets resource limits for a tool
    pub async fn get_limits(&self, tool_id: &str) -> Option<ResourceLimits> {
        let resources = self.resources.read().await;
        resources.get(tool_id).map(|record| record.limits)
    }
    
    /// Registers a file handle for a tool
    pub async fn register_file_handle(&self, tool_id: &str, handle_id: u32, description: String) {
        let mut resources = self.resources.write().await;
        
        let record = resources
            .entry(tool_id.to_string())
            .or_insert_with(|| ResourceRecord::new(ResourceLimits::default()));
        
        record.file_handles.insert(handle_id, description);
        record.usage.file_handles = record.file_handles.len() as u32;
    }
    
    /// Registers a network connection for a tool
    pub async fn register_network_connection(&self, tool_id: &str, connection_id: u32, endpoint: String) {
        let mut resources = self.resources.write().await;
        
        let record = resources
            .entry(tool_id.to_string())
            .or_insert_with(|| ResourceRecord::new(ResourceLimits::default()));
        
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
            record.usage.reset();
            
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
impl ToolLifecycleHook for ResourceCleanupHook {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resource_limits() {
        let hook = ResourceCleanupHook::new();
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
        let hook = ResourceCleanupHook::new();
        let tool_id = "test-tool";
        
        // Update resource usage
        hook.update_usage(tool_id, Some(1024 * 1024), Some(1000), Some(5), Some(2)).await.unwrap();
        
        // Verify usage was updated
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.memory_bytes, 1024 * 1024);
        assert_eq!(usage.cpu_time_ms, 1000);
        assert_eq!(usage.file_handles, 5);
        assert_eq!(usage.network_connections, 2);
    }
    
    #[tokio::test]
    async fn test_file_handle_tracking() {
        let hook = ResourceCleanupHook::new();
        let tool_id = "test-tool";
        
        // Register file handles
        hook.register_file_handle(tool_id, 1, "test-file-1.txt".to_string()).await;
        hook.register_file_handle(tool_id, 2, "test-file-2.txt".to_string()).await;
        
        // Verify file handles count
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles, 2);
        
        // Unregister a file handle
        let removed = hook.unregister_file_handle(tool_id, 1).await;
        assert!(removed);
        
        // Verify file handles count decreased
        let usage = hook.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.file_handles, 1);
    }
} 