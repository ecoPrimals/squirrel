use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use super::resource_manager::{ResourceLimits, ResourceManager, ResourceUsage};
use super::resource_tracker::ResourceTracker;
use crate::tool::ToolError;

/// A basic resource manager implementation
#[derive(Debug)]
pub struct BasicResourceManager {
    /// Resource trackers by tool ID
    trackers: Arc<RwLock<HashMap<String, Arc<ResourceTracker>>>>,
    /// Resource limits by tool ID
    limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,
    /// Maximum allowed limits by tool ID
    max_limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,
}

impl Default for BasicResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicResourceManager {
    /// Creates a new resource manager
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(RwLock::new(HashMap::new())),
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ResourceManager for BasicResourceManager {
    #[instrument(skip(self))]
    async fn initialize_tool(
        &self,
        tool_id: &str,
        base_limits: ResourceLimits,
        max_limits: ResourceLimits,
    ) -> Result<(), ToolError> {
        info!("Initializing resource tracking for tool: {}", tool_id);

        // Create tracker
        let tracker = Arc::new(ResourceTracker::new(tool_id));

        // Store in maps
        {
            let mut trackers = self.trackers.write().await;
            let mut limits = self.limits.write().await;
            let mut max_limits_map = self.max_limits.write().await;

            trackers.insert(tool_id.to_string(), tracker);
            limits.insert(tool_id.to_string(), base_limits);
            max_limits_map.insert(tool_id.to_string(), max_limits);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Cleaning up resources for tool: {}", tool_id);

        // Get the tracker
        let tracker = {
            let trackers = self.trackers.read().await;
            match trackers.get(tool_id) {
                Some(t) => t.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            }
        };

        // Perform cleanup
        tracker.cleanup().await.map_err(|err| {
            ToolError::ResourceError(format!("Failed to clean up resources: {}", err))
        })?;

        // Remove from maps
        {
            let mut trackers = self.trackers.write().await;
            let mut limits = self.limits.write().await;
            let mut max_limits = self.max_limits.write().await;

            trackers.remove(tool_id);
            limits.remove(tool_id);
            max_limits.remove(tool_id);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update_limits(
        &self,
        tool_id: &str,
        new_limits: ResourceLimits,
    ) -> Result<(), ToolError> {
        info!("Updating resource limits for tool: {}", tool_id);

        // Check if the new limits exceed max limits
        {
            let max_limits = self.max_limits.read().await;
            if let Some(max) = max_limits.get(tool_id) {
                if new_limits.max_memory_bytes > max.max_memory_bytes
                    || new_limits.max_cpu_time_ms > max.max_cpu_time_ms
                    || new_limits.max_file_handles > max.max_file_handles
                    || new_limits.max_network_connections > max.max_network_connections
                {
                    return Err(ToolError::ResourceError(format!(
                        "Requested limits exceed maximum allowed limits for tool {}",
                        tool_id
                    )));
                }
            } else {
                return Err(ToolError::ToolNotFound(tool_id.to_string()));
            }
        }

        // Update limits
        {
            let mut limits = self.limits.write().await;
            limits.insert(tool_id.to_string(), new_limits);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Resetting resource tracking for tool: {}", tool_id);

        // Get the tracker
        let tracker = {
            let trackers = self.trackers.read().await;
            match trackers.get(tool_id) {
                Some(t) => t.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            }
        };

        // Reset the tracker
        tracker.reset().await.map_err(|err| {
            ToolError::ResourceError(format!("Failed to reset resources: {}", err))
        })?;

        // No need to update limits
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_usage(&self, tool_id: &str) -> Result<ResourceUsage, ToolError> {
        // Get the tracker
        let tracker = {
            let trackers = self.trackers.read().await;
            match trackers.get(tool_id) {
                Some(t) => t.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            }
        };

        // Get current usage
        let usage = tracker.get_current_usage().await.map_err(|err| {
            ToolError::ResourceError(format!("Failed to get resource usage: {}", err))
        })?;

        // Convert to ResourceUsage
        Ok(ResourceUsage {
            memory_bytes: usage.memory_bytes,
            cpu_time_ms: usage.cpu_time_ms,
            file_handles: usage.file_handles.len(),
            network_connections: usage.network_connections.len(),
        })
    }

    #[instrument(skip(self))]
    async fn check_limits(&self, tool_id: &str) -> Result<bool, ToolError> {
        // Get the tracker and limits
        let (tracker, tool_limits) = {
            let trackers = self.trackers.read().await;
            let limits = self.limits.read().await;

            let t = match trackers.get(tool_id) {
                Some(t) => t.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            };

            let l = match limits.get(tool_id) {
                Some(l) => l.clone(),
                None => return Err(ToolError::ToolNotFound(tool_id.to_string())),
            };

            (t, l)
        };

        // Get current usage
        let usage = tracker.get_current_usage().await.map_err(|err| {
            ToolError::ResourceError(format!("Failed to get resource usage: {}", err))
        })?;

        // Check against limits
        let within_limits = usage.memory_bytes <= tool_limits.max_memory_bytes
            && usage.cpu_time_ms <= tool_limits.max_cpu_time_ms
            && usage.file_handles.len() <= tool_limits.max_file_handles
            && usage.network_connections.len() <= tool_limits.max_network_connections;

        Ok(within_limits)
    }
}
