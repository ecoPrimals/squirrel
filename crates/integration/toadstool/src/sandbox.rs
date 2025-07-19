//! Sandbox policy abstraction for Toadstool integration

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Security policy for sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Policy identifier
    pub policy_id: String,
    /// Security level (strict, standard, permissive)
    pub security_level: SecurityLevel,
    /// Allowed permissions
    pub permissions: HashSet<SandboxPermission>,
    /// Network restrictions
    pub network_policy: NetworkPolicy,
    /// File system restrictions
    pub filesystem_policy: FilesystemPolicy,
    /// System call restrictions
    pub syscall_policy: SyscallPolicy,
}

/// Security levels for sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Strictest security - minimal permissions
    Strict,
    /// Standard security - common permissions
    Standard,
    /// Permissive security - broader permissions
    Permissive,
    /// Custom security - user-defined permissions
    Custom,
}

/// Sandbox permissions that can be granted
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SandboxPermission {
    /// Allow network access
    NetworkAccess,
    /// Allow file system read access
    FilesystemRead,
    /// Allow file system write access
    FilesystemWrite,
    /// Allow process execution
    ProcessExecution,
    /// Allow inter-process communication
    InterProcessCommunication,
    /// Allow device access
    DeviceAccess,
    /// Allow system information access
    SystemInformation,
    /// Allow environment variable access
    EnvironmentVariables,
    /// Allow temporary file creation
    TemporaryFiles,
    /// Allow signal handling
    SignalHandling,
}

/// Network access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Whether network access is allowed
    pub allow_network: bool,
    /// Allowed domains (if network access is enabled)
    pub allowed_domains: HashSet<String>,
    /// Blocked domains
    pub blocked_domains: HashSet<String>,
    /// Allowed ports
    pub allowed_ports: HashSet<u16>,
    /// Maximum concurrent connections
    pub max_connections: Option<u32>,
}

/// File system access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemPolicy {
    /// Whether file system access is allowed
    pub allow_filesystem: bool,
    /// Allowed read paths
    pub allowed_read_paths: HashSet<String>,
    /// Allowed write paths
    pub allowed_write_paths: HashSet<String>,
    /// Blocked paths
    pub blocked_paths: HashSet<String>,
    /// Maximum file size for operations
    pub max_file_size: Option<u64>,
    /// Maximum number of open files
    pub max_open_files: Option<u32>,
}

/// System call policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallPolicy {
    /// Allowed system calls
    pub allowed_syscalls: HashSet<String>,
    /// Blocked system calls
    pub blocked_syscalls: HashSet<String>,
    /// Default action for unlisted syscalls
    pub default_action: SyscallAction,
}

/// Action to take for system calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyscallAction {
    /// Allow the system call
    Allow,
    /// Block the system call
    Block,
    /// Log the system call and allow
    LogAndAllow,
    /// Log the system call and block
    LogAndBlock,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            policy_id: "default".to_string(),
            security_level: SecurityLevel::Standard,
            permissions: Self::default_permissions(),
            network_policy: NetworkPolicy::default(),
            filesystem_policy: FilesystemPolicy::default(),
            syscall_policy: SyscallPolicy::default(),
        }
    }
}

impl SandboxPolicy {
    /// Create a new sandbox policy
    pub fn new(policy_id: String, security_level: SecurityLevel) -> Self {
        let permissions = match security_level {
            SecurityLevel::Strict => Self::strict_permissions(),
            SecurityLevel::Standard => Self::default_permissions(),
            SecurityLevel::Permissive => Self::permissive_permissions(),
            SecurityLevel::Custom => HashSet::new(),
        };

        Self {
            policy_id,
            security_level,
            permissions,
            network_policy: NetworkPolicy::default(),
            filesystem_policy: FilesystemPolicy::default(),
            syscall_policy: SyscallPolicy::default(),
        }
    }

    /// Get strict permissions
    fn strict_permissions() -> HashSet<SandboxPermission> {
        let mut permissions = HashSet::new();
        permissions.insert(SandboxPermission::TemporaryFiles);
        permissions
    }

    /// Get default permissions
    fn default_permissions() -> HashSet<SandboxPermission> {
        let mut permissions = HashSet::new();
        permissions.insert(SandboxPermission::FilesystemRead);
        permissions.insert(SandboxPermission::TemporaryFiles);
        permissions.insert(SandboxPermission::EnvironmentVariables);
        permissions.insert(SandboxPermission::SystemInformation);
        permissions
    }

    /// Get permissive permissions
    fn permissive_permissions() -> HashSet<SandboxPermission> {
        let mut permissions = HashSet::new();
        permissions.insert(SandboxPermission::NetworkAccess);
        permissions.insert(SandboxPermission::FilesystemRead);
        permissions.insert(SandboxPermission::FilesystemWrite);
        permissions.insert(SandboxPermission::ProcessExecution);
        permissions.insert(SandboxPermission::TemporaryFiles);
        permissions.insert(SandboxPermission::EnvironmentVariables);
        permissions.insert(SandboxPermission::SystemInformation);
        permissions
    }

    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &SandboxPermission) -> bool {
        self.permissions.contains(permission)
    }

    /// Add a permission
    pub fn add_permission(&mut self, permission: SandboxPermission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission
    pub fn remove_permission(&mut self, permission: &SandboxPermission) {
        self.permissions.remove(permission);
    }
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            allow_network: false,
            allowed_domains: HashSet::new(),
            blocked_domains: HashSet::new(),
            allowed_ports: HashSet::new(),
            max_connections: Some(10),
        }
    }
}

impl Default for FilesystemPolicy {
    fn default() -> Self {
        Self {
            allow_filesystem: true,
            allowed_read_paths: HashSet::new(),
            allowed_write_paths: HashSet::new(),
            blocked_paths: HashSet::new(),
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_open_files: Some(100),
        }
    }
}

impl Default for SyscallPolicy {
    fn default() -> Self {
        Self {
            allowed_syscalls: HashSet::new(),
            blocked_syscalls: HashSet::new(),
            default_action: SyscallAction::Allow,
        }
    }
}
