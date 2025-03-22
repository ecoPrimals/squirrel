use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{error, info};
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::error::Result;

/// Plugin permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// System level access (most privileged)
    System,
    /// User level access
    User,
    /// Restricted access
    Restricted,
}

/// Plugin resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum storage usage in bytes
    pub max_storage_bytes: usize,
    /// Maximum network usage in bytes
    pub max_network_bytes: usize,
    /// Maximum file handles
    pub max_file_handles: usize,
    /// Maximum threads
    pub max_threads: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,                // 10% CPU
            max_storage_bytes: 10 * 1024 * 1024, // 10 MB
            max_network_bytes: 5 * 1024 * 1024,  // 5 MB
            max_file_handles: 10,
            max_threads: 2,
        }
    }
}

/// Plugin security context containing permission and resource information
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Permission level
    pub permission_level: PermissionLevel,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Allowed capabilities
    pub allowed_capabilities: HashSet<String>,
    /// Allowed file paths
    pub allowed_paths: HashSet<PathBuf>,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        let mut allowed_capabilities = HashSet::new();
        allowed_capabilities.insert("read".to_string());
        
        let mut allowed_paths = HashSet::new();
        allowed_paths.insert(PathBuf::from("./plugins"));
        
        Self {
            permission_level: PermissionLevel::Restricted,
            resource_limits: ResourceLimits::default(),
            allowed_capabilities,
            allowed_paths,
            created_at: Utc::now(),
        }
    }
}

/// Current resource usage for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: usize,
    /// Current CPU usage percentage
    pub cpu_percent: u8,
    /// Current storage usage in bytes
    pub storage_bytes: usize,
    /// Current network usage in bytes
    pub network_bytes: usize,
    /// Current file handles
    pub file_handles: usize,
    /// Current threads
    pub threads: usize,
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            cpu_percent: 0,
            storage_bytes: 0,
            network_bytes: 0,
            file_handles: 0,
            threads: 0,
            timestamp: Utc::now(),
        }
    }
}

/// Security errors that can occur during plugin operations
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Insufficient permissions for the operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    /// Operation not allowed in sandbox
    #[error("Operation not allowed in sandbox: {0}")]
    SandboxViolation(String),
    
    /// Security validation failed
    #[error("Security validation failed: {0}")]
    ValidationFailed(String),
    
    /// Path access denied
    #[error("Path access denied: {0}")]
    PathAccessDenied(String),
    
    /// Capability not allowed
    #[error("Capability not allowed: {0}")]
    CapabilityNotAllowed(String),
}

impl From<SecurityError> for crate::error::SquirrelError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::PermissionDenied(msg) => Self::security(format!("Permission denied: {}", msg)),
            SecurityError::ResourceLimitExceeded(msg) => Self::security(format!("Resource limit exceeded: {}", msg)),
            SecurityError::SandboxViolation(msg) => Self::security(format!("Sandbox violation: {}", msg)),
            SecurityError::ValidationFailed(msg) => Self::security(format!("Security validation failed: {}", msg)),
            SecurityError::PathAccessDenied(msg) => Self::security(format!("Path access denied: {}", msg)),
            SecurityError::CapabilityNotAllowed(msg) => Self::security(format!("Capability not allowed: {}", msg)),
        }
    }
}

// Add the implementation for CoreError conversion
impl From<SecurityError> for crate::error::CoreError {
    fn from(err: SecurityError) -> Self {
        Self::Plugin(format!("Security error: {}", err))
    }
}

/// Plugin sandbox for managing plugin security
#[async_trait::async_trait]
pub trait PluginSandbox: Send + Sync + std::fmt::Debug {
    /// Create sandbox for plugin
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Destroy sandbox for plugin
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Check if operation is allowed for the plugin
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()>;
    
    /// Track resource usage for the plugin
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
    
    /// Check if a path is allowed for access
    async fn check_path_access(&self, plugin_id: Uuid, path: &PathBuf, write: bool) -> Result<()>;
    
    /// Check if a capability is allowed for the plugin
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()>;
}

/// Basic plugin sandbox implementation
#[derive(Debug)]
pub struct BasicPluginSandbox {
    /// Plugin security contexts
    security_contexts: Arc<RwLock<std::collections::HashMap<Uuid, SecurityContext>>>,
    /// Plugin resource usage
    resource_usage: Arc<RwLock<std::collections::HashMap<Uuid, ResourceUsage>>>,
}

