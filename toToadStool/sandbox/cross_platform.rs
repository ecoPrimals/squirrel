//! Cross-Platform Plugin Sandbox Implementation
//!
//! This module provides a cross-platform sandbox that chooses the appropriate
//! platform-specific implementation or falls back to the basic sandbox.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{Result, CoreError};
use crate::plugin::security::SecurityContext;
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use super::errors::SandboxError;
use super::traits::PluginSandbox;
use super::basic::BasicPluginSandbox;

/// Cross-platform plugin sandbox implementation that chooses appropriate sandbox for the current platform
#[derive(Debug)]
pub struct CrossPlatformSandbox {
    /// Platform-specific implementation
    platform_impl: Box<dyn PluginSandbox>,
    /// Security contexts
    security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    /// Platform name for logging and capability detection
    platform_name: String,
    /// Whether platform-specific implementation was successfully loaded
    has_native_sandbox: bool,
}

impl CrossPlatformSandbox {
    /// Create a new cross-platform sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        let platform_name = ResourceMonitor::get_platform_name();
        let mut has_native_sandbox = true;
        
        // Create platform-specific implementation
        let platform_impl: Box<dyn PluginSandbox> = match platform_name {
            "windows" => {
                #[cfg(target_os = "windows")]
                {
                    match super::windows::WindowsSandbox::new(resource_monitor.clone()) {
                        Ok(sandbox) => Box::new(sandbox),
                        Err(e) => {
                            warn!("Failed to create Windows sandbox: {}", e);
                            has_native_sandbox = false;
                            Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    warn!("Windows sandbox unavailable on this build target, using basic sandbox instead");
                    has_native_sandbox = false;
                    Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                }
            }
            "linux" => {
                #[cfg(target_family = "unix")]
                {
                    match super::linux::LinuxCgroupSandbox::new(resource_monitor.clone()) {
                        Ok(sandbox) => Box::new(sandbox),
                        Err(e) => {
                            warn!("Failed to create Linux sandbox: {}", e);
                            has_native_sandbox = false;
                            Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                        }
                    }
                }
                #[cfg(not(target_family = "unix"))]
                {
                    warn!("Linux sandbox unavailable on this build target, using basic sandbox instead");
                    has_native_sandbox = false;
                    Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                }
            }
            "macos" => {
                #[cfg(target_os = "macos")]
                {
                    match super::macos::MacOsSandbox::new(resource_monitor.clone()) {
                        Ok(sandbox) => Box::new(sandbox),
                        Err(e) => {
                            warn!("Failed to create macOS sandbox: {}", e);
                            has_native_sandbox = false;
                            Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    warn!("macOS sandbox unavailable on this build target, using basic sandbox instead");
                    has_native_sandbox = false;
                    Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
                }
            }
            _ => {
                warn!("No native sandbox available for platform {}, using basic sandbox", platform_name);
                has_native_sandbox = false;
                Box::new(BasicPluginSandbox::new(resource_monitor.clone()))
            }
        };
        
        // Create security contexts map
        let security_contexts = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            platform_impl,
            security_contexts,
            resource_monitor,
            platform_name: platform_name.to_owned(),
            has_native_sandbox,
        })
    }
    
    /// Get the platform capabilities
    pub fn get_platform_capabilities(&self) -> HashSet<String> {
        let mut capabilities = HashSet::new();
        
        // Add common capabilities
        capabilities.insert("basic_isolation".to_string());
        capabilities.insert("resource_monitoring".to_string());
        capabilities.insert("path_validation".to_string());
        capabilities.insert("plugin_lifecycle".to_string());
        
        if cfg!(target_os = "windows") {
            // Windows-specific capabilities
            capabilities.insert("windows_job_objects".to_string());
            capabilities.insert("process_priority_control".to_string());
            capabilities.insert("memory_limits".to_string());
            capabilities.insert("cpu_limits".to_string());
            capabilities.insert("process_limits".to_string());
        } else if cfg!(target_os = "linux") {
            // Linux-specific capabilities
            capabilities.insert("cgroups".to_string());
            capabilities.insert("process_limits".to_string());
            capabilities.insert("memory_limits".to_string());
            capabilities.insert("cpu_limits".to_string());
            
            #[cfg(target_os = "linux")]
            {
                capabilities.insert("cgroups_v2".to_string());
                capabilities.insert("seccomp".to_string());
                capabilities.insert("namespaces".to_string());
            }
        } else if cfg!(target_os = "macos") {
            // macOS-specific capabilities
            capabilities.insert("resource_limits".to_string());
            capabilities.insert("memory_limits".to_string());
            capabilities.insert("cpu_limits".to_string());
            
            #[cfg(target_os = "macos")]
            {
                capabilities.insert("app_sandbox".to_string());
                capabilities.insert("system_integrity_protection".to_string());
            }
        }
        
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        {
            capabilities.insert("native_sandbox".to_string());
        }
        
        capabilities
    }
    
    /// Get information about the active platform implementation
    pub fn get_platform_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("platform".to_string(), self.platform_name.clone());
        info.insert("has_native_sandbox".to_string(), self.has_native_sandbox.to_string());
        
        // Add implementation-specific details using reflection
        let type_name = std::any::type_name_of_val(&*self.platform_impl);
        info.insert("implementation".to_string(), type_name.to_string());
        
        info
    }
    
