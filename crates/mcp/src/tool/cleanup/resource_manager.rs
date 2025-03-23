use async_trait::async_trait;

use crate::tool::ToolError;

/// Resource usage data structure
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Number of open file handles
    pub file_handles: usize,
    /// Number of network connections
    pub network_connections: usize,
}

/// Resource limits for a tool
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
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
            max_memory_bytes: 1024 * 1024 * 100, // 100MB
            max_cpu_time_ms: 10000,              // 10 seconds
            max_file_handles: 10,
            max_network_connections: 5,
        }
    }
}

/// ResourceManager handles tracking and managing tool resource usage
#[async_trait]
pub trait ResourceManager: Send + Sync + std::fmt::Debug {
    /// Initializes resource tracking for a tool
    ///
    /// This method is called during tool initialization to set up resource tracking
    /// and limits for the tool.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to initialize
    /// * `base_limits` - Initial resource limits for the tool
    /// * `max_limits` - Maximum allowed resource limits for the tool
    async fn initialize_tool(
        &self,
        tool_id: &str,
        base_limits: ResourceLimits,
        max_limits: ResourceLimits,
    ) -> Result<(), ToolError>;

    /// Cleans up resources for a tool
    ///
    /// This method is called during tool cleanup to release any resources
    /// that were allocated to the tool.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to clean up
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError>;

    /// Resets resource tracking for a tool
    ///
    /// This method is called when a tool is reset during recovery. It should
    /// clear resource usage counters but maintain the tool's registration and limits.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to reset
    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError>;

    /// Update resource limits for a tool
    ///
    /// This method allows dynamically updating the resource limits for a tool
    /// during its lifecycle. The new limits must not exceed the maximum limits
    /// set during initialization.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to update
    /// * `new_limits` - The new resource limits to apply
    async fn update_limits(
        &self,
        tool_id: &str,
        new_limits: ResourceLimits,
    ) -> Result<(), ToolError>;

    /// Gets the current resource usage for a tool
    ///
    /// This method returns the current resource usage statistics for a tool.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to get usage for
    async fn get_usage(&self, tool_id: &str) -> Result<ResourceUsage, ToolError>;

    /// Checks if a tool's resource usage is within limits
    ///
    /// This method checks whether the tool's current resource usage is within
    /// the limits set for it.
    ///
    /// # Arguments
    /// * `tool_id` - The ID of the tool to check
    ///
    /// # Returns
    /// `true` if the tool is within limits, `false` otherwise
    async fn check_limits(&self, tool_id: &str) -> Result<bool, ToolError>;
}
