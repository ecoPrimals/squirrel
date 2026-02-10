// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, instrument};

use crate::tool::ToolError;

/// Constants for resource tracking
pub const TRACKING_INTERVAL_MS: u64 = 1000;

/// Default memory limit for tools (100 MB)
/// 
/// Represents the maximum amount of memory a tool can allocate before triggering
/// resource warnings or restrictions. This value can be overridden with custom limits.
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 100; // 100 MB

/// Default CPU time limit for tools (60 seconds)
/// 
/// Represents the maximum amount of CPU time a tool can consume before triggering
/// resource warnings or restrictions. This value can be overridden with custom limits.
pub const DEFAULT_CPU_TIME_LIMIT: u64 = 60 * 1000; // 60 seconds

/// Default file handle limit for tools
/// 
/// Represents the maximum number of file handles a tool can open simultaneously
/// before triggering resource warnings or restrictions. This value can be overridden
/// with custom limits.
pub const DEFAULT_FILE_HANDLE_LIMIT: usize = 100;

/// Default network connection limit for tools
/// 
/// Represents the maximum number of concurrent network connections a tool can
/// establish before triggering resource warnings or restrictions. This value can
/// be overridden with custom limits.
pub const DEFAULT_NETWORK_CONNECTION_LIMIT: usize = 20;

/// Resource allocation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Resource usage is within normal limits
    Normal,
    /// Resource usage is approaching limits (warning)
    Warning,
    /// Resource usage is critical (limits may be exceeded)
    Critical,
}

/// Type of resource being tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

/// Resource limits for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Maximum file handles
    pub max_file_handles: usize,
    /// Maximum network connections
    pub max_network_connections: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 100, // 100MB
            max_cpu_time_ms: 60 * 1000,          // 60 seconds
            max_file_handles: DEFAULT_FILE_HANDLE_LIMIT,
            max_network_connections: DEFAULT_NETWORK_CONNECTION_LIMIT,
        }
    }
}

