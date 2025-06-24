//! Resource monitoring and enforcement methods for LinuxCgroupSandbox

use std::path::Path;
use tracing::{debug, info};
use uuid::Uuid;
use chrono;

use crate::error::Result;
use crate::plugin::resource_monitor::ResourceUsage;
use crate::plugin::sandbox::SandboxError;

use super::sandbox::LinuxCgroupSandbox;

impl LinuxCgroupSandbox {
    /// Get resource usage from cgroup
    pub async fn get_cgroup_resource_usage(&self, cgroup_path: &Path) -> Result<ResourceUsage> {
        let mut usage = ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            disk_mb: 0.0,
            network_mb: 0.0,
            timestamp: chrono::Utc::now(),
        };
        
        // Get memory usage from memory.current
        if let Ok(memory_str) = self.read_cgroup_file(cgroup_path, "memory.current").await {
            if let Ok(memory) = memory_str.trim().parse::<u64>() {
                usage.memory_bytes = memory;
            }
        }
        
        // Get CPU usage from cpu.stat
        let mut uptime_usec = 0.0;
        if let Ok(stat) = self.read_cgroup_file(cgroup_path, "cpu.stat").await {
            for line in stat.lines() {
                if line.starts_with("usage_usec") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        if let Ok(usec) = value.parse::<f64>() {
                            uptime_usec = usec;
                            break;
                        }
                    }
                }
            }
        }
        
        // Calculate CPU percentage based on usage time
        // Convert microseconds to seconds and then calculate percentage based on number of cores
        let usage_sec = uptime_usec / 1_000_000.0;
        
        // Get uptime of the system to calculate CPU percentage
        let system_uptime = match std::fs::read_to_string("/proc/uptime") {
            Ok(uptime_str) => {
                if let Some(uptime_sec_str) = uptime_str.split_whitespace().next() {
                    uptime_sec_str.parse::<f64>().unwrap_or(60.0)
                } else {
                    60.0 // Default to 60 seconds if uptime not available
                }
            }
            Err(_) => 60.0,
        };
        
        // Calculate CPU percentage - adjust based on available time
        // Use a simpler approach since we don't have num_cpus crate
        const NUM_CORES: f64 = 4.0; // Default to 4 cores as a reasonable estimate
        usage.cpu_percent = ((usage_sec / system_uptime) * 100.0 * NUM_CORES) as f32;
        
        // Get disk usage from io.stat if available
        if let Ok(io_stat) = self.read_cgroup_file(cgroup_path, "io.stat").await {
            let mut total_bytes: f64 = 0.0;
            
            for line in io_stat.lines() {
                if let Some(rbytes_pos) = line.find("rbytes=") {
                    let rbytes_str = &line[rbytes_pos + 7..];
                    if let Some(end) = rbytes_str.find(' ') {
                        if let Ok(rbytes) = rbytes_str[..end].parse::<u64>() {
                            total_bytes += rbytes as f64;
                        }
                    }
                }
                
                if let Some(wbytes_pos) = line.find("wbytes=") {
                    let wbytes_str = &line[wbytes_pos + 7..];
                    if let Some(end) = wbytes_str.find(' ') {
                        if let Ok(wbytes) = wbytes_str[..end].parse::<u64>() {
                            total_bytes += wbytes as f64;
                        }
                    }
                }
            }
            
            usage.disk_mb = (total_bytes / (1024.0 * 1024.0)) as f32; // Convert bytes to MB
        }
        
        Ok(usage)
    }
    
    /// Apply Linux-specific optimizations
    pub async fn apply_linux_optimizations(&self, cgroup_path: &Path) -> Result<()> {
        // Apply various Linux-specific optimizations for plugin cgroups
        
        // Limit swap usage to reduce chance of system-wide thrashing
        self.write_cgroup_file(cgroup_path, "memory.swappiness", b"10").await.ok();
        
        // Ensure all processes in cgroup are killed together during OOM
        self.write_cgroup_file(cgroup_path, "memory.oom.group", b"1").await.ok();
        
        // Use memory hierarchy so child cgroups inherit memory limits
        self.write_cgroup_file(cgroup_path, "memory.use_hierarchy", b"1").await.ok();
        
        // Set default I/O weight to prioritize system processes
        self.write_cgroup_file(cgroup_path, "io.weight", b"50").await.ok();
        
        // Set default CPU weight to prioritize system processes
        self.write_cgroup_file(cgroup_path, "cpu.weight", b"50").await.ok();
        
        // Set up pressure monitoring (requires recent kernels)
        self.write_cgroup_file(cgroup_path, "memory.pressure", b"some 5000 10000").await.ok();
        
        Ok(())
    }
    
    /// Enforce memory limit by killing processes
    pub async fn enforce_memory_limit(&self, plugin_id: Uuid) -> Result<()> {
        // Get the process ID from the resource monitor
        let process_id_result = self.resource_monitor.get_process_id(plugin_id).await;
        
        if let Ok(process_id) = process_id_result {
            // Get the cgroup path
            let cgroup_paths = self.cgroup_paths.read().await;
            if let Some(cgroup_path) = cgroup_paths.get(&plugin_id) {
                // Get the security context to determine memory limit
                let contexts = self.security_contexts.read().await;
                if let Some(context) = contexts.get(&plugin_id) {
                    // Apply memory limit based on security context
                    let high_threshold = (context.resource_limits.max_memory_bytes as f64 * 0.9) as u64;
                    
                    // Set memory.high (soft limit with throttling)
                    self.write_cgroup_file(cgroup_path, "memory.high", high_threshold.to_string().as_bytes()).await?;
                    
                    debug!("Enforced memory limit for plugin {}: high={} bytes", plugin_id, high_threshold);
                    
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            } else {
                Err(SandboxError::PluginNotFound(plugin_id).into())
            }
        } else {
            Err(SandboxError::PluginNotFound(plugin_id).into())
        }
    }
    
    /// Enforce CPU limit by reducing quota
    pub async fn enforce_cpu_limit(&self, plugin_id: Uuid) -> Result<()> {
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
} 