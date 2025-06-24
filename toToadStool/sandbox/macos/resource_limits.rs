//! Resource Limits and Monitoring
//!
//! This module handles resource limit enforcement and monitoring for sandboxed plugins,
//! including memory limits, CPU usage, and process resource tracking.

use super::*;
use std::process::Command;
use sysinfo::{System, SystemExt, ProcessExt, PidExt};

/// Generate resource limits rules for sandbox profiles
pub async fn generate_resource_limits_rules(context: &SecurityContext) -> Result<String> {
    let mut rules = String::new();
    
    let limits = &context.resource_limits;
    
    // Memory limits
    if limits.max_memory_bytes > 0 {
        rules.push_str(&format!("; Memory limit: {} bytes\n", limits.max_memory_bytes));
        rules.push_str(&format!("(allow mach-task-name (target mach-task-self))\n"));
    }
    
    // CPU limits (time-based)
    if limits.max_cpu_percent > 0 {
        rules.push_str(&format!("; CPU limit: {}%\n", limits.max_cpu_percent));
    }
    
    // Thread limits
    if limits.max_threads > 0 {
        rules.push_str(&format!("; Thread limit: {}\n", limits.max_threads));
    }
    
    // Disk limits
    if limits.max_disk_mb > 0 {
        rules.push_str(&format!("; Disk limit: {} MB\n", limits.max_disk_mb));
    }
    
    Ok(rules)
}

impl MacOsSandbox {
    /// Enforce memory limit for a plugin
    pub async fn enforce_memory_limit(&self, plugin_id: Uuid) -> Result<()> {
        let context = self.get_security_context(plugin_id).await?;
        
        // Get the process ID
        let process_ids = self.process_ids.read().await;
        let process_id = process_ids.get(&plugin_id)
            .copied()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // Check memory usage
        let usage = self.get_process_resource_usage(process_id).await?;
        
        let limits = &context.resource_limits;
        if limits.max_memory_bytes > 0 {
            if usage.memory_bytes > limits.max_memory_bytes {
                warn!(
                    "Plugin {} exceeded memory limit: {} bytes > {} bytes",
                    plugin_id, usage.memory_bytes, limits.max_memory_bytes
                );
                
                // Terminate the process if it exceeds memory limits
                self.terminate_process(process_id).await?;
                
                return Err(SandboxError::ResourceLimitExceeded(format!(
                    "Memory limit exceeded: {} MB > {} MB",
                    usage.memory_bytes / 1024 / 1024,
                    limits.max_memory_bytes / 1024 / 1024
                )).into());
            }
        }
        
        Ok(())
    }
    
    /// Get resource usage for a process
    pub async fn get_process_resource_usage(&self, process_id: u32) -> Result<ResourceUsage> {
        let pid = process_id;
        
        // Use blocking task to interact with system info
        let usage = tokio::task::spawn_blocking(move || {
            let mut system = System::new_all();
            system.refresh_processes();
            
            if let Some(process) = system.process(sysinfo::Pid::from(pid as usize)) {
                ResourceUsage {
                    memory_bytes: process.memory() * 1024, // sysinfo returns KB, we want bytes
                    cpu_percent: process.cpu_usage(),
                    disk_mb: 0.0, // Would need separate implementation
                    network_mb: 0.0, // Would need separate implementation
                    timestamp: chrono::Utc::now(),
                }
            } else {
                ResourceUsage::default()
            }
        }).await?;
        
        Ok(usage)
    }
    
