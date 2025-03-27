#![cfg(target_family = "unix")]

//! Linux-specific plugin sandbox implementation using cgroups v2
//!
//! This module provides a Linux-specific implementation of the PluginSandbox trait
//! using cgroups v2 for process isolation and resource limits.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, error, info, trace, warn};
use std::any::Any;
use tokio::task;
use std::os::unix::process::CommandExt;
use std::collections::HashSet;
use std::time::Duration;

use crate::error::{Result, SquirrelError};
use crate::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use crate::plugin::sandbox::{PluginSandbox, SandboxError};

const CGROUP_BASE_PATH: &str = "/sys/fs/cgroup/squirrel";

/// Seccomp filter configuration
#[derive(Debug, Clone)]
struct SeccompConfig {
    /// Plugin ID that this filter is for
    plugin_id: Uuid,
    /// System calls that are allowed
    allowed_syscalls: HashSet<String>,
    /// Whether the filter is applied
    is_applied: bool,
    /// Filter action for unmatched syscalls (e.g. "kill", "trap", "errno")
    default_action: String,
    /// Path to the BPF program if generated
    bpf_path: Option<PathBuf>,
    /// Whether to log syscalls for debugging
    log_syscalls: bool,
}

impl SeccompConfig {
    /// Create a default seccomp configuration
    fn default(plugin_id: Uuid) -> Self {
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
    fn from_security_context(plugin_id: Uuid, context: &SecurityContext) -> Self {
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
                if context.capabilities.contains("file:read") || 
                   context.capabilities.contains("file:write") {
                    config.allow_file_operations();
                }
                
                if context.capabilities.contains("network:connect") || 
                   context.capabilities.contains("network:listen") {
                    config.allow_network();
                }
                
                if context.capabilities.contains("system:resources") {
                    config.allow_process_management();
                }
                
                if context.capabilities.contains("plugin:execute") {
                    config.allow_execution();
                }
            },
            PermissionLevel::Restricted => {
                // Restricted level gets strict seccomp with kill for dangerous syscalls
                config.default_action = "errno".to_string();
                
                // Only add operation categories explicitly allowed by capabilities
                if context.capabilities.contains("file:read") {
                    config.allow_read_operations();
                }
                
                if context.capabilities.contains("network:connect") {
                    config.allow_client_network();
                }
                
                if context.capabilities.contains("plugin:execute") {
                    config.allow_limited_execution();
                }
            }
        }
        