impl BasicPluginSandbox {
    /// Create a new basic plugin sandbox
    pub fn new() -> Self {
        Self {
            security_contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            resource_usage: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Add a security context for a plugin
    pub async fn add_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        
        // Initialize resource usage tracking
        let mut usage = self.resource_usage.write().await;
        usage.insert(plugin_id, ResourceUsage::default());
        
        Ok(())
    }
    
    /// Get the security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Option<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id).cloned()
    }
    
    /// Update resource usage for a plugin
    pub async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()> {
        // Check if resources exceed limits
        let exceed_check = {
            let contexts = self.security_contexts.read().await;
            if let Some(context) = contexts.get(&plugin_id) {
                let limits = &context.resource_limits;
                if usage.memory_bytes > limits.max_memory_bytes {
                    return Err(SecurityError::ResourceLimitExceeded(format!("Memory usage exceeded: {} > {}", 
                        usage.memory_bytes, limits.max_memory_bytes)).into());
                }
                if usage.cpu_percent > limits.max_cpu_percent {
                    return Err(SecurityError::ResourceLimitExceeded(format!("CPU usage exceeded: {}% > {}%", 
                        usage.cpu_percent, limits.max_cpu_percent)).into());
                }
                if usage.storage_bytes > limits.max_storage_bytes {
                    return Err(SecurityError::ResourceLimitExceeded(format!("Storage usage exceeded: {} > {}", 
                        usage.storage_bytes, limits.max_storage_bytes)).into());
                }
                if usage.network_bytes > limits.max_network_bytes {
                    return Err(SecurityError::ResourceLimitExceeded(format!("Network usage exceeded: {} > {}", 
                        usage.network_bytes, limits.max_network_bytes)).into());
                }
                if usage.file_handles > limits.max_file_handles {
                    return Err(SecurityError::ResourceLimitExceeded(format!("File handles exceeded: {} > {}", 
                        usage.file_handles, limits.max_file_handles)).into());
                }
                if usage.threads > limits.max_threads {
                    return Err(SecurityError::ResourceLimitExceeded(format!("Threads exceeded: {} > {}", 
                        usage.threads, limits.max_threads)).into());
                }
            }
            true
        };
        
        if exceed_check {
            let mut usages = self.resource_usage.write().await;
            usages.insert(plugin_id, usage);
        }
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl PluginSandbox for BasicPluginSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        // Check if security context exists
        let contexts = self.security_contexts.read().await;
        if !contexts.contains_key(&plugin_id) {
            // Create default security context if none exists
            drop(contexts);
            let default_context = SecurityContext::default();
            self.add_security_context(plugin_id, default_context).await?;
        }
        
        info!("Created sandbox for plugin: {}", plugin_id);
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        let mut usages = self.resource_usage.write().await;
        usages.remove(&plugin_id);
        
        info!("Destroyed sandbox for plugin: {}", plugin_id);
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        
        if let Some(context) = contexts.get(&plugin_id) {
            // System level can do anything
            if context.permission_level == PermissionLevel::System {
                return Ok(());
            }
            
            // Check if operation is allowed for the current permission level
            match (context.permission_level, operation) {
                (PermissionLevel::User, "read") => Ok(()),
                (PermissionLevel::User, "write") => Ok(()),
                (PermissionLevel::User, "network") => Ok(()),
                (PermissionLevel::Restricted, "read") => Ok(()),
                _ => Err(SecurityError::PermissionDenied(format!(
                    "Operation '{}' not allowed for permission level {:?}", 
                    operation, context.permission_level)).into()),
            }
        } else {
            Err(SecurityError::ValidationFailed(format!(
                "No security context found for plugin: {}", plugin_id)).into())
        }
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // In a real implementation, this would measure actual resource usage
        // For now, we just return the last known resource usage
        let usages = self.resource_usage.read().await;
        
        if let Some(usage) = usages.get(&plugin_id) {
            Ok(usage.clone())
        } else {
            Ok(ResourceUsage::default())
        }
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &PathBuf, write: bool) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        
        if let Some(context) = contexts.get(&plugin_id) {
            // System level can access any path
            if context.permission_level == PermissionLevel::System {
                return Ok(());
            }
            
            // User level can write to allowed paths, Restricted can only read
            if write && context.permission_level == PermissionLevel::Restricted {
                return Err(SecurityError::PermissionDenied(format!(
                    "Write access to path '{}' not allowed for restricted plugins", 
                    path.display())).into());
            }
            
            // Check if path is in allowed paths
            let is_allowed = context.allowed_paths.iter().any(|allowed| {
                path.starts_with(allowed)
            });
            
            if is_allowed {
                Ok(())
            } else {
                Err(SecurityError::PathAccessDenied(format!(
                    "Access to path '{}' not allowed", path.display())).into())
            }
        } else {
            Err(SecurityError::ValidationFailed(format!(
                "No security context found for plugin: {}", plugin_id)).into())
        }
    }
    
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        
        if let Some(context) = contexts.get(&plugin_id) {
            // System level has all capabilities
            if context.permission_level == PermissionLevel::System {
                return Ok(());
            }
            
            // Check if capability is in allowed capabilities
            if context.allowed_capabilities.contains(capability) {
                Ok(())
            } else {
                Err(SecurityError::CapabilityNotAllowed(format!(
                    "Capability '{}' not allowed", capability)).into())
            }
        } else {
            Err(SecurityError::ValidationFailed(format!(
                "No security context found for plugin: {}", plugin_id)).into())
        }
    }
}

