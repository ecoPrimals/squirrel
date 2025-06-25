//! Plugin Sandbox Security Module
//!
//! This module provides security isolation and permission management for WASM plugins.
//! It ensures that plugins can only access resources they're explicitly granted permission to use.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::error::{PluginError, PluginResult};

/// Permission types that can be granted to plugins
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Allow network access to specific domains
    NetworkAccess(String),
    /// Allow reading files from specific paths
    FileSystemRead(String),
    /// Allow writing files to specific paths
    FileSystemWrite(String),
    /// Allow access to local storage
    LocalStorage,
    /// Allow access to session storage
    SessionStorage,
    /// Allow access to clipboard
    Clipboard,
    /// Allow access to geolocation
    Geolocation,
    /// Allow access to notifications
    Notifications,
    /// Allow access to camera
    Camera,
    /// Allow access to microphone
    Microphone,
    /// Allow execution of external commands
    ExecuteCommands,
    /// Allow access to plugin APIs
    PluginAPI(String),
}

impl Permission {
    /// Convert permission to string representation
    pub fn as_str(&self) -> String {
        match self {
            Permission::NetworkAccess(domain) => format!("network_access:{}", domain),
            Permission::FileSystemRead(path) => format!("filesystem_read:{}", path),
            Permission::FileSystemWrite(path) => format!("filesystem_write:{}", path),
            Permission::LocalStorage => "local_storage".to_string(),
            Permission::SessionStorage => "session_storage".to_string(),
            Permission::Clipboard => "clipboard".to_string(),
            Permission::Geolocation => "geolocation".to_string(),
            Permission::Notifications => "notifications".to_string(),
            Permission::Camera => "camera".to_string(),
            Permission::Microphone => "microphone".to_string(),
            Permission::ExecuteCommands => "execute_commands".to_string(),
            Permission::PluginAPI(api) => format!("plugin_api:{}", api),
        }
    }

    /// Parse permission from string
    pub fn from_str(s: &str) -> Option<Self> {
        if let Some(domain) = s.strip_prefix("network_access:") {
            Some(Permission::NetworkAccess(domain.to_string()))
        } else if let Some(path) = s.strip_prefix("filesystem_read:") {
            Some(Permission::FileSystemRead(path.to_string()))
        } else if let Some(path) = s.strip_prefix("filesystem_write:") {
            Some(Permission::FileSystemWrite(path.to_string()))
        } else if let Some(api) = s.strip_prefix("plugin_api:") {
            Some(Permission::PluginAPI(api.to_string()))
        } else {
            match s {
                "local_storage" => Some(Permission::LocalStorage),
                "session_storage" => Some(Permission::SessionStorage),
                "clipboard" => Some(Permission::Clipboard),
                "geolocation" => Some(Permission::Geolocation),
                "notifications" => Some(Permission::Notifications),
                "camera" => Some(Permission::Camera),
                "microphone" => Some(Permission::Microphone),
                "execute_commands" => Some(Permission::ExecuteCommands),
                _ => None,
            }
        }
    }
}

/// Plugin security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Plugin identifier
    pub plugin_id: String,
    /// Set of granted permissions
    pub permissions: HashSet<Permission>,
    /// Resource quotas
    pub quotas: ResourceQuotas,
    /// Time-based restrictions
    pub time_restrictions: Option<TimeRestrictions>,
    /// Content Security Policy for web resources
    pub csp: Option<String>,
}

/// Resource usage quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotas {
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time: Option<u64>,
    /// Maximum network requests per minute
    pub max_network_requests: Option<u32>,
    /// Maximum file system operations per minute
    pub max_fs_operations: Option<u32>,
    /// Maximum storage size in bytes
    pub max_storage_size: Option<u64>,
}