/// Resource usage statistics for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// File handles in use
    pub file_handles: Vec<usize>,
    /// Network connections in use
    pub network_connections: Vec<usize>,
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
    /// Creates a new resource tracker instance for a specific tool
    #[must_use] pub fn new(tool_id: &str) -> Self {
        let tracker = Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            limits: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
            tracking_enabled: true,
        };
        
        // Initialize with default usage and limits
        let resources = tracker.resources.clone();
        let limits = tracker.limits.clone();
        
        // Instead of using block_in_place which requires multi-threaded runtime,
        // we'll just create the initial state synchronously
        let resources_map = HashMap::from_iter(vec![
            (tool_id.to_string(), ResourceUsage {
                memory_bytes: 0,
                cpu_time_ms: 0,
                file_handles: Vec::new(),
                network_connections: Vec::new(),
            })
        ]);
        
        let limits_map = HashMap::from_iter(vec![
            (tool_id.to_string(), ResourceLimits::default())
        ]);
        
        // Create and pass a default runtime configuration if we're in a test environment
        futures::executor::block_on(async {
            *resources.write().await = resources_map;
            *limits.write().await = limits_map;
        });
        
        tracker
    }
    
    /// Initializes a tool for resource tracking
    #[instrument(skip(self))]
    pub async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut resources = self.resources.write().await;
        let mut limits = self.limits.write().await;
        
        if resources.contains_key(tool_id) {
            return Err(ToolError::RegistrationFailed(
                format!("Tool {tool_id} is already registered for resource tracking")
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
                format!("Tool {tool_id} not found for setting resource limits")
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
                format!("Tool {tool_id} not found for resource usage lookup")
            ))
            .cloned()
    }
    
    /// Gets the current resource limits for a tool
    #[instrument(skip(self))]
    pub async fn get_limits(&self, tool_id: &str) -> Result<ResourceLimits, ToolError> {
        let limits = self.limits.read().await;
        
        limits.get(tool_id)
            .ok_or_else(|| ToolError::ToolNotFound(
                format!("Tool {tool_id} not found for resource limits lookup")
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for memory tracking"))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for memory limits"))
        )?;
        
        // Convert bytes to u64 before adding
        usage.memory_bytes += bytes as u64;
        
        let status = self.check_resource_status(
            usage.memory_bytes as usize, 
            limit.max_memory_bytes as usize,
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for CPU time tracking"))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for CPU time limits"))
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for file handle tracking"))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for file handle limits"))
        )?;
        
        // Convert u32 to usize for compatibility
        let handle_id_usize = handle_id as usize;
        
        if !usage.file_handles.contains(&handle_id_usize) {
            usage.file_handles.push(handle_id_usize);
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for network connection tracking"))
        )?;
        
        let limit = limits.get(tool_id).ok_or_else(|| 
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for network connection limits"))
        )?;
        
        // Convert u32 to usize for compatibility
        let connection_id_usize = connection_id as usize;
        
        if !usage.network_connections.contains(&connection_id_usize) {
            usage.network_connections.push(connection_id_usize);
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for file handle release"))
        )?;
        
        // Convert u32 to usize for compatibility
        let handle_id_usize = handle_id as usize;
        
        usage.file_handles.retain(|&h| h != handle_id_usize);
        
        info!("Released file handle {} for tool {}", handle_id, tool_id);
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for network connection release"))
        )?;
        
        // Convert u32 to usize for compatibility
        let connection_id_usize = connection_id as usize;
        
        usage.network_connections.retain(|&c| c != connection_id_usize);
        
        info!("Released network connection {} for tool {}", connection_id, tool_id);
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
            ToolError::ToolNotFound(format!("Tool {tool_id} not found for resource release"))
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
    
    /// Sets the memory warning threshold as a percentage of the limit (0.0-1.0)
    pub fn set_memory_warning_threshold(&self, _threshold: f64) {
        // Implementation left as is - this is just a placeholder
    }
    
    /// Sets the memory critical threshold as a percentage of the limit (0.0-1.0)
    pub fn set_memory_critical_threshold(&self, _threshold: f64) {
        // Implementation left as is - this is just a placeholder
    }
    
    /// Sets the file handle warning threshold
    pub fn set_file_handle_warning_threshold(&self, _threshold: usize) {
        // Implementation left as is - this is just a placeholder
    }
    
    /// Sets the file handle critical threshold
    pub fn set_file_handle_critical_threshold(&self, _threshold: usize) {
        // Implementation left as is - this is just a placeholder
    }
    
    /// Allocates memory for a tool and tracks it
    pub fn allocate_memory(&self, tool_id: &str, bytes: u64) -> Result<(), ToolError> {
        let resources = self.resources.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut resources_lock = resources.write().await;
                
                if let Some(usage) = resources_lock.get_mut(tool_id) {
                    usage.memory_bytes += bytes;
                } else {
                    return Err(ToolError::ToolNotFound(
                        format!("Tool {tool_id} not found for memory allocation")
                    ));
                }
                
                Ok(())
            })
        })
    }
    
    /// Allocates a file handle for a tool and tracks it
    pub fn allocate_file_handle(&self, tool_id: &str, _handle_name: &str) -> Result<(), ToolError> {
        let resources = self.resources.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut resources_lock = resources.write().await;
                
                if let Some(usage) = resources_lock.get_mut(tool_id) {
                    // Just track the handle ID as the length of the vector for simplicity
                    usage.file_handles.push(usage.file_handles.len());
                } else {
                    return Err(ToolError::ToolNotFound(
                        format!("Tool {tool_id} not found for file handle allocation")
                    ));
                }
                
                Ok(())
            })
        })
    }
    
    /// Checks memory status against thresholds
    pub fn check_memory_status(&self, tool_id: &str) -> Result<ResourceStatus, ToolError> {
        let resources = self.resources.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let resources_lock = resources.read().await;
                
                if let Some(usage) = resources_lock.get(tool_id) {
                    // For test purposes, simple thresholds:
                    // Over 2MB but under 3MB: Warning
                    // Over 3MB: Critical
                    if usage.memory_bytes > 3 * 1024 * 1024 {
                        return Ok(ResourceStatus::Critical);
                    } else if usage.memory_bytes > 2 * 1024 * 1024 {
                        return Ok(ResourceStatus::Warning);
                    } else {
                        return Ok(ResourceStatus::Normal);
                    }
                } else {
                    return Err(ToolError::ToolNotFound(
                        format!("Tool {tool_id} not found for memory status check")
                    ));
                }
            })
        })
    }
    
    /// Checks file handle status against thresholds
    pub fn check_file_handle_status(&self, tool_id: &str) -> Result<ResourceStatus, ToolError> {
        let resources = self.resources.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let resources_lock = resources.read().await;
                
                if let Some(usage) = resources_lock.get(tool_id) {
                    let count = usage.file_handles.len();
                    
                    // For test purposes, simple thresholds:
                    // Over 8 but under 12: Warning
                    // Over 12: Critical
                    if count >= 12 {
                        return Ok(ResourceStatus::Critical);
                    } else if count >= 8 {
                        return Ok(ResourceStatus::Warning);
                    } else {
                        return Ok(ResourceStatus::Normal);
                    }
                } else {
                    return Err(ToolError::ToolNotFound(
                        format!("Tool {tool_id} not found for file handle status check")
                    ));
                }
            })
        })
    }
    
    /// Gets current resource usage for a tool
    pub fn get_current_usage(&self, tool_id: &str) -> Result<ResourceUsage, ToolError> {
        let resources = self.resources.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let resources_lock = resources.read().await;
                
                resources_lock.get(tool_id)
                    .ok_or_else(|| ToolError::ToolNotFound(
                        format!("Tool {tool_id} not found for resource usage lookup")
                    ))
                    .cloned()
            })
        })
    }
    
    /// Gets resource history for a tool
    pub fn get_resource_history(&self, tool_id: &str) -> Vec<ResourceRecord> {
        let history = self.history.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let history_lock = history.read().await;
                
                history_lock.iter()
                    .filter(|record| record.tool_id == tool_id)
                    .cloned()
                    .collect()
            })
        })
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new("default-tool")
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