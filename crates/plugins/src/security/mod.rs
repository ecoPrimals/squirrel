//! Plugin security module
//!
//! This module provides security functionality for plugins.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::plugin::{Plugin, PluginMetadata};

pub mod resource_monitor;
pub mod signature;

pub use resource_monitor::{ResourceMonitor, ResourceMonitorConfig, ResourceStats, ResourceLimitAction};
pub use signature::{
    SignatureVerifier, SignatureVerifierConfig, PluginSignature, 
    SignatureAlgorithm, SignatureScope, VerificationResult
};

/// Security permission
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Permission name
    pub name: String,
    
    /// Permission description
    pub description: String,
    
    /// Permission scope
    pub scope: PermissionScope,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
}

/// Permission scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PermissionScope {
    /// System scope
    System,
    /// Plugin scope
    Plugin,
    /// User scope
    User,
}

/// Security role
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    
    /// Role description
    pub description: String,
    
    /// Role permissions
    pub permissions: Vec<String>,
}

/// Plugin sandbox configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum memory usage
    pub max_memory: Option<u64>,
    
    /// Maximum CPU usage
    pub max_cpu: Option<f64>,
    
    /// Maximum disk space
    pub max_disk: Option<u64>,
    
    /// Allowed network access
    pub network_access: bool,
    
    /// Allowed file system access
    pub filesystem_access: bool,
    
    /// Allowed environment variables
    pub allowed_env_vars: Vec<String>,
    
    /// Timeout
    pub timeout: Option<u64>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: Some(100 * 1024 * 1024), // 100 MB
            max_cpu: Some(0.5), // 50% of one CPU core
            max_disk: Some(10 * 1024 * 1024), // 10 MB
            network_access: false,
            filesystem_access: false,
            allowed_env_vars: Vec::new(),
            timeout: Some(5000), // 5 seconds
        }
    }
}

/// Resource usage tracking for plugins
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current memory usage
    pub memory_usage: u64,
    
    /// Current CPU usage
    pub cpu_usage: f64,
    
    /// Current disk usage
    pub disk_usage: u64,
    
    /// Network usage (bytes sent/received)
    pub network_usage: u64,
    
    /// Last updated timestamp
    pub last_updated: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            disk_usage: 0,
            network_usage: 0,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Security manager trait
#[async_trait]
pub trait SecurityManager: Send + Sync + Debug {
    /// Verify plugin security
    async fn verify_plugin(&self, plugin: &dyn Plugin) -> Result<()>;
    
    /// Check if a plugin has a permission
    async fn has_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool>;
    
    /// Grant a permission to a plugin
    async fn grant_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()>;
    
    /// Revoke a permission from a plugin
    async fn revoke_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()>;
    
    /// Get plugin permissions
    async fn get_plugin_permissions(&self, plugin_id: Uuid) -> Result<Vec<String>>;
    
    /// Create a plugin sandbox
    async fn create_sandbox(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()>;
    
    /// Destroy a plugin sandbox
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Check if a plugin is in a sandbox
    async fn is_sandboxed(&self, plugin_id: Uuid) -> Result<bool>;
    
    /// Get sandbox configuration
    async fn get_sandbox_config(&self, plugin_id: Uuid) -> Result<Option<SandboxConfig>>;
    
    /// Set sandbox configuration
    async fn set_sandbox_config(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()>;
    
    /// Verify plugin signature
    async fn verify_signature(&self, metadata: &PluginMetadata, signature: &[u8]) -> Result<bool>;
    
    /// Get plugin resource usage
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
    
    /// Update plugin resource usage
    async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()>;
    
    /// Check if plugin exceeds resource limits
    async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<bool>;
    
    /// Create a security report for a plugin
    async fn create_security_report(&self, plugin_id: Uuid) -> Result<SecurityReport>;
}

/// Security report for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Plugin name
    pub plugin_name: String,
    
    /// Permissions granted
    pub permissions: Vec<String>,
    
    /// Resource usage
    pub resource_usage: ResourceUsage,
    
    /// Security issues found
    pub security_issues: Vec<SecurityIssue>,
    
    /// Sandbox configuration
    pub sandbox_config: Option<SandboxConfig>,
    
    /// Is sandboxed
    pub is_sandboxed: bool,
    
    /// Security score (0-100)
    pub security_score: u8,
    
    /// Report timestamp
    pub timestamp: u64,
}

/// Security issue type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityIssueType {
    /// Excessive permissions
    ExcessivePermissions,
    
