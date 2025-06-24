//! Process Management
//!
//! This module handles process launching, monitoring, and termination for sandboxed plugins.

use super::*;
use std::process::Command;

impl MacOsSandbox {
    /// Launch a plugin with sandbox constraints
    pub async fn launch_with_sandbox(&self, plugin_id: Uuid, executable: &Path, args: &[&str]) -> Result<u32> {
        let context = self.get_security_context(plugin_id).await?;
        let profile_path = self.create_sandbox_profile(plugin_id, &context).await?;
        
        debug!("Launching plugin {} with sandbox profile {:?}", plugin_id, profile_path);
        
        // Build sandbox-exec command
        let mut cmd = Command::new("sandbox-exec");
        cmd.arg("-f").arg(&profile_path);
        cmd.arg(executable);
        cmd.args(args);
        
        // Launch the process
        let child = cmd.spawn()
            .map_err(|e| SandboxError::ProcessLaunch(format!("Failed to launch sandboxed process: {}", e)))?;
        
        let process_id = child.id();
        
        // Store process ID
        let mut process_ids = self.process_ids.write().await;
        process_ids.insert(plugin_id, process_id);
        
        // Store sandbox profile path
        let mut profiles = self.sandbox_profiles.write().await;
        profiles.insert(plugin_id, profile_path);
        
        info!("Launched plugin {} with process ID {}", plugin_id, process_id);
        
        Ok(process_id)
    }
    
    /// Apply sandbox to an existing process
    pub async fn apply_sandbox_to_process(&self, plugin_id: Uuid, process_id: u32, profile_path: &Path) -> Result<()> {
        debug!("Applying sandbox to existing process {} for plugin {}", process_id, plugin_id);
        
        // Note: macOS doesn't support applying sandbox to existing processes
        // The process must be launched with sandbox-exec from the start
        warn!("Cannot apply sandbox to existing process {} - process must be relaunched", process_id);
        
        Err(SandboxError::ProcessSandboxing(
            "macOS requires processes to be launched with sandbox from the start".to_string()
        ).into())
    }
    
    /// Check if a process is sandboxed
    pub async fn is_process_sandboxed(&self, process_id: u32) -> Result<bool> {
        // Check if process was launched by us (and therefore sandboxed)
        let process_ids = self.process_ids.read().await;
        let is_our_process = process_ids.values().any(|&pid| pid == process_id);
        
        if is_our_process {
            return Ok(true);
        }
        
        // Check using codesign to see if the process has sandbox entitlements
        let output = Command::new("codesign")
            .args(&["-d", "--entitlements", "-", &format!("/proc/{}/exe", process_id)])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let entitlements = String::from_utf8_lossy(&output.stdout);
                Ok(entitlements.contains("com.apple.security.app-sandbox"))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    /// Get all sandboxed processes managed by this instance
    pub async fn get_all_sandboxed_processes(&self) -> Result<Vec<(Uuid, u32)>> {
        let process_ids = self.process_ids.read().await;
        Ok(process_ids.iter().map(|(&plugin_id, &process_id)| (plugin_id, process_id)).collect())
    }
} 