/// Time-based access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Allowed hours (0-23)
    pub allowed_hours: Option<Vec<u8>>,
    /// Allowed days of week (0=Sunday, 6=Saturday)
    pub allowed_days: Option<Vec<u8>>,
    /// Maximum session duration in seconds
    pub max_session_duration: Option<u64>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Current memory usage
    pub memory_used: u64,
    /// CPU time used in milliseconds
    pub cpu_time_used: u64,
    /// Network requests made in current minute
    pub network_requests: u32,
    /// File system operations in current minute
    pub fs_operations: u32,
    /// Storage space used
    pub storage_used: u64,
    /// Session start time
    pub session_start: Option<u64>,
}

/// Sandbox manager for plugin security
#[derive(Debug)]
pub struct SandboxManager {
    /// Security policies for each plugin
    policies: HashMap<String, SecurityPolicy>,
    /// Resource usage tracking
    usage: HashMap<String, ResourceUsage>,
    /// Global security settings
    global_settings: GlobalSecuritySettings,
}

/// Global security settings
#[derive(Debug, Clone)]
pub struct GlobalSecuritySettings {
    /// Whether to enforce strict mode
    pub strict_mode: bool,
    /// Default permissions for new plugins
    pub default_permissions: HashSet<Permission>,
    /// Global resource limits
    pub global_limits: ResourceQuotas,
    /// Allowed domains for network access
    pub allowed_domains: HashSet<String>,
    /// Blocked domains
    pub blocked_domains: HashSet<String>,
}

