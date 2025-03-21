use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, instrument};

use crate::tool::ToolError;
use super::{ResourceUsage, ResourceLimits};

/// Constants for resource tracking
pub const TRACKING_INTERVAL_MS: u64 = 1000;
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 100; // 100 MB
pub const DEFAULT_CPU_TIME_LIMIT: u64 = 60 * 1000; // 60 seconds
pub const DEFAULT_FILE_HANDLE_LIMIT: usize = 100;
pub const DEFAULT_NETWORK_CONNECTION_LIMIT: usize = 20;

/// Resource allocation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Resources within normal limits
    Normal,
    /// Resource usage nearing limits
    Warning,
    /// Resource usage exceeded limits
    Critical,
}

/// Type of resource being tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory allocation
    Memory,
    /// CPU time usage
    CpuTime,
    /// File handle
    FileHandle,
    /// Network connection
    NetworkConnection,
}

/// Record of resource usage for logging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecord {
    /// ID of the tool using the resource
    pub tool_id: String,
    /// Timestamp of the record
    pub timestamp: DateTime<Utc>,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Current usage value
    pub usage: String,
    /// Status of the resource
    pub status: ResourceStatus,
}

/// Resource event types for tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceEvent {
    /// Resource allocation event
    Allocation(ResourceType, String),
    /// Resource release event
    Release(ResourceType, String),
    /// Resource limit exceeded event
    LimitExceeded(ResourceType, String),
}

/// Tracks and manages resource usage for tools
#[derive(Debug)]
pub struct ResourceTracker {
    /// Current resource usage by tool
    resources: Arc<RwLock<HashMap<String, ResourceUsage>>>,
    /// Resource limits by tool
    limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,
    /// History of resource usage records
    history: Arc<RwLock<Vec<ResourceRecord>>>,
    /// Maximum number of history records to keep
    max_history_size: usize,
    /// Whether tracking is enabled
    tracking_enabled: bool,
}

