//! Seccomp configuration for Linux sandbox

use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

use crate::plugin::security::{SecurityContext, PermissionLevel};

/// Seccomp filter configuration
#[derive(Debug, Clone)]
pub struct SeccompConfig {
    /// Plugin ID that this filter is for
    pub(crate) plugin_id: Uuid,
    /// System calls that are allowed
    pub(crate) allowed_syscalls: HashSet<String>,
    /// Whether the filter is applied
    pub(crate) is_applied: bool,
    /// Filter action for unmatched syscalls (e.g. "kill", "trap", "errno")
    pub(crate) default_action: String,
    /// Path to the BPF program if generated
    pub(crate) bpf_path: Option<PathBuf>,
    /// Whether to log syscalls for debugging
    pub(crate) log_syscalls: bool,
}

impl SeccompConfig {
    /// Create a default seccomp configuration
    pub fn default(plugin_id: Uuid) -> Self {
        // Default allowed syscalls for basic operation
        let mut allowed_syscalls = HashSet::new();
        
        // Basic process operations
        allowed_syscalls.insert("read".to_string());
        allowed_syscalls.insert("write".to_string());
        allowed_syscalls.insert("open".to_string());
        allowed_syscalls.insert("close".to_string());
        allowed_syscalls.insert("stat".to_string());
        allowed_syscalls.insert("fstat".to_string());
        allowed_syscalls.insert("lstat".to_string());
        allowed_syscalls.insert("poll".to_string());
        allowed_syscalls.insert("lseek".to_string());
        allowed_syscalls.insert("mmap".to_string());
        allowed_syscalls.insert("mprotect".to_string());
        allowed_syscalls.insert("munmap".to_string());
        allowed_syscalls.insert("brk".to_string());
        allowed_syscalls.insert("rt_sigaction".to_string());
        allowed_syscalls.insert("rt_sigprocmask".to_string());
        allowed_syscalls.insert("ioctl".to_string());
        allowed_syscalls.insert("pread64".to_string());
        allowed_syscalls.insert("pwrite64".to_string());
        allowed_syscalls.insert("readv".to_string());
        allowed_syscalls.insert("writev".to_string());
        allowed_syscalls.insert("access".to_string());
        allowed_syscalls.insert("pipe".to_string());
        allowed_syscalls.insert("select".to_string());
        allowed_syscalls.insert("sched_yield".to_string());
        allowed_syscalls.insert("mremap".to_string());
        allowed_syscalls.insert("msync".to_string());
        allowed_syscalls.insert("mincore".to_string());
        allowed_syscalls.insert("madvise".to_string());
        allowed_syscalls.insert("pause".to_string());
        allowed_syscalls.insert("nanosleep".to_string());
        allowed_syscalls.insert("getitimer".to_string());
        allowed_syscalls.insert("alarm".to_string());
        allowed_syscalls.insert("setitimer".to_string());
        allowed_syscalls.insert("getpid".to_string());
        allowed_syscalls.insert("sendfile".to_string());
        allowed_syscalls.insert("socket".to_string());
        allowed_syscalls.insert("connect".to_string());
        allowed_syscalls.insert("exit".to_string());
        allowed_syscalls.insert("exit_group".to_string());
        allowed_syscalls.insert("gettid".to_string());
        allowed_syscalls.insert("futex".to_string());
        allowed_syscalls.insert("getdents64".to_string());
        allowed_syscalls.insert("clock_gettime".to_string());
        allowed_syscalls.insert("clock_nanosleep".to_string());
        allowed_syscalls.insert("sysinfo".to_string());
        allowed_syscalls.insert("uname".to_string());
        allowed_syscalls.insert("memfd_create".to_string());
        
        Self {
            plugin_id,
            allowed_syscalls,
            is_applied: false,
            default_action: "errno".to_string(),  // Less restrictive default
            bpf_path: None,
            log_syscalls: cfg!(debug_assertions),
        }
    }
    
