// Resource Management Module
//
// This module provides functionality for monitoring and limiting
// resource usage by plugins.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory usage
    Memory,
    
    /// CPU usage
    Cpu,
    
    /// Disk usage
    Disk,
    
    /// Network usage
    Network,
    
    /// File handles
    FileHandles,
    
    /// Thread count
    Threads,
}

/// Resource limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    
    /// Maximum CPU usage percentage (0.0 - 1.0)
    pub max_cpu: Option<f64>,
    
    /// Maximum disk usage in bytes
    pub max_disk: Option<u64>,
    
    /// Maximum network usage in bytes per second
    pub max_network: Option<u64>,
    
    /// Maximum file handles
    pub max_file_handles: Option<u32>,
    
    /// Maximum thread count
    pub max_threads: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(100 * 1024 * 1024), // 100 MB
            max_cpu: Some(0.5),                  // 50% CPU
            max_disk: Some(10 * 1024 * 1024),    // 10 MB
            max_network: Some(1024 * 1024),      // 1 MB/s
            max_file_handles: Some(100),         // 100 files
            max_threads: Some(10),               // 10 threads
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory: u64,
    
    /// Current CPU usage percentage (0.0 - 1.0)
    pub cpu: f64,
    
    /// Current disk usage in bytes
    pub disk: u64,
    
    /// Current network usage in bytes per second
    pub network: u64,
    
    /// Current file handles
    pub file_handles: u32,
    
    /// Current thread count
    pub threads: u32,
    
    /// Timestamp when the usage was measured
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory: 0,
            cpu: 0.0,
            disk: 0,
            network: 0,
            file_handles: 0,
            threads: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Resource violation action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationAction {
    /// Log the violation but take no action
    Log,
    
    /// Pause the plugin
    Pause,
    
    /// Stop the plugin
    Stop,
    
    /// Restart the plugin
    Restart,
}

/// Resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceViolation {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Current usage
    pub current_usage: f64,
    
    /// Limit
    pub limit: f64,
    
    /// Violation time
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Action taken
    pub action: ViolationAction,
}

/// Resource monitor interface
#[async_trait]
pub trait ResourceMonitor: Send + Sync + Debug {
    /// Set resource limits for a plugin
    async fn set_limits(&self, plugin_id: Uuid, limits: ResourceLimits) -> Result<()>;
    
    /// Get resource limits for a plugin
    async fn get_limits(&self, plugin_id: Uuid) -> Result<ResourceLimits>;
    
    /// Get current resource usage for a plugin
    async fn get_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
    
    /// Check if a plugin is exceeding any resource limits
    async fn check_limits(&self, plugin_id: Uuid) -> Result<Vec<ResourceViolation>>;
    
    /// Report resource allocation
    async fn report_allocation(&self, plugin_id: Uuid, resource_type: ResourceType, amount: u64) -> Result<()>;
    
    /// Report resource deallocation
    async fn report_deallocation(&self, plugin_id: Uuid, resource_type: ResourceType, amount: u64) -> Result<()>;
    
    /// Start monitoring a plugin
    async fn start_monitoring(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Stop monitoring a plugin
    async fn stop_monitoring(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Get violation history for a plugin
    async fn get_violations(&self, plugin_id: Uuid) -> Result<Vec<ResourceViolation>>;
}

/// Resource monitor implementation
#[derive(Debug)]
pub struct ResourceMonitorImpl {
    /// Plugin resource limits
    limits: RwLock<HashMap<Uuid, ResourceLimits>>,
    
    /// Plugin resource usage
    usage: RwLock<HashMap<Uuid, ResourceUsage>>,
    
    /// Plugin resource violations
    violations: RwLock<HashMap<Uuid, Vec<ResourceViolation>>>,
    
    /// Last CPU measurement time
    last_cpu_time: RwLock<HashMap<Uuid, (Instant, f64)>>,
    
    /// Default action for violations
    default_action: ViolationAction,
    
    /// Monitoring interval
    monitoring_interval: Duration,
    
    /// Whether monitoring is enabled
    enabled: bool,
}

impl ResourceMonitorImpl {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            limits: RwLock::new(HashMap::new()),
            usage: RwLock::new(HashMap::new()),
            violations: RwLock::new(HashMap::new()),
            last_cpu_time: RwLock::new(HashMap::new()),
            default_action: ViolationAction::Log,
            monitoring_interval: Duration::from_secs(1),
            enabled: true,
        }
    }
    
