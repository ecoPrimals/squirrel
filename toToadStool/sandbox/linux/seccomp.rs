//! Seccomp filtering functionality for the Linux sandbox

use std::path::Path;
use tracing::{debug, warn};
use uuid::Uuid;
use std::fs;

use crate::error::Result;
use crate::plugin::sandbox::SandboxError;

use super::sandbox::LinuxCgroupSandbox;

impl LinuxCgroupSandbox {
    /// Apply seccomp filter to a plugin process
    pub async fn apply_seccomp_filter(&self, plugin_id: Uuid) -> Result<()> {
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
        use crate::plugin::sandbox::seccomp::SeccompFilterBuilder;
        
        let filter = SeccompFilterBuilder::from_security_context(plugin_id, &context);
        
        debug!("Applying seccomp filter to plugin {} (PID: {})", plugin_id, process_id);
        filter.apply_to_process(process_id)?;
        
        debug!("Seccomp filter applied successfully");
        Ok(())
    }
    
    /// Generate a seccomp BPF program
    pub async fn generate_seccomp_bpf(&self, plugin_id: Uuid, output_path: &Path) -> Result<()> {
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