impl Default for GlobalSecuritySettings {
    fn default() -> Self {
        let mut default_permissions = HashSet::new();
        default_permissions.insert(Permission::LocalStorage);
        default_permissions.insert(Permission::SessionStorage);

        Self {
            strict_mode: true,
            default_permissions,
            global_limits: ResourceQuotas {
                max_memory: Some(100 * 1024 * 1024), // 100MB
                max_cpu_time: Some(30 * 1000), // 30 seconds
                max_network_requests: Some(100),
                max_fs_operations: Some(50),
                max_storage_size: Some(10 * 1024 * 1024), // 10MB
            },
            allowed_domains: HashSet::new(),
            blocked_domains: HashSet::new(),
        }
    }
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            usage: HashMap::new(),
            global_settings: GlobalSecuritySettings::default(),
        }
    }

    /// Register a plugin with security policy
    pub fn register_plugin(&mut self, policy: SecurityPolicy) -> PluginResult<()> {
        // Validate policy against global settings
        self.validate_policy(&policy)?;
        
        let plugin_id = policy.plugin_id.clone();
        self.policies.insert(plugin_id.clone(), policy);
        self.usage.insert(plugin_id, ResourceUsage::default());
        
        Ok(())
    }

    /// Check if a plugin has a specific permission
    pub fn check_permission(&self, plugin_id: &str, permission: &str) -> bool {
        if let Some(policy) = self.policies.get(plugin_id) {
            if let Some(perm) = Permission::from_str(permission) {
                policy.permissions.contains(&perm)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Validate network access for a plugin
    pub fn validate_network_access(&self, plugin_id: &str, hostname: &str) -> PluginResult<()> {
        // Check if domain is globally blocked
        if self.global_settings.blocked_domains.contains(hostname) {
            return Err(PluginError::PermissionDenied(
                format!("Domain {} is globally blocked", hostname)
            ));
        }

        // Check plugin-specific permissions
        if let Some(policy) = self.policies.get(plugin_id) {
            let has_general_access = policy.permissions.contains(&Permission::NetworkAccess("*".to_string()));
            let has_specific_access = policy.permissions.contains(&Permission::NetworkAccess(hostname.to_string()));
            
            if !has_general_access && !has_specific_access {
                return Err(PluginError::PermissionDenied(
                    format!("Network access to {} not allowed for plugin {}", hostname, plugin_id)
                ));
            }
        } else {
            return Err(PluginError::PermissionDenied(
                format!("Plugin {} not registered", plugin_id)
            ));
        }

        // Check resource quotas
        self.check_network_quota(plugin_id)?;

        Ok(())
    }

    /// Validate file system access
    pub fn validate_file_access(&self, plugin_id: &str, path: &str, write: bool) -> PluginResult<()> {
        if let Some(policy) = self.policies.get(plugin_id) {
            let required_permission = if write {
                Permission::FileSystemWrite(path.to_string())
            } else {
                Permission::FileSystemRead(path.to_string())
            };

            // Check for specific path permission or wildcard
            let has_specific = policy.permissions.contains(&required_permission);
            let has_wildcard = if write {
                policy.permissions.contains(&Permission::FileSystemWrite("*".to_string()))
            } else {
                policy.permissions.contains(&Permission::FileSystemRead("*".to_string()))
            };

            if !has_specific && !has_wildcard {
                return Err(PluginError::PermissionDenied(
                    format!("File system access to {} not allowed for plugin {}", path, plugin_id)
                ));
            }
        } else {
            return Err(PluginError::PermissionDenied(
                format!("Plugin {} not registered", plugin_id)
            ));
        }

        // Check resource quotas
        self.check_fs_quota(plugin_id)?;

        Ok(())
    }

    /// Update resource usage for a plugin
    pub fn update_resource_usage(&mut self, plugin_id: &str, usage_type: ResourceUsageType) -> PluginResult<()> {
        if let Some(usage) = self.usage.get_mut(plugin_id) {
            match usage_type {
                ResourceUsageType::NetworkRequest => {
                    usage.network_requests += 1;
                }
                ResourceUsageType::FileSystemOperation => {
                    usage.fs_operations += 1;
                }
                ResourceUsageType::MemoryAllocation(bytes) => {
                    usage.memory_used += bytes;
                }
                ResourceUsageType::CpuTime(ms) => {
                    usage.cpu_time_used += ms;
                }
                ResourceUsageType::StorageUsed(bytes) => {
                    usage.storage_used += bytes;
                }
            }

            // Check if usage exceeds quotas
            self.validate_resource_usage(plugin_id)?;
        }

        Ok(())
    }

    /// Get current resource usage for a plugin
    pub fn get_resource_usage(&self, plugin_id: &str) -> Option<&ResourceUsage> {
        self.usage.get(plugin_id)
    }

    /// Reset resource usage counters (called periodically)
    pub fn reset_periodic_counters(&mut self) {
        for usage in self.usage.values_mut() {
            usage.network_requests = 0;
            usage.fs_operations = 0;
        }
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        self.policies.remove(plugin_id);
        self.usage.remove(plugin_id);
    }

    // Private helper methods

    fn validate_policy(&self, policy: &SecurityPolicy) -> PluginResult<()> {
        if self.global_settings.strict_mode {
            // In strict mode, validate all permissions against global settings
            for permission in &policy.permissions {
                if let Permission::NetworkAccess(domain) = permission {
                    if !self.global_settings.allowed_domains.is_empty() 
                        && !self.global_settings.allowed_domains.contains(domain) 
                        && domain != "*" {
                        return Err(PluginError::PermissionDenied(
                            format!("Domain {} not in allowed list", domain)
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_network_quota(&self, plugin_id: &str) -> PluginResult<()> {
        if let Some(policy) = self.policies.get(plugin_id) {
            if let Some(usage) = self.usage.get(plugin_id) {
                if let Some(max_requests) = policy.quotas.max_network_requests {
                    if usage.network_requests >= max_requests {
                        return Err(PluginError::QuotaExceeded(
                            "Network request quota exceeded".to_string()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_fs_quota(&self, plugin_id: &str) -> PluginResult<()> {
        if let Some(policy) = self.policies.get(plugin_id) {
            if let Some(usage) = self.usage.get(plugin_id) {
                if let Some(max_ops) = policy.quotas.max_fs_operations {
                    if usage.fs_operations >= max_ops {
                        return Err(PluginError::QuotaExceeded(
                            "File system operation quota exceeded".to_string()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_resource_usage(&self, plugin_id: &str) -> PluginResult<()> {
        if let Some(policy) = self.policies.get(plugin_id) {
            if let Some(usage) = self.usage.get(plugin_id) {
                // Check memory quota
                if let Some(max_memory) = policy.quotas.max_memory {
                    if usage.memory_used > max_memory {
                        return Err(PluginError::QuotaExceeded(
                            "Memory quota exceeded".to_string()
                        ));
                    }
                }

                // Check CPU time quota
                if let Some(max_cpu) = policy.quotas.max_cpu_time {
                    if usage.cpu_time_used > max_cpu {
                        return Err(PluginError::QuotaExceeded(
                            "CPU time quota exceeded".to_string()
                        ));
                    }
                }

                // Check storage quota
                if let Some(max_storage) = policy.quotas.max_storage_size {
                    if usage.storage_used > max_storage {
                        return Err(PluginError::QuotaExceeded(
                            "Storage quota exceeded".to_string()
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Types of resource usage that can be tracked
#[derive(Debug, Clone)]
pub enum ResourceUsageType {
    NetworkRequest,
    FileSystemOperation,
    MemoryAllocation(u64),
    CpuTime(u64),
    StorageUsed(u64),
}

/// Global sandbox manager instance
static SANDBOX_MANAGER: Mutex<Option<Arc<Mutex<SandboxManager>>>> = Mutex::new(None);

/// Initialize the global sandbox manager
pub fn init_sandbox_manager() -> Arc<Mutex<SandboxManager>> {
    let mut global = SANDBOX_MANAGER.lock().unwrap();
    if global.is_none() {
        *global = Some(Arc::new(Mutex::new(SandboxManager::new())));
    }
    global.as_ref().unwrap().clone()
}

/// Get the global sandbox manager
pub fn get_sandbox_manager() -> Arc<Mutex<SandboxManager>> {
    let global = SANDBOX_MANAGER.lock().unwrap();
    global.as_ref().expect("Sandbox manager not initialized").clone()
}

/// Create a default security policy for a plugin
pub fn create_default_policy(plugin_id: String) -> SecurityPolicy {
    let mut permissions = HashSet::new();
    permissions.insert(Permission::LocalStorage);
    permissions.insert(Permission::SessionStorage);

    SecurityPolicy {
        plugin_id,
        permissions,
        quotas: ResourceQuotas {
            max_memory: Some(50 * 1024 * 1024), // 50MB
            max_cpu_time: Some(10 * 1000), // 10 seconds
            max_network_requests: Some(50),
            max_fs_operations: Some(25),
            max_storage_size: Some(5 * 1024 * 1024), // 5MB
        },
        time_restrictions: None,
        csp: Some("default-src 'self'".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::NetworkAccess("example.com".to_string());
        let perm_str = perm.as_str();
        assert_eq!(perm_str, "network_access:example.com");
        
        let parsed = Permission::from_str(&perm_str);
        assert_eq!(parsed, Some(perm));
    }

    #[test]
    fn test_sandbox_manager() {
        let mut manager = SandboxManager::new();
        let policy = create_default_policy("test_plugin".to_string());
        
        manager.register_plugin(policy).unwrap();
        
        assert!(manager.check_permission("test_plugin", "local_storage"));
        assert!(!manager.check_permission("test_plugin", "network_access:example.com"));
    }

    #[test]
    fn test_resource_quota_enforcement() {
        let mut manager = SandboxManager::new();
        let mut policy = create_default_policy("test_plugin".to_string());
        policy.quotas.max_network_requests = Some(1);
        
        manager.register_plugin(policy).unwrap();
        
        // First request should succeed
        manager.update_resource_usage("test_plugin", ResourceUsageType::NetworkRequest).unwrap();
        
        // Second request should fail
        let result = manager.update_resource_usage("test_plugin", ResourceUsageType::NetworkRequest);
        assert!(result.is_err());
    }
} 