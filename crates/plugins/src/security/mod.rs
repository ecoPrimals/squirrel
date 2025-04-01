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
#[cfg(feature = "mcp")]
use squirrel_mcp::security::{
    SecurityManager as MCPSecurityManager, // Import as a type alias
    BasicRBACManager, 
    DefaultAuditService, 
    DefaultCryptoProvider, 
    DefaultIdentityManager,
    DefaultTokenManager, 
    InMemoryKeyStorage,
};

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

/// Resource usage limits for plugins
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in bytes (0 = no limit)
    pub memory_limit: u64,
    
    /// CPU usage limit (0.0 = no limit)
    pub cpu_limit: f64,
    
    /// Disk usage limit in bytes (0 = no limit)
    pub disk_limit: u64,
    
    /// Network usage limit in bytes (0 = no limit)
    pub network_limit: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit: 100 * 1024 * 1024, // 100 MB
            cpu_limit: 0.5, // 50% of one CPU core
            disk_limit: 10 * 1024 * 1024, // 10 MB
            network_limit: 1024 * 1024, // 1 MB
        }
    }
}

impl ResourceUsage {
    /// Check if this resource usage exceeds the given limits
    pub fn exceeds_limits(&self, limits: &ResourceLimits) -> bool {
        (limits.memory_limit > 0 && self.memory_usage > limits.memory_limit) ||
        (limits.cpu_limit > 0.0 && self.cpu_usage > limits.cpu_limit) ||
        (limits.disk_limit > 0 && self.disk_usage > limits.disk_limit) ||
        (limits.network_limit > 0 && self.network_usage > limits.network_limit)
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

/// Security manager adapter for non-MCP builds
#[cfg(not(feature = "mcp"))]
#[derive(Debug)]
pub struct SecurityManagerAdapter {
    /// Plugin permissions
    permissions: RwLock<HashMap<Uuid, HashSet<String>>>,
    
    /// Plugin sandbox configurations
    sandbox_configs: RwLock<HashMap<Uuid, SandboxConfig>>,
    
    /// Plugin resource usage
    resource_usage: RwLock<HashMap<Uuid, ResourceUsage>>,
    
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    
    /// Signature verifier
    signature_verifier: Arc<SignatureVerifier>,
}

#[cfg(not(feature = "mcp"))]
impl SecurityManagerAdapter {
    /// Create a default security manager adapter
    pub fn default() -> Self {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let signature_verifier = Arc::new(SignatureVerifier::new());
        
        Self {
            permissions: RwLock::new(HashMap::new()),
            sandbox_configs: RwLock::new(HashMap::new()),
            resource_usage: RwLock::new(HashMap::new()),
            resource_monitor,
            signature_verifier,
        }
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

/// Plugin security manager implementation
#[cfg(feature = "mcp")]
pub struct SecurityManagerAdapter {
    /// MCP security manager implementation
    inner: Arc<dyn MCPSecurityManager>,
    
    /// Plugin permissions
    permissions: RwLock<HashMap<Uuid, HashSet<String>>>,
    
    /// Plugin sandbox configurations
    sandbox_configs: RwLock<HashMap<Uuid, SandboxConfig>>,
    
    /// Plugin resource usage
    resource_usage: RwLock<HashMap<Uuid, ResourceUsage>>,
    
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    
    /// Signature verifier
    signature_verifier: Arc<SignatureVerifier>,
}

#[cfg(feature = "mcp")]
impl SecurityManagerAdapter {
    /// Create a new security manager adapter with the provided MCP security manager
    pub fn new(mcp_security_manager: Arc<dyn MCPSecurityManager>) -> Self {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let signature_verifier = Arc::new(SignatureVerifier::new());
        
        Self {
            inner: mcp_security_manager,
            permissions: RwLock::new(HashMap::new()),
            sandbox_configs: RwLock::new(HashMap::new()),
            resource_usage: RwLock::new(HashMap::new()),
            resource_monitor,
            signature_verifier,
        }
    }
    
    /// Create a default security manager adapter
    pub fn default() -> Self {
        // Create default implementations of all required components
        let key_storage = Arc::new(InMemoryKeyStorage::new());
        let crypto_provider = Arc::new(DefaultCryptoProvider::new());
        let token_manager = Arc::new(DefaultTokenManager::new(
            key_storage.clone(),
            crypto_provider.clone(),
        ));
        let identity_manager = Arc::new(DefaultIdentityManager::new());
        let rbac_manager = Arc::new(BasicRBACManager::new());
        let audit_service = Arc::new(DefaultAuditService::new());
        
        // Import the SecurityManagerImpl type just for this function
        #[cfg(feature = "mcp")]
        use squirrel_mcp::security::SecurityManagerImpl;
        
        // Create the MCP security manager with these components
        let inner = Arc::new(SecurityManagerImpl::new(
            crypto_provider,
            token_manager,
            identity_manager,
            rbac_manager,
            audit_service,
        )) as Arc<dyn MCPSecurityManager>;
        
        Self::new(inner)
    }
    
    /// Get the inner MCP security manager
    pub fn inner(&self) -> Arc<dyn MCPSecurityManager> {
        self.inner.clone()
    }
}

#[async_trait]
impl SecurityManager for SecurityManagerAdapter {
    async fn verify_plugin(&self, plugin: &dyn Plugin) -> Result<()> {
        // Perform basic verification
        if plugin.metadata().id.is_nil() {
            return Err(anyhow::anyhow!("Plugin ID is nil"));
        }
        
        Ok(())
    }
    
    async fn has_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool> {
        let permissions = self.permissions.read().await;
        let has_perm = permissions
            .get(&plugin_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false);
        
        Ok(has_perm)
    }
    
    async fn grant_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        let plugin_perms = permissions.entry(plugin_id).or_insert_with(HashSet::new);
        plugin_perms.insert(permission.to_string());
        
        Ok(())
    }
    
    async fn revoke_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        if let Some(plugin_perms) = permissions.get_mut(&plugin_id) {
            plugin_perms.remove(permission);
        }
        
        Ok(())
    }
    
    async fn get_plugin_permissions(&self, plugin_id: Uuid) -> Result<Vec<String>> {
        let permissions = self.permissions.read().await;
        let perms = permissions
            .get(&plugin_id)
            .map(|perms| perms.iter().cloned().collect())
            .unwrap_or_default();
        
        Ok(perms)
    }
    
    async fn create_sandbox(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.remove(&plugin_id);
        
        Ok(())
    }
    
    async fn is_sandboxed(&self, plugin_id: Uuid) -> Result<bool> {
        let sandbox_configs = self.sandbox_configs.read().await;
        Ok(sandbox_configs.contains_key(&plugin_id))
    }
    
    async fn get_sandbox_config(&self, plugin_id: Uuid) -> Result<Option<SandboxConfig>> {
        let sandbox_configs = self.sandbox_configs.read().await;
        let config = sandbox_configs.get(&plugin_id).cloned();
        
        Ok(config)
    }
    
    async fn set_sandbox_config(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        Ok(())
    }
    
    async fn verify_signature(&self, metadata: &PluginMetadata, signature: &[u8]) -> Result<bool> {
        // Use the signature verifier to verify the signature
        self.signature_verifier.verify(metadata, signature).await
    }
    
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        let resource_usage = self.resource_usage.read().await;
        let usage = resource_usage
            .get(&plugin_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(usage)
    }
    
    async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()> {
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.insert(plugin_id, usage);
        
        Ok(())
    }
    
    async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<bool> {
        // Get current resource usage
        let usage = self.get_resource_usage(plugin_id).await?;
        
        // Get sandbox config
        let sandbox_config = match self.get_sandbox_config(plugin_id).await? {
            Some(config) => config,
            None => return Ok(true), // No limits if not sandboxed
        };
        
        // Create resource limits from sandbox config
        let limits = ResourceLimits {
            memory_limit: sandbox_config.max_memory.unwrap_or(0),
            cpu_limit: sandbox_config.max_cpu.unwrap_or(0.0),
            disk_limit: sandbox_config.max_disk.unwrap_or(0),
            network_limit: if sandbox_config.network_access { 0 } else { 1 }, // Limit to 1 byte if no network access
        };
        
        // Check if usage exceeds limits
        Ok(!usage.exceeds_limits(&limits))
    }
    
    async fn create_security_report(&self, plugin_id: Uuid) -> Result<SecurityReport> {
        let permissions = self.get_plugin_permissions(plugin_id).await?;
        let resource_usage = self.get_resource_usage(plugin_id).await?;
        let sandbox_config = self.get_sandbox_config(plugin_id).await?;
        let is_sandboxed = self.is_sandboxed(plugin_id).await?;
        
        // Collect security issues
        let mut security_issues = Vec::new();
        
        // Check resource limits
        if !self.check_resource_limits(plugin_id).await? {
            security_issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimitExceeded,
                description: "Plugin exceeds resource limits".to_string(),
                severity: 70,
                recommended_action: "Adjust resource limits or optimize plugin".to_string(),
            });
        }
        
        // Calculate security score (simple implementation)
        let security_score = if security_issues.is_empty() {
            100
        } else {
            let total_severity: u32 = security_issues.iter()
                .map(|issue: &SecurityIssue| u32::from(issue.severity))
                .sum();
            let score = 100u8.saturating_sub((total_severity / security_issues.len() as u32) as u8);
            score.max(1) // Minimum score of 1
        };
        
        Ok(SecurityReport {
            plugin_id,
            plugin_name: "Unknown".to_string(), // This would be set by the caller
            permissions,
            resource_usage,
            security_issues,
            sandbox_config,
            is_sandboxed,
            security_score,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
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
    
    /// Register a signature
    pub async fn register_signature(&self, plugin_id: Uuid, signature: Vec<u8>) -> Result<()> {
        let mut signatures = self.signatures.write().await;
        signatures.insert(plugin_id, signature);
        Ok(())
    }
    
    /// Check if a plugin has a signature
    pub async fn has_signature(&self, plugin_id: Uuid) -> Result<bool> {
        let signatures = self.signatures.read().await;
        Ok(signatures.contains_key(&plugin_id))
    }
    
    /// Get a plugin's signature
    pub async fn get_signature(&self, plugin_id: Uuid) -> Result<Option<Vec<u8>>> {
        let signatures = self.signatures.read().await;
        Ok(signatures.get(&plugin_id).cloned())
    }
    
    /// Get the resource monitor
    pub fn resource_monitor(&self) -> Arc<ResourceMonitor> {
        self.resource_monitor.clone()
    }

    /// Get the signature verifier
    pub fn signature_verifier(&self) -> Arc<SignatureVerifier> {
        self.signature_verifier.clone()
    }
    
    /// Get the available permissions
    pub async fn available_permissions(&self) -> HashMap<String, Permission> {
        self.available_permissions.read().await.clone()
    }
    
    /// Register a new permission
    pub async fn register_permission(&self, permission: Permission) -> Result<()> {
        let mut available_permissions = self.available_permissions.write().await;
        available_permissions.insert(permission.name.clone(), permission);
        Ok(())
    }
}

impl Default for EnhancedSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecurityManager for EnhancedSecurityManager {
    async fn verify_plugin(&self, plugin: &dyn Plugin) -> Result<()> {
        // Perform basic verification
        if plugin.metadata().id.is_nil() {
            return Err(anyhow::anyhow!("Plugin ID is nil"));
        }
        
        Ok(())
    }
    
    async fn has_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool> {
        let permissions = self.permissions.read().await;
        let has_perm = permissions
            .get(&plugin_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false);
        
        Ok(has_perm)
    }
    
    async fn grant_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        let plugin_perms = permissions.entry(plugin_id).or_insert_with(HashSet::new);
        plugin_perms.insert(permission.to_string());
        
        Ok(())
    }
    
    async fn revoke_permission(&self, plugin_id: Uuid, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        if let Some(plugin_perms) = permissions.get_mut(&plugin_id) {
            plugin_perms.remove(permission);
        }
        
        Ok(())
    }
    
    async fn get_plugin_permissions(&self, plugin_id: Uuid) -> Result<Vec<String>> {
        let permissions = self.permissions.read().await;
        let perms = permissions
            .get(&plugin_id)
            .map(|perms| perms.iter().cloned().collect())
            .unwrap_or_default();
        
        Ok(perms)
    }
    
    async fn create_sandbox(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.remove(&plugin_id);
        
        Ok(())
    }
    
    async fn is_sandboxed(&self, plugin_id: Uuid) -> Result<bool> {
        let sandbox_configs = self.sandbox_configs.read().await;
        Ok(sandbox_configs.contains_key(&plugin_id))
    }
    
    async fn get_sandbox_config(&self, plugin_id: Uuid) -> Result<Option<SandboxConfig>> {
        let sandbox_configs = self.sandbox_configs.read().await;
        let config = sandbox_configs.get(&plugin_id).cloned();
        
        Ok(config)
    }
    
    async fn set_sandbox_config(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        Ok(())
    }
    
    async fn verify_signature(&self, metadata: &PluginMetadata, signature: &[u8]) -> Result<bool> {
        self.signature_verifier.verify(metadata, signature).await
    }
    
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        let resource_usage = self.resource_usage.read().await;
        let usage = resource_usage
            .get(&plugin_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(usage)
    }
    
    async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()> {
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.insert(plugin_id, usage);
        
        Ok(())
    }
    
    async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<bool> {
        // Get current resource usage
        let usage = self.get_resource_usage(plugin_id).await?;
        
        // Get sandbox config
        let sandbox_config = match self.get_sandbox_config(plugin_id).await? {
            Some(config) => config,
            None => return Ok(true), // No limits if not sandboxed
        };
        
        // Create resource limits from sandbox config
        let limits = ResourceLimits {
            memory_limit: sandbox_config.max_memory.unwrap_or(0),
            cpu_limit: sandbox_config.max_cpu.unwrap_or(0.0),
            disk_limit: sandbox_config.max_disk.unwrap_or(0),
            network_limit: if sandbox_config.network_access { 0 } else { 1 }, // Limit to 1 byte if no network access
        };
        
        // Check if usage exceeds limits
        Ok(!usage.exceeds_limits(&limits))
    }
    
    async fn create_security_report(&self, plugin_id: Uuid) -> Result<SecurityReport> {
        let permissions = self.get_plugin_permissions(plugin_id).await?;
        let resource_usage = self.get_resource_usage(plugin_id).await?;
        let sandbox_config = self.get_sandbox_config(plugin_id).await?;
        let is_sandboxed = self.is_sandboxed(plugin_id).await?;
        
        // Create a list of security issues
        let mut security_issues = Vec::new();
        
        // Check for resource limits
        if let Some(config) = &sandbox_config {
            // Create resource limits from sandbox config
            let limits = ResourceLimits {
                memory_limit: config.max_memory.unwrap_or(0),
                cpu_limit: config.max_cpu.unwrap_or(0.0),
                disk_limit: config.max_disk.unwrap_or(0),
                network_limit: if config.network_access { 0 } else { 1 }, // Limit to 1 byte if no network access
            };
            
            // Check if usage exceeds limits
            if resource_usage.exceeds_limits(&limits) {
                security_issues.push(SecurityIssue {
                    issue_type: SecurityIssueType::ResourceLimitExceeded,
                    description: "Plugin exceeds resource limits".to_string(),
                    severity: 70,
                    recommended_action: "Adjust resource limits or optimize plugin".to_string(),
                });
            }
        }
        
        // Check for excessive permissions
        if permissions.len() > 10 {
            security_issues.push(SecurityIssue {
                issue_type: SecurityIssueType::ExcessivePermissions,
                description: format!("Plugin has {} permissions granted", permissions.len()),
                severity: 40,
                recommended_action: "Review and reduce the plugin's permissions".to_string(),
            });
        }
        
        // Calculate security score (simple implementation)
        let security_score = if security_issues.is_empty() {
            100
        } else {
            let total_severity: u32 = security_issues.iter()
                .map(|issue: &SecurityIssue| u32::from(issue.severity))
                .sum();
            let score = 100u8.saturating_sub((total_severity / security_issues.len() as u32) as u8);
            score.max(1) // Minimum score of 1
        };
        
        Ok(SecurityReport {
            plugin_id,
            plugin_name: "Unknown".to_string(), // This would be set by the caller
            permissions,
            resource_usage,
            security_issues,
            sandbox_config,
            is_sandboxed,
            security_score,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
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

/// Create a security manager adapter with the provided MCP security manager
#[cfg(feature = "mcp")]
pub fn with_mcp_security_manager(mcp_security_manager: Arc<dyn SecurityManager>) -> SecurityManagerAdapter {
    SecurityManagerAdapter::new(mcp_security_manager)
} 