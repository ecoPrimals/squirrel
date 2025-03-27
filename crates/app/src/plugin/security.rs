use std::collections::{HashSet, HashMap};
use std::path::{PathBuf, Path};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{error, info, warn};
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::error::Result;
use super::resource_monitor::{ResourceMonitor, ResourceLimits as ResourceMonitorLimits, ResourceUsage as ResourceMonitorUsage};
use super::sandbox::{PluginSandbox, BasicPluginSandbox};

/// Plugin permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
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
    pub max_memory_bytes: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum disk usage in MB
    pub max_disk_mb: u64,
    /// Maximum threads
    pub max_threads: u16,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,                // 10% CPU
            max_disk_mb: 100,                   // 100 MB
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
    pub allowed_paths: Vec<PathBuf>,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        let mut allowed_capabilities = HashSet::new();
        // Use namespace:action format consistently
        allowed_capabilities.insert("plugin:read".to_string());
        allowed_capabilities.insert("file:read".to_string());
        allowed_capabilities.insert("config:read".to_string());
        
        let allowed_paths = vec![PathBuf::from("./plugins")];
        
        Self {
            permission_level: PermissionLevel::Restricted,
            resource_limits: ResourceLimits::default(),
            allowed_capabilities,
            allowed_paths,
            created_at: Utc::now(),
        }
    }
}

impl SecurityContext {
    /// Create a new security context with capabilities appropriate for the given permission level
    pub fn for_permission_level(level: PermissionLevel) -> Self {
        let mut context = Self::default();
        context.permission_level = level;
        
        // Set capabilities based on permission level
        let mut capabilities = HashSet::new();
        
        match level {
            PermissionLevel::System => {
                // System can do anything, but we'll add some common ones for explicitness
                capabilities.insert("file:*".to_string());
                capabilities.insert("plugin:*".to_string());
                capabilities.insert("network:*".to_string());
                capabilities.insert("config:*".to_string());
                capabilities.insert("system:*".to_string());
            },
            PermissionLevel::User => {
                capabilities.insert("file:read".to_string());
                capabilities.insert("file:write".to_string());
                capabilities.insert("plugin:read".to_string());
                capabilities.insert("plugin:execute".to_string());
                capabilities.insert("network:connect".to_string());
                capabilities.insert("config:read".to_string());
                capabilities.insert("config:write".to_string());
                capabilities.insert("system:info".to_string());
            },
            PermissionLevel::Restricted => {
                capabilities.insert("file:read".to_string());
                capabilities.insert("plugin:read".to_string());
                capabilities.insert("config:read".to_string());
            }
        }
        
        context.allowed_capabilities = capabilities;
        context
    }
}

// Use the ResourceMonitorUsage directly for consistency
/// Resource usage data for a plugin, representing CPU, memory, disk and network usage
/// This is a type alias to ResourceMonitorUsage for consistency with the resource_monitor module
pub type ResourceUsage = ResourceMonitorUsage;

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
            SecurityError::PermissionDenied(msg) => Self::security(format!("Permission denied: {msg}")),
            SecurityError::ResourceLimitExceeded(msg) => Self::security(format!("Resource limit exceeded: {msg}")),
            SecurityError::SandboxViolation(msg) => Self::security(format!("Sandbox violation: {msg}")),
            SecurityError::ValidationFailed(msg) => Self::security(format!("Security validation failed: {msg}")),
            SecurityError::PathAccessDenied(msg) => Self::security(format!("Path access denied: {msg}")),
            SecurityError::CapabilityNotAllowed(msg) => Self::security(format!("Capability not allowed: {msg}")),
        }
    }
}

/// Security audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEntry {
    /// Plugin ID
    pub plugin_id: Uuid,
    /// Plugin name (if available)
    pub plugin_name: Option<String>,
    /// Operation requested
    pub operation: String,
    /// Whether the operation was allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Timestamp of the audit
    pub timestamp: DateTime<Utc>,
}