/// Security validator for plugins
#[derive(Debug)]
pub struct SecurityValidator {
    sandbox: Arc<dyn PluginSandbox>,
}

impl SecurityValidator {
    /// Create a new security validator with the given sandbox
    pub fn new(sandbox: Arc<dyn PluginSandbox>) -> Self {
        Self { sandbox }
    }
    
    /// Create a new security validator with a basic sandbox
    pub fn with_basic_sandbox() -> Self {
        Self {
            sandbox: Arc::new(BasicPluginSandbox::new()),
        }
    }
    
    /// Validate that a plugin can perform an operation
    pub async fn validate_operation(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        self.sandbox.check_permission(plugin_id, operation).await
    }
    
    /// Validate that a plugin can access a path
    pub async fn validate_path_access(&self, plugin_id: Uuid, path: &PathBuf, write: bool) -> Result<()> {
        self.sandbox.check_path_access(plugin_id, path, write).await
    }
    
    /// Validate that a plugin can use a capability
    pub async fn validate_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        self.sandbox.check_capability(plugin_id, capability).await
    }
    
    /// Get the sandbox implementation
    pub fn sandbox(&self) -> Arc<dyn PluginSandbox> {
        self.sandbox.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_sandbox_permissions() {
        let sandbox = BasicPluginSandbox::new();
        let plugin_id = Uuid::new_v4();
        
        // Create sandbox with default (restricted) permissions
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Reading should be allowed
        assert!(sandbox.check_permission(plugin_id, "read").await.is_ok());
        
        // Writing should be denied for restricted plugins
        assert!(sandbox.check_permission(plugin_id, "write").await.is_err());
        
        // Create a user level security context
        let mut user_context = SecurityContext::default();
        user_context.permission_level = PermissionLevel::User;
        sandbox.add_security_context(plugin_id, user_context).await.unwrap();
        
        // Now writing should be allowed
        assert!(sandbox.check_permission(plugin_id, "write").await.is_ok());
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let sandbox = BasicPluginSandbox::new();
        let plugin_id = Uuid::new_v4();
        
        // Create sandbox with default limits
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Update with usage below limits
        let mut usage = ResourceUsage::default();
        usage.memory_bytes = 50 * 1024 * 1024; // 50 MB (below 100 MB limit)
        assert!(sandbox.update_resource_usage(plugin_id, usage.clone()).await.is_ok());
        
        // Update with usage above limits
        usage.memory_bytes = 200 * 1024 * 1024; // 200 MB (above 100 MB limit)
        assert!(sandbox.update_resource_usage(plugin_id, usage).await.is_err());
    }
    
    #[tokio::test]
    async fn test_path_access() {
        let sandbox = BasicPluginSandbox::new();
        let plugin_id = Uuid::new_v4();
        
        // Create sandbox with default paths
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Access to plugins directory should be allowed
        assert!(sandbox.check_path_access(
            plugin_id, &PathBuf::from("./plugins/test.txt"), false).await.is_ok());
        
        // Write access should be denied for restricted plugins
        assert!(sandbox.check_path_access(
            plugin_id, &PathBuf::from("./plugins/test.txt"), true).await.is_err());
        
        // Access to system directory should be denied
        assert!(sandbox.check_path_access(
            plugin_id, &PathBuf::from("/etc/passwd"), false).await.is_err());
        
        // Create a user level security context
        let mut user_context = SecurityContext::default();
        user_context.permission_level = PermissionLevel::User;
        sandbox.add_security_context(plugin_id, user_context).await.unwrap();
        
        // Now write access should be allowed
        assert!(sandbox.check_path_access(
            plugin_id, &PathBuf::from("./plugins/test.txt"), true).await.is_ok());
    }
} 