    /// Resource limit exceeded
    ResourceLimitExceeded,
    
    /// Untrusted source
    UntrustedSource,
    
    /// Missing signature
    MissingSignature,
    
    /// Invalid signature
    InvalidSignature,
    
    /// Potential malware
    PotentialMalware,
    
    /// Other issue
    Other(String),
}

/// Security issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Issue type
    pub issue_type: SecurityIssueType,
    
    /// Issue description
    pub description: String,
    
    /// Issue severity (0-100)
    pub severity: u8,
    
    /// Recommended action
    pub recommended_action: String,
}

/// Enhanced security manager
#[derive(Debug)]
pub struct EnhancedSecurityManager {
    /// Plugin permissions
    permissions: RwLock<HashMap<Uuid, HashSet<String>>>,
    
    /// Available permissions
    available_permissions: RwLock<HashMap<String, Permission>>,
    
    /// Plugin roles
    roles: RwLock<HashMap<Uuid, Vec<String>>>,
    
    /// Role definitions
    role_definitions: RwLock<HashMap<String, Role>>,
    
    /// Plugin sandbox configurations
    sandbox_configs: RwLock<HashMap<Uuid, SandboxConfig>>,
    
    /// Plugin resource usage
    resource_usage: RwLock<HashMap<Uuid, ResourceUsage>>,
    
    /// Plugin signatures
    signatures: RwLock<HashMap<Uuid, Vec<u8>>>,
    
    /// Security storage path
    storage_path: PathBuf,
    
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    
    /// Signature verifier
    signature_verifier: Arc<SignatureVerifier>,
}

impl EnhancedSecurityManager {
    /// Create a new enhanced security manager
    pub fn new() -> Self {
        let storage_path = PathBuf::from("./data/security");
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let signature_verifier = Arc::new(SignatureVerifier::new());
        
        Self {
            permissions: RwLock::new(HashMap::new()),
            available_permissions: RwLock::new(Self::default_permissions()),
            roles: RwLock::new(HashMap::new()),
            role_definitions: RwLock::new(Self::default_roles()),
            sandbox_configs: RwLock::new(HashMap::new()),
            resource_usage: RwLock::new(HashMap::new()),
            signatures: RwLock::new(HashMap::new()),
            storage_path,
            resource_monitor,
            signature_verifier,
        }
    }
    
    /// Create a new enhanced security manager with a custom storage path
    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let signature_verifier = Arc::new(SignatureVerifier::with_storage_dir(storage_path.join("signatures")));
        
