#![cfg(target_os = "windows")]

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, error, info};
use uuid::Uuid;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, FALSE, GetLastError};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject,
    CreateJobObjectW,
    JOBOBJECT_BASIC_LIMIT_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JobObjectBasicLimitInformation,
    JobObjectExtendedLimitInformation,
    JobObjectCpuRateControlInformation,
    SetInformationJobObject,
    TerminateJobObject,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_AFFINITY,
    JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    JOB_OBJECT_LIMIT_WORKINGSET,
    JOB_OBJECT_LIMIT_PRIORITY_CLASS,
    JOB_OBJECT_LIMIT_SCHEDULING_CLASS,
    JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT,
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
    JOB_OBJECT_CPU_RATE_CONTROL,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, IDLE_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, ABOVE_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS};
use windows::core::{PCWSTR, PWSTR};
use std::ffi::OsString;
use std::os::windows::prelude::*;
use winapi::um::winnt::OSVERSIONINFOW;
use winapi::shared::winerror::S_OK;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;

use crate::error::{Result, CoreError};
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage, ResourceLimits};
use crate::plugin::sandbox::{SandboxError, PluginSandbox};
use crate::plugin::security::{SecurityContext, PermissionLevel};

/// Windows Sandbox implementation using Job Objects
pub struct WindowsSandbox {
    /// Store security contexts for each plugin
    security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Store process IDs for each plugin
    process_ids: Arc<RwLock<HashMap<Uuid, u32>>>,
    /// Store job handles for each plugin
    job_handles: Arc<RwLock<HashMap<Uuid, HANDLE>>>,
    /// Resource monitor for tracking plugin resource usage
    resource_monitor: Arc<ResourceMonitor>,
}

impl fmt::Debug for WindowsSandbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowsSandbox")
            .field("security_contexts", &"RwLock<HashMap>")
            .field("process_ids", &"RwLock<HashMap>")
            .field("job_handles", &"RwLock<HashMap>")
            .field("resource_monitor", &"Arc<ResourceMonitor>")
            .finish()
    }
}