/// Security validator for plugins
#[derive(Debug)]
pub struct SecurityValidator {
    /// Plugin security contexts
    security_contexts: Arc<RwLock<std::collections::HashMap<Uuid, SecurityContext>>>,
    /// Plugin audit log
    audit_log: Arc<RwLock<Vec<SecurityAuditEntry>>>,
    /// Sandbox implementation
    sandbox: Arc<dyn PluginSandbox>,
    /// Audit enabled flag
    audit_enabled: bool,
}

impl Default for SecurityValidator {
    fn default() -> Self {
        // Use the new sandbox module's BasicPluginSandbox
        let sandbox = Arc::new(super::sandbox::BasicPluginSandbox::new(
            Arc::new(ResourceMonitor::new())
        ));
        
        Self::with_sandbox(sandbox)
    }
}

impl SecurityValidator {
    /// Create a new security validator
    #[must_use] pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a security validator with a custom sandbox
    pub fn with_sandbox(sandbox: Arc<dyn PluginSandbox>) -> Self {
        Self {
            security_contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            sandbox,
            audit_enabled: true,
        }
    }
    
    /// Set security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        Ok(())
    }
    
    /// Get security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Option<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id).cloned()
    }
    
    /// Audit an operation
    async fn audit(&self, plugin_id: Uuid, operation: &str, allowed: bool, reason: &str) {
        if !self.audit_enabled {
            return;
        }
        
        let entry = SecurityAuditEntry {
            plugin_id,
            plugin_name: None, // Could be retrieved if needed
            operation: operation.to_string(),
            allowed,
            reason: reason.to_string(),
            timestamp: Utc::now(),
        };
        
        let mut audit_log = self.audit_log.write().await;
        audit_log.push(entry);
        
        // Limit audit log size
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }
    
    /// Validate if an operation is allowed for a plugin
    pub async fn validate_operation(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        // Delegate to sandbox
        let result = self.sandbox.check_permission(plugin_id, operation).await;
        
        // Audit the operation
        match &result {
            Ok(_) => {
                self.audit(plugin_id, operation, true, "Operation allowed").await;
            }
            Err(e) => {
                self.audit(plugin_id, operation, false, &e.to_string()).await;
            }
        }
        
        result
    }
    
    /// Validate if a plugin has access to a path
    pub async fn validate_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        // Delegate to sandbox
        let result = self.sandbox.check_path_access(plugin_id, path, write).await;
        
        // Audit the operation
        let operation = if write {
            format!("Write access to {}", path.display())
        } else {
            format!("Read access to {}", path.display())
        };
        
        match &result {
            Ok(_) => {
                self.audit(plugin_id, &operation, true, "Path access allowed").await;
            }
            Err(e) => {
                self.audit(plugin_id, &operation, false, &e.to_string()).await;
            }
        }
        
        result
    }
    
    /// Validate if a plugin has a capability
    pub async fn validate_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        let result = self.sandbox.check_capability(plugin_id, capability).await?;
        
        if result {
            Ok(())
        } else {
            Err(SecurityError::CapabilityNotAllowed(format!(
                "Capability not allowed: {} for plugin {}",
                capability, plugin_id
            )).into())
        }
    }
    
    /// Get the sandbox
    pub fn sandbox(&self) -> Arc<dyn PluginSandbox> {
        self.sandbox.clone()
    }
    
    /// Get the audit log
    pub async fn get_audit_log(&self, plugin_id: Option<Uuid>, limit: usize) -> Vec<SecurityAuditEntry> {
        let audit_log = self.audit_log.read().await;
        
        let mut filtered_log: Vec<SecurityAuditEntry> = if let Some(id) = plugin_id {
            audit_log.iter().filter(|entry| entry.plugin_id == id).cloned().collect()
        } else {
            audit_log.clone()
        };
        
        // Sort by timestamp (most recent first) and limit
        filtered_log.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if filtered_log.len() > limit {
            filtered_log.truncate(limit);
        }
        
        filtered_log
    }
}

