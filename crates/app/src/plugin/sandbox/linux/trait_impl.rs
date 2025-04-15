//! Implementation of the PluginSandbox trait for LinuxCgroupSandbox

use std::any::Any;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::Result;
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use crate::plugin::sandbox::{PluginSandbox, SandboxError};
use crate::plugin::security::SecurityContext;

use super::sandbox::LinuxCgroupSandbox;

#[async_trait::async_trait]
impl PluginSandbox for LinuxCgroupSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating Linux cgroup sandbox for plugin {}", plugin_id);
        
        // Create default security context if needed
        let mut contexts = self.security_contexts.write().await;
        if !contexts.contains_key(&plugin_id) {
            contexts.insert(plugin_id, SecurityContext::default());
        }
        let context = contexts.get(&plugin_id).unwrap().clone();
        drop(contexts);
        
        // Create cgroup
        let cgroup_path = self.create_cgroup(plugin_id).await?;
        
        // Enable controllers - try to enable all available controllers
        // This allows for more comprehensive resource control
        if let Err(e) = self.enable_controllers(&cgroup_path).await {
            warn!("Failed to enable cgroup controllers: {}", e);
            // Continue anyway - some limits may not work
        }
        
        // Set resource limits based on security context
        self.set_cgroup_limits(&cgroup_path, &context.resource_limits).await?;
        
        // Add additional memory safeguards - set memory.swap.max to same as memory.max
        // to prevent excessive swap usage
        if context.resource_limits.max_memory_bytes > 0 {
            let mem_limit = context.resource_limits.max_memory_bytes.to_string();
            if let Err(e) = self.write_cgroup_file(&cgroup_path, "memory.swap.max", mem_limit.as_bytes()).await {
                debug!("Could not set swap limit for plugin {}: {}", plugin_id, e);
                // Continue despite failure - swap controller may not be available
            } else {
                debug!("Set swap limit for plugin {}: {} bytes", plugin_id, mem_limit);
            }
        }
        
        // Register cgroup path
        let mut cgroup_paths = self.cgroup_paths.write().await;
        cgroup_paths.insert(plugin_id, cgroup_path.clone());
        drop(cgroup_paths);
        
        // Get process ID
        let process_id = match self.resource_monitor.get_process_id(plugin_id).await {
            Ok(pid) => pid, 
            Err(e) => {
                // If no process ID is registered or error, use the current process
                warn!("Failed to get process ID for plugin {}, using current process: {}", plugin_id, e);
                let current_pid = std::process::id();
                
                // Register with resource monitor
                let executable_path = std::env::current_exe()?;
                if let Err(e) = self.resource_monitor.register_process(plugin_id, current_pid, &executable_path).await {
                    warn!("Failed to register process with resource monitor: {}", e);
                }
                
                current_pid
            }
        };
        
        // Add process to cgroup
        self.add_process_to_cgroup(&cgroup_path, process_id).await?;
        
        // Apply additional Linux optimizations
        if let Err(e) = self.apply_linux_optimizations(&cgroup_path).await {
            debug!("Could not apply Linux optimizations for plugin {}: {}", plugin_id, e);
            // Continue despite failure - optimizations are optional
        }
        
        // Set up OOM notifications if possible
        let memory_events_path = cgroup_path.join("memory.events");
        if memory_events_path.exists() {
            debug!("Enabled OOM monitoring for plugin {}", plugin_id);
            // In a production implementation, we'd start a thread here to monitor for OOM events
        }
        
        info!("Linux cgroup sandbox created for plugin {} with process {}", plugin_id, process_id);
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying Linux cgroup sandbox for plugin {}", plugin_id);
        
        // Get the process ID
        let maybe_process_id = self.resource_monitor.get_process_id(plugin_id).await;
        
        // Get the cgroup path
        let maybe_cgroup_path = {
            let cgroup_paths = self.cgroup_paths.read().await;
            cgroup_paths.get(&plugin_id).cloned()
        };
        
        // Kill processes in the cgroup
        if let Some(cgroup_path) = &maybe_cgroup_path {
            if let Err(e) = self.kill_cgroup_processes(cgroup_path).await {
                warn!("Failed to kill processes in cgroup for plugin {}: {}", plugin_id, e);
            }
        }
        
        // Remove the cgroup
        if let Some(cgroup_path) = maybe_cgroup_path {
            if let Err(e) = self.remove_cgroup(plugin_id).await {
                warn!("Failed to remove cgroup for plugin {}: {}", plugin_id, e);
            }
        }
        
        // Remove security context
        {
            let mut contexts = self.security_contexts.write().await;
            contexts.remove(&plugin_id);
        }
        
        // Unregister from resource monitor
        if let Err(e) = self.resource_monitor.unregister_process(plugin_id).await {
            warn!("Failed to unregister plugin {} from resource monitor: {}", plugin_id, e);
        }
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // Map operations to capabilities
        let required_capability = match operation {
            // Filesystem operations
            "filesystem:read" => "file:read",
            "filesystem:write" => "file:write",
            "filesystem:delete" => "file:delete",
            "filesystem:execute" => "file:execute",
            
            // Network operations
            "network:connect" => "network:connect",
            "network:listen" => "network:listen",
            
            // Process operations
            "process:spawn" => "system:resources",
            "process:kill" => "system:admin",
            
            // Plugin operations
            "plugin:load" => "plugin:read",
            "plugin:install" => "plugin:install",
            "plugin:uninstall" => "plugin:uninstall",
            "plugin:execute" => "plugin:execute",
            "plugin:update" => "plugin:write",
            
            // Config operations
            "config:read" => "config:read",
            "config:write" => "config:write",
            
            // System operations
            "system:info" => "system:info",
            "system:admin" => "system:admin",
            
            // For any unmapped operation, use the operation directly as capability
            _ => operation,
        };
        
        debug!("Mapped operation '{}' to capability '{}'", operation, required_capability);
        
        // System level has implicit access to all operations
        if context.permission_level == crate::plugin::security::PermissionLevel::System {
            return Ok(());
        }
        
        // Use check_capability to verify the capability
        // We'll drop the context lock first to avoid deadlock when check_capability acquires it
        drop(contexts);
        let has_capability = self.check_capability(plugin_id, required_capability).await?;
        
        if has_capability {
            Ok(())
        } else {
            Err(SandboxError::Permission(format!(
                "Plugin {} lacks capability {} required for operation {}",
                plugin_id, required_capability, operation
            )).into())
        }
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // Check if we have a cgroup path
        let cgroup_paths = self.cgroup_paths.read().await;
        if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
            // Get resource usage from cgroup
            let mut usage = self.get_cgroup_resource_usage(cgroup_path).await?;
            
            // Try to get network usage from resource monitor
            if let Ok(Some(monitor_usage)) = self.resource_monitor.get_resource_usage(plugin_id).await {
                usage.cpu_percent = monitor_usage.cpu_percent; // Use CPU% from monitor
                usage.network_mb = monitor_usage.network_mb;
            }
            
            return Ok(usage);
        }
        
        // Fallback to resource monitor
        match self.resource_monitor.get_resource_usage(plugin_id).await? {
            Some(usage) => Ok(usage),
            None => Err(SandboxError::PluginNotFound(plugin_id).into()),
        }
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level can access any path
        if context.permission_level == crate::plugin::security::PermissionLevel::System {
            debug!("Plugin {} has system permission level, allowing access to {}", 
                   plugin_id, path.display());
            return Ok(());
        }
        
        // For write access, we need to be more restrictive
        if write && context.permission_level == crate::plugin::security::PermissionLevel::Restricted {
            debug!("Plugin {} with restricted permission denied write access to {}", 
                   plugin_id, path.display());
            return Err(SandboxError::PathAccess(format!(
                "Plugin {} with restricted permission level does not have write access to path {}",
                plugin_id, path.display()
            )).into());
        }
        
        // Check if the path is a symlink - symlinks require special handling
        let metadata = std::fs::metadata(path);
        if let Ok(meta) = metadata {
            if meta.file_type().is_symlink() {
                debug!("Path {} is a symlink, performing additional validation", path.display());
                // For symlinks, we should check both the link and its target
                if let Ok(target) = std::fs::read_link(path) {
                    // Also check the target path
                    let target_path = if target.is_absolute() {
                        target
                    } else {
                        // Convert relative symlinks to absolute
                        if let Some(parent) = path.parent() {
                            parent.join(target)
                        } else {
                            target
                        }
                    };
                    
                    debug!("Symlink {} points to {}, checking target", 
                          path.display(), target_path.display());
                    
                    // Recursively check the target path
                    return self.check_path_access(plugin_id, &target_path, write).await;
                }
            }
        }
        
        // Check if path is in allowed paths
        let canonical_path = path.canonicalize().map_err(|e| SandboxError::PathAccess(format!(
            "Could not canonicalize path {}: {}", path.display(), e
        )))?;
        
        // Deny access to sensitive system directories by default
        let sensitive_paths = [
            Path::new("/etc"),
            Path::new("/var/log"),
            Path::new("/var/run"),
            Path::new("/boot"),
            Path::new("/proc"),
            Path::new("/sys"),
            Path::new("/dev"),
        ];
        
        for sensitive in &sensitive_paths {
            if canonical_path.starts_with(sensitive) && context.permission_level != crate::plugin::security::PermissionLevel::System {
                debug!("Plugin {} denied access to sensitive path {}", 
                      plugin_id, canonical_path.display());
                return Err(SandboxError::PathAccess(format!(
                    "Plugin {} with non-admin permission level cannot access system path {}",
                    plugin_id, canonical_path.display()
                )).into());
            }
        }
        
        let is_allowed = context.allowed_paths.iter().any(|allowed_path| {
            // Try to canonicalize the allowed path and check if target path is a subdirectory
            if let Ok(canonical_allowed) = allowed_path.canonicalize() {
                canonical_path.starts_with(&canonical_allowed)
            } else {
                false
            }
        });
        
        // Also check if path is in secure namespace
        let is_secure = self.is_path_in_secure_namespace(&canonical_path);
        
        if is_allowed || is_secure {
            debug!("Path access granted for plugin {} to {}", plugin_id, path.display());
            Ok(())
        } else {
            debug!("Path access denied for plugin {} to {}", plugin_id, path.display());
            Err(SandboxError::PathAccess(format!(
                "Plugin {} does not have access to path {}",
                plugin_id, path.display()
            )).into())
        }
    }
    
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<bool> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level has all capabilities
        if context.permission_level == crate::plugin::security::PermissionLevel::System {
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
        Ok(false)
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        // Handle Linux-specific features
        match feature {
            "cgroups" => {
                // Cgroups are already used in this implementation
                debug!("Cgroups feature is already enabled");
                Ok(())
            }
            "memory_limit" => {
                // Memory limits are already set when creating the sandbox
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
                
                let cgroup_paths = self.cgroup_paths.read().await;
                if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
                    // Re-apply memory limits
                    let memory_bytes = context.resource_limits.max_memory_bytes;
                    self.write_cgroup_file(
                        cgroup_path, 
                        "memory.max", 
                        memory_bytes.to_string().as_bytes()
                    ).await?;
                    
                    debug!("Re-applied memory limits for plugin {}", plugin_id);
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            }
            "cpu_limit" => {
                // CPU limits are already set when creating the sandbox
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
                
                let cgroup_paths = self.cgroup_paths.read().await;
                if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
                    // Re-apply CPU limits
                    let period = 100000;
                    let quota = (period as f64 * (context.resource_limits.max_cpu_percent as f64 / 100.0)) as u64;
                    
                    self.write_cgroup_file(
                        cgroup_path,
                        "cpu.max",
                        format!("{} {}", quota, period).as_bytes()
                    ).await?;
                    
                    debug!("Re-applied CPU limits for plugin {}", plugin_id);
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            }
            "seccomp" => {
                // Apply seccomp filtering
                self.apply_seccomp_filter(plugin_id).await?;
                
                debug!("Applied seccomp filtering for plugin {}", plugin_id);
                Ok(())
            }
            "seccomp_export" => {
                // Export seccomp filter to a BPF file
                let plugin_id_str = plugin_id.to_string();
                let output_path = std::env::temp_dir().join(format!("seccomp_{}.bpf", plugin_id_str));
                
                self.generate_seccomp_bpf(plugin_id, &output_path).await?;
                
                debug!("Exported seccomp BPF filter to {}", output_path.display());
                Ok(())
            }
            _ => {
                Err(SandboxError::Unsupported(format!(
                    "Feature '{}' is not supported by Linux sandbox", feature
                )).into())
            }
        }
    }
    
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        Some(self.resource_monitor.clone())
    }
} 