        Self {
            permissions: RwLock::new(HashMap::new()),
            available_permissions: RwLock::new(Self::default_permissions()),
            roles: RwLock::new(HashMap::new()),
            role_definitions: RwLock::new(Self::default_roles()),
            sandbox_configs: RwLock::new(HashMap::new()),
            resource_usage: RwLock::new(HashMap::new()),
            signatures: RwLock::new(HashMap::new()),
            storage_path,
            resource_monitor,
            signature_verifier,
        }
    }
    
    /// Create default permissions
    fn default_permissions() -> HashMap<String, Permission> {
        let mut permissions = HashMap::new();
        
        // Define common permissions
        let file_read = Permission {
            name: "file:read".to_string(),
            description: "Permission to read files".to_string(),
            scope: PermissionScope::System,
            required_permissions: Vec::new(),
        };
        
        let file_write = Permission {
            name: "file:write".to_string(),
            description: "Permission to write files".to_string(),
            scope: PermissionScope::System,
            required_permissions: vec!["file:read".to_string()],
        };
        
        let network_connect = Permission {
            name: "network:connect".to_string(),
            description: "Permission to connect to the network".to_string(),
            scope: PermissionScope::System,
            required_permissions: Vec::new(),
        };
        
        let command_exec = Permission {
            name: "command:execute".to_string(),
            description: "Permission to execute commands".to_string(),
            scope: PermissionScope::System,
            required_permissions: Vec::new(),
        };
        
        let plugin_load = Permission {
            name: "plugin:load".to_string(),
            description: "Permission to load plugins".to_string(),
            scope: PermissionScope::System,
            required_permissions: Vec::new(),
        };
        
        // Add permissions to the map
        permissions.insert(file_read.name.clone(), file_read);
        permissions.insert(file_write.name.clone(), file_write);
        permissions.insert(network_connect.name.clone(), network_connect);
        permissions.insert(command_exec.name.clone(), command_exec);
        permissions.insert(plugin_load.name.clone(), plugin_load);
        
        permissions
    }
    
    /// Create default roles
    fn default_roles() -> HashMap<String, Role> {
        let mut roles = HashMap::new();
        
        // Define common roles
        let basic_role = Role {
            name: "basic".to_string(),
            description: "Basic role with minimal permissions".to_string(),
            permissions: vec!["file:read".to_string()],
        };
        
        let standard_role = Role {
            name: "standard".to_string(),
            description: "Standard role with common permissions".to_string(),
            permissions: vec![
                "file:read".to_string(),
                "file:write".to_string(),
                "network:connect".to_string(),
            ],
        };
        
        let admin_role = Role {
            name: "admin".to_string(),
            description: "Admin role with elevated permissions".to_string(),
            permissions: vec![
                "file:read".to_string(),
                "file:write".to_string(),
                "network:connect".to_string(),
                "command:execute".to_string(),
                "plugin:load".to_string(),
            ],
        };
        
        // Add roles to the map
        roles.insert(basic_role.name.clone(), basic_role);
        roles.insert(standard_role.name.clone(), standard_role);
        roles.insert(admin_role.name.clone(), admin_role);
        
        roles
    }
    
    /// Assign a role to a plugin
    pub async fn assign_role(&self, plugin_id: Uuid, role_name: &str) -> Result<()> {
        // Get the role definition
        let role_definitions = self.role_definitions.read().await;
        let role = role_definitions.get(role_name).ok_or_else(|| {
            anyhow::anyhow!("Role not found: {}", role_name)
        })?;
        
        // Grant all permissions from the role
        for permission in &role.permissions {
            self.grant_permission(plugin_id, permission).await?;
        }
        
        // Add the role to the plugin's roles
        let mut roles = self.roles.write().await;
        let plugin_roles = roles.entry(plugin_id).or_insert_with(Vec::new);
        if !plugin_roles.contains(&role_name.to_string()) {
            plugin_roles.push(role_name.to_string());
        }
        
        Ok(())
    }
    
    /// Check if a plugin has a role
    pub async fn has_role(&self, plugin_id: Uuid, role_name: &str) -> Result<bool> {
        let roles = self.roles.read().await;
        if let Some(plugin_roles) = roles.get(&plugin_id) {
            Ok(plugin_roles.contains(&role_name.to_string()))
        } else {
            Ok(false)
        }
    }
    
    /// Load security data from disk
    pub async fn load(&self) -> Result<()> {
        // Implement loading security data from disk
        // This would read permissions, roles, and sandbox configs from files
        
        // Example implementation:
        let permissions_path = self.storage_path.join("permissions.json");
        if permissions_path.exists() {
            let data = tokio::fs::read_to_string(&permissions_path).await?;
            let loaded_permissions: HashMap<Uuid, HashSet<String>> = serde_json::from_str(&data)?;
            
            let mut permissions = self.permissions.write().await;
            *permissions = loaded_permissions;
        }
        
        // Similar loading for other data would be implemented here
        
        Ok(())
    }
    
    /// Save security data to disk
    pub async fn save(&self) -> Result<()> {
        // Implement saving security data to disk
        // This would write permissions, roles, and sandbox configs to files
        
        // Example implementation:
        let permissions_path = self.storage_path.join("permissions.json");
        let permissions = self.permissions.read().await;
        let data = serde_json::to_string_pretty(&*permissions)?;
        
        tokio::fs::write(&permissions_path, data).await?;
        
        // Similar saving for other data would be implemented here
        
        Ok(())
    }
    
    /// Verify all security aspects of a plugin
    async fn verify_security(&self, plugin: &dyn Plugin) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();
        
        // Check for signature
        let metadata = plugin.metadata();
        let signatures = self.signatures.read().await;
        if let Some(signature) = signatures.get(&metadata.id) {
            if !self.verify_signature(metadata, signature).await? {
                issues.push(SecurityIssue {
                    issue_type: SecurityIssueType::InvalidSignature,
                    description: "Plugin signature is invalid".to_string(),
                    severity: 80,
                    recommended_action: "Verify the plugin source or resign the plugin".to_string(),
                });
            }
        } else {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::MissingSignature,
                description: "Plugin is not signed".to_string(),
                severity: 60,
                recommended_action: "Sign the plugin or use only trusted plugins".to_string(),
            });
        }
        
        // Check for excessive permissions
        let permissions = self.get_plugin_permissions(metadata.id).await?;
        if permissions.len() > 10 {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ExcessivePermissions,
                description: format!("Plugin has {} permissions granted", permissions.len()),
                severity: 40,
                recommended_action: "Review and reduce the plugin's permissions".to_string(),
            });
        }
        
        // Check if plugin exceeds resource limits
        if self.check_resource_limits(metadata.id).await? {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimitExceeded,
                description: "Plugin exceeds resource limits".to_string(),
                severity: 70,
                recommended_action: "Adjust resource limits or optimize the plugin".to_string(),
            });
        }
        
        Ok(issues)
    }

    /// Get the resource monitor
    pub fn resource_monitor(&self) -> Arc<ResourceMonitor> {
        self.resource_monitor.clone()
    }

    /// Get the signature verifier
    pub fn signature_verifier(&self) -> Arc<SignatureVerifier> {
        self.signature_verifier.clone()
    }
}

