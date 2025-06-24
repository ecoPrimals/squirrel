//! I/O and file operation methods for LinuxCgroupSandbox

use std::fs;
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::error::{Result, CoreError};
use crate::plugin::sandbox::SandboxError;
use crate::plugin::security::ResourceLimits;

use super::sandbox::LinuxCgroupSandbox;

impl LinuxCgroupSandbox {
    /// Create a cgroup for a plugin
    pub async fn create_cgroup(&self, plugin_id: Uuid) -> Result<PathBuf> {
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
    pub async fn set_cgroup_limits(&self, cgroup_path: &Path, limits: &ResourceLimits) -> Result<()> {
        debug!("Setting cgroup limits at {}", cgroup_path.display());
        
        // Set memory limit (in bytes)
        self.write_cgroup_file(
            cgroup_path,
            "memory.max",
            limits.max_memory_bytes.to_string().as_bytes()
        ).await?;
        
        // Set CPU limit
        // We use a period of 100000 microseconds as the base and calculate quota based on percentage
        let period = 100000;
        let quota = (period as f64 * (limits.max_cpu_percent as f64 / 100.0)) as u64;
        
        self.write_cgroup_file(
            cgroup_path,
            "cpu.max",
            format!("{} {}", quota, period).as_bytes()
        ).await?;
        
        // Set IO limits
        self.write_cgroup_file(
            cgroup_path,
            "io.max",
            format!("rbps=max wbps={}", limits.max_disk_mb * 1024 * 1024).as_bytes()
        ).await?;
        
        // Set process count limit
        // First, get security context to determine permission level
        let contexts = self.security_contexts.read().await;
        let pids_max = if let Some(plugin_id) = self.get_plugin_id_from_cgroup_path(cgroup_path) {
            if let Some(context) = contexts.get(&plugin_id) {
                match context.permission_level {
                    crate::plugin::security::PermissionLevel::System => 1024, // System gets more processes
                    crate::plugin::security::PermissionLevel::User => 256,    // User gets moderate processes
                    crate::plugin::security::PermissionLevel::Restricted => limits.max_threads as u64, // Restricted follows the limit exactly
                }
            } else {
                // Default to max_threads if no context
                limits.max_threads as u64
            }
        } else {
            // Default to max_threads if can't determine plugin ID
            limits.max_threads as u64
        };
        
        self.write_cgroup_file(
            cgroup_path,
            "pids.max",
            pids_max.to_string().as_bytes()
        ).await?;
        
        // Set OOM behavior - prefer not to kill essential system processes
        self.write_cgroup_file(
            cgroup_path,
            "memory.oom.group",
            b"1"
        ).await?;
        
        Ok(())
    }
    
    /// Add a process to a cgroup
    pub async fn add_process_to_cgroup(&self, cgroup_path: &Path, process_id: u32) -> Result<()> {
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
    pub async fn apply_process_specific_settings(&self, cgroup_path: &Path, process_id: u32) -> Result<()> {
        // Extract plugin ID from cgroup path
        let cgroup_name = cgroup_path.file_name()
            .ok_or_else(|| SandboxError::Internal("Invalid cgroup path".to_string()))?
            .to_string_lossy();
        
        if let Ok(plugin_id) = Uuid::parse_str(&cgroup_name) {
            // Get security context
            if let Ok(context) = self.get_security_context(plugin_id).await {
                // Apply different settings based on permission level
                match context.permission_level {
                    crate::plugin::security::PermissionLevel::Restricted => {
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
    pub async fn enable_controllers(&self, cgroup_path: &Path) -> Result<()> {
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

    /// Write data to a cgroup file
    pub async fn write_cgroup_file(&self, cgroup_path: &Path, filename: &str, data: &[u8]) -> Result<()> {
        let file_path = cgroup_path.join(filename);
        let data_owned = data.to_vec(); // Clone the data to owned vector

        task::spawn_blocking(move || {
            fs::write(&file_path, data_owned).map_err(|e| {
                error!("Failed to write to cgroup file {}: {}", file_path.display(), e);
                CoreError::from(SandboxError::Platform(format!(
                    "Failed to write to cgroup file {}: {}",
                    file_path.display(), e
                )))
            })
        }).await.map_err(|e| {
            error!("Failed to join task for writing to cgroup file: {}", e);
            CoreError::from(SandboxError::Platform(format!(
                "Failed to join task for writing to cgroup file: {}", e
            )))
        })?
    }

    /// Read content from a cgroup file
    pub async fn read_cgroup_file(&self, cgroup_path: &Path, filename: &str) -> Result<String> {
        let file_path = cgroup_path.join(filename);
        debug!("Reading from cgroup file: {}", file_path.display());
        
        // Perform the filesystem operation in a blocking task
        task::spawn_blocking(move || {
            fs::read_to_string(&file_path).map_err(|e| {
                error!("Failed to read from cgroup file {}: {}", file_path.display(), e);
                CoreError::from(SandboxError::Platform(format!(
                    "Failed to read from cgroup file {}: {}", 
                    file_path.display(), e
                )))
            })
        }).await.map_err(|e| {
            CoreError::from(SandboxError::Platform(format!(
                "Failed to spawn blocking task for reading cgroup file: {}", e
            )))
        })?
    }
    
    /// Kill processes in cgroup
    pub async fn kill_cgroup_processes(&self, cgroup_path: &Path) -> Result<()> {
        // First try writing to cgroup.kill if available (newer kernels)
        self.write_cgroup_file(cgroup_path, "cgroup.kill", b"1").await?;
        
        // Also try reading cgroup.procs and killing each process
        let procs = self.read_cgroup_file(cgroup_path, "cgroup.procs").await?;
        for line in procs.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                debug!("Sending SIGTERM to process {} in cgroup {}", pid, cgroup_path.display());
                // Send SIGTERM via kill command
                let kill_result = tokio::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output()
                    .await;
                
                if let Err(e) = kill_result {
                    warn!("Failed to terminate process {}: {}", pid, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Remove a cgroup for a plugin
    pub async fn remove_cgroup(&self, plugin_id: Uuid) -> Result<()> {
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
} 