    /// Create a seccomp configuration with appropriate permissions based on security context
    pub fn from_security_context(plugin_id: Uuid, context: &SecurityContext) -> Self {
        let mut config = Self::default(plugin_id);
        
        // Set default action based on permission level
        match context.permission_level {
            PermissionLevel::System => {
                // System level gets very permissive seccomp (practically no restrictions)
                config.default_action = "log".to_string();
                
                // Allow all syscalls by adding common ones, kernel will allow unlisted ones
                // due to permissive default_action
                config.allow_all_operations();
            },
            PermissionLevel::User => {
                // User level gets most operations but with restrictions
                config.default_action = "errno".to_string();
                
                // Add various operations based on capabilities
                if context.allowed_capabilities.contains("file:read") || 
                   context.allowed_capabilities.contains("file:write") {
                    config.allow_file_operations();
                }
                
                if context.allowed_capabilities.contains("network:connect") || 
                   context.allowed_capabilities.contains("network:listen") {
                    config.allow_network();
                }
                
                if context.allowed_capabilities.contains("system:resources") {
                    config.allow_process_management();
                }
                
                if context.allowed_capabilities.contains("plugin:execute") {
                    config.allow_execution();
                }
            },
            PermissionLevel::Restricted => {
                // Restricted level gets strict seccomp with kill for dangerous syscalls
                config.default_action = "errno".to_string();
                
                // Only add operation categories explicitly allowed by capabilities
                if context.allowed_capabilities.contains("file:read") {
                    config.allow_read_operations();
                }
                
                if context.allowed_capabilities.contains("network:connect") {
                    config.allow_client_network();
                }
                
                if context.allowed_capabilities.contains("plugin:execute") {
                    config.allow_limited_execution();
                }
            }
        }
        
        config
    }
    
    /// Add allowed syscalls for network operations
    pub fn allow_network(&mut self) {
        self.allowed_syscalls.insert("socket".to_string());
        self.allowed_syscalls.insert("connect".to_string());
        self.allowed_syscalls.insert("accept".to_string());
        self.allowed_syscalls.insert("accept4".to_string());
        self.allowed_syscalls.insert("bind".to_string());
        self.allowed_syscalls.insert("listen".to_string());
        self.allowed_syscalls.insert("sendto".to_string());
        self.allowed_syscalls.insert("recvfrom".to_string());
        self.allowed_syscalls.insert("setsockopt".to_string());
        self.allowed_syscalls.insert("getsockopt".to_string());
        self.allowed_syscalls.insert("shutdown".to_string());
        self.allowed_syscalls.insert("sendmsg".to_string());
        self.allowed_syscalls.insert("recvmsg".to_string());
        self.allowed_syscalls.insert("getpeername".to_string());
        self.allowed_syscalls.insert("getsockname".to_string());
    }
    
    /// Add allowed syscalls for client network operations only
    pub fn allow_client_network(&mut self) {
        self.allowed_syscalls.insert("socket".to_string());
        self.allowed_syscalls.insert("connect".to_string());
        self.allowed_syscalls.insert("sendto".to_string());
        self.allowed_syscalls.insert("recvfrom".to_string());
        self.allowed_syscalls.insert("setsockopt".to_string());
        self.allowed_syscalls.insert("getsockopt".to_string());
        self.allowed_syscalls.insert("shutdown".to_string());
        self.allowed_syscalls.insert("sendmsg".to_string());
        self.allowed_syscalls.insert("recvmsg".to_string());
        self.allowed_syscalls.insert("getpeername".to_string());
        self.allowed_syscalls.insert("getsockname".to_string());
        // Notably missing: bind, listen, accept
    }
    
    /// Add allowed syscalls for file operations
    pub fn allow_file_operations(&mut self) {
        self.allowed_syscalls.insert("mkdir".to_string());
        self.allowed_syscalls.insert("rmdir".to_string());
        self.allowed_syscalls.insert("unlink".to_string());
        self.allowed_syscalls.insert("rename".to_string());
        self.allowed_syscalls.insert("readlink".to_string());
        self.allowed_syscalls.insert("symlink".to_string());
        self.allowed_syscalls.insert("chmod".to_string());
        self.allowed_syscalls.insert("fchmod".to_string());
        self.allowed_syscalls.insert("chown".to_string());
        self.allowed_syscalls.insert("fchown".to_string());
        self.allowed_syscalls.insert("lchown".to_string());
        self.allowed_syscalls.insert("umask".to_string());
        self.allowed_syscalls.insert("fcntl".to_string());
        self.allowed_syscalls.insert("truncate".to_string());
        self.allowed_syscalls.insert("ftruncate".to_string());
        self.allowed_syscalls.insert("fallocate".to_string());
        self.allowed_syscalls.insert("fsync".to_string());
        self.allowed_syscalls.insert("fdatasync".to_string());
        self.allowed_syscalls.insert("openat".to_string());
        self.allowed_syscalls.insert("newfstatat".to_string());
        self.allowed_syscalls.insert("readlinkat".to_string());
        self.allowed_syscalls.insert("mkdirat".to_string());
        self.allowed_syscalls.insert("unlinkat".to_string());
        self.allowed_syscalls.insert("fchmodat".to_string());
        self.allowed_syscalls.insert("fchownat".to_string());
        self.allowed_syscalls.insert("renameat".to_string());
        self.allowed_syscalls.insert("renameat2".to_string());
    }
    