    /// Create a new resource monitor with custom configuration
    pub fn with_config(default_action: ViolationAction, monitoring_interval: Duration, enabled: bool) -> Self {
        Self {
            limits: RwLock::new(HashMap::new()),
            usage: RwLock::new(HashMap::new()),
            violations: RwLock::new(HashMap::new()),
            last_cpu_time: RwLock::new(HashMap::new()),
            default_action,
            monitoring_interval,
            enabled,
        }
    }
    
    /// Measure resource usage for a plugin
    async fn measure_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // For an actual implementation, we would use platform-specific APIs
        // to measure resource usage. For now, we'll return dummy values.
        
        // On Windows, we might use Process APIs
        // On Linux, we might use /proc filesystem
        // On macOS, we might use task_info
        
        let mut usage = ResourceUsage::default();
        
        #[cfg(target_os = "windows")]
        {
            // Windows implementation would go here
            // For example, using Windows Process APIs
            
            // This is a placeholder, actual implementation would use WinAPI
            usage.memory = 10 * 1024 * 1024; // 10 MB
            usage.cpu = 0.1; // 10% CPU
            usage.disk = 1024 * 1024; // 1 MB
            usage.network = 10 * 1024; // 10 KB/s
            usage.file_handles = 5;
            usage.threads = 2;
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux implementation would go here
            // For example, using /proc filesystem
            
            // This is a placeholder, actual implementation would read from /proc
            usage.memory = 10 * 1024 * 1024; // 10 MB
            usage.cpu = 0.1; // 10% CPU
            usage.disk = 1024 * 1024; // 1 MB
            usage.network = 10 * 1024; // 10 KB/s
            usage.file_handles = 5;
            usage.threads = 2;
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS implementation would go here
            // For example, using task_info
            
            // This is a placeholder, actual implementation would use task_info
            usage.memory = 10 * 1024 * 1024; // 10 MB
            usage.cpu = 0.1; // 10% CPU
            usage.disk = 1024 * 1024; // 1 MB
            usage.network = 10 * 1024; // 10 KB/s
            usage.file_handles = 5;
            usage.threads = 2;
        }
        
        usage.timestamp = chrono::Utc::now();
        
        // Store the usage
        {
            let mut usage_map = self.usage.write().await;
            usage_map.insert(plugin_id, usage.clone());
        }
        
        Ok(usage)
    }
    
    /// Record a resource violation
    async fn record_violation(
        &self,
        plugin_id: Uuid,
        resource_type: ResourceType,
        current_usage: f64,
        limit: f64,
        action: ViolationAction,
    ) -> Result<ResourceViolation> {
        let violation = ResourceViolation {
            plugin_id,
            resource_type,
            current_usage,
            limit,
            timestamp: chrono::Utc::now(),
            action,
        };
        
        // Record the violation
        {
            let mut violations_map = self.violations.write().await;
            violations_map
                .entry(plugin_id)
                .or_insert_with(Vec::new)
                .push(violation.clone());
        }
        
        // Log the violation
        warn!(
            "Resource violation: plugin {} exceeded {} limit ({:.2} > {:.2}), action: {:?}",
            plugin_id, resource_type, current_usage, limit, action
        );
        
        Ok(violation)
    }
    
    /// Start a background task to monitor plugin resources
    pub async fn start_background_monitoring(monitor: Arc<Self>) {
        if !monitor.enabled {
            return;
        }
        
        tokio::spawn(async move {
            loop {
                // Sleep first to allow initialization
                tokio::time::sleep(monitor.monitoring_interval).await;
                
                // Get all plugin IDs
                let plugin_ids = {
                    let limits_map = monitor.limits.read().await;
                    limits_map.keys().cloned().collect::<Vec<_>>()
                };
                
                // Check each plugin
                for plugin_id in plugin_ids {
                    if let Err(e) = monitor.check_limits(plugin_id).await {
                        error!("Error checking resource limits for plugin {}: {}", plugin_id, e);
                    }
                }
            }
        });
    }
}

