//! Sandbox functionality for secure plugin execution
//!
//! This module provides security and isolation mechanisms for WASM plugins,
//! ensuring they operate within defined boundaries and permissions.

use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use js_sys::Date;
use std::sync::{Mutex, OnceLock};

/// Security level for plugin execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Minimal restrictions, full access
    Unrestricted,
    /// Standard restrictions, most operations allowed
    Standard,
    /// High restrictions, limited operations
    Restricted,
    /// Maximum restrictions, minimal operations
    Sandboxed,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

impl SecurityLevel {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unrestricted => "unrestricted",
            Self::Standard => "standard",
            Self::Restricted => "restricted",
            Self::Sandboxed => "sandboxed",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unrestricted" => Some(Self::Unrestricted),
            "standard" => Some(Self::Standard),
            "restricted" => Some(Self::Restricted),
            "sandboxed" => Some(Self::Sandboxed),
            _ => None,
        }
    }
}

/// Permission types for plugin operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Network access permission
    NetworkAccess,
    /// File system read permission
    FileSystemRead,
    /// File system write permission
    FileSystemWrite,
    /// Environment variable access
    EnvironmentAccess,
    /// Process execution permission
    ProcessExecution,
    /// System information access
    SystemInfo,
    /// Inter-plugin communication
    PluginCommunication,
    /// Custom permission
    Custom(String),
}

impl Permission {
    /// Convert to string representation
    pub fn as_str(&self) -> String {
        match self {
            Self::NetworkAccess => "network_access".to_string(),
            Self::FileSystemRead => "filesystem_read".to_string(),
            Self::FileSystemWrite => "filesystem_write".to_string(),
            Self::EnvironmentAccess => "environment_access".to_string(),
            Self::ProcessExecution => "process_execution".to_string(),
            Self::SystemInfo => "system_info".to_string(),
            Self::PluginCommunication => "plugin_communication".to_string(),
            Self::Custom(name) => format!("custom:{}", name),
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "network_access" => Some(Self::NetworkAccess),
            "filesystem_read" => Some(Self::FileSystemRead),
            "filesystem_write" => Some(Self::FileSystemWrite),
            "environment_access" => Some(Self::EnvironmentAccess),
            "process_execution" => Some(Self::ProcessExecution),
            "system_info" => Some(Self::SystemInfo),
            "plugin_communication" => Some(Self::PluginCommunication),
            s if s.starts_with("custom:") => {
                Some(Self::Custom(s.strip_prefix("custom:").unwrap().to_string()))
            }
            _ => None,
        }
    }
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum number of file handles
    pub max_file_handles: Option<u32>,
    /// Maximum network connections
    pub max_network_connections: Option<u32>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(256 * 1024 * 1024), // 256MB
            max_cpu_time_ms: Some(30_000), // 30 seconds
            max_file_handles: Some(100),
            max_network_connections: Some(10),
            max_execution_time_ms: Some(60_000), // 60 seconds
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Security level for the plugin
    pub security_level: SecurityLevel,
    /// List of allowed permissions
    pub permissions: Vec<Permission>,
    /// Resource limits for the plugin
    pub resource_limits: ResourceLimits,
    /// Allowed domains for network access
    pub domain_whitelist: Vec<String>,
    /// Allowed file paths for access
    pub file_read_whitelist: Vec<String>,
    /// Allowed file paths for writing
    pub file_write_whitelist: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            permissions: vec![
                Permission::NetworkAccess,
                Permission::FileSystemRead,
            ],
            resource_limits: ResourceLimits::default(),
            domain_whitelist: vec![],
            file_read_whitelist: vec![],
            file_write_whitelist: vec![],
        }
    }
}

/// Sandbox context for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Sandbox configuration
    pub config: SandboxConfig,
    /// Current resource usage
    pub resource_usage: ResourceUsage,
    /// Active permissions
    pub active_permissions: Vec<Permission>,
}

/// Current resource usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Current CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Current file handles
    pub file_handles: u32,
    /// Current network connections
    pub network_connections: u32,
    /// Execution start time
    pub execution_start_ms: u64,
}

/// Sandbox manager for controlling plugin execution
#[wasm_bindgen]
pub struct SandboxManager {
    contexts: HashMap<String, SandboxContext>,
}

#[wasm_bindgen]
impl SandboxManager {
    /// Create a new sandbox manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }
    
    /// Create a sandbox context for a plugin
    #[wasm_bindgen]
    pub fn create_context(&mut self, plugin_id: String, config: JsValue) -> Result<(), JsValue> {
        let config: SandboxConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid sandbox config: {}", e)))?;
        
        let context = SandboxContext {
            plugin_id: plugin_id.clone(),
            config,
            resource_usage: ResourceUsage::default(),
            active_permissions: vec![],
        };
        
        self.contexts.insert(plugin_id, context);
        Ok(())
    }
    
    /// Check if a permission is allowed for a plugin
    #[wasm_bindgen]
    pub fn check_permission(&self, plugin_id: &str, permission: &str) -> bool {
        if let Some(context) = self.contexts.get(plugin_id) {
            if let Some(perm) = Permission::from_str(permission) {
                return context.config.permissions.contains(&perm);
            }
        }
        false
    }
    