    /// Set security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Update the stored security context
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context.clone());
        drop(contexts); // Drop the write lock before calling into platform_impl
        
        // Update the platform implementation's security context
        self.platform_impl.set_security_context(plugin_id, context).await
    }
    
    /// Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        // Try our internal registry first
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&plugin_id) {
            return Ok(context.clone());
        }
        
        // If not found, check with platform implementation
        drop(contexts);
        match self.platform_impl.as_any().downcast_ref::<BasicPluginSandbox>() {
            Some(basic_sandbox) => basic_sandbox.get_security_context(plugin_id).await,
            None => Err(SandboxError::PluginNotFound(plugin_id).into()),
        }
    }
    
    /// Apply a feature with graceful degradation when not supported
    pub async fn apply_feature_with_degradation(&self, plugin_id: Uuid, feature: &str) -> Result<bool> {
        debug!("Applying feature with degradation support: {} for plugin {}", feature, plugin_id);
        
        // Try using platform-specific implementation first
        let result = self.platform_impl.apply_feature(plugin_id, feature).await;
        
        match result {
            Ok(_) => {
                debug!("Successfully applied feature {} using native implementation", feature);
                Ok(true)
            },
            Err(err) => {
                // Check if the error message indicates the feature is not supported
                let err_str = err.to_string();
                if err_str.contains("not supported") || err_str.contains("Feature not supported") {
                    warn!("Feature {} not natively supported, trying fallback implementation: {}", feature, err_str);
                    
                    // Try fallback implementation
                    match self.apply_feature_fallback(plugin_id, feature).await {
                        Ok(_) => {
                            info!("Applied feature {} with fallback implementation", feature);
                            return Ok(false); // Feature applied with fallback
                        },
                        Err(fallback_err) => {
                            warn!("Fallback implementation also failed: {}", fallback_err);
                            return Err(fallback_err);
                        }
                    }
                }
                
                // For all other errors, propagate the original error
                Err(err)
            }
        }
    }
    
    /// Apply a fallback implementation for a feature
    async fn apply_feature_fallback(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        match feature {
            "job_objects" | "cgroups" | "sandbox_profiles" => {
                // These all relate to process isolation, use resource monitoring as fallback
                self.apply_resource_limits_fallback(plugin_id).await
            }
            "resource_limits" => {
                // Apply resource monitoring
                self.apply_resource_limits_fallback(plugin_id).await
            }
            "seccomp" | "entitlements" => {
                // These relate to syscall filtering, use path restrictions as fallback
                self.apply_basic_isolation_fallback(plugin_id).await
            }
            _ => {
                warn!("No fallback available for feature '{}'", feature);
                Err(SandboxError::Unsupported(format!(
                    "Feature '{}' is not supported on this platform and no fallback is available",
                    feature
                )).into())
            }
        }
    }
    
    /// Apply resource limits fallback
    async fn apply_resource_limits_fallback(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Using resource limits fallback for plugin {}", plugin_id);
        
        // Fallback just monitors resources and logs warnings
        let monitor_clone = self.resource_monitor.clone();
        let contexts_clone = self.security_contexts.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Check if plugin still exists
                let contexts = contexts_clone.read().await;
                if !contexts.contains_key(&plugin_id) {
                    break;
                }
                drop(contexts);
                
                // Get resource usage and check limits
                if let Ok(Some(usage)) = monitor_clone.get_resource_usage(plugin_id).await {
                    let contexts = contexts_clone.read().await;
                    if let Some(context) = contexts.get(&plugin_id) {
                        let limits = &context.resource_limits;
                        
                        // Check if any limits are exceeded
                        if (usage.memory_bytes as f64) > (limits.max_memory_bytes as f64) {
                            warn!(
                                "Plugin {} exceeding memory limit: {:.2} MB > {:.2} MB",
                                plugin_id,
                                (usage.memory_bytes as f64) / (1024.0 * 1024.0),
                                (limits.max_memory_bytes as f64) / (1024.0 * 1024.0)
                            );
                        }
                        
                        if usage.cpu_percent as u8 > limits.max_cpu_percent {
                            warn!(
                                "Plugin {} exceeding CPU limit: {:.2}% > {}%",
                                plugin_id,
                                usage.cpu_percent,
                                limits.max_cpu_percent
                            );
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Apply basic isolation fallback
    async fn apply_basic_isolation_fallback(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Using basic isolation fallback for plugin {}", plugin_id);
        warn!("Plugin {} requested isolation, but only path restrictions are available", plugin_id);
        Ok(())
    }

    /// Standardize platform-specific errors to consistent error types
    fn standardize_error(&self, error: CoreError, operation: &str, plugin_id: Uuid) -> CoreError {
        match error {
            CoreError::Io(msg) | CoreError::Plugin(msg) | CoreError::Config(msg) | 
            CoreError::Database(msg) | CoreError::Context(msg) | CoreError::Command(msg) |
            CoreError::Monitoring(msg) | CoreError::Serialization(msg) | CoreError::Sync(msg) => {
                let msg_lower = msg.to_lowercase();
                
                if msg_lower.contains("permission denied") || msg_lower.contains("access denied") {
                    CoreError::Security(format!("{}: {} for plugin {}", operation, msg, plugin_id))
                } else if msg_lower.contains("not found") && msg_lower.contains("plugin") {
                    CoreError::Plugin(format!("Plugin not found: {}", plugin_id))
                } else {
                    CoreError::Plugin(format!("{}: {} for plugin {}", operation, msg, plugin_id))
                }
            },
            CoreError::Security(msg) => {
                CoreError::Security(format!("{}: {} for plugin {}", operation, msg, plugin_id))
            },
        }
    }

    /// Helper method to add error context to results
    fn add_context<T, S: AsRef<str>>(&self, result: Result<T>, context: S) -> Result<T> {
        result.map_err(|err| {
            let msg = format!("{}: {}", context.as_ref(), err);
            CoreError::Context(msg)
        })
    }
}

#[async_trait::async_trait]
impl PluginSandbox for CrossPlatformSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating {} sandbox for plugin {}", self.platform_name, plugin_id);
        
        // First update our security contexts
        let contexts = self.security_contexts.read().await;
        if !contexts.contains_key(&plugin_id) {
            drop(contexts);
            let mut contexts = self.security_contexts.write().await;
            contexts.insert(plugin_id, SecurityContext::default());
        }
        
        // Then delegate to platform implementation
        match self.platform_impl.create_sandbox(plugin_id).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if self.has_native_sandbox {
                    error!("Failed to create sandbox for plugin {}: {}", plugin_id, e);
                    Err(e)
                } else {
                    warn!("Basic sandbox initialization issue for plugin {}: {}", plugin_id, e);
                    Ok(())
                }
            }
        }
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying {} sandbox for plugin {}", self.platform_name, plugin_id);
        
        let result = self.platform_impl.destroy_sandbox(plugin_id).await;
        
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        if let Err(e) = &result {
            warn!("Error during sandbox destruction for plugin {}: {}", plugin_id, e);
            
            if self.has_native_sandbox {
                match e {
                    CoreError::Plugin(msg) if msg.contains("not found") => {
                        return Ok(());
                    },
                    _ => return Err(e.clone()),
                }
            } else {
                return Ok(());
            }
        }
        
        result
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        match self.platform_impl.check_permission(plugin_id, operation).await {
            Ok(_) => Ok(()),
            Err(error) => {
                Err(self.standardize_error(error, &format!("check_permission::{}", operation), plugin_id))
            }
        }
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        self.platform_impl.track_resources(plugin_id).await
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write_access: bool) -> Result<()> {
        let operation = if write_access { "write" } else { "read" };
        
        match self.platform_impl.check_path_access(plugin_id, path, write_access).await {
            Ok(_) => Ok(()),
            Err(err) => Err(self.standardize_error(
                err, 
                &format!("Path access check ({})", operation), 
                plugin_id
            ))
        }
    }
    
    async fn check_capability(&self, plugin_id: Uuid, required_capability: &str) -> Result<bool> {
        self.add_context(
            self.platform_impl.check_capability(plugin_id, required_capability).await,
            format!("Failed to check capability {} for plugin {}", 
                required_capability, plugin_id)
        )
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        self.platform_impl.apply_feature(plugin_id, feature).await
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
} 