#[async_trait]
impl ResourceMonitor for ResourceMonitorImpl {
    async fn set_limits(&self, plugin_id: Uuid, limits: ResourceLimits) -> Result<()> {
        let mut limits_map = self.limits.write().await;
        limits_map.insert(plugin_id, limits);
        Ok(())
    }
    
    async fn get_limits(&self, plugin_id: Uuid) -> Result<ResourceLimits> {
        let limits_map = self.limits.read().await;
        limits_map
            .get(&plugin_id)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))
    }
    
    async fn get_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // First try to get cached usage
        {
            let usage_map = self.usage.read().await;
            if let Some(usage) = usage_map.get(&plugin_id) {
                return Ok(usage.clone());
            }
        }
        
        // If not found, measure it
        self.measure_usage(plugin_id).await
    }
    
    async fn check_limits(&self, plugin_id: Uuid) -> Result<Vec<ResourceViolation>> {
        let limits = self.get_limits(plugin_id).await?;
        let usage = self.get_usage(plugin_id).await?;
        
        let mut violations = Vec::new();
        
        // Check memory limit
        if let Some(max_memory) = limits.max_memory {
            if usage.memory > max_memory {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::Memory,
                        usage.memory as f64,
                        max_memory as f64,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        // Check CPU limit
        if let Some(max_cpu) = limits.max_cpu {
            if usage.cpu > max_cpu {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::Cpu,
                        usage.cpu,
                        max_cpu,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        // Check disk limit
        if let Some(max_disk) = limits.max_disk {
            if usage.disk > max_disk {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::Disk,
                        usage.disk as f64,
                        max_disk as f64,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        // Check network limit
        if let Some(max_network) = limits.max_network {
            if usage.network > max_network {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::Network,
                        usage.network as f64,
                        max_network as f64,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        // Check file handles limit
        if let Some(max_file_handles) = limits.max_file_handles {
            if usage.file_handles > max_file_handles {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::FileHandles,
                        usage.file_handles as f64,
                        max_file_handles as f64,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        // Check thread limit
        if let Some(max_threads) = limits.max_threads {
            if usage.threads > max_threads {
                let violation = self
                    .record_violation(
                        plugin_id,
                        ResourceType::Threads,
                        usage.threads as f64,
                        max_threads as f64,
                        self.default_action,
                    )
                    .await?;
                violations.push(violation);
            }
        }
        
        Ok(violations)
    }
    
    async fn report_allocation(&self, plugin_id: Uuid, resource_type: ResourceType, amount: u64) -> Result<()> {
        // Update the usage
        let mut usage_map = self.usage.write().await;
        let usage = usage_map.entry(plugin_id).or_insert_with(ResourceUsage::default);
        
        match resource_type {
            ResourceType::Memory => usage.memory += amount,
            ResourceType::Disk => usage.disk += amount,
            ResourceType::Network => usage.network += amount,
            ResourceType::FileHandles => usage.file_handles += amount as u32,
            ResourceType::Threads => usage.threads += amount as u32,
            _ => {
                return Err(PluginError::InvalidOperation(format!(
                    "Cannot report allocation for resource type: {:?}",
                    resource_type
                )))
            }
        }
        
        usage.timestamp = chrono::Utc::now();
        
        Ok(())
    }
    
    async fn report_deallocation(&self, plugin_id: Uuid, resource_type: ResourceType, amount: u64) -> Result<()> {
        // Update the usage
        let mut usage_map = self.usage.write().await;
        let usage = usage_map.entry(plugin_id).or_insert_with(ResourceUsage::default);
        
        match resource_type {
            ResourceType::Memory => {
                usage.memory = usage.memory.saturating_sub(amount);
            }
            ResourceType::Disk => {
                usage.disk = usage.disk.saturating_sub(amount);
            }
            ResourceType::Network => {
                usage.network = usage.network.saturating_sub(amount);
            }
            ResourceType::FileHandles => {
                usage.file_handles = usage.file_handles.saturating_sub(amount as u32);
            }
            ResourceType::Threads => {
                usage.threads = usage.threads.saturating_sub(amount as u32);
            }
            _ => {
                return Err(PluginError::InvalidOperation(format!(
                    "Cannot report deallocation for resource type: {:?}",
                    resource_type
                )))
            }
        }
        
        usage.timestamp = chrono::Utc::now();
        
        Ok(())
    }
    
    async fn start_monitoring(&self, plugin_id: Uuid) -> Result<()> {
        // Ensure we have limits set
        if !self.limits.read().await.contains_key(&plugin_id) {
            let default_limits = ResourceLimits::default();
            self.set_limits(plugin_id, default_limits).await?;
        }
        
        // Initialize usage
        let _ = self.measure_usage(plugin_id).await?;
        
        Ok(())
    }
    
    async fn stop_monitoring(&self, plugin_id: Uuid) -> Result<()> {
        // Clean up resources
        {
            let mut limits_map = self.limits.write().await;
            limits_map.remove(&plugin_id);
        }
        
        {
            let mut usage_map = self.usage.write().await;
            usage_map.remove(&plugin_id);
        }
        
        {
            let mut last_cpu_time_map = self.last_cpu_time.write().await;
            last_cpu_time_map.remove(&plugin_id);
        }
        
        Ok(())
    }
    
    async fn get_violations(&self, plugin_id: Uuid) -> Result<Vec<ResourceViolation>> {
        let violations_map = self.violations.read().await;
        Ok(violations_map.get(&plugin_id).cloned().unwrap_or_default())
    }
}

/// Tests for the resource monitor
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_resource_limits() {
        let monitor = Arc::new(ResourceMonitorImpl::new());
        let plugin_id = Uuid::new_v4();
        
        // Set limits
        let limits = ResourceLimits {
            max_memory: Some(100 * 1024 * 1024), // 100 MB
            max_cpu: Some(0.5),                  // 50% CPU
            max_disk: Some(10 * 1024 * 1024),    // 10 MB
            max_network: Some(1024 * 1024),      // 1 MB/s
            max_file_handles: Some(100),         // 100 files
            max_threads: Some(10),               // 10 threads
        };
        
        assert!(monitor.set_limits(plugin_id, limits.clone()).await.is_ok());
        
        // Get limits
        let retrieved_limits = monitor.get_limits(plugin_id).await.unwrap();
        assert_eq!(retrieved_limits.max_memory, limits.max_memory);
        assert_eq!(retrieved_limits.max_cpu, limits.max_cpu);
        assert_eq!(retrieved_limits.max_disk, limits.max_disk);
        assert_eq!(retrieved_limits.max_network, limits.max_network);
        assert_eq!(retrieved_limits.max_file_handles, limits.max_file_handles);
        assert_eq!(retrieved_limits.max_threads, limits.max_threads);
    }
    
    #[tokio::test]
    async fn test_resource_usage() {
        let monitor = Arc::new(ResourceMonitorImpl::new());
        let plugin_id = Uuid::new_v4();
        
        // Start monitoring
        assert!(monitor.start_monitoring(plugin_id).await.is_ok());
        
        // Get usage
        let usage = monitor.get_usage(plugin_id).await.unwrap();
        assert!(usage.memory > 0);
        
        // Report allocation
        assert!(monitor
            .report_allocation(plugin_id, ResourceType::Memory, 1024 * 1024)
            .await
            .is_ok());
        
        // Get updated usage
        let updated_usage = monitor.get_usage(plugin_id).await.unwrap();
        assert!(updated_usage.memory >= usage.memory + 1024 * 1024);
        
        // Report deallocation
        assert!(monitor
            .report_deallocation(plugin_id, ResourceType::Memory, 1024 * 1024)
            .await
            .is_ok());
        
        // Stop monitoring
        assert!(monitor.stop_monitoring(plugin_id).await.is_ok());
    }
} 