    /// Update resource usage for a plugin
    #[wasm_bindgen]
    pub fn update_resource_usage(&mut self, plugin_id: &str, usage: JsValue) -> Result<(), JsValue> {
        let new_usage: ResourceUsage = serde_wasm_bindgen::from_value(usage)
            .map_err(|e| JsValue::from_str(&format!("Invalid usage data: {}", e)))?;
        
        if let Some(context) = self.contexts.get_mut(plugin_id) {
            context.resource_usage = new_usage;
            
            // Check limits after update - we need to clone the context to avoid borrowing issues
            let context_clone = SandboxContext {
                plugin_id: context.plugin_id.clone(),
                config: context.config.clone(),
                resource_usage: context.resource_usage.clone(),
                active_permissions: context.active_permissions.clone(),
            };
            self.check_resource_limits(&context_clone)?;
        }

        Ok(())
    }
    
    /// Get current resource usage for a plugin
    #[wasm_bindgen]
    pub fn get_resource_usage(&self, plugin_id: &str) -> Result<JsValue, JsValue> {
        if let Some(context) = self.contexts.get(plugin_id) {
            serde_wasm_bindgen::to_value(&context.resource_usage)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        } else {
            Err(JsValue::from_str("Plugin context not found"))
        }
    }
    
    /// Remove sandbox context for a plugin
    #[wasm_bindgen]
    pub fn remove_context(&mut self, plugin_id: &str) {
        self.contexts.remove(plugin_id);
    }
}

impl SandboxManager {
    /// Check resource limits for a context
    fn check_resource_limits(&self, context: &SandboxContext) -> Result<(), JsValue> {
        let limits = &context.config.resource_limits;
        let usage = &context.resource_usage;

        // Check memory limit
        if let Some(max_memory) = limits.max_memory_bytes {
            if usage.memory_bytes > max_memory {
                return Err(JsValue::from_str(&format!(
                    "Memory limit exceeded: {} > {}",
                    usage.memory_bytes, max_memory
                )));
            }
        }

        // Check CPU time limit
        if let Some(max_cpu) = limits.max_cpu_time_ms {
            if usage.cpu_time_ms > max_cpu {
                return Err(JsValue::from_str(&format!(
                    "CPU time limit exceeded: {} > {}",
                    usage.cpu_time_ms, max_cpu
                )));
            }
        }

        // Check execution time limit
        if let Some(max_execution) = limits.max_execution_time_ms {
            let current_time = Date::now() as u64;
            if current_time - usage.execution_start_ms > max_execution {
                return Err(JsValue::from_str(&format!(
                    "Execution time limit exceeded: {} > {}",
                    current_time - usage.execution_start_ms, max_execution
                )));
            }
        }

        Ok(())
    }
    