        config
    }
    
    /// Add allowed syscalls for network operations
    fn allow_network(&mut self) {
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
    fn allow_client_network(&mut self) {
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
    fn allow_file_operations(&mut self) {
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
    fn allow_read_operations(&mut self) {
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
    fn allow_process_management(&mut self) {
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
    fn allow_execution(&mut self) {
        self.allowed_syscalls.insert("execve".to_string());
        self.allowed_syscalls.insert("execveat".to_string());
        self.allowed_syscalls.insert("wait4".to_string());
        self.allowed_syscalls.insert("waitid".to_string());
        self.allowed_syscalls.insert("kill".to_string());
    }
    
    /// Add allowed syscalls for limited execution (highly restricted)
    fn allow_limited_execution(&mut self) {
        self.allowed_syscalls.insert("execve".to_string());
        self.allowed_syscalls.insert("wait4".to_string());
    }
    
    /// Allow all operations (for system level)
    fn allow_all_operations(&mut self) {
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

/// Linux-specific plugin sandbox implementation using cgroups v2
#[derive(Debug)]
pub struct LinuxCgroupSandbox {
    /// Cgroup paths for plugins
    cgroup_paths: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
    /// Security contexts for plugins
    security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    /// Base path for cgroups
    base_path: PathBuf,
    /// Seccomp configurations per plugin
    seccomp_configs: RwLock<HashMap<Uuid, SeccompConfig>>,
}

impl LinuxCgroupSandbox {
    /// Create a new Linux cgroup sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        // Create base path for cgroups if it doesn't exist
        let base_path = PathBuf::from(CGROUP_BASE_PATH);
        
        // Ensure cgroup filesystem is mounted
        Self::ensure_cgroup_mounted()?;
        
        // Create squirrel cgroup if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .map_err(|e| SandboxError::Creation(format!("Failed to create cgroup directory: {}", e)))?;
        }
        
        Ok(Self {
            cgroup_paths: Arc::new(RwLock::new(HashMap::new())),
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
            base_path,
            seccomp_configs: RwLock::new(HashMap::new()),
        })
    }
    
    /// Ensure cgroup filesystem is mounted
    fn ensure_cgroup_mounted() -> Result<()> {
        // Check if cgroup2 is mounted
        let cgroup_mount = Path::new("/sys/fs/cgroup");
        if !cgroup_mount.exists() {
            return Err(SandboxError::Platform(
                "Cgroup filesystem is not mounted at /sys/fs/cgroup".to_string()
            ).into());
        }
        
        // Check if it's cgroup v2
        let cgroup_type_path = cgroup_mount.join("cgroup.controllers");
        if !cgroup_type_path.exists() {
            return Err(SandboxError::Platform(
                "Cgroup v2 is required but not available on this system".to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Get cgroup path for a plugin
    fn get_cgroup_path(&self, plugin_id: &Uuid) -> PathBuf {
        self.base_path.join(plugin_id.to_string())
    }
    
    /// Create a cgroup for a plugin
    async fn create_cgroup(&self, plugin_id: Uuid) -> Result<PathBuf> {
        let cgroup_path = self.get_cgroup_path(&plugin_id);
        let cgroup_path_clone = cgroup_path.clone();
        
        // Create the cgroup directory
        task::spawn_blocking(move || {
            fs::create_dir_all(&cgroup_path_clone)
                .map_err(|e| SandboxError::Creation(format!(
                    "Failed to create cgroup directory for plugin {}: {}", 
                    plugin_id, e
                )))
        }).await??;
        
        debug!("Created cgroup at {:?} for plugin {}", cgroup_path, plugin_id);
        
        Ok(cgroup_path)
    }
    
    /// Set resource limits on a cgroup
    async fn set_cgroup_limits(&self, cgroup_path: &Path, limits: &ResourceLimits) -> Result<()> {
        // Set memory limit
        if limits.max_memory_bytes > 0 {
            // Write memory.max (hard limit)
            self.write_cgroup_file(
                cgroup_path, 
                "memory.max", 
                limits.max_memory_bytes.to_string().as_bytes()
            ).await?;
            
            // Set memory high threshold at 90% of max
            let memory_high = (limits.max_memory_bytes as f64 * 0.9) as u64;
            self.write_cgroup_file(
                cgroup_path, 
                "memory.high", 
                memory_high.to_string().as_bytes()
            ).await?;
            
            // Set memory.swap.max to prevent swap usage as an escape
            self.write_cgroup_file(
                cgroup_path,
                "memory.swap.max",
                "0".as_bytes()
            ).await.ok(); // This may fail if swap control is not available
            
            // Set memory.oom.group to make OOM kill all processes in the cgroup
            self.write_cgroup_file(
                cgroup_path,
                "memory.oom.group",
                "1".as_bytes()
            ).await.ok(); // This may fail if not supported
            
            debug!("Set memory limits for cgroup {}: max={}, high={}", 
                cgroup_path.display(), limits.max_memory_bytes, memory_high);
        }
        
        // Set CPU limit
        if limits.max_cpu_percent < 100 {
            // Convert percentage to cgroup cpu.max format (quota period)
            // With a 100ms period (100000us), percentage becomes quota value
            let period = 100000;
            let quota = (period as f64 * (limits.max_cpu_percent as f64 / 100.0)) as u64;
            
            // Write CPU quota/period
            self.write_cgroup_file(
                cgroup_path,
                "cpu.max",
                format!("{} {}", quota, period).as_bytes()
            ).await?;
            
            // Also set cpu.weight for proportional CPU distribution
            // Convert percentage to weight (1-10000, with 100 being default)
            let weight = ((limits.max_cpu_percent as f64 / 100.0) * 10000.0) as u64;
            let weight = weight.clamp(1, 10000); // Ensure within valid range
            
            self.write_cgroup_file(
                cgroup_path,
                "cpu.weight",
                weight.to_string().as_bytes()
            ).await.ok(); // May fail if not supported
            
            debug!("Set CPU limits for cgroup {}: quota={}, period={}, weight={}", 
                cgroup_path.display(), quota, period, weight);
        }
        
        // Set I/O limit if available
        if limits.max_disk_mb > 0 {
            // Try to set io.max for all devices with rbps (read bandwidth) and wbps (write bandwidth)
            // Convert MB to bytes
            let bytes_per_sec = (limits.max_disk_mb as u64 * 1024 * 1024) / 60; // MB per minute to bytes per second
            
            // Get available devices from io.stat
            let io_stat = match self.read_cgroup_file(cgroup_path, "io.stat").await {
                Ok(stat) => stat,
                Err(_) => {
                    debug!("io.stat not available for cgroup {}, skipping I/O limits", cgroup_path.display());
                    String::new()
                }
            };
            
            if !io_stat.is_empty() {
                // For each device in io.stat, set limits
                // Format is: "dev_id rbps=rate wbps=rate riops=rate wiops=rate"
                let devices = io_stat.lines()
                    .map(|line| line.split_whitespace().next().unwrap_or(""))
                    .filter(|dev| !dev.is_empty())
                    .collect::<Vec<_>>();
                
                for dev in devices {
                    let io_max = format!("{} rbps={} wbps={}", dev, bytes_per_sec, bytes_per_sec);
                    
                    self.write_cgroup_file(
                        cgroup_path,
                        "io.max",
                        io_max.as_bytes()
                    ).await.ok(); // May fail if specific device doesn't support limits
                    
                    debug!("Set I/O limits for device {} in cgroup {}: {}", 
                        dev, cgroup_path.display(), bytes_per_sec);
                }
            } else {
                // Try to set io.max with default device (8:0 = sda)
                let io_max = format!("8:0 rbps={} wbps={}", bytes_per_sec, bytes_per_sec);
                
                let result = self.write_cgroup_file(
                    cgroup_path,
                    "io.max",
                    io_max.as_bytes()
                ).await;
                
                if result.is_ok() {
                    debug!("Set I/O limits for default device in cgroup {}: {}", 
                        cgroup_path.display(), bytes_per_sec);
                } else {
                    debug!("Failed to set I/O limits for cgroup {}", cgroup_path.display());
                }
            }
        }
        
        // Set PIDs limit to prevent fork bombs
        // Default is based on permission level in the security context
        let pids_max = match limits.permission_level {
            PermissionLevel::System => 10000, // Very generous for system level
            PermissionLevel::User => 1000,    // Moderate for user level
            PermissionLevel::Restricted => 100, // Very restrictive
        };
        
        self.write_cgroup_file(
            cgroup_path,
            "pids.max",
            pids_max.to_string().as_bytes()
        ).await.ok(); // May fail if not supported
        
        debug!("Set PIDs limit for cgroup {}: {}", cgroup_path.display(), pids_max);
        
        Ok(())
    }
    
    /// Add a process to a cgroup
    async fn add_process_to_cgroup(&self, cgroup_path: &Path, process_id: u32) -> Result<()> {
        let pid_str = process_id.to_string();
        
        // Write process ID to cgroup.procs file
        self.write_cgroup_file(
            cgroup_path,
            "cgroup.procs",
            pid_str.as_bytes()
        ).await?;
        
        debug!("Added process {} to cgroup {}", process_id, cgroup_path.display());
        
        // Verify the process was added successfully by reading cgroup.procs
        let procs = self.read_cgroup_file(cgroup_path, "cgroup.procs").await?;
        if !procs.contains(&pid_str) {
            return Err(SandboxError::Creation(format!(
                "Failed to add process {} to cgroup {}: process not found in cgroup.procs",
                process_id, cgroup_path.display()
            )).into());
        }
        
        // Set any specific kernel parameters for process
        self.apply_process_specific_settings(cgroup_path, process_id).await?;
        
        Ok(())
    }
    
    /// Apply process-specific settings based on cgroup and security context
    async fn apply_process_specific_settings(&self, cgroup_path: &Path, process_id: u32) -> Result<()> {
        // Extract plugin ID from cgroup path
        let cgroup_name = cgroup_path.file_name()
            .ok_or_else(|| SandboxError::Internal("Invalid cgroup path".to_string()))?
            .to_string_lossy();
        
        if let Ok(plugin_id) = Uuid::parse_str(&cgroup_name) {
            // Get security context
            if let Ok(context) = self.get_security_context(plugin_id).await {
                // Apply different settings based on permission level
                match context.permission_level {
                    PermissionLevel::Restricted => {
                        // For restricted processes, apply additional kernel limits
                        // This would typically set process-specific attributes through /proc
                        // For example, disabling ptrace capabilities:
                        let _ = task::spawn_blocking(move || {
                            let yama_path = Path::new("/proc")
                                .join(process_id.to_string())
                                .join("yama")
                                .join("ptrace_scope");
                            
                            if yama_path.exists() {
                                let _ = std::fs::write(yama_path, "3"); // Disable ptrace
                            }
                        }).await;
                    },
                    _ => {
                        // No special settings for other permission levels
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Enable required controllers in the cgroup
    async fn enable_controllers(&self, cgroup_path: &Path) -> Result<()> {
        // Read available controllers
        let subtree_control = match self.read_cgroup_file(cgroup_path.parent().unwrap_or(Path::new("/sys/fs/cgroup")), "cgroup.controllers").await {
            Ok(controllers) => controllers,
            Err(e) => {
                warn!("Could not read cgroup controllers: {}", e);
                String::new()
            }
        };
        
        // If we have controllers, enable them
        if !subtree_control.is_empty() {
            // Format controllers as +controller for each one
            let controllers = subtree_control
                .split_whitespace()
                .map(|c| format!("+{}", c))
                .collect::<Vec<_>>()
                .join(" ");
            
            // Write to cgroup.subtree_control
            let parent_path = cgroup_path.parent().unwrap_or(Path::new("/sys/fs/cgroup"));
            let result = self.write_cgroup_file(
                parent_path,
                "cgroup.subtree_control",
                controllers.as_bytes()
            ).await;
            
            if let Err(e) = result {
                warn!("Could not enable cgroup controllers: {}", e);
                // Continue anyway, as some controllers might still work
            } else {
                debug!("Enabled cgroup controllers: {}", controllers);
            }
        }
        
        Ok(())
    }
    
    /// Get resource usage from cgroup
    async fn get_cgroup_resource_usage(&self, cgroup_path: &Path) -> Result<ResourceUsage> {
        let mut usage = ResourceUsage::default();
        usage.timestamp = chrono::Utc::now();
        
        // Get memory usage from memory.current
        match self.read_cgroup_file(cgroup_path, "memory.current").await {
            Ok(mem_str) => {
                if let Ok(mem_bytes) = mem_str.trim().parse::<f64>() {
                    usage.memory_mb = mem_bytes / (1024.0 * 1024.0); // Convert bytes to MB
                }
            }
            Err(e) => {
                warn!("Failed to read memory usage from cgroup: {}", e);
            }
        }
        
        // Get CPU usage - this is more complex in cgroups v2
        // We'll use the cpu.stat file
        match self.read_cgroup_file(cgroup_path, "cpu.stat").await {
            Ok(cpu_stat) => {
                // Parse usage_usec from cpu.stat
                for line in cpu_stat.lines() {
                    if line.starts_with("usage_usec") {
                        if let Some(value_str) = line.split_whitespace().nth(1) {
                            if let Ok(usage_usec) = value_str.parse::<f64>() {
                                // Convert microseconds to percentage based on time since creation
                                // This is approximate and will need tuning in a real implementation
                                
                                // Get uptime in microseconds
                                let uptime_usec = match self.read_cgroup_file(cgroup_path, "cpu.stat") {
                                    Ok(stat) => {
                                        for line in stat.lines() {
                                            if line.starts_with("usage_usec") {
                                                if let Some(value) = line.split_whitespace().nth(1) {
                                                    if let Ok(usec) = value.parse::<f64>() {
                                                        usec
                                                    } else {
                                                        0.0
                                                    }
                                                } else {
                                                    0.0
                                                }
                                            }
                                        }
                                        0.0 // Default if not found
                                    }
                                    Err(_) => 0.0,
                                };
                                
                                if uptime_usec > 0.0 {
                                    // Calculate percentage based on uptime and usage
                                    usage.cpu_percent = ((usage_usec / uptime_usec) * 100.0) as f32;
                                } else {
                                    // Fallback calculation based on system uptime
                                    let system_uptime = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_micros() as f64;
                                    
                                    if system_uptime > 0.0 {
                                        usage.cpu_percent = ((usage_usec / system_uptime) * 100.0) as f32;
                                    }
                                }
                                
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read CPU usage from cgroup: {}", e);
            }
        }
        
        // Get disk usage from io.stat
        match self.read_cgroup_file(cgroup_path, "io.stat").await {
            Ok(io_stat) => {
                // Parse rbytes and wbytes from io.stat
                let mut total_bytes = 0.0;
                
                for line in io_stat.lines() {
                    for field in line.split_whitespace() {
                        if field.starts_with("rbytes=") {
                            if let Some(value_str) = field.strip_prefix("rbytes=") {
                                if let Ok(bytes) = value_str.parse::<f64>() {
                                    total_bytes += bytes;
                                }
                            }
                        } else if field.starts_with("wbytes=") {
                            if let Some(value_str) = field.strip_prefix("wbytes=") {
                                if let Ok(bytes) = value_str.parse::<f64>() {
                                    total_bytes += bytes;
                                }
                            }
                        }
                    }
                }
                
                usage.disk_mb = total_bytes / (1024.0 * 1024.0); // Convert bytes to MB
            }
            Err(e) => {
                // io.stat may not be available, not a critical error
                debug!("Failed to read I/O usage from cgroup: {}", e);
            }
        }
        
        // Add any other metrics as needed
        
        Ok(usage)
    }
    
    /// Check if path is inside secure namespace
    fn is_path_in_secure_namespace(&self, path: &Path) -> bool {
        // Check if the path is under /tmp or the home directory of the current user
        let tmp_path = Path::new("/tmp");
        if path.starts_with(tmp_path) {
            return true;
        }
        
        // Check for home directory
        if let Ok(home) = std::env::var("HOME") {
            let home_path = Path::new(&home);
            if path.starts_with(home_path) {
                return true;
            }
        }
        
        false
    }
    
    /// Kill processes in cgroup
    async fn kill_cgroup_processes(&self, cgroup_path: &Path) -> Result<()> {
        // Write "1" to cgroup.kill to terminate all processes
        self.write_cgroup_file(cgroup_path, "cgroup.kill", b"1").await?;
        debug!("Killed processes in cgroup {:?}", cgroup_path);
        Ok(())
    }
    
    /// Apply Linux-specific optimizations
    async fn apply_linux_optimizations(&self, cgroup_path: &Path) -> Result<()> {
        // Set swappiness to low value to reduce swap usage
        self.write_cgroup_file(cgroup_path, "memory.swappiness", b"10").await.ok();
        
        // Enable memory.oom.group to kill all processes in the cgroup on OOM
        self.write_cgroup_file(cgroup_path, "memory.oom.group", b"1").await.ok();
        
        // Enable memory use_hierarchy
        self.write_cgroup_file(cgroup_path, "memory.use_hierarchy", b"1").await.ok();
        
        // Set io controller weight to prioritize system processes over plugins
        self.write_cgroup_file(cgroup_path, "io.weight", b"50").await.ok();
        
        // Set cpu.weight to lower value (default is 100)
        self.write_cgroup_file(cgroup_path, "cpu.weight", b"50").await.ok();
        
        // Enable pressure stall information monitoring
        self.write_cgroup_file(cgroup_path, "memory.pressure", b"some 5000 10000").await.ok();
        
        // Set memory.high to 90% of memory.max to get early warnings before OOM
        // We'll read the current memory.max value first
        if let Ok(max_memory) = fs::read_to_string(cgroup_path.join("memory.max")) {
            if let Ok(max_bytes) = max_memory.trim().parse::<u64>() {
                if max_bytes > 0 {  // Only if there's an actual limit
                    let high_threshold = (max_bytes as f64 * 0.9) as u64;
                    self.write_cgroup_file(cgroup_path, "memory.high", high_threshold.to_string().as_bytes()).await.ok();
                    debug!("Set memory.high to {} bytes (90% of memory.max)", high_threshold);
                }
            }
        }
        
        Ok(())
    }
    
    /// Enforce memory limit by killing processes
    async fn enforce_memory_limit(&self, plugin_id: Uuid) -> Result<()> {
        let cgroup_paths = self.cgroup_paths.read().await;
        if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
            // Kill all processes in the cgroup
            self.kill_cgroup_processes(cgroup_path).await?;
            
            info!("Killed processes in cgroup for plugin {} due to memory limit violation", plugin_id);
        }
        
        Ok(())
    }
    
    /// Enforce CPU limit by reducing quota
    async fn enforce_cpu_limit(&self, plugin_id: Uuid) -> Result<()> {
        let cgroup_paths = self.cgroup_paths.read().await;
        if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
            let contexts = self.security_contexts.read().await;
            if let Some(context) = contexts.get(&plugin_id) {
                let limits = &context.resource_limits;
                
                // Reduce CPU limit further
                let reduced_cpu_percent = (limits.max_cpu_percent / 2).max(1);
                
                // Convert percentage to cgroup cpu.max format (quota period)
                let period = 100000;
                let quota = (period as f64 * (reduced_cpu_percent as f64 / 100.0)) as u64;
                
                self.write_cgroup_file(
                    cgroup_path,
                    "cpu.max",
                    format!("{} {}", quota, period).as_bytes()
                ).await?;
                
                info!(
                    "Reduced CPU limit for plugin {} from {}% to {}% due to CPU usage violation",
                    plugin_id, limits.max_cpu_percent, reduced_cpu_percent
                );
            }
        }
        
        Ok(())
    }

    /// Remove a cgroup for a plugin
    async fn remove_cgroup(&self, plugin_id: Uuid) -> Result<()> {
        let cgroup_paths = self.cgroup_paths.read().await;
        let cgroup_path = cgroup_paths.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // First kill all processes in the cgroup
        self.kill_cgroup_processes(cgroup_path).await?;
        
        // Remove the cgroup directory
        let cgroup_path_clone = cgroup_path.clone();
        task::spawn_blocking(move || {
            // First ensure cgroup is empty
            // Write "0" to cgroup.kill to terminate all processes
            if let Err(e) = fs::write(cgroup_path_clone.join("cgroup.kill"), "1") {
                warn!("Failed to kill processes in cgroup for plugin {}: {}", plugin_id, e);
                // Continue anyway - try to clean up as much as possible
            }
            
            // Wait a moment for processes to terminate
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Now remove the cgroup directory
            fs::remove_dir(cgroup_path_clone)
                .map_err(|e| SandboxError::Destruction(format!(
                    "Failed to remove cgroup directory for plugin {}: {}", 
                    plugin_id, e
                )))
        }).await??;
        
        // Remove from the map
        drop(cgroup_paths);
        let mut cgroup_paths = self.cgroup_paths.write().await;
        cgroup_paths.remove(&plugin_id);
        
        debug!("Removed cgroup for plugin {}", plugin_id);
        
        Ok(())
    }

    /// Apply seccomp filter to a plugin process
    async fn apply_seccomp_filter(&self, plugin_id: Uuid) -> Result<()> {
        // Check if we're on Linux
        if !cfg!(target_os = "linux") {
            warn!("Seccomp filtering not available on this platform");
            return Ok(());
        }
        
        // Get process ID
        let process_id = {
            let cgroup_paths = self.cgroup_paths.read().await;
            if !cgroup_paths.contains_key(&plugin_id) {
                return Err(SandboxError::PluginNotFound(plugin_id).into());
            }
            
            // Get the process IDs in the cgroup
            let cgroup_path = &cgroup_paths[&plugin_id];
            let procs_path = cgroup_path.join("cgroup.procs");
            
            if !procs_path.exists() {
                return Err(SandboxError::Internal(format!(
                    "Could not find cgroup.procs file at {}",
                    procs_path.display()
                ))
                .into());
            }
            
            let procs_content = fs::read_to_string(&procs_path)
                .map_err(|e| SandboxError::Internal(format!("Failed to read cgroup.procs: {}", e)))?;
            
            // Get the first process ID in the cgroup
            let pid = procs_content
                .lines()
                .next()
                .ok_or_else(|| {
                    SandboxError::Internal("No processes found in cgroup".to_string())
                })?
                .parse::<u32>()
                .map_err(|e| SandboxError::Internal(format!("Failed to parse PID: {}", e)))?;
            
            pid
        };
        
        // Get the seccomp configuration for this plugin
        let context = self.get_security_context(plugin_id).await?;
        
        // Use the new SeccompFilterBuilder to create and apply the filter
        use crate::plugin::sandbox::seccomp::{SeccompFilterBuilder, SeccompAction};
        
        let filter = SeccompFilterBuilder::from_security_context(plugin_id, &context);
        
        debug!("Applying seccomp filter to plugin {} (PID: {})", plugin_id, process_id);
        filter.apply_to_process(process_id)?;
        
        debug!("Seccomp filter applied successfully");
        Ok(())
    }
    
    /// Generate a seccomp BPF program
    async fn generate_seccomp_bpf(&self, plugin_id: Uuid, output_path: &Path) -> Result<()> {
        // Check if we're on Linux
        if !cfg!(target_os = "linux") {
            warn!("Seccomp filtering not available on this platform");
            return Ok(());
        }
        
        // Get the seccomp configuration for this plugin
        let context = self.get_security_context(plugin_id).await?;
        
        // Use the new SeccompFilterBuilder to generate the BPF program
        use crate::plugin::sandbox::seccomp::SeccompFilterBuilder;
        
        let filter = SeccompFilterBuilder::from_security_context(plugin_id, &context);
        
        debug!("Generating seccomp BPF program for plugin {} at {}", 
               plugin_id, output_path.display());
        
        filter.generate_bpf(output_path)?;
        
        debug!("Seccomp BPF program generated successfully");
        Ok(())
    }
}

#[async_trait::async_trait]
impl PluginSandbox for LinuxCgroupSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating Linux cgroup sandbox for plugin {}", plugin_id);
        
        // Create default security context if needed
        let mut contexts = self.security_contexts.write().await;
        if !contexts.contains_key(&plugin_id) {
            contexts.insert(plugin_id, SecurityContext::default());
        }
        let context = contexts.get(&plugin_id).unwrap().clone();
        drop(contexts);
        
        // Create cgroup
        let cgroup_path = self.create_cgroup(plugin_id).await?;
        
        // Enable controllers - try to enable all available controllers
        // This allows for more comprehensive resource control
        if let Err(e) = self.enable_controllers(&cgroup_path).await {
            warn!("Failed to enable cgroup controllers: {}", e);
            // Continue anyway - some limits may not work
        }
        
        // Set resource limits based on security context
        self.set_cgroup_limits(&cgroup_path, &context.resource_limits).await?;
        
        // Add additional memory safeguards - set memory.swap.max to same as memory.max
        // to prevent excessive swap usage
        if context.resource_limits.max_memory_bytes > 0 {
            let mem_limit = context.resource_limits.max_memory_bytes.to_string();
            if let Err(e) = self.write_cgroup_file(&cgroup_path, "memory.swap.max", mem_limit.as_bytes()).await {
                debug!("Could not set swap limit for plugin {}: {}", plugin_id, e);
                // Continue despite failure - swap controller may not be available
            } else {
                debug!("Set swap limit for plugin {}: {} bytes", plugin_id, mem_limit);
            }
        }
        
        // Register cgroup path
        let mut cgroup_paths = self.cgroup_paths.write().await;
        cgroup_paths.insert(plugin_id, cgroup_path.clone());
        drop(cgroup_paths);
        
        // Get process ID
        let process_id = match self.resource_monitor.get_process_id(plugin_id).await? {
            Some(pid) => pid,
            None => {
                // If no process ID is registered, use the current process
                let current_pid = std::process::id();
                
                // Register with resource monitor
                let executable_path = std::env::current_exe()?;
                self.resource_monitor.register_process(plugin_id, current_pid, &executable_path).await?;
                
                current_pid
            }
        };
        
        // Add process to cgroup
        self.add_process_to_cgroup(&cgroup_path, process_id).await?;
        
        // Apply additional Linux optimizations
        if let Err(e) = self.apply_linux_optimizations(&cgroup_path).await {
            debug!("Could not apply Linux optimizations for plugin {}: {}", plugin_id, e);
            // Continue despite failure - optimizations are optional
        }
        
        // Set up OOM notifications if possible
        if let Ok(file) = OpenOptions::new()
            .read(true)
            .open(cgroup_path.join("memory.events"))
        {
            debug!("Enabled OOM monitoring for plugin {}", plugin_id);
            // In a production implementation, we'd start a thread here to monitor for OOM events
        }
        
        info!("Linux cgroup sandbox created for plugin {} with process {}", plugin_id, process_id);
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying Linux cgroup sandbox for plugin {}", plugin_id);
        
        // First kill processes and remove cgroup
        if let Err(e) = self.remove_cgroup(plugin_id).await {
            warn!("Error removing cgroup for plugin {}: {}", plugin_id, e);
            // Continue with cleanup even if cgroup removal fails
        }
        
        // Unregister from resource monitor
        if let Err(e) = self.resource_monitor.unregister_process(plugin_id).await {
            warn!("Error unregistering process for plugin {}: {}", plugin_id, e);
        }
        
        // Remove security context
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // Map operations to capabilities
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
        self.check_capability(plugin_id, required_capability).await
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // Check if we have a cgroup path
        let cgroup_paths = self.cgroup_paths.read().await;
        if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
            // Get resource usage from cgroup
            let mut usage = self.get_cgroup_resource_usage(cgroup_path).await?;
            
            // Try to get network usage from resource monitor
            if let Ok(Some(monitor_usage)) = self.resource_monitor.get_resource_usage(plugin_id).await {
                usage.cpu_percent = monitor_usage.cpu_percent; // Use CPU% from monitor
                usage.network_mb = monitor_usage.network_mb;
            }
            
            return Ok(usage);
        }
        
        // Fallback to resource monitor
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
            debug!("Plugin {} has system permission level, allowing access to {}", 
                   plugin_id, path.display());
            return Ok(());
        }
        
        // For write access, we need to be more restrictive
        if write && context.permission_level == PermissionLevel::Restricted {
            debug!("Plugin {} with restricted permission denied write access to {}", 
                   plugin_id, path.display());
            return Err(SandboxError::PathAccess(format!(
                "Plugin {} with restricted permission level does not have write access to path {}",
                plugin_id, path.display()
            )).into());
        }
        
        // Check if the path is a symlink - symlinks require special handling
        let metadata = fs::metadata(path);
        if let Ok(meta) = metadata {
            if meta.file_type().is_symlink() {
                debug!("Path {} is a symlink, performing additional validation", path.display());
                // For symlinks, we should check both the link and its target
                if let Ok(target) = fs::read_link(path) {
                    // Also check the target path
                    let target_path = if target.is_absolute() {
                        target
                    } else {
                        // Convert relative symlinks to absolute
                        if let Some(parent) = path.parent() {
                            parent.join(target)
                        } else {
                            target
                        }
                    };
                    
                    debug!("Symlink {} points to {}, checking target", 
                          path.display(), target_path.display());
                    
                    // Recursively check the target path
                    return self.check_path_access(plugin_id, &target_path, write).await;
                }
            }
        }
        
        // Check if path is in allowed paths
        let canonical_path = path.canonicalize().map_err(|e| SandboxError::PathAccess(format!(
            "Could not canonicalize path {}: {}", path.display(), e
        )))?;
        
        // Deny access to sensitive system directories by default
        let sensitive_paths = [
            Path::new("/etc"),
            Path::new("/var/log"),
            Path::new("/var/run"),
            Path::new("/boot"),
            Path::new("/proc"),
            Path::new("/sys"),
            Path::new("/dev"),
        ];
        
        for sensitive in &sensitive_paths {
            if canonical_path.starts_with(sensitive) && context.permission_level != PermissionLevel::System {
                debug!("Plugin {} denied access to sensitive path {}", 
                      plugin_id, canonical_path.display());
                return Err(SandboxError::PathAccess(format!(
                    "Plugin {} with non-admin permission level cannot access system path {}",
                    plugin_id, canonical_path.display()
                )).into());
            }
        }
        
        let is_allowed = context.allowed_paths.iter().any(|allowed_path| {
            // Try to canonicalize the allowed path and check if target path is a subdirectory
            if let Ok(canonical_allowed) = allowed_path.canonicalize() {
                canonical_path.starts_with(&canonical_allowed)
            } else {
                false
            }
        });
        
        // Also check if path is in secure namespace
        let is_secure = self.is_path_in_secure_namespace(&canonical_path);
        
        if is_allowed || is_secure {
            debug!("Path access granted for plugin {} to {}", plugin_id, path.display());
            Ok(())
        } else {
            debug!("Path access denied for plugin {} to {}", plugin_id, path.display());
            Err(SandboxError::PathAccess(format!(
                "Plugin {} does not have access to path {}",
                plugin_id, path.display()
            )).into())
        }
    }
    
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level has all capabilities
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Check for exact match
        if context.allowed_capabilities.contains(capability) {
            return Ok(());
        }
        
        // Check for wildcard namespace match
        if let Some(namespace) = capability.split(':').next() {
            let wildcard = format!("{}:*", namespace);
            if context.allowed_capabilities.contains(&wildcard) {
                return Ok(());
            }
        }
        
        Err(SandboxError::Capability(format!(
            "Plugin {} does not have capability '{}'",
            plugin_id, capability
        )).into())
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        // Handle Linux-specific features
        match feature {
            "cgroups" => {
                // Cgroups are already used in this implementation
                debug!("Cgroups feature is already enabled");
                Ok(())
            }
            "memory_limit" => {
                // Memory limits are already set when creating the sandbox
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
                
                let cgroup_paths = self.cgroup_paths.read().await;
                if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
                    // Re-apply memory limits
                    let memory_bytes = context.resource_limits.max_memory_bytes;
                    self.write_cgroup_file(
                        cgroup_path, 
                        "memory.max", 
                        memory_bytes.to_string().as_bytes()
                    ).await?;
                    
                    debug!("Re-applied memory limits for plugin {}", plugin_id);
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            }
            "cpu_limit" => {
                // CPU limits are already set when creating the sandbox
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
                
                let cgroup_paths = self.cgroup_paths.read().await;
                if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
                    // Re-apply CPU limits
                    let period = 100000;
                    let quota = (period as f64 * (context.resource_limits.max_cpu_percent as f64 / 100.0)) as u64;
                    
                    self.write_cgroup_file(
                        cgroup_path,
                        "cpu.max",
                        format!("{} {}", quota, period).as_bytes()
                    ).await?;
                    
                    debug!("Re-applied CPU limits for plugin {}", plugin_id);
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            }
            "seccomp" => {
                // Apply seccomp filtering
                self.apply_seccomp_filter(plugin_id).await?;
                
                debug!("Applied seccomp filtering for plugin {}", plugin_id);
                Ok(())
            }
            "seccomp_export" => {
                // Export seccomp filter to a BPF file
                let plugin_id_str = plugin_id.to_string();
                let output_path = std::env::temp_dir().join(format!("seccomp_{}.bpf", plugin_id_str));
                
                self.generate_seccomp_bpf(plugin_id, &output_path).await?;
                
                debug!("Exported seccomp BPF filter to {}", output_path.display());
                Ok(())
            }
            _ => {
                Err(SandboxError::Unsupported(format!(
                    "Feature '{}' is not supported by Linux sandbox", feature
                )).into())
            }
        }
    }
    
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Delegate to the existing method
        self.set_security_context(plugin_id, context).await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        Some(self.resource_monitor.clone())
    }
}

/// Check if cgroups v2 is available on the system
#[cfg(target_os = "linux")]
pub fn has_cgroups_v2() -> bool {
    // Check if cgroups v2 is mounted
    if let Ok(mount_output) = std::fs::read_to_string("/proc/mounts") {
        // Look for cgroup2 fs type
        return mount_output.contains("cgroup2");
    }
    
    // Alternative check: see if unified hierarchy is available
    std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
}

/// Check if seccomp is available on the system
#[cfg(target_os = "linux")]
pub fn has_seccomp() -> bool {
    // Check if seccomp is available in the kernel
    // By checking if the seccomp directory exists in the kernel config
    if let Ok(config) = std::fs::read_to_string("/proc/config.gz") {
        return config.contains("CONFIG_SECCOMP=y");
    }
    
    // Try checking for seccomp presence via prctl
    unsafe {
        // PR_GET_SECCOMP = 21
        let ret = libc::prctl(21, 0, 0, 0, 0);
        ret >= 0
    }
}

/// Check if namespaces are available on the system
#[cfg(target_os = "linux")]
pub fn has_namespaces() -> bool {
    // Check if we can create a user namespace for the current process
    // This is a simple way to check if namespaces are supported
    unsafe {
        // CLONE_NEWUSER = 0x10000000
        let ret = libc::unshare(0x10000000);
        
        // If successful, restore the original namespace
        if ret == 0 {
            // We successfully created a user namespace, now revert
            let _ = libc::setns(libc::open("/proc/self/ns/user\0".as_ptr() as *const i8, libc::O_RDONLY), 0);
            return true;
        }
        
        // If we couldn't create a user namespace, check if other namespaces are available
        std::path::Path::new("/proc/self/ns").exists()
    }
}

/// Get the list of available namespaces on the system
#[cfg(target_os = "linux")]
pub fn get_available_namespaces() -> Vec<String> {
    let mut namespaces = Vec::new();
    
    // Check if namespace entries exist in /proc/self/ns/
    let ns_path = std::path::Path::new("/proc/self/ns");
    if let Ok(entries) = std::fs::read_dir(ns_path) {
        for entry in entries.filter_map(Result::ok) {
            if let Some(filename) = entry.file_name().to_str() {
                namespaces.push(filename.to_string());
            }
        }
    }
    
    namespaces
}

// For non-Linux platforms, provide stub implementations
#[cfg(not(target_os = "linux"))]
pub fn has_cgroups_v2() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn has_seccomp() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn has_namespaces() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn get_available_namespaces() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::security::{SecurityContext, PermissionLevel};
    use std::env;
    use tokio::runtime::Runtime;
    use uuid::Uuid;

    // Helper function to check if we can run cgroup tests
    fn can_run_cgroup_tests() -> bool {
        // Check if we are running with sufficient permissions and on Linux
        if !cfg!(target_os = "linux") {
            return false;
        }

        // Check if cgroup v2 is mounted and accessible
        let cgroup_mount = Path::new("/sys/fs/cgroup");
        if !cgroup_mount.exists() {
            return false;
        }

        // Check if it's cgroup v2
        let cgroup_type_path = cgroup_mount.join("cgroup.controllers");
        if !cgroup_type_path.exists() {
            return false;
        }

        // Check if we have permission to create cgroups
        if !cgroup_mount.join("cgroup.procs").exists() {
            return false;
        }

        true
    }

    // Create a test security context with different permission levels
    fn create_test_context(level: PermissionLevel) -> SecurityContext {
        let mut context = SecurityContext::default();
        context.permission_level = level;
        
        match level {
            PermissionLevel::System => {
                context.capabilities.insert("system:admin".to_string());
                context.capabilities.insert("file:*".to_string());
                context.capabilities.insert("network:*".to_string());
            },
            PermissionLevel::User => {
                context.capabilities.insert("file:read".to_string());
                context.capabilities.insert("file:write".to_string());
                context.capabilities.insert("network:connect".to_string());
            },
            PermissionLevel::Restricted => {
                context.capabilities.insert("file:read".to_string());
                context.capabilities.insert("plugin:execute".to_string());
            },
        }
        
        // Add allowed paths
        let temp_dir = env::temp_dir();
        context.allowed_paths.push(temp_dir.clone());
        
        // Add write paths only for User and System
        if level != PermissionLevel::Restricted {
            context.write_allowed_paths.push(temp_dir);
        }
        
        // Set resource limits based on permission level
        match level {
            PermissionLevel::System => {
                context.resource_limits.max_memory_bytes = 1024 * 1024 * 1024; // 1GB
                context.resource_limits.max_cpu_percent = 100;
            },
            PermissionLevel::User => {
                context.resource_limits.max_memory_bytes = 512 * 1024 * 1024; // 512MB
                context.resource_limits.max_cpu_percent = 50;
            },
            PermissionLevel::Restricted => {
                context.resource_limits.max_memory_bytes = 256 * 1024 * 1024; // 256MB
                context.resource_limits.max_cpu_percent = 25;
            },
        }
        
        context
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_create_destroy_sandbox() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_create_destroy_sandbox: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check if cgroup path exists
        let cgroup_path = sandbox.get_cgroup_path(&plugin_id);
        assert!(cgroup_path.exists(), "Cgroup path does not exist");
        
        // Check if security context was created
        let context = sandbox.get_security_context(plugin_id).await;
        assert!(context.is_ok(), "Failed to get security context");
        
        // Destroy sandbox
        let result = sandbox.destroy_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to destroy sandbox: {:?}", result);
        
        // Check if cgroup path was removed
        assert!(!cgroup_path.exists(), "Cgroup path still exists after destruction");
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_security_contexts() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_security_contexts: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context
        let context = create_test_context(PermissionLevel::User);
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Get security context
        let retrieved_context = sandbox.get_security_context(plugin_id).await.unwrap();
        assert_eq!(retrieved_context.permission_level, PermissionLevel::User);
        assert!(retrieved_context.capabilities.contains("file:read"));
        
        // Create sandbox with the context
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_resource_limits() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_resource_limits: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context with resource limits
        let mut context = create_test_context(PermissionLevel::User);
        context.resource_limits.max_memory_bytes = 100 * 1024 * 1024; // 100MB
        context.resource_limits.max_cpu_percent = 30;
        
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check cgroup settings
        let cgroup_path = sandbox.get_cgroup_path(&plugin_id);
        
        // Verify memory limit was set properly
        let memory_max = sandbox.read_cgroup_file(&cgroup_path, "memory.max").await;
        assert!(memory_max.is_ok(), "Failed to read memory.max: {:?}", memory_max);
        let memory_max = memory_max.unwrap().trim().parse::<u64>();
        assert!(memory_max.is_ok(), "Failed to parse memory.max: {:?}", memory_max);
        let memory_max = memory_max.unwrap();
        assert_eq!(memory_max, 100 * 1024 * 1024, "Memory limit was not set correctly");
        
        // Verify CPU limit
        let cpu_max = sandbox.read_cgroup_file(&cgroup_path, "cpu.max").await;
        assert!(cpu_max.is_ok(), "Failed to read cpu.max: {:?}", cpu_max);
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_path_access() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_path_access: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context with specific path permissions
        let mut context = create_test_context(PermissionLevel::Restricted);
        let temp_file = env::temp_dir().join("test_file.txt");
        let home_dir = PathBuf::from("/home");
        
        context.allowed_paths.push(env::temp_dir());
        
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check access to allowed path
        let result = sandbox.check_path_access(plugin_id, &temp_file, false).await;
        assert!(result.is_ok(), "Access to allowed path denied: {:?}", result);
        
        // Check access to disallowed path
        let result = sandbox.check_path_access(plugin_id, &home_dir, false).await;
        assert!(result.is_err(), "Access to disallowed path granted");
        
        // Check write access (should be denied for Restricted level)
        let result = sandbox.check_path_access(plugin_id, &temp_file, true).await;
        assert!(result.is_err(), "Write access to path granted for Restricted level");
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_seccomp_filtering() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_seccomp_filtering: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context
        let context = create_test_context(PermissionLevel::Restricted);
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Apply seccomp filtering
        let result = sandbox.apply_feature(plugin_id, "seccomp").await;
        assert!(result.is_ok(), "Failed to apply seccomp filtering: {:?}", result);
        
        // Check seccomp config
        let seccomp_configs = sandbox.seccomp_configs.read().await;
        let config = seccomp_configs.get(&plugin_id);
        assert!(config.is_some(), "Seccomp config not found");
        let config = config.unwrap();
        assert!(config.is_applied, "Seccomp filter not marked as applied");
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_resource_monitoring() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_resource_monitoring: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor.clone()).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context
        let context = create_test_context(PermissionLevel::User);
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Mock process ID for the plugin
        let mock_pid = std::process::id(); // Use current process for testing
        
        // Register the process with resource monitor
        let result = resource_monitor.register_process(plugin_id, mock_pid, &env::current_exe().unwrap()).await;
        assert!(result.is_ok(), "Failed to register process: {:?}", result);
        
        // Track resources
        let result = sandbox.track_resources(plugin_id).await;
        assert!(result.is_ok(), "Failed to track resources: {:?}", result);
        
        let usage = result.unwrap();
        assert!(usage.memory_mb >= 0.0, "Invalid memory usage: {}", usage.memory_mb);
        assert!(usage.cpu_percent >= 0.0, "Invalid CPU usage: {}", usage.cpu_percent);
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
        let _ = resource_monitor.unregister_process(plugin_id).await;
    }
} 