    /// Get detailed process information
    pub async fn get_detailed_process_info(&self, process_id: u32) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        
        // Use ps command to get detailed process information
        let output = Command::new("ps")
            .args(&["-p", &process_id.to_string(), "-o", "pid,ppid,user,pri,ni,vsz,rss,pcpu,pmem,time,comm"])
            .output()
            .map_err(|e| SandboxError::ResourceMonitoring(format!("Failed to get process info: {}", e)))?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            
            if lines.len() >= 2 {
                let headers: Vec<&str> = lines[0].split_whitespace().collect();
                let values: Vec<&str> = lines[1].split_whitespace().collect();
                
                for (header, value) in headers.iter().zip(values.iter()) {
                    info.insert(header.to_string(), value.to_string());
                }
            }
        }
        
        // Add sandbox-specific information
        info.insert("sandbox_enabled".to_string(), "true".to_string());
        info.insert("sandbox_profile".to_string(), 
                   self.get_sandbox_profile_path(
                       self.get_plugin_id_by_process(process_id).await?
                   ).to_string_lossy().to_string());
        
        // Get memory regions information
        let mem_output = Command::new("vmmap")
            .arg(process_id.to_string())
            .output();
        
        if let Ok(mem_output) = mem_output {
            if mem_output.status.success() {
                let mem_str = String::from_utf8_lossy(&mem_output.stdout);
                
                // Parse vmmap output for memory regions
                let mut total_virtual = 0u64;
                let mut total_resident = 0u64;
                
                for line in mem_str.lines() {
                    if line.contains("TOTAL") {
                        // Parse the total line
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            if let Ok(virtual_size) = parse_memory_size(parts[1]) {
                                total_virtual = virtual_size;
                            }
                            if let Ok(resident_size) = parse_memory_size(parts[2]) {
                                total_resident = resident_size;
                            }
                        }
                        break;
                    }
                }
                
                info.insert("virtual_memory".to_string(), format!("{}", total_virtual));
                info.insert("resident_memory".to_string(), format!("{}", total_resident));
            }
        }
        
        // Get open file descriptors
        let lsof_output = Command::new("lsof")
            .args(&["-p", &process_id.to_string()])
            .output();
        
        if let Ok(lsof_output) = lsof_output {
            if lsof_output.status.success() {
                let lsof_str = String::from_utf8_lossy(&lsof_output.stdout);
                let fd_count = lsof_str.lines().count().saturating_sub(1); // Subtract header line
                info.insert("open_file_descriptors".to_string(), fd_count.to_string());
            }
        }
        
        // Get network connections
        let netstat_output = Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output();
        
        if let Ok(netstat_output) = netstat_output {
            if netstat_output.status.success() {
                let netstat_str = String::from_utf8_lossy(&netstat_output.stdout);
                let mut connection_count = 0;
                
                for line in netstat_str.lines() {
                    if line.contains(&format!(".{}", process_id)) {
                        connection_count += 1;
                    }
                }
                
                info.insert("network_connections".to_string(), connection_count.to_string());
            }
        }
        
        Ok(info)
    }
    
    /// Get plugin ID by process ID (reverse lookup)
    async fn get_plugin_id_by_process(&self, process_id: u32) -> Result<Uuid> {
        let process_ids = self.process_ids.read().await;
        
        for (plugin_id, &pid) in process_ids.iter() {
            if pid == process_id {
                return Ok(*plugin_id);
            }
        }
        
        Err(SandboxError::PluginNotFound(Uuid::new_v4()).into()) // Placeholder UUID
    }
    
    /// Terminate a process
    pub async fn terminate_process(&self, process_id: u32) -> Result<()> {
        debug!("Terminating process {}", process_id);
        
        // Try graceful termination first (SIGTERM)
        let term_result = Command::new("kill")
            .args(&["-TERM", &process_id.to_string()])
            .output();
        
        if let Ok(output) = term_result {
            if output.status.success() {
                // Wait a bit for graceful shutdown
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                
                // Check if process is still running
                let check_result = Command::new("kill")
                    .args(&["-0", &process_id.to_string()])
                    .output();
                
                if let Ok(check_output) = check_result {
                    if !check_output.status.success() {
                        // Process terminated successfully
                        info!("Process {} terminated gracefully", process_id);
                        return Ok(());
                    }
                }
                
                // If still running, force kill (SIGKILL)
                warn!("Process {} didn't respond to SIGTERM, using SIGKILL", process_id);
                let kill_result = Command::new("kill")
                    .args(&["-KILL", &process_id.to_string()])
                    .output();
                
                if let Ok(kill_output) = kill_result {
                    if kill_output.status.success() {
                        info!("Process {} force terminated", process_id);
                        return Ok(());
                    }
                }
            }
        }
        
        Err(SandboxError::ProcessTermination(format!("Failed to terminate process {}", process_id)).into())
    }
}

/// Parse memory size from vmmap output (e.g., "123K", "45M", "2G")
fn parse_memory_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim();
    
    if size_str.is_empty() {
        return Ok(0);
    }
    
    let (number_part, unit) = if size_str.ends_with('K') {
        (&size_str[..size_str.len()-1], 1024)
    } else if size_str.ends_with('M') {
        (&size_str[..size_str.len()-1], 1024 * 1024)
    } else if size_str.ends_with('G') {
        (&size_str[..size_str.len()-1], 1024 * 1024 * 1024)
    } else {
        (size_str, 1)
    };
    
    let number: f64 = number_part.parse()
        .map_err(|_| SandboxError::ResourceMonitoring(format!("Invalid memory size: {}", size_str)))?;
    
    Ok((number * unit as f64) as u64)
} 