    /// Validate network access
    pub fn validate_network_access(&self, plugin_id: &str, domain: &str) -> PluginResult<()> {
        if let Some(context) = self.contexts.get(plugin_id) {
            if !context.config.permissions.contains(&Permission::NetworkAccess) {
                return Err(PluginError::PermissionDenied("Network access not allowed".to_string()));
            }
            
            if !context.config.domain_whitelist.is_empty() {
                let allowed = context.config.domain_whitelist.iter()
                    .any(|allowed_domain| domain.ends_with(allowed_domain));
                
                if !allowed {
                    return Err(PluginError::PermissionDenied(
                        format!("Domain {} not in allowed list", domain)
                    ));
                }
            }
        } else {
            return Err(PluginError::InvalidConfiguration("Plugin context not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate file access permissions
    pub fn validate_file_access(&self, plugin_id: &str, path: &str, write: bool) -> PluginResult<()> {
        if let Some(context) = self.contexts.get(plugin_id) {
            // Check if path is in whitelist
            let allowed = if write {
                context.config.file_write_whitelist.iter().any(|pattern| path.starts_with(pattern))
            } else {
                context.config.file_read_whitelist.iter().any(|pattern| path.starts_with(pattern))
            };
            
            if !allowed {
                return Err(PluginError::SecurityViolation(format!(
                    "File {} access denied for path: {}", 
                    if write { "write" } else { "read" }, 
                    path
                )));
            }
        }
        
        Ok(())
    }
}

/// Get the global sandbox manager instance
pub fn get_sandbox_manager() -> &'static Mutex<SandboxManager> {
    static MANAGER: OnceLock<Mutex<SandboxManager>> = OnceLock::new();
    MANAGER.get_or_init(|| Mutex::new(SandboxManager::new()))
}

/// Initialize sandbox for a plugin
pub fn init_sandbox(plugin_id: String, config: SandboxConfig) -> PluginResult<()> {
    let manager = get_sandbox_manager();
    let mut manager_guard = manager.lock().unwrap();
    let js_config = serde_wasm_bindgen::to_value(&config)
        .map_err(|e| PluginError::InternalError(format!("Config serialization error: {}", e)))?;
    
    manager_guard.create_context(plugin_id, js_config)
        .map_err(|e| PluginError::InternalError(format!("Sandbox creation error: {:?}", e)))?;
    
    Ok(())
}

/// Check permission for current plugin
pub fn check_permission(permission: Permission) -> bool {
    // In a real implementation, this would get the current plugin ID from context
    let plugin_id = "current_plugin"; // Placeholder
    let manager = get_sandbox_manager();
    let manager_guard = manager.lock().unwrap();
    manager_guard.check_permission(plugin_id, &permission.as_str())
}

/// Utility functions for sandbox operations
pub mod utils {
    use super::*;
    
    /// Create a default sandbox config for a security level
    pub fn create_config_for_level(level: SecurityLevel) -> SandboxConfig {
        let mut config = SandboxConfig::default();
        config.security_level = level;
        
        match level {
            SecurityLevel::Unrestricted => {
                config.permissions = vec![
                    Permission::NetworkAccess,
                    Permission::FileSystemRead,
                    Permission::FileSystemWrite,
                    Permission::EnvironmentAccess,
                    Permission::ProcessExecution,
                    Permission::SystemInfo,
                    Permission::PluginCommunication,
                ];
                config.resource_limits = ResourceLimits {
                    max_memory_bytes: None,
                    max_cpu_time_ms: None,
                    max_file_handles: None,
                    max_network_connections: None,
                    max_execution_time_ms: None,
                };
            }
            SecurityLevel::Standard => {
                // Default config is already standard
            }
            SecurityLevel::Restricted => {
                config.permissions = vec![
                    Permission::FileSystemRead,
                    Permission::SystemInfo,
                ];
                config.resource_limits.max_memory_bytes = Some(128 * 1024 * 1024); // 128MB
                config.resource_limits.max_cpu_time_ms = Some(15_000); // 15 seconds
            }
            SecurityLevel::Sandboxed => {
                config.permissions = vec![Permission::SystemInfo];
                config.resource_limits.max_memory_bytes = Some(64 * 1024 * 1024); // 64MB
                config.resource_limits.max_cpu_time_ms = Some(5_000); // 5 seconds
                config.resource_limits.max_file_handles = Some(10);
                config.resource_limits.max_network_connections = Some(0);
            }
        }
        
        config
    }
    
    /// Validate sandbox configuration
    pub fn validate_config(config: &SandboxConfig) -> PluginResult<()> {
        // Check for conflicting permissions
        if config.permissions.contains(&Permission::FileSystemWrite) 
            && !config.permissions.contains(&Permission::FileSystemRead) {
            return Err(PluginError::InvalidConfiguration(
                "Write permission requires read permission".to_string()
            ));
        }
        
        // Validate resource limits
        if let Some(memory) = config.resource_limits.max_memory_bytes {
            if memory < 1024 * 1024 { // Minimum 1MB
                return Err(PluginError::InvalidConfiguration(
                    "Memory limit too low (minimum 1MB)".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[test]
    fn test_security_level_conversion() {
        assert_eq!(SecurityLevel::Standard.as_str(), "standard");
        assert_eq!(SecurityLevel::from_str("restricted"), Some(SecurityLevel::Restricted));
        assert_eq!(SecurityLevel::from_str("invalid"), None);
    }
    
    #[test]
    fn test_permission_conversion() {
        let perm = Permission::NetworkAccess;
        assert_eq!(perm.as_str(), "network_access");
        assert_eq!(Permission::from_str("network_access"), Some(Permission::NetworkAccess));
        
        let custom = Permission::Custom("test".to_string());
        assert_eq!(custom.as_str(), "custom:test");
        assert_eq!(Permission::from_str("custom:test"), Some(Permission::Custom("test".to_string())));
    }
    
    #[wasm_bindgen_test]
    fn test_sandbox_manager() {
        let mut manager = SandboxManager::new();
        let config = SandboxConfig::default();
        let js_config = serde_wasm_bindgen::to_value(&config).unwrap();
        
        manager.create_context("test-plugin".to_string(), js_config).unwrap();
        assert!(manager.check_permission("test-plugin", "filesystem_read"));
        assert!(!manager.check_permission("test-plugin", "network_access"));
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = SandboxConfig::default();
        config.permissions = vec![Permission::FileSystemWrite]; // Missing read permission
        
        assert!(utils::validate_config(&config).is_err());
        
        config.permissions.push(Permission::FileSystemRead);
        assert!(utils::validate_config(&config).is_ok());
    }
    
    #[test]
    fn test_security_level_configs() {
        let unrestricted = utils::create_config_for_level(SecurityLevel::Unrestricted);
        assert!(unrestricted.permissions.contains(&Permission::NetworkAccess));
        assert!(unrestricted.resource_limits.max_memory_bytes.is_none());
        
        let sandboxed = utils::create_config_for_level(SecurityLevel::Sandboxed);
        assert!(!sandboxed.permissions.contains(&Permission::NetworkAccess));
        assert!(sandboxed.resource_limits.max_memory_bytes.is_some());
    }
} 