impl WindowsSandbox {
    /// Create a new Windows sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        Ok(WindowsSandbox {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            process_ids: Arc::new(RwLock::new(HashMap::new())),
            job_handles: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
        })
    }

    /// Set a security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Update the stored security context
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        
        // Apply the new limits if a job already exists
        let job_handle_opt = {
            let job_handles = self.job_handles.read().await;
            job_handles.get(&plugin_id).map(|h| *h)
        };
        
        if let Some(job_handle) = job_handle_opt {
            // Apply resource limits based on the security context
            self.apply_resource_limits(plugin_id, job_handle).await?;
        }
        
        Ok(())
    }
    
    /// Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id)
            .cloned()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id).into())
    }

    /// Check if a path is allowed by security context
    pub async fn is_path_allowed(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<bool> {
        let context = self.get_security_context(plugin_id).await?;
        
        // System level has access to all paths
        if context.permission_level == PermissionLevel::System {
            return Ok(true);
        }
        
        // For write access, check allowed_paths if writing
        if write {
            for allowed_path in &context.allowed_paths {
                if path.starts_with(allowed_path) {
                    return Ok(true);
                }
            }
            // If no write path matches, writing is not allowed
            return Ok(false);
        }
        
        // For read access, check allowed_paths
        for allowed_path in &context.allowed_paths {
            if path.starts_with(allowed_path) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Create a Windows Job Object for process isolation
    async fn create_job_object(&self, plugin_id: Uuid) -> Result<HANDLE> {
        // Create a unique job name based on the plugin ID
        let job_name = format!("SquirrelPlugin_{}", plugin_id);
        let wide_name: Vec<u16> = job_name.encode_utf16().chain(std::iter::once(0)).collect();
        
        // Create the job object
        let job_handle = unsafe {
            match CreateJobObjectW(None, PCWSTR(wide_name.as_ptr())) {
                Ok(handle) => handle,
                Err(e) => {
                    return Err(SandboxError::Creation(format!(
                        "Failed to create Windows Job Object for plugin {}: {}",
                        plugin_id, e
                    )).into());
                }
            }
        };
        
        // Store the job handle
        let mut job_handles = self.job_handles.write().await;
        job_handles.insert(plugin_id, job_handle);
        
        debug!("Created Windows Job Object for plugin {}", plugin_id);
        
        Ok(job_handle)
    }
    
    /// Apply resource limits to a Windows Job Object
    async fn apply_resource_limits(&self, plugin_id: Uuid, job_handle: HANDLE) -> Result<()> {
        // Get the security context to determine resource limits
        let context = self.get_security_context(plugin_id).await?;
        let resource_limits = &context.resource_limits;
        
        // Use max_memory_bytes directly
        let memory_limit_bytes = resource_limits.max_memory_bytes;
        
        // Set basic limits information
        let mut basic_limits = JOBOBJECT_BASIC_LIMIT_INFORMATION::default();
        
        // Set appropriate limit flags based on the permission level
        let mut limit_flags = JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        
        // Add more restrictive flags for User and Restricted permission levels
        if context.permission_level != PermissionLevel::System {
            limit_flags = limit_flags | JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
            
            // Add even more restrictions for Restricted level
            if context.permission_level == PermissionLevel::Restricted {
                limit_flags = limit_flags | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PRIORITY_CLASS;
                
                // Set lower priority class for restricted plugins
                basic_limits.PriorityClass = BELOW_NORMAL_PRIORITY_CLASS.0;
            } else {
                // Set normal priority for User level
                basic_limits.PriorityClass = NORMAL_PRIORITY_CLASS.0;
            }
        } else {
            // System level gets higher priority
            basic_limits.PriorityClass = ABOVE_NORMAL_PRIORITY_CLASS.0;
        }
        
        // Set the limit flags
        basic_limits.LimitFlags = limit_flags;
        
        // Set the active process limit based on permission level
        match context.permission_level {
            PermissionLevel::System => basic_limits.ActiveProcessLimit = 0, // No limit
            PermissionLevel::User => basic_limits.ActiveProcessLimit = 5,   // Moderate limit
            PermissionLevel::Restricted => basic_limits.ActiveProcessLimit = 2, // Very restrictive
        }
        
        // Set extended limits information 
        let mut extended_limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        extended_limits.BasicLimitInformation = basic_limits;
        
        // Set job and process memory limits - using usize for compatibility
        let memory_bytes_usize = if memory_limit_bytes > usize::MAX as u64 {
            usize::MAX
        } else {
            memory_limit_bytes as usize 
        };
        
        extended_limits.ProcessMemoryLimit = memory_bytes_usize;
        extended_limits.JobMemoryLimit = memory_bytes_usize;
        
        // Apply the extended limits
        let result = unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                &extended_limits as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        
        if let Err(e) = result {
            return Err(SandboxError::Creation(format!(
                "Failed to set job memory limits for plugin {}: {}",
                plugin_id, e
            )).into());
        }
        
        debug!("Setting memory limit to {} MB for plugin {}", resource_limits.max_memory_bytes / (1024 * 1024), plugin_id);
        
        // Set CPU rate control information
        let mut cpu_rate_info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION::default();
        
        // The CPU rate is 100 times the percentage (e.g., 80% = 8000)
        let cpu_rate = (resource_limits.max_cpu_percent as u32) * 100;
        
        // Set control flags - enable CPU rate control with hard cap
        let mut cpu_control_flags = JOB_OBJECT_CPU_RATE_CONTROL_ENABLE;
        
        // For restricted permission level, enforce a hard cap
        if context.permission_level == PermissionLevel::Restricted {
            cpu_control_flags = cpu_control_flags | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP;
        }
        
        cpu_rate_info.ControlFlags = cpu_control_flags;
        cpu_rate_info.Anonymous.CpuRate = cpu_rate;
        
        let result = unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectCpuRateControlInformation,
                &cpu_rate_info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            )
        };
        
        if let Err(e) = result {
            return Err(SandboxError::Creation(format!(
                "Failed to set job CPU limits for plugin {}: {}",
                plugin_id, e
            )).into());
        }
        
        debug!("Applied CPU limits for plugin {}: {}% with control flags: {:?}", 
            plugin_id, resource_limits.max_cpu_percent, cpu_control_flags);
        
        Ok(())
    }
    
    /// Assign a process to a job object
    async fn assign_process_to_job(&self, plugin_id: Uuid, process_id: u32) -> Result<()> {
        // Get the job handle for the plugin
        let job_handles = self.job_handles.read().await;
        let job_handle = match job_handles.get(&plugin_id) {
            Some(handle) => *handle,
            None => {
                return Err(SandboxError::PluginNotFound(plugin_id).into());
            }
        };
        
        // Open the process with appropriate permissions
        let process_handle = unsafe {
            match OpenProcess(PROCESS_ALL_ACCESS, FALSE, process_id) {
                Ok(handle) => handle,
                Err(e) => {
                    return Err(SandboxError::Creation(format!(
                        "Failed to open process {} for plugin {}: {}",
                        process_id, plugin_id, e
                    )).into());
                }
            }
        };
        
        // Assign the process to the job object
        let result = unsafe {
            let result = AssignProcessToJobObject(job_handle, process_handle);
            
            // Clean up temporary handle even if operation succeeded
            let _ = CloseHandle(process_handle);
            
            result
        };
        
        if let Err(e) = result {
            return Err(SandboxError::Creation(format!(
                "Failed to assign process {} to job object for plugin {}: {}",
                process_id, plugin_id, e
            )).into());
        }
        
        // Store the process ID for future reference
        let mut process_ids = self.process_ids.write().await;
        process_ids.insert(plugin_id, process_id);
        
        debug!("Assigned process {} to job object for plugin {}", process_id, plugin_id);
        
        // Register the process with the resource monitor
        let exe_path = std::env::current_exe()?;
        if let Err(e) = self.resource_monitor.register_process(plugin_id, process_id, &exe_path).await {
            warn!("Failed to register process with resource monitor: {}", e);
        }
        
        Ok(())
    }
    
    /// Terminate a job and all its processes
    async fn terminate_job(&self, plugin_id: Uuid) -> Result<()> {
        // Get the job handle for the plugin
        let job_handles = self.job_handles.read().await;
        let job_handle = match job_handles.get(&plugin_id) {
            Some(handle) => *handle,
            None => {
                return Err(SandboxError::PluginNotFound(plugin_id).into());
            }
        };
        
        // Keep the handle value but drop the lock so we don't hold it during the termination
        let job_handle = job_handle;
        drop(job_handles);
        
        // Terminate the job
        let result = unsafe {
            TerminateJobObject(job_handle, 0)
        };
        
        if let Err(e) = result {
            return Err(SandboxError::Destruction(format!(
                "Failed to terminate job object for plugin {}: {}",
                plugin_id, e
            )).into());
        }
        
        // Clean up job handle
        let _ = unsafe { CloseHandle(job_handle) };
        
        // Remove the job handle from the map
        let mut job_handles = self.job_handles.write().await;
        job_handles.remove(&plugin_id);
        
        // Remove the process ID from the map
        let mut process_ids = self.process_ids.write().await;
        process_ids.remove(&plugin_id);
        
        // Unregister the process from the resource monitor
        if let Err(e) = self.resource_monitor.unregister_process(plugin_id).await {
            warn!("Failed to unregister process from resource monitor: {}", e);
        }
        
        debug!("Terminated job object for plugin {}", plugin_id);
        
        Ok(())
    }

    /// Check if a plugin has access to a path
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        // Get security context to check allowed paths
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| CoreError::Plugin(format!("Plugin not found: {}", plugin_id)))?;
        
        // System level has access to all paths
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Check if the path is within any of the allowed paths
        for allowed_path in &context.allowed_paths {
            if path_is_within(path, allowed_path) {
                // For write operations, need to check write permission
                if write && context.permission_level == PermissionLevel::Restricted {
                    // Restricted permission level cannot write
                    return Err(SandboxError::PathAccess(format!(
                        "Plugin {} with restricted permission level cannot write to path: {}",
                        plugin_id, path.display()
                    )).into());
                }
                return Ok(());
            }
        }
        
        // Path is not allowed
        let message = if write {
            format!("Plugin {} does not have write access to path: {}", plugin_id, path.display())
        } else {
            format!("Plugin {} does not have read access to path: {}", plugin_id, path.display())
        };
        
        Err(SandboxError::PathAccess(message).into())
    }

    /// Check if Windows sandbox has priority control capability
    fn has_priority_control(&self) -> bool {
        // Job objects can set priority, so this is always available
        true
    }
    
    /// Check if Windows sandbox has app container capability
    fn has_app_container(&self) -> bool {
        // Check if app containers are available on this Windows version
        // App containers were introduced in Windows 8+
        true
    }
    
    /// Check if Windows sandbox has integrity levels capability  
    fn has_integrity_levels(&self) -> bool {
        // Integrity levels are available on Windows Vista and later
        true
    }
}