#[async_trait]
impl SecurityManager for EnhancedSecurityManager {
    async fn verify_plugin(&self, plugin: &dyn Plugin) -> Result<()> {
        // Check for security issues
        let issues = self.verify_security(plugin).await?;
        
        // If there are any high-severity issues, return an error
        for issue in &issues {
            if issue.severity > 70 {
                return Err(anyhow::anyhow!("Security verification failed: {}", issue.description));
            }
        }
        
        // Initialize resource tracking for this plugin
        let metadata = plugin.metadata();
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.entry(metadata.id).or_insert_with(ResourceUsage::default);
        
        Ok(())
    }
    
    async fn has_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool> {
        let permissions = self.permissions.read().await;
        if let Some(plugin_permissions) = permissions.get(&plugin_id) {
            Ok(plugin_permissions.contains(permission))
        } else {
            Ok(false)
        }
    }
    
    async fn grant_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        // Check if the permission exists
        let available_permissions = self.available_permissions.read().await;
        if !available_permissions.contains_key(permission) {
            return Err(anyhow::anyhow!("Permission does not exist: {}", permission));
        }
        
        // Grant the permission
        let mut permissions = self.permissions.write().await;
        let plugin_permissions = permissions.entry(plugin_id).or_insert_with(HashSet::new);
        plugin_permissions.insert(permission.to_string());
        
        // Also grant any required permissions
        if let Some(perm_def) = available_permissions.get(permission) {
            for req_perm in &perm_def.required_permissions {
                let plugin_permissions = permissions.entry(plugin_id).or_insert_with(HashSet::new);
                plugin_permissions.insert(req_perm.clone());
            }
        }
        