impl ResourceTracker {
    /// Creates a new resource tracker with the specified history size
    pub fn new(max_history_size: usize) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            limits: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::with_capacity(max_history_size))),
            max_history_size,
            tracking_enabled: true,
        }
    }
    
    /// Initializes a tool for resource tracking
    #[instrument(skip(self))]
    pub async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut resources = self.resources.write().await;
        let mut limits = self.limits.write().await;
        
        if resources.contains_key(tool_id) {
            return Err(ToolError::RegistrationFailed(
                format!("Tool {} is already registered for resource tracking", tool_id)
            ));
        }
        
        // Initialize with default usage and limits
        resources.insert(tool_id.to_string(), ResourceUsage {
            memory_bytes: 0,
            cpu_time_ms: 0,
            file_handles: Vec::new(),
            network_connections: Vec::new(),
        });
        
        limits.insert(tool_id.to_string(), ResourceLimits::default());
        
        info!("Initialized resource tracking for tool {}", tool_id);
        Ok(())
    }
    
    /// Sets resource limits for a tool
    #[instrument(skip(self))]
    pub async fn set_limits(&self, tool_id: &str, limits: ResourceLimits) -> Result<(), ToolError> {
        let mut limits_map = self.limits.write().await;
        
        if !limits_map.contains_key(tool_id) && !self.resources.read().await.contains_key(tool_id) {
            return Err(ToolError::ToolNotFound(
                format!("Tool {} not found for setting resource limits", tool_id)
            ));
        }
        
        limits_map.insert(tool_id.to_string(), limits);
        info!("Set resource limits for tool {}", tool_id);
        Ok(())
    }
    
    /// Gets the current resource usage for a tool
    #[instrument(skip(self))]
    pub async fn get_usage(&self, tool_id: &str) -> Result<ResourceUsage, ToolError> {
        let resources = self.resources.read().await;
        
        resources.get(tool_id)
            .ok_or_else(|| ToolError::ToolNotFound(
                format!("Tool {} not found for resource usage lookup", tool_id)
            ))
            .cloned()
    }
    
    /// Gets the current resource limits for a tool
    #[instrument(skip(self))]
    pub async fn get_limits(&self, tool_id: &str) -> Result<ResourceLimits, ToolError> {
        let limits = self.limits.read().await;
        
        limits.get(tool_id)
            .ok_or_else(|| ToolError::ToolNotFound(
                format!("Tool {} not found for resource limits lookup", tool_id)
            ))
            .cloned()
    }
    
    /// Tracks memory allocation for a tool
    #[instrument(skip(self))]
    pub async fn track_memory_allocation(&self, tool_id: &str, bytes: usize) -> Result<ResourceStatus, ToolError> {
        if !self.tracking_enabled {
            return Ok(ResourceStatus::Normal);
        }

        let mut resources = self.resources.write().await;
        let limits = self.limits.read().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for memory tracking", tool_id))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for memory limits", tool_id))
        )?;
        
        usage.memory_bytes += bytes;
        
        let status = self.check_resource_status(
            usage.memory_bytes, 
            limit.max_memory_bytes,
            ResourceType::Memory,
            tool_id,
            &format!("{} bytes", usage.memory_bytes)
        ).await;
        
        Ok(status)
    }
    
    /// Tracks CPU time usage for a tool
    #[instrument(skip(self))]
    pub async fn track_cpu_time(&self, tool_id: &str, time_ms: u64) -> Result<ResourceStatus, ToolError> {
        if !self.tracking_enabled {
            return Ok(ResourceStatus::Normal);
        }

        let mut resources = self.resources.write().await;
        let limits = self.limits.read().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for CPU time tracking", tool_id))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for CPU time limits", tool_id))
        )?;
        
        usage.cpu_time_ms += time_ms;
        
        let status = self.check_resource_status(
            usage.cpu_time_ms as usize, 
            limit.max_cpu_time_ms as usize,
            ResourceType::CpuTime,
            tool_id,
            &format!("{} ms", usage.cpu_time_ms)
        ).await;
        
        Ok(status)
    }
    
    /// Tracks file handle allocation for a tool
    #[instrument(skip(self))]
    pub async fn track_file_handle(&self, tool_id: &str, handle_id: u32) -> Result<ResourceStatus, ToolError> {
        if !self.tracking_enabled {
            return Ok(ResourceStatus::Normal);
        }

        let mut resources = self.resources.write().await;
        let limits = self.limits.read().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for file handle tracking", tool_id))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for file handle limits", tool_id))
        )?;
        
        // Add file handle if not already tracked
        if !usage.file_handles.contains(&handle_id) {
            usage.file_handles.push(handle_id);
        }
        
        let status = self.check_resource_status(
            usage.file_handles.len(), 
            limit.max_file_handles,
            ResourceType::FileHandle,
            tool_id,
            &format!("{} handles", usage.file_handles.len())
        ).await;
        
        Ok(status)
    }
    
    /// Tracks network connection for a tool
    #[instrument(skip(self))]
    pub async fn track_network_connection(&self, tool_id: &str, connection_id: u32) -> Result<ResourceStatus, ToolError> {
        if !self.tracking_enabled {
            return Ok(ResourceStatus::Normal);
        }

        let mut resources = self.resources.write().await;
        let limits = self.limits.read().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for network connection tracking", tool_id))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for network connection limits", tool_id))
        )?;
        
        // Add connection if not already tracked
        if !usage.network_connections.contains(&connection_id) {
            usage.network_connections.push(connection_id);
        }
        
        let status = self.check_resource_status(
            usage.network_connections.len(), 
            limit.max_network_connections,
            ResourceType::NetworkConnection,
            tool_id,
            &format!("{} connections", usage.network_connections.len())
        ).await;
        
        Ok(status)
    }
    
    /// Releases a file handle for a tool
    #[instrument(skip(self))]
    pub async fn release_file_handle(&self, tool_id: &str, handle_id: u32) -> Result<(), ToolError> {
        if !self.tracking_enabled {
            return Ok(());
        }

        let mut resources = self.resources.write().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for file handle release", tool_id))
        )?;
        
        usage.file_handles.retain(|&h| h != handle_id);
        
        self.record_resource_event(
            ResourceEvent::Release(ResourceType::FileHandle, handle_id.to_string()),
            tool_id
        ).await;
        
        Ok(())
    }
    
    /// Releases a network connection for a tool
    #[instrument(skip(self))]
    pub async fn release_network_connection(&self, tool_id: &str, connection_id: u32) -> Result<(), ToolError> {
        if !self.tracking_enabled {
            return Ok(());
        }

        let mut resources = self.resources.write().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for network connection release", tool_id))
        )?;
        
        usage.network_connections.retain(|&c| c != connection_id);
        
        self.record_resource_event(
            ResourceEvent::Release(ResourceType::NetworkConnection, connection_id.to_string()),
            tool_id
        ).await;
        
        Ok(())
    }
    
    /// Releases all resources for a tool
    #[instrument(skip(self))]
    pub async fn release_all_resources(&self, tool_id: &str) -> Result<(), ToolError> {
        if !self.tracking_enabled {
            return Ok(());
        }

        let mut resources = self.resources.write().await;
        
        let usage = resources.get_mut(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {} not found for resource release", tool_id))
        )?;
        
        // Record file handle releases
        for handle in usage.file_handles.drain(..) {
            self.record_resource_event(
                ResourceEvent::Release(ResourceType::FileHandle, handle.to_string()),
                tool_id
            ).await;
        }
        
        // Record network connection releases
        for conn in usage.network_connections.drain(..) {
            self.record_resource_event(
                ResourceEvent::Release(ResourceType::NetworkConnection, conn.to_string()),
                tool_id
            ).await;
        }
        
        // Reset memory and CPU time
        usage.memory_bytes = 0;
        usage.cpu_time_ms = 0;
        
        Ok(())
    }
    
    /// Removes a tool from tracking
    #[instrument(skip(self))]
    pub async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Release all resources first
        self.release_all_resources(tool_id).await?;
        
        // Remove from tracking
        let mut resources = self.resources.write().await;
        let mut limits = self.limits.write().await;
        
        resources.remove(tool_id);
        limits.remove(tool_id);
        
        info!("Cleaned up resource tracking for tool {}", tool_id);
        Ok(())
    }
    
    /// Gets historical resource usage records for a tool
    #[instrument(skip(self))]
    pub async fn get_history(&self, tool_id: &str) -> Vec<ResourceRecord> {
        let history = self.history.read().await;
        history.iter()
            .filter(|record| record.tool_id == tool_id)
            .cloned()
            .collect()
    }
    
    /// Enables resource tracking
    pub fn enable_tracking(&mut self) {
        self.tracking_enabled = true;
        info!("Resource tracking enabled");
    }
    
    /// Disables resource tracking
    pub fn disable_tracking(&mut self) {
        self.tracking_enabled = false;
        info!("Resource tracking disabled");
    }
    
    /// Records resource usage in history
    async fn record_usage(&self, resource_type: ResourceType, tool_id: &str, usage: &str, status: ResourceStatus) {
        if !self.tracking_enabled {
            return;
        }
        
        let record = ResourceRecord {
            tool_id: tool_id.to_string(),
            timestamp: Utc::now(),
            resource_type,
            usage: usage.to_string(),
            status,
        };
        
        let mut history = self.history.write().await;
        
        // Maintain history size limit
        if history.len() >= self.max_history_size {
            history.remove(0);
        }
        
        history.push(record);
    }
    
    /// Records a resource event
    async fn record_resource_event(&self, event: ResourceEvent, tool_id: &str) {
        if !self.tracking_enabled {
            return;
        }
        
        match &event {
            ResourceEvent::Allocation(resource_type, details) => {
                info!("Resource allocated: {:?} - {} for tool {}", resource_type, details, tool_id);
            },
            ResourceEvent::Release(resource_type, details) => {
                info!("Resource released: {:?} - {} for tool {}", resource_type, details, tool_id);
            },
            ResourceEvent::LimitExceeded(resource_type, details) => {
                warn!("Resource limit exceeded: {:?} - {} for tool {}", resource_type, details, tool_id);
            },
        }
    }
    
    /// Checks resource status against limits
    async fn check_resource_status(
        &self, 
        current: usize, 
        limit: usize, 
        resource_type: ResourceType,
        tool_id: &str,
        usage_str: &str
    ) -> ResourceStatus {
        let status = if current >= limit {
            self.record_resource_event(
                ResourceEvent::LimitExceeded(resource_type, usage_str.to_string()),
                tool_id
            ).await;
            ResourceStatus::Critical
        } else if current >= (limit * 80) / 100 {
            ResourceStatus::Warning
        } else {
            ResourceStatus::Normal
        };
        
        self.record_usage(resource_type, tool_id, usage_str, status).await;
        
        status
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new(1000) // Default to storing 1000 history records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resource_tracking_initialization() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        let result = tracker.initialize_tool(tool_id).await;
        assert!(result.is_ok());
        
        // Check that the tool has been initialized
        let usage = tracker.get_usage(tool_id).await.unwrap();
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_time_ms, 0);
        assert!(usage.file_handles.is_empty());
        assert!(usage.network_connections.is_empty());
        
        // Attempting to initialize again should fail
        let result = tracker.initialize_tool(tool_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_memory_tracking() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
        
        // Track memory allocation
        let status = tracker.track_memory_allocation(tool_id, 10 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        
        // Verify status
        assert_eq!(status, ResourceStatus::Normal);
        
        // Get usage
        let usage = tracker.get_usage(tool_id).await.expect("Failed to get usage");
        
        // Verify memory usage
        assert_eq!(usage.memory_bytes, 10 * 1024 * 1024);
        
        // Track more memory allocation to exceed limit
        let status = tracker.track_memory_allocation(tool_id, 95 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        
        // Verify status (should be critical because we exceeded 100MB)
        assert_eq!(status, ResourceStatus::Critical);
    }
    
    #[tokio::test]
    async fn test_file_handle_tracking() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
        
        // Track file handle allocation
        for i in 0..10 {
            let status = tracker.track_file_handle(tool_id, i as u32).await
                .expect("Failed to track file handle");
            
            // Verify status
            assert_eq!(status, ResourceStatus::Normal);
        }
        
        // Get usage
        let usage = tracker.get_usage(tool_id).await.expect("Failed to get usage");
        
        // Verify file handle usage
        assert_eq!(usage.file_handles.len(), 10);
        
        // Release a file handle
        tracker.release_file_handle(tool_id, 5 as u32).await
            .expect("Failed to release file handle");
        
        // Get usage again
        let usage = tracker.get_usage(tool_id).await
            .expect("Failed to get usage");
        
        // Verify file handle usage after release
        assert_eq!(usage.file_handles.len(), 9);
        assert!(!usage.file_handles.contains(&5));
    }
    
    #[tokio::test]
    async fn test_release_all_resources() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
        
        // Track various resources
        tracker.track_memory_allocation(tool_id, 10 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        tracker.track_cpu_time(tool_id, 1000).await
            .expect("Failed to track CPU time");
        tracker.track_file_handle(tool_id, 1 as u32).await
            .expect("Failed to track file handle");
        tracker.track_network_connection(tool_id, 1 as u32).await
            .expect("Failed to track network connection");
        
        // Release all resources
        tracker.release_all_resources(tool_id).await
            .expect("Failed to release all resources");
        
        // Get usage
        let usage = tracker.get_usage(tool_id).await.expect("Failed to get usage");
        
        // Verify all resources released
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_time_ms, 0);
        assert_eq!(usage.file_handles.len(), 0);
        assert_eq!(usage.network_connections.len(), 0);
    }
    
    #[tokio::test]
    async fn test_cleanup_tool() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
        
        // Track a resource
        tracker.track_memory_allocation(tool_id, 10 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        
        // Cleanup tool
        tracker.cleanup_tool(tool_id).await
            .expect("Failed to cleanup tool");
        
        // Verify tool is removed from tracking
        let result = tracker.get_usage(tool_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_custom_limits() {
        let tracker = ResourceTracker::default();
        let tool_id = "test-tool";
        
        // Initialize tool
        tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
        
        // Set custom limits
        let custom_limits = ResourceLimits {
            max_memory_bytes: 5 * 1024 * 1024, // 5MB
            max_cpu_time_ms: 5000,             // 5 seconds
            max_file_handles: 50,
            max_network_connections: 10,
        };
        
        tracker.set_limits(tool_id, custom_limits.clone()).await
            .expect("Failed to set limits");
        
        // Get limits
        let limits = tracker.get_limits(tool_id).await.expect("Failed to get limits");
        
        // Verify custom limits
        assert_eq!(limits.max_memory_bytes, custom_limits.max_memory_bytes);
        assert_eq!(limits.max_cpu_time_ms, custom_limits.max_cpu_time_ms);
        assert_eq!(limits.max_file_handles, custom_limits.max_file_handles);
        assert_eq!(limits.max_network_connections, custom_limits.max_network_connections);
        
        // Track memory allocation that exceeds new limit
        let status = tracker.track_memory_allocation(tool_id, 4 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        
        // Should be warning (>75% of 5MB)
        assert_eq!(status, ResourceStatus::Warning);
        
        // Track more memory to exceed limit
        let status = tracker.track_memory_allocation(tool_id, 2 * 1024 * 1024).await
            .expect("Failed to track memory allocation");
        
        // Should be critical (>100% of 5MB)
        assert_eq!(status, ResourceStatus::Critical);
    }
} 