#[async_trait::async_trait]
impl PluginSandbox for WindowsSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating Windows sandbox for plugin {}", plugin_id);
        
        // Create default security context if none exists
        let contexts = self.security_contexts.read().await;
        let has_context = contexts.contains_key(&plugin_id);
        drop(contexts);
        
        if !has_context {
            let mut contexts = self.security_contexts.write().await;
            contexts.insert(plugin_id, SecurityContext::default());
        }
        
        // Create job object for process isolation
        let job_handle = self.create_job_object(plugin_id).await?;
        
        // Apply resource limits to the job
        self.apply_resource_limits(plugin_id, job_handle).await?;
        
        // Get the process ID for the plugin
        let process_id = match self.resource_monitor.get_process_id(plugin_id).await {
            Ok(pid) => pid,
            Err(_) => {
                return Err(SandboxError::PluginNotFound(plugin_id).into());
            }
        };
        
        // Assign the process to the job
        self.assign_process_to_job(plugin_id, process_id).await?;
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying Windows sandbox for plugin {}", plugin_id);
        
        // Terminate the job object and all processes
        match self.terminate_job(plugin_id).await {
            Ok(()) => {},
            Err(e) => {
                error!("Error terminating job for plugin {}: {}", plugin_id, e);
                // Continue cleanup despite errors
            }
        }
        
        // Remove security context
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        debug!("Windows sandbox for plugin {} destroyed", plugin_id);
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        // Get security context to check permissions
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| CoreError::Plugin(format!("Plugin not found: {}", plugin_id)))?;
        
        // Determine minimum permission level and capability needed for the operation
        let (min_permission_level, required_capability) = match operation {
            "filesystem:read" => (PermissionLevel::Restricted, "file:read"),
            "filesystem:write" => (PermissionLevel::User, "file:write"),
            "plugin:list" => (PermissionLevel::Restricted, "plugin:read"),
            "plugin:install" => (PermissionLevel::User, "plugin:install"),
            "plugin:uninstall" => (PermissionLevel::User, "plugin:uninstall"),
            "network:connect" => (PermissionLevel::User, "network:connect"),
            "process:create" => (PermissionLevel::User, "process:create"),
            "system:admin" => (PermissionLevel::System, "system:admin"),
            // Default to requiring the operation itself as capability
            _ => (PermissionLevel::User, operation),
        };
        
        // Check permission level first
        if context.permission_level < min_permission_level {
            return Err(SandboxError::Permission(format!(
                "Plugin {} with permission level {:?} lacks required permission level {:?} for operation {}",
                plugin_id, context.permission_level, min_permission_level, operation
            )).into());
        }
        
        // System level bypasses capability check
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Check capability for user and restricted levels
        let result = self.check_capability(plugin_id, required_capability).await?;
        if result {
            Ok(())
        } else {
            Err(SandboxError::Permission(format!(
                "Plugin {} lacks capability {} required for operation {}",
                plugin_id, required_capability, operation
            )).into())
        }
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // Use resource monitor to track resources
        match self.resource_monitor.get_resource_usage(plugin_id).await? {
            Some(usage) => Ok(usage),
            None => {
                // Try to measure resources directly
                match self.resource_monitor.get_process_id(plugin_id).await {
                    Ok(pid) => {
                        // Measure resources and update the resource monitor
                        let usage = ResourceUsage::default(); // Replace with actual measurement
                        // In a real implementation, would measure resources for the PID here
                        Ok(usage)
                    }
                    Err(_) => Err(SandboxError::PluginNotFound(plugin_id).into()),
                }
            }
        }
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        self.check_path_access(plugin_id, path, write).await
    }
    
    /// Check if plugin has specific capability
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<bool> {
        // Get security context to check capabilities
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| CoreError::Plugin(format!("Plugin not found: {}", plugin_id)))?;
        
        // System level has all capabilities
        if context.permission_level == PermissionLevel::System {
            return Ok(true);
        }
        
        // Check for exact match
        if context.allowed_capabilities.contains(capability) {
            return Ok(true);
        }
        
        // Check for wildcard namespace match
        if let Some(namespace) = capability.split(':').next() {
            let wildcard = format!("{}:*", namespace);
            if context.allowed_capabilities.contains(&wildcard) {
                return Ok(true);
            }
        }
        
        // Capability not allowed
        if capability.starts_with("windows:") {
            // Check Windows-specific capabilities
            match capability {
                "windows:process_priority" => Ok(self.has_priority_control()),
                "windows:job_objects" => Ok(true), // Windows sandbox always has job objects
                "windows:app_container" => Ok(self.has_app_container()),
                "windows:integrity_levels" => Ok(self.has_integrity_levels()),
                _ => Ok(false)
            }
        } else {
            // Generic capability not found
            Ok(false)
        }
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        // Check if feature is supported on Windows
        match feature {
            "process:isolation" => {
                // Already implemented via Job Objects
                debug!("Plugin {} has process isolation via Windows Job Objects", plugin_id);
                Ok(())
            }
            "memory:limit" => {
                // Already implemented via Job Objects
                debug!("Plugin {} has memory limiting via Windows Job Objects", plugin_id);
                Ok(())
            }
            "cpu:limit" => {
                // Already implemented via Job Objects
                debug!("Plugin {} has CPU limiting via Windows Job Objects", plugin_id);
                Ok(())
            }
            _ => {
                Err(SandboxError::Unsupported(format!(
                    "Feature {} is not supported on Windows for plugin {}",
                    feature, plugin_id
                )).into())
            }
        }
    }
    
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        self.set_security_context(plugin_id, context).await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        Some(self.resource_monitor.clone())
    }
    
    fn is_sandbox_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::security::{SecurityContext, PermissionLevel};
    use std::env;
    
    // Test helper function to create a WindowsSandbox for testing
    async fn create_test_sandbox() -> (WindowsSandbox, Arc<ResourceMonitor>) {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = WindowsSandbox::new(resource_monitor.clone()).unwrap();
        (sandbox, resource_monitor)
    }
    
    #[tokio::test]
    async fn test_windows_sandbox_creation() {
        let (sandbox, resource_monitor) = create_test_sandbox().await;
        let plugin_id = Uuid::new_v4();
        
        // First register the process ID with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Verify security context exists
        let context = sandbox.get_security_context(plugin_id).await.unwrap();
        assert_eq!(context.permission_level, PermissionLevel::Restricted); // Default
        
        // Cleanup
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_windows_sandbox_security_context() {
        let (sandbox, resource_monitor) = create_test_sandbox().await;
        let plugin_id = Uuid::new_v4();
        
        // First register the process ID with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Create custom security context
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::Restricted;
        context.allowed_capabilities = vec!["test:capability".to_string()].into_iter().collect();
        
        // Set context and create sandbox
        sandbox.set_security_context(plugin_id, context.clone()).await.unwrap();
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Verify context was saved
        let saved_context = sandbox.get_security_context(plugin_id).await.unwrap();
        assert_eq!(saved_context.permission_level, PermissionLevel::Restricted);
        assert!(saved_context.allowed_capabilities.contains("test:capability"));
        
        // Cleanup
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
}

/// Check if integrity levels are available
#[cfg(target_os = "windows")]
pub fn has_integrity_levels() -> bool {
    // Integrity levels are available on Windows Vista and later
    // Since we only support Windows 10+, this should always be true
    true
}

/// Check if desktop isolation is available
#[cfg(target_os = "windows")]
pub fn has_desktop_isolation() -> bool {
    // This is available on Windows 10 and later, use a simplified check
    true // Simply assume Windows 10+ for now, which is our target anyway
}

/// Check if firewall integration is available
#[cfg(target_os = "windows")]
pub fn has_firewall_integration() -> bool {
    // Available on all supported Windows versions
    true
}

/// Check if application container isolation is available
#[cfg(target_os = "windows")]
pub fn has_app_container() -> bool {
    // Available on Windows 8 and later
    true
}

// For non-Windows platforms, provide stub implementations
#[cfg(not(target_os = "windows"))]
pub fn has_integrity_levels() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn has_desktop_isolation() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn has_firewall_integration() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn has_app_container() -> bool {
    false
}

/// Check if a path is within another path
fn path_is_within(path: &Path, base_path: &Path) -> bool {
    // Try to get canonical paths if possible
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        // If path doesn't exist, we'll still try with the provided path
        Err(_) => path.to_path_buf(),
    };
    
    let canonical_base = match base_path.canonicalize() {
        Ok(p) => p,
        // If base path doesn't exist, we can't determine containment
        Err(_) => return false,
    };
    
    // Check if path starts with the base path
    canonical_path.starts_with(&canonical_base)
} 