        // Save changes to disk
        drop(permissions);
        self.save().await?;
        
        Ok(())
    }
    
    async fn revoke_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        if let Some(plugin_permissions) = permissions.get_mut(&plugin_id) {
            plugin_permissions.remove(permission);
        }
        
        // Save changes to disk
        drop(permissions);
        self.save().await?;
        
        Ok(())
    }
    
    async fn get_plugin_permissions(&self, plugin_id: Uuid) -> Result<Vec<String>> {
        let permissions = self.permissions.read().await;
        if let Some(plugin_permissions) = permissions.get(&plugin_id) {
            Ok(plugin_permissions.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn create_sandbox(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        // Store the sandbox configuration
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config.clone());
        
        // Also update the resource monitor
        self.resource_monitor.set_sandbox_config(plugin_id, config).await?;
        
        // In a real implementation, this would create the actual sandbox
        // For example, using OS features or containers
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        // Remove the sandbox configuration
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.remove(&plugin_id);
        
        // Also update the resource monitor
        self.resource_monitor.remove_sandbox_config(&plugin_id).await?;
        
        // In a real implementation, this would destroy the actual sandbox
        
        Ok(())
    }
    
    async fn is_sandboxed(&self, plugin_id: Uuid) -> Result<bool> {
        let sandbox_configs = self.sandbox_configs.read().await;
        Ok(sandbox_configs.contains_key(&plugin_id))
    }
    
    async fn get_sandbox_config(&self, plugin_id: Uuid) -> Result<Option<SandboxConfig>> {
        let sandbox_configs = self.sandbox_configs.read().await;
        Ok(sandbox_configs.get(&plugin_id).cloned())
    }
    
    async fn set_sandbox_config(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        // Save changes to disk
        drop(sandbox_configs);
        self.save().await?;
        
        Ok(())
    }
    
    async fn verify_signature(&self, metadata: &PluginMetadata, signature: &[u8]) -> Result<bool> {
        // Store the signature bytes for later use
        let mut signatures = self.signatures.write().await;
        signatures.insert(metadata.id, signature.to_vec());
        
        // Create a PluginSignature object
        let plugin_signature = PluginSignature {
            plugin_id: metadata.id,
            signature: signature.to_vec(),
            algorithm: SignatureAlgorithm::Ed25519,
            public_key: Vec::new(), // This would be retrieved from metadata in a real implementation
            signer: "Unknown".to_string(), // This would be retrieved from metadata in a real implementation
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            scope: SignatureScope::Metadata,
        };
        
        // Register the signature with the verifier
        if let Err(e) = self.signature_verifier.register_signature(plugin_signature).await {
            log::error!("Failed to register signature: {}", e);
            return Ok(false);
        }
        
        // Verify the signature
        match self.signature_verifier.verify_plugin_signature(metadata, None).await {
            Ok(result) => Ok(result.valid),
            Err(e) => {
                log::error!("Signature verification error: {}", e);
                Ok(false)
            }
        }
    }
    
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // Try to get usage from the resource monitor
        match self.resource_monitor.get_resource_stats(plugin_id).await {
            Ok(stats) => Ok(stats.current_usage),
            Err(_) => {
                // Fall back to our internal tracking
                let resource_usage = self.resource_usage.read().await;
                Ok(resource_usage.get(&plugin_id).cloned().unwrap_or_default())
            }
        }
    }
    
    async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()> {
        // Update both our internal tracking and the resource monitor
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.insert(plugin_id, usage.clone());
        
        // Also update the resource monitor
        self.resource_monitor.update_resource_usage(plugin_id, usage).await?;
        
        Ok(())
    }
    
    async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<bool> {
        // Sample current usage
        let usage = self.resource_monitor.sample_plugin_resource_usage(plugin_id).await?;
        
        // Update the resource usage
        self.update_resource_usage(plugin_id, usage).await?;
        
        // Check if any plugins need to be stopped
        let plugins_to_stop = self.resource_monitor.get_plugins_to_stop().await?;
        let exceeded = plugins_to_stop.contains(&plugin_id);
        
        Ok(exceeded)
    }
    
    async fn create_security_report(&self, plugin_id: Uuid) -> Result<SecurityReport> {
        // Get the plugin information
        let permissions = self.get_plugin_permissions(plugin_id).await?;
        let resource_usage = self.get_resource_usage(plugin_id).await?;
        let sandbox_config = self.get_sandbox_config(plugin_id).await?;
        let is_sandboxed = self.is_sandboxed(plugin_id).await?;
        
        // For a real report, we would need the plugin object to check more things
        // This is a simplified version
        let plugin_name = format!("Plugin {}", plugin_id);
        
        // Calculate a security score based on permissions, sandboxing, etc.
        let mut security_score: u8 = 100;
        
        // More permissions = lower score
        security_score = security_score.saturating_sub((permissions.len() as u8).min(50));
        
        // Not sandboxed = lower score
        if !is_sandboxed {
            security_score = security_score.saturating_sub(30);
        }
        
        // Create the report
        let report = SecurityReport {
            plugin_id,
            plugin_name,
            permissions,
            resource_usage,
            security_issues: Vec::new(), // Would normally be populated
            sandbox_config,
            is_sandboxed,
            security_score,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        Ok(report)
    }
}

impl Default for EnhancedSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

// Still provide the old DefaultSecurityManager for backward compatibility
/// Default security manager implementation (legacy)
#[derive(Debug, Clone, Copy)]
pub struct DefaultSecurityManager {
    // Implementation details
}

impl DefaultSecurityManager {
    /// Create a new default security manager
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SecurityManager for DefaultSecurityManager {
    async fn verify_plugin(&self, _plugin: &dyn Plugin) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn has_permission(&self, _plugin_id: Uuid, _permission: &str) -> Result<bool> {
        // Basic implementation
        Ok(true)
    }
    
    async fn grant_permission(&self, _plugin_id: Uuid, _permission: &str) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn revoke_permission(&self, _plugin_id: Uuid, _permission: &str) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn get_plugin_permissions(&self, _plugin_id: Uuid) -> Result<Vec<String>> {
        // Basic implementation
        Ok(Vec::new())
    }
    
    async fn create_sandbox(&self, _plugin_id: Uuid, _config: SandboxConfig) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn destroy_sandbox(&self, _plugin_id: Uuid) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn is_sandboxed(&self, _plugin_id: Uuid) -> Result<bool> {
        // Basic implementation
        Ok(false)
    }
    
    async fn get_sandbox_config(&self, _plugin_id: Uuid) -> Result<Option<SandboxConfig>> {
        // Basic implementation
        Ok(None)
    }
    
    async fn set_sandbox_config(&self, _plugin_id: Uuid, _config: SandboxConfig) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn verify_signature(&self, _metadata: &PluginMetadata, _signature: &[u8]) -> Result<bool> {
        // Basic implementation
        Ok(true)
    }
    
    async fn get_resource_usage(&self, _plugin_id: Uuid) -> Result<ResourceUsage> {
        // Basic implementation
        Ok(ResourceUsage::default())
    }
    
    async fn update_resource_usage(&self, _plugin_id: Uuid, _usage: ResourceUsage) -> Result<()> {
        // Basic implementation
        Ok(())
    }
    
    async fn check_resource_limits(&self, _plugin_id: Uuid) -> Result<bool> {
        // Basic implementation
        Ok(false)
    }
    
    async fn create_security_report(&self, plugin_id: Uuid) -> Result<SecurityReport> {
        // Basic implementation
        Ok(SecurityReport {
            plugin_id,
            plugin_name: format!("Plugin {}", plugin_id),
            permissions: Vec::new(),
            resource_usage: ResourceUsage::default(),
            security_issues: Vec::new(),
            sandbox_config: None,
            is_sandboxed: false,
            security_score: 50,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
} 