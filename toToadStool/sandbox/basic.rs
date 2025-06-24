//! Basic Plugin Sandbox Implementation
//!
//! This module provides a basic fallback sandbox implementation that works
//! on all platforms, providing fundamental security and resource monitoring.

use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::{Result, CoreError};
use crate::plugin::security::{SecurityContext, PermissionLevel};
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use super::errors::SandboxError;
use super::traits::PluginSandbox;

/// Basic plugin sandbox implementation for fallback on unsupported platforms
#[derive(Debug)]
pub struct BasicPluginSandbox {
    /// Security contexts for plugins
    security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
}

impl BasicPluginSandbox {
    /// Create a new basic plugin sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Self {
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
        }
    }
    
    /// Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id)
            .cloned()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id).into())
    }
    
    /// Get the resource monitor for this sandbox
    pub fn get_resource_monitor(&self) -> Arc<ResourceMonitor> {
        self.resource_monitor.clone()
    }

    /// Standardize platform-specific errors to consistent error types
    fn standardize_error(&self, error: CoreError, operation: &str, plugin_id: Uuid) -> CoreError {
        match error {
            CoreError::Io(msg) | CoreError::Plugin(msg) | CoreError::Config(msg) | 
            CoreError::Database(msg) | CoreError::Context(msg) | CoreError::Command(msg) |
            CoreError::Monitoring(msg) | CoreError::Serialization(msg) | CoreError::Sync(msg) => {
                // Check message content for specific error types
                let msg_lower = msg.to_lowercase();
                
                if msg_lower.contains("permission denied") || msg_lower.contains("access denied") {
                    CoreError::Security(format!("{}: {} for plugin {}", operation, msg, plugin_id))
                } else if msg_lower.contains("not found") && msg_lower.contains("plugin") {
                    CoreError::Plugin(format!("Plugin not found: {}", plugin_id))
                } else if msg_lower.contains("resource limit") || msg_lower.contains("memory limit") || msg_lower.contains("cpu limit") {
                    CoreError::Security(format!("Resource limit exceeded: {}: {} for plugin {}", operation, msg, plugin_id))
                } else if msg_lower.contains("path") && (msg_lower.contains("denied") || msg_lower.contains("not allowed")) {
                    CoreError::Security(format!("Path access denied: {}: {} for plugin {}", operation, msg, plugin_id))
                } else if msg_lower.contains("unsupported") || msg_lower.contains("not supported") {
                    CoreError::Plugin(format!("Feature not supported: {}: {} for plugin {}", operation, msg, plugin_id))
                } else if msg_lower.contains("platform") {
                    CoreError::Plugin(format!("Platform error: {}: {} for plugin {}", operation, msg, plugin_id))
                } else {
                    // Keep as generic if not matching any specific category
                    CoreError::Plugin(format!("{}: {} for plugin {}", operation, msg, plugin_id))
                }
            },
            CoreError::Security(msg) => {
                // Keep security errors as is, but add context
                CoreError::Security(format!("{}: {} for plugin {}", operation, msg, plugin_id))
            },
        }
    }
}

#[async_trait::async_trait]
impl PluginSandbox for BasicPluginSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        // Basic sandbox just registers the plugin ID
        debug!("Creating basic sandbox for plugin {}", plugin_id);
        
        // Create default security context if none exists
        let contexts = self.security_contexts.read().await;
        if !contexts.contains_key(&plugin_id) {
            drop(contexts);
            let mut contexts = self.security_contexts.write().await;
            contexts.insert(plugin_id, SecurityContext::default());
        }
        
        // Register with resource monitor if possible
        if let Err(e) = self.resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe()?).await {
            warn!("Could not register plugin with resource monitor: {}", e);
        }
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying basic sandbox for plugin {}", plugin_id);
        
        // Remove security context
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        // Unregister from resource monitor
        if let Err(e) = self.resource_monitor.unregister_process(plugin_id).await {
            warn!("Could not unregister plugin from resource monitor: {}", e);
        }
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // Map operations to required capabilities
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
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Use check_capability to verify the capability
        // We'll drop the context lock first to avoid deadlock when check_capability acquires it
        drop(contexts);
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
            None => Err(SandboxError::PluginNotFound(plugin_id).into()),
        }
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level can access any path
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // For write access, we need to be more restrictive
        if write && context.permission_level == PermissionLevel::Restricted {
            return Err(SandboxError::PathAccess(format!(
                "Plugin {} with restricted permission level does not have write access to path {}",
                plugin_id, path.display()
            )).into());
        }
        
        // Check if path is in allowed paths
        // First try to canonicalize the path directly
        // If it doesn't exist, try to canonicalize its parent directory
        let canonical_path = match path.canonicalize() {
            Ok(canonical) => canonical,
            Err(_) => {
                // If the path doesn't exist, check if its parent directory exists and is allowed
                if let Some(parent) = path.parent() {
                    match parent.canonicalize() {
                        Ok(parent_canonical) => {
                            // For non-existent files, check if their parent directory is allowed
                            // If checking a file like "parent/file.txt", we need to check if "parent" is allowed
                            if let Some(file_name) = path.file_name() {
                                // Clone parent_canonical before joining to avoid borrow issues
                                parent_canonical.join(file_name)
                            } else {
                                parent_canonical
                            }
                        }
                        Err(e) => return Err(SandboxError::PathAccess(format!(
                            "Could not canonicalize parent path of {}: {}", path.display(), e
                        )).into())
                    }
                } else {
                    return Err(SandboxError::PathAccess(format!(
                        "Path {} has no parent directory", path.display()
                    )).into());
                }
            }
        };
        
        let is_allowed = context.allowed_paths.iter().any(|allowed_path| {
            // Try to canonicalize the allowed path and check if target path is a subdirectory
            if let Ok(canonical_allowed) = allowed_path.canonicalize() {
                // Check if path starts with allowed_path
                canonical_path.starts_with(&canonical_allowed)
            } else {
                false
            }
        });
        
        if is_allowed {
            Ok(())
        } else {
            Err(SandboxError::PathAccess(format!(
                "Plugin {} does not have access to path {}",
                plugin_id, path.display()
            )).into())
        }
    }
    
    /// Check for capability support
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
        Ok(false)
    }
    
    async fn apply_feature(&self, _plugin_id: Uuid, feature: &str) -> Result<()> {
        // Basic sandbox doesn't support any advanced features
        Err(SandboxError::Unsupported(format!(
            "Feature '{}' is not supported by basic sandbox", feature
        )).into())
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