    /// Add allowed syscalls for read-only operations
    pub fn allow_read_operations(&mut self) {
        self.allowed_syscalls.insert("readlink".to_string());
        self.allowed_syscalls.insert("readlinkat".to_string());
        self.allowed_syscalls.insert("getcwd".to_string());
        self.allowed_syscalls.insert("getdents".to_string());
        self.allowed_syscalls.insert("getdents64".to_string());
        self.allowed_syscalls.insert("newfstatat".to_string());
        self.allowed_syscalls.insert("fstat".to_string());
        self.allowed_syscalls.insert("fstatfs".to_string());
        self.allowed_syscalls.insert("statfs".to_string());
        self.allowed_syscalls.insert("statx".to_string());
        
        // Restricted read-only open
        // real syscall filtering would need argument inspection
        self.allowed_syscalls.insert("open".to_string());
        self.allowed_syscalls.insert("openat".to_string());
    }
    
    /// Add allowed syscalls for process management
    pub fn allow_process_management(&mut self) {
        self.allowed_syscalls.insert("clone".to_string());
        self.allowed_syscalls.insert("fork".to_string());
        self.allowed_syscalls.insert("vfork".to_string());
        self.allowed_syscalls.insert("execve".to_string());
        self.allowed_syscalls.insert("execveat".to_string());
        self.allowed_syscalls.insert("wait4".to_string());
        self.allowed_syscalls.insert("waitid".to_string());
        self.allowed_syscalls.insert("kill".to_string());
        self.allowed_syscalls.insert("tkill".to_string());
        self.allowed_syscalls.insert("tgkill".to_string());
        self.allowed_syscalls.insert("rt_sigqueueinfo".to_string());
        self.allowed_syscalls.insert("rt_tgsigqueueinfo".to_string());
        self.allowed_syscalls.insert("setpriority".to_string());
        self.allowed_syscalls.insert("sched_setaffinity".to_string());
        self.allowed_syscalls.insert("sched_getaffinity".to_string());
        self.allowed_syscalls.insert("sched_setscheduler".to_string());
        self.allowed_syscalls.insert("sched_getscheduler".to_string());
        self.allowed_syscalls.insert("sched_setparam".to_string());
        self.allowed_syscalls.insert("sched_getparam".to_string());
    }
    
    /// Add allowed syscalls for execution (subset of process management)
    pub fn allow_execution(&mut self) {
        self.allowed_syscalls.insert("execve".to_string());
        self.allowed_syscalls.insert("execveat".to_string());
        self.allowed_syscalls.insert("wait4".to_string());
        self.allowed_syscalls.insert("waitid".to_string());
        self.allowed_syscalls.insert("kill".to_string());
    }
    
    /// Add allowed syscalls for limited execution (highly restricted)
    pub fn allow_limited_execution(&mut self) {
        self.allowed_syscalls.insert("execve".to_string());
        self.allowed_syscalls.insert("wait4".to_string());
    }
    
    /// Allow all operations (for system level)
    pub fn allow_all_operations(&mut self) {
        // Allow network
        self.allow_network();
        
        // Allow file operations
        self.allow_file_operations();
        
        // Allow process management
        self.allow_process_management();
        
        // Allow other system operations
        self.allowed_syscalls.insert("mount".to_string());
        self.allowed_syscalls.insert("umount2".to_string());
        self.allowed_syscalls.insert("ptrace".to_string());
        self.allowed_syscalls.insert("setuid".to_string());
        self.allowed_syscalls.insert("setgid".to_string());
        self.allowed_syscalls.insert("setfsuid".to_string());
        self.allowed_syscalls.insert("setfsgid".to_string());
        self.allowed_syscalls.insert("setresuid".to_string());
        self.allowed_syscalls.insert("setresgid".to_string());
        self.allowed_syscalls.insert("setgroups".to_string());
        self.allowed_syscalls.insert("capset".to_string());
        self.allowed_syscalls.insert("chroot".to_string());
        self.allowed_syscalls.insert("pivot_root".to_string());
        self.allowed_syscalls.insert("sethostname".to_string());
        self.allowed_syscalls.insert("setdomainname".to_string());
        self.allowed_syscalls.insert("reboot".to_string());
        // Many more system-level operations
    }
} 