/// Enhanced security validator with more features
#[derive(Debug)]
pub struct EnhancedSecurityValidator {
    /// Plugin security contexts
    security_contexts: Arc<RwLock<std::collections::HashMap<Uuid, SecurityContext>>>,
    /// Plugin audit log
    audit_log: Arc<RwLock<Vec<SecurityAuditEntry>>>,
    /// Sandbox implementation
    sandbox: Arc<dyn PluginSandbox>,
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    /// Audit enabled flag
    audit_enabled: bool,
}

impl Default for EnhancedSecurityValidator {
    fn default() -> Self {
        // First create the resource monitor that both the validator and sandbox will use
        let resource_monitor = Arc::new(ResourceMonitor::default());
        
        // Create a basic sandbox with the resource monitor
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));
        
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            sandbox,
            resource_monitor,
            audit_enabled: true,
        }
    }
}

impl EnhancedSecurityValidator {
    /// Create a new enhanced security validator
    #[must_use] pub fn new() -> Self {
        // First create the resource monitor that both the validator and sandbox will use
        let resource_monitor = Arc::new(ResourceMonitor::default());
        
        // Create a basic sandbox with the resource monitor
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));
        
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            sandbox,
            resource_monitor,
            audit_enabled: true,
        }
    }
    
    /// Create an enhanced security validator with a custom sandbox
    pub fn new_with_sandbox(sandbox: Arc<dyn PluginSandbox>) -> Self {
        // Get the resource monitor from the sandbox if available, otherwise create a new one
        let resource_monitor = sandbox.get_resource_monitor()
            .unwrap_or_else(|| Arc::new(ResourceMonitor::default()));
        
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            sandbox,
            resource_monitor,
            audit_enabled: true,
        }
    }
    
    /// Create an enhanced security validator with a custom sandbox and resource monitor
    pub fn with_sandbox_and_monitor(sandbox: Arc<dyn PluginSandbox>, resource_monitor: Arc<ResourceMonitor>) -> Self {
        Self {
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            sandbox,
            resource_monitor,
            audit_enabled: true,
        }
    }
    
    /// Set security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context.clone());
        
        // If using the CrossPlatformSandbox, update its security context as well
        if let Some(cross_platform) = self.sandbox.as_any().downcast_ref::<super::sandbox::CrossPlatformSandbox>() {
            cross_platform.set_security_context(plugin_id, context).await?;
        }
        
        Ok(())
    }
    
    /// Get security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Option<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id).cloned()
    }
    
    /// Audit an operation
    async fn audit(&self, plugin_id: Uuid, operation: &str, allowed: bool, reason: &str) {
        if !self.audit_enabled {
            return;
        }
        
        let entry = SecurityAuditEntry {
            plugin_id,
            plugin_name: None, // Could be retrieved if needed
            operation: operation.to_string(),
            allowed,
            reason: reason.to_string(),
            timestamp: Utc::now(),
        };
        
        let mut audit_log = self.audit_log.write().await;
        audit_log.push(entry);
        
        // Limit audit log size
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }
    
    /// Validate if an operation is allowed for a plugin
    pub async fn validate_operation(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        // Delegate to sandbox
        let result = self.sandbox.check_permission(plugin_id, operation).await;
        
        // Audit the operation
        match &result {
            Ok(_) => {
                self.audit(plugin_id, operation, true, "Operation allowed").await;
            }
            Err(e) => {
                self.audit(plugin_id, operation, false, &e.to_string()).await;
            }
        }
        
        result
    }
    
    /// Validate if a plugin has access to a path
    pub async fn validate_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        // Delegate to sandbox
        let result = self.sandbox.check_path_access(plugin_id, path, write).await;
        
        // Audit the operation
        let operation = if write {
            format!("Write access to {}", path.display())
        } else {
            format!("Read access to {}", path.display())
        };
        
        match &result {
            Ok(_) => {
                self.audit(plugin_id, &operation, true, "Path access allowed").await;
            }
            Err(e) => {
                self.audit(plugin_id, &operation, false, &e.to_string()).await;
            }
        }
        
        result
    }
    
    /// Validate if a plugin has a capability
    pub async fn validate_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        let result = self.sandbox.check_capability(plugin_id, capability).await?;
        
        if result {
            Ok(())
        } else {
            Err(SecurityError::CapabilityNotAllowed(format!(
                "Capability not allowed: {} for plugin {}",
                capability, plugin_id
            )).into())
        }
    }
    
    /// Check if a plugin has exceeded resource limits
    pub async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<bool> {
        // Get resource usage
        let usage = self.sandbox.track_resources(plugin_id).await?;
        
        // Get security context
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id).ok_or_else(|| {
            SecurityError::ValidationFailed(format!("Security context not found for plugin {}", plugin_id))
        })?;
        
        // Check each resource against limits
        let exceeded = if usage.memory_bytes > context.resource_limits.max_memory_bytes {
            warn!("Plugin {} exceeded memory limit: {} bytes > {} bytes", 
                plugin_id, usage.memory_bytes, context.resource_limits.max_memory_bytes);
            Some("Memory limit exceeded")
        } else if usage.cpu_percent > context.resource_limits.max_cpu_percent as f32 {
            warn!("Plugin {} exceeded CPU limit: {}% > {}%", 
                plugin_id, usage.cpu_percent, context.resource_limits.max_cpu_percent as f32);
            Some("CPU limit exceeded")
        } else if usage.disk_mb > context.resource_limits.max_disk_mb as f32 {
            warn!("Plugin {} exceeded disk limit: {} MB > {} MB", 
                plugin_id, usage.disk_mb, context.resource_limits.max_disk_mb as f32);
            Some("Disk limit exceeded")
        } else {
            None
        };
        
        // Audit if exceeded
        if let Some(reason) = exceeded {
            self.audit(
                plugin_id,
                "Resource usage check",
                false,
                reason
            ).await;
        }
        
        Ok(exceeded.is_some())
    }
    
    /// Get the sandbox
    pub fn sandbox(&self) -> Arc<dyn PluginSandbox> {
        self.sandbox.clone()
    }
    
    /// Get the resource monitor
    pub fn resource_monitor(&self) -> Arc<ResourceMonitor> {
        self.resource_monitor.clone()
    }
    
    /// Get the resource monitor with the get_ prefix to match expected usage
    pub fn get_resource_monitor(&self) -> Arc<ResourceMonitor> {
        self.resource_monitor.clone()
    }
    
    /// Get the audit log entries
    pub async fn get_audit_log(&self, plugin_id: Option<Uuid>, limit: usize) -> Vec<SecurityAuditEntry> {
        let audit_log = self.audit_log.read().await;
        
        let mut filtered_log: Vec<SecurityAuditEntry> = if let Some(id) = plugin_id {
            audit_log.iter().filter(|entry| entry.plugin_id == id).cloned().collect()
        } else {
            audit_log.clone()
        };
        
        // Sort by timestamp (most recent first) and limit
        filtered_log.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if filtered_log.len() > limit {
            filtered_log.truncate(limit);
        }
        
        filtered_log
    }
    
    /// Apply a security feature
    pub async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        self.sandbox.apply_feature(plugin_id, feature).await
    }

    /// Create a test instance for unit testing
    #[cfg(test)]
    pub async fn create_test_instance() -> Self {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));
        let validator = Self::new_with_sandbox(sandbox);
        
        // Add some contexts for testing
        {
            let mut contexts = validator.security_contexts.write().await;
            contexts.insert(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                SecurityContext::default()
            );
        } // contexts is dropped here
        
        validator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_basic_sandbox_permissions() {
        let resource_monitor = Arc::new(ResourceMonitor::default());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        let plugin_id = Uuid::new_v4();
        
        // Create a sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // System level should be able to do anything
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::System;
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Should succeed
        sandbox.check_permission(plugin_id, "filesystem:write").await.unwrap();
        
        // User level should be able to do some things
        let mut context = SecurityContext::for_permission_level(PermissionLevel::User);
        
        // Ensure the specific capabilities we'll test are added
        context.allowed_capabilities.insert("file:write".to_string());
        context.allowed_capabilities.insert("network:connect".to_string());
        
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Should succeed
        sandbox.check_permission(plugin_id, "filesystem:write").await.unwrap();
        
        // Should fail
        assert!(sandbox.check_permission(plugin_id, "system:admin").await.is_err());
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let resource_monitor = Arc::new(ResourceMonitor::default());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        let plugin_id = Uuid::new_v4();
        
        // Create a sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Set resource limits
        let mut context = SecurityContext::default();
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 1024 * 1024 * 100, // 100 MB
            max_cpu_percent: 10,
            max_disk_mb: 1024, // 1 GB
            max_threads: 5,
        };
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Should be able to get the context back
        let context = sandbox.get_security_context(plugin_id).await.unwrap();
        assert_eq!(context.resource_limits.max_memory_bytes, 1024 * 1024 * 100);
        assert_eq!(context.resource_limits.max_cpu_percent, 10);
        assert_eq!(context.resource_limits.max_disk_mb, 1024);
        assert_eq!(context.resource_limits.max_threads, 5);
    }

    #[tokio::test]
    async fn test_path_access() {
        let resource_monitor = Arc::new(ResourceMonitor::default());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        let plugin_id = Uuid::new_v4();
        
        // Create a sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Set allowed paths
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::User;
        context.allowed_paths = vec![
            PathBuf::from(".")
        ];
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Should be able to access allowed paths
        sandbox.check_path_access(plugin_id, &PathBuf::from("."), false).await.unwrap();
        
        // Should not be able to access paths outside of allowed paths
        // This test might fail depending on the current directory, but it's a good example
        assert!(sandbox.check_path_access(plugin_id, &PathBuf::from("/"), true).await.is_err());
    }
    
    #[tokio::test]
    async fn test_enhanced_security_validator() {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));
        let validator = EnhancedSecurityValidator::with_sandbox_and_monitor(sandbox.clone(), resource_monitor);
        let plugin_id = Uuid::new_v4();
        
        // Create the sandbox for the plugin first
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Set up a security context using our new helper method
        let mut context = SecurityContext::for_permission_level(PermissionLevel::User);
        
        // Add a custom capability for testing
        context.allowed_capabilities.insert("custom:test".to_string());
        
        // Update both the validator and the sandbox with the security context
        validator.set_security_context(plugin_id, context.clone()).await.unwrap();
        
        // Also set the security context directly on the sandbox since the validator doesn't propagate
        // to BasicPluginSandbox (only to CrossPlatformSandbox)
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Test standard operations that should be allowed for User level
        validator.validate_operation(plugin_id, "filesystem:write").await.unwrap();
        assert!(validator.validate_operation(plugin_id, "system:admin").await.is_err());
        
        // Test capabilities
        validator.validate_capability(plugin_id, "plugin:read").await.unwrap(); // Standard capability
        validator.validate_capability(plugin_id, "file:read").await.unwrap(); // Standard capability
        validator.validate_capability(plugin_id, "custom:test").await.unwrap(); // Custom capability
        validator.validate_capability(plugin_id, "network:connect").await.unwrap(); // Standard capability
        
        // These should fail
        assert!(validator.validate_capability(plugin_id, "system:admin").await.is_err());
        assert!(validator.validate_capability(plugin_id, "plugin:install").await.is_err());
        
        // Test with restricted permissions
        let restricted_plugin_id = Uuid::new_v4();
        sandbox.create_sandbox(restricted_plugin_id).await.unwrap();
        
        let restricted_context = SecurityContext::for_permission_level(PermissionLevel::Restricted);
        validator.set_security_context(restricted_plugin_id, restricted_context.clone()).await.unwrap();
        sandbox.set_security_context(restricted_plugin_id, restricted_context).await.unwrap();
        
        // Check that appropriate permissions are enforced
        validator.validate_capability(restricted_plugin_id, "file:read").await.unwrap(); // Should work
        assert!(validator.validate_capability(restricted_plugin_id, "file:write").await.is_err()); // Should fail
    }
} 
