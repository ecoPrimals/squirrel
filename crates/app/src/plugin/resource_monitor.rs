use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use regex::Regex;
use tokio::time;
use tracing::{error, warn};
use std::path::Path;

use crate::error::{Result, CoreError};

/// Errors that can occur during resource monitoring operations
#[derive(Debug, Clone)]
pub enum ResourceMonitorError {
    /// The process was not registered with the resource monitor
    ProcessNotRegistered(Uuid),
    /// The process could not be found on the system
    ProcessNotFound(Uuid),
    /// A command to measure resources failed to execute
    CommandExecutionFailed(String),
    /// Failed to parse the output of a resource monitoring command
    ParseError(String),
    /// The current platform is not supported for resource monitoring
    PlatformNotSupported(String),
    /// A resource limit was exceeded by a process
    ResourceLimitExceeded { 
        /// The ID of the process that exceeded a resource limit
        process_id: Uuid, 
        /// The type of resource that was exceeded (CPU, Memory, etc.)
        resource_type: String, 
        /// The current value of the resource usage
        current_value: f64, 
        /// The limit that was exceeded
        limit: f64 
    },
    /// Any other error that occurred during resource monitoring
    Other(String),
}

impl std::fmt::Display for ResourceMonitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceMonitorError::ProcessNotRegistered(id) => write!(f, "Process not registered: {}", id),
            ResourceMonitorError::ProcessNotFound(id) => write!(f, "Process not found: {}", id),
            ResourceMonitorError::CommandExecutionFailed(err) => write!(f, "Command execution failed: {}", err),
            ResourceMonitorError::ParseError(err) => write!(f, "Parse error: {}", err),
            ResourceMonitorError::PlatformNotSupported(platform) => write!(f, "Platform not supported: {}", platform),
            ResourceMonitorError::ResourceLimitExceeded { process_id, resource_type, current_value, limit } => {
                write!(f, "Resource limit exceeded for process {}: {} current value {} exceeds limit {}", 
                    process_id, resource_type, current_value, limit)
            },
            ResourceMonitorError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for ResourceMonitorError {}

impl From<ResourceMonitorError> for CoreError {
    fn from(err: ResourceMonitorError) -> Self {
        CoreError::Plugin(format!("Resource monitor error: {}", err))
    }
}

/// Resource limits for a process or plugin
///
/// This struct defines the maximum allowed values for various system resources.
/// These limits are used to monitor and control resource usage of plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage (0-100)
    pub max_cpu_percent: u8,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum disk usage in MB
    pub max_disk_mb: u64,
    /// Maximum threads
    pub max_threads: u16,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            max_cpu_percent: 80,
            max_memory_bytes: 1024 * 1024 * 1024, // 1 GB
            max_disk_mb: 1024,
            max_threads: 4,
        }
    }
}

/// Resource usage measurements for a process or plugin
///
/// This struct contains the current usage values for various system resources.
/// These measurements are used to monitor and control resource usage of plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Current disk usage in megabytes
    pub disk_mb: f32,
    /// Current network usage in megabytes
    pub network_mb: f32,
    /// Timestamp when these measurements were taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            disk_mb: 0.0,
            network_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    process_handle: u32,
    executable_path: std::path::PathBuf,
    start_time: Instant,
}

/// Resource monitoring system for plugin processes
///
/// This struct provides functionality to monitor resource usage of processes
/// associated with plugins, enforce resource limits, and take action when
/// limits are exceeded.
#[derive(Debug)]
pub struct ResourceMonitor {
    process_map: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    usage_data: Arc<RwLock<HashMap<Uuid, ResourceUsage>>>,
    resource_limits: Arc<RwLock<HashMap<Uuid, ResourceLimits>>>,
    is_monitoring: bool,
    monitor_interval: Duration,
}

impl ResourceMonitor {
    /// Creates a new ResourceMonitor instance
    ///
    /// The created monitor is not automatically started. Call `start_monitoring`
    /// to begin the monitoring process after enabling monitoring.
    pub fn new() -> Self {
        ResourceMonitor {
            process_map: Arc::new(RwLock::new(HashMap::new())),
            usage_data: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: Arc::new(RwLock::new(HashMap::new())),
            is_monitoring: false,
            monitor_interval: Duration::from_secs(30),
        }
    }

    /// Sets the interval at which resource monitoring occurs
    ///
    /// # Arguments
    ///
    /// * `interval` - The duration between resource measurement checks
    pub fn set_monitor_interval(&mut self, interval: Duration) {
        self.monitor_interval = interval;
    }

    /// Enables the resource monitoring system
    ///
    /// Call `start_monitoring` after enabling to begin the actual monitoring process.
    pub fn enable_monitoring(&mut self) {
        self.is_monitoring = true;
    }

    /// Disables the resource monitoring system
    pub fn disable_monitoring(&mut self) {
        self.is_monitoring = false;
    }

    /// Registers a process for resource monitoring
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID associated with the plugin
    /// * `process_handle` - The system process ID (PID) to monitor
    /// * `executable_path` - The path to the process executable
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the process was registered successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process could not be registered.
    pub async fn register_process(&self, process_id: Uuid, process_handle: u32, executable_path: &Path) -> Result<()> {
        let mut process_map = self.process_map.write().await;
        let process_info = ProcessInfo {
            process_handle,
            executable_path: executable_path.to_path_buf(),
            start_time: Instant::now(),
        };
        
        process_map.insert(process_id, process_info);
        Ok(())
    }

    /// Sets resource limits for a monitored process
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process
    /// * `limits` - The resource limits to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the limits were set successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not registered or limits could not be set.
    pub async fn set_resource_limits(&self, process_id: Uuid, limits: ResourceLimits) -> Result<()> {
        let mut resource_limits = self.resource_limits.write().await;
        
        if !self.process_map.read().await.contains_key(&process_id) {
            return Err(ResourceMonitorError::ProcessNotRegistered(process_id).into());
        }
        
        resource_limits.insert(process_id, limits);
        Ok(())
    }

    /// Gets the current resource limits for a monitored process
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process
    ///
    /// # Returns
    ///
    /// Returns Ok(ResourceLimits) if the limits were retrieved successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not registered or limits could not be retrieved.
    pub async fn get_resource_limits(&self, process_id: Uuid) -> Result<ResourceLimits> {
        let resource_limits = self.resource_limits.read().await;
        
        if !self.process_map.read().await.contains_key(&process_id) {
            return Err(ResourceMonitorError::ProcessNotRegistered(process_id).into());
        }
        
        match resource_limits.get(&process_id) {
            Some(limits) => Ok(limits.clone()),
            None => Err(ResourceMonitorError::ProcessNotFound(process_id).into())
        }
    }

    /// Checks if a process is registered with the resource monitor
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin to check
    ///
    /// # Returns
    ///
    /// Returns Ok(true) if the process is registered, Ok(false) otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if checking the process registration status fails.
    pub async fn is_process_registered(&self, process_id: Uuid) -> Result<bool> {
        let process_map = self.process_map.read().await;
        Ok(process_map.contains_key(&process_id))
    }

    /// Unregisters a process from resource monitoring
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process to unregister
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the process was unregistered successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process was not registered.
    pub async fn unregister_process(&self, process_id: Uuid) -> Result<()> {
        let mut process_map = self.process_map.write().await;
        
        if !process_map.contains_key(&process_id) {
            return Err(ResourceMonitorError::ProcessNotRegistered(process_id).into());
        }
        
        process_map.remove(&process_id);
        
        // Also clean up any usage data and resource limits
        {
            let mut usage_data = self.usage_data.write().await;
            usage_data.remove(&process_id);
        }
        
        {
            let mut resource_limits = self.resource_limits.write().await;
            resource_limits.remove(&process_id);
        }
        
        Ok(())
    }

    /// Starts the resource monitoring background task
    ///
    /// This spawns a background task that periodically measures resource usage
    /// of all registered processes and checks against their limits.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if monitoring was started successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if monitoring could not be started.
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.is_monitoring {
            return Ok(());
        }

        let process_map_clone = Arc::clone(&self.process_map);
        let usage_data_clone = Arc::clone(&self.usage_data);
        let resource_limits_clone = Arc::clone(&self.resource_limits);
        let interval = self.monitor_interval;

        tokio::spawn(async move {
            let mut interval_timer = time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let process_ids = {
                    let process_map = process_map_clone.read().await;
                    process_map.keys().cloned().collect::<Vec<_>>()
                };
                
                for process_id in process_ids {
                    let process_info_opt = {
                        let process_map = process_map_clone.read().await;
                        process_map.get(&process_id).cloned()
                    };
                    
                    if let Some(process_info) = process_info_opt {
                        match Self::measure_process_resources(process_info.process_handle, process_info.executable_path.as_path()).await {
                            Ok(usage) => {
                                // Store the usage data
                                {
                                    let mut usage_data = usage_data_clone.write().await;
                                    usage_data.insert(process_id, usage.clone());
                                }
                                
                                // Check against limits
                                let limits_opt = {
                                    let resource_limits = resource_limits_clone.read().await;
                                    resource_limits.get(&process_id).cloned()
                                };
                                
                                if let Some(limits) = limits_opt {
                                    if let Err(e) = Self::check_resource_limits(process_id, &usage, &limits).await {
                                        error!("Resource limit exceeded for process {}: {}", process_id, e);
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to measure resources for process {}: {}", process_id, e);
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Measures resources for all registered processes
    ///
    /// This method forces an immediate measurement of resource usage for all
    /// registered processes and updates the internal usage data.
    ///
    /// # Returns
    ///
    /// Returns a HashMap mapping process IDs to their resource usage.
    ///
    /// # Errors
    ///
    /// Returns an error if measuring resources fails.
    pub async fn measure_all_resources(&self) -> Result<HashMap<Uuid, ResourceUsage>> {
        let process_ids = {
            let process_map = self.process_map.read().await;
            process_map.keys().cloned().collect::<Vec<_>>()
        };
        
        let mut all_usage = HashMap::new();
        
        for process_id in process_ids {
            let process_info_opt = {
                let process_map = self.process_map.read().await;
                process_map.get(&process_id).cloned()
            };
            
            if let Some(process_info) = process_info_opt {
                match Self::measure_process_resources(process_info.process_handle, process_info.executable_path.as_path()).await {
                    Ok(usage) => {
                        all_usage.insert(process_id, usage.clone());
                        
                        // Also update the internal usage data
                        let mut usage_data = self.usage_data.write().await;
                        usage_data.insert(process_id, usage);
                    },
                    Err(e) => {
                        error!("Failed to measure resources for process {}: {}", process_id, e);
                    }
                }
            }
        }
        
        Ok(all_usage)
    }

    async fn check_resource_limits(process_id: Uuid, usage: &ResourceUsage, limits: &ResourceLimits) -> std::result::Result<(), ResourceMonitorError> {
        if (usage.cpu_percent as f64) > (limits.max_cpu_percent as f64) {
            return Err(ResourceMonitorError::ResourceLimitExceeded { 
                process_id, 
                resource_type: "CPU".to_string(), 
                current_value: usage.cpu_percent as f64, 
                limit: limits.max_cpu_percent as f64 
            });
        }
        
        if (usage.memory_bytes as f64) > (limits.max_memory_bytes as f64) {
            return Err(ResourceMonitorError::ResourceLimitExceeded { 
                process_id, 
                resource_type: "Memory".to_string(), 
                current_value: usage.memory_bytes as f64, 
                limit: limits.max_memory_bytes as f64 
            });
        }
        
        if (usage.disk_mb as f64) > (limits.max_disk_mb as f64) {
            return Err(ResourceMonitorError::ResourceLimitExceeded { 
                process_id, 
                resource_type: "Disk".to_string(), 
                current_value: usage.disk_mb as f64, 
                limit: limits.max_disk_mb as f64 
            });
        }
        
        Ok(())
    }

    async fn measure_process_resources(pid: u32, executable_path: &Path) -> std::result::Result<ResourceUsage, ResourceMonitorError> {
        #[cfg(target_os = "windows")]
        {
            Self::measure_windows_resources(pid).await
        }
        
        #[cfg(target_os = "linux")]
        {
            Self::measure_linux_resources(pid).await
        }
        
        #[cfg(target_os = "macos")]
        {
            Self::measure_macos_resources(pid).await
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(ResourceMonitorError::PlatformNotSupported("Unsupported platform".to_string()))
        }
    }

    #[cfg(target_os = "windows")]
    async fn measure_windows_resources(pid: u32) -> std::result::Result<ResourceUsage, ResourceMonitorError> {
        // Use PowerShell to get process info for Windows
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Get-Process -Id {} | Select-Object -Property Name,CPU,WorkingSet,PagedMemorySize | ConvertTo-Json",
                    pid
                ),
            ])
            .output()
            .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(ResourceMonitorError::CommandExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let json_output = String::from_utf8_lossy(&output.stdout).to_string();
        Self::parse_powershell_output(json_output)
    }

    fn parse_powershell_output(output: String) -> std::result::Result<ResourceUsage, ResourceMonitorError> {
        // Parse the PowerShell JSON output
        fn parse_powershell_value(output: &str, field: &str) -> std::result::Result<f64, ResourceMonitorError> {
            if let Some(cap) = Regex::new(&format!(r#""{field}"\s*:\s*(\d+)"#))
                .map_err(|e| ResourceMonitorError::ParseError(e.to_string()))?
                .captures(output)
            {
                if let Some(value) = cap.get(1) {
                    return value.as_str().parse::<f64>()
                        .map_err(|e| ResourceMonitorError::ParseError(e.to_string()));
                }
            }
            Err(ResourceMonitorError::ParseError(format!("Failed to extract {} from output", field)))
        }

        let cpu = parse_powershell_value(&output, "CPU")?;
        let memory = parse_powershell_value(&output, "WorkingSet")? / (1024.0 * 1024.0); // Convert to MB
        
        Ok(ResourceUsage {
            cpu_percent: cpu as f32,
            memory_bytes: memory as u64,
            disk_mb: 0.0,       // Not easily available in PowerShell output, would need additional commands
            network_mb: 0.0,     // Not easily available in PowerShell output, would need additional commands
            timestamp: chrono::Utc::now(),
        })
    }

    #[cfg(target_os = "linux")]
    async fn measure_linux_resources(pid: u32) -> std::result::Result<ResourceUsage, ResourceMonitorError> {
        // Get CPU and memory usage from /proc filesystem
        let status_output = Command::new("cat")
            .arg(format!("/proc/{}/status", pid))
            .output()
            .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;

        if !status_output.status.success() {
            return Err(ResourceMonitorError::CommandExecutionFailed(
                String::from_utf8_lossy(&status_output.stderr).to_string(),
            ));
        }

        let status_str = String::from_utf8_lossy(&status_output.stdout).to_string();
        
        // Get CPU usage
        let cpu_output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "%cpu", "--no-headers"])
            .output()
            .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;

        if !cpu_output.status.success() {
            return Err(ResourceMonitorError::CommandExecutionFailed(
                String::from_utf8_lossy(&cpu_output.stderr).to_string(),
            ));
        }

        let cpu_str = String::from_utf8_lossy(&cpu_output.stdout).trim().to_string();
        let cpu_percent = cpu_str.parse::<f32>()
            .map_err(|e| ResourceMonitorError::ParseError(format!("Failed to parse CPU usage: {}", e)))?;

        // Parse memory usage from status output
        let memory_mb = Self::parse_proc_status_value(&status_str, "VmRSS")? / 1024.0; // Convert from KB to MB

        Ok(ResourceUsage {
            cpu_percent,
            memory_bytes: memory_mb as u64,
            disk_mb: 0.0,      // Would need additional commands to track disk usage per process
            network_mb: 0.0,    // Would need additional commands to track network usage per process
            timestamp: chrono::Utc::now(),
        })
    }

    #[cfg(target_os = "linux")]
    fn parse_proc_status_value(status: &str, field: &str) -> std::result::Result<f32, ResourceMonitorError> {
        for line in status.lines() {
            if line.starts_with(field) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<f32>()
                        .map_err(|e| ResourceMonitorError::ParseError(format!("Failed to parse {}: {}", field, e)));
                }
            }
        }
        Err(ResourceMonitorError::ParseError(format!("Field {} not found in status output", field)))
    }

    #[cfg(target_os = "macos")]
    async fn measure_macos_resources(pid: u32) -> std::result::Result<ResourceUsage, ResourceMonitorError> {
        // For macOS, use ps command for CPU and memory usage
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "%cpu,%mem,rss", "--no-headers"])
            .output()
            .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(ResourceMonitorError::CommandExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = output_str.split_whitespace().collect();
        
        if parts.len() < 3 {
            return Err(ResourceMonitorError::ParseError("Invalid output format from ps command".to_string()));
        }

        let cpu_percent = parts[0].parse::<f32>()
            .map_err(|e| ResourceMonitorError::ParseError(format!("Failed to parse CPU usage: {}", e)))?;
            
        let memory_mb = parts[2].parse::<f32>()
            .map_err(|e| ResourceMonitorError::ParseError(format!("Failed to parse RSS: {}", e)))? / 1024.0; // Convert from KB to MB

        Ok(ResourceUsage {
            cpu_percent,
            memory_bytes: memory_mb as u64,
            disk_mb: 0.0,      // Would need additional commands to track disk usage per process
            network_mb: 0.0,    // Would need additional commands to track network usage per process
            timestamp: chrono::Utc::now(),
        })
    }

    /// Terminates a monitored process
    ///
    /// This method forcibly kills a process that is being monitored and
    /// unregisters it from the monitoring system.
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process to kill
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the process was killed successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process was not registered or could not be killed.
    pub async fn kill_process(&self, process_id: Uuid) -> Result<()> {
        let process_handle_opt = {
            let process_map = self.process_map.read().await;
            process_map.get(&process_id).map(|info| info.process_handle)
        };

        match process_handle_opt {
            Some(process_handle) => {
                #[cfg(target_os = "windows")]
                {
                    Command::new("taskkill")
                        .args(["/F", "/PID", &process_handle.to_string()])
                        .output()
                        .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;
                }

                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    Command::new("kill")
                        .args(["-9", &process_handle.to_string()])
                        .output()
                        .map_err(|e| ResourceMonitorError::CommandExecutionFailed(e.to_string()))?;
                }

                self.unregister_process(process_id).await
            }
            None => Err(ResourceMonitorError::ProcessNotRegistered(process_id).into()),
        }
    }

    /// Gets the current resource usage for a monitored process
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process
    ///
    /// # Returns
    ///
    /// Returns Ok(Option<ResourceUsage>) if the usage was retrieved successfully.
    /// The Option will be None if no usage data is available for the process yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not registered.
    pub async fn get_resource_usage(&self, process_id: Uuid) -> Result<Option<ResourceUsage>> {
        let usage_data = self.usage_data.read().await;
        
        if !self.process_map.read().await.contains_key(&process_id) {
            return Err(ResourceMonitorError::ProcessNotRegistered(process_id).into());
        }
        
        Ok(usage_data.get(&process_id).cloned())
    }

    /// Sets resource usage data for a process (used primarily for testing)
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID of the plugin/process
    /// * `usage` - The resource usage data to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the usage data was set successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not registered.
    #[cfg(test)]
    pub async fn set_resource_usage_for_testing(&self, process_id: Uuid, usage: ResourceUsage) -> Result<()> {
        if !self.process_map.read().await.contains_key(&process_id) {
            return Err(ResourceMonitorError::ProcessNotRegistered(process_id).into());
        }
        
        let mut usage_data = self.usage_data.write().await;
        usage_data.insert(process_id, usage);
        Ok(())
    }

    /// Gets the platform name
    ///
    /// # Returns
    ///
    /// The platform name as a string
    pub fn get_platform_name() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            return "windows";
        }
        
        #[cfg(target_os = "linux")]
        {
            return "linux";
        }
        
        #[cfg(target_os = "macos")]
        {
            return "macos";
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            return "unknown";
        }
    }

    /// Gets the process ID of a registered process
    ///
    /// # Arguments
    ///
    /// * `process_id` - The UUID associated with the plugin
    ///
    /// # Returns
    ///
    /// Returns the system process ID (PID) if the process is registered.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is not registered.
    pub async fn get_process_id(&self, process_id: Uuid) -> Result<u32> {
        let process_map = self.process_map.read().await;
        if let Some(process_info) = process_map.get(&process_id) {
            Ok(process_info.process_handle)
        } else {
            Err(ResourceMonitorError::ProcessNotRegistered(process_id).into())
        }
    }

    /// Check if advanced metrics are available
    pub fn has_advanced_metrics() -> bool {
        let platform = Self::get_platform_name();
        match platform {
            "windows" => {
                // Check for Performance Data Helper (PDH) API
                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    
                    // Try to check if PDH is available by running a simple PowerShell command
                    let pdh_available = Command::new("powershell")
                        .arg("-Command")
                        .arg("Get-Counter -ListSet * | Select-Object -First 1")
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false);
                        
                    pdh_available
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            },
            "linux" => {
                // Check for advanced monitoring tools like perf
                #[cfg(target_os = "linux")]
                {
                    // Check for /proc/pid/smaps (detailed memory information)
                    let smaps_exists = std::path::Path::new("/proc/self/smaps").exists();
                    
                    // Check for perf
                    let perf_available = std::process::Command::new("which")
                        .arg("perf")
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false);
                        
                    smaps_exists && perf_available
                }
                
                #[cfg(not(target_os = "linux"))]
                {
                    false
                }
            },
            "macos" => {
                // Check for advanced monitoring tools
                #[cfg(target_os = "macos")]
                {
                    // Check for dtrace
                    let dtrace_available = std::process::Command::new("which")
                        .arg("dtrace")
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false);
                        
                    dtrace_available
                }
                
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            },
            _ => false,
        }
    }

    /// Check if resource throttling is supported
    pub fn supports_resource_throttling() -> bool {
        let platform = Self::get_platform_name();
        match platform {
            "windows" => {
                // Job Objects on Windows support throttling
                #[cfg(target_os = "windows")]
                {
                    // Job Objects are available on all supported Windows versions
                    true
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            },
            "linux" => {
                // Check for cgroups which support throttling
                #[cfg(target_os = "linux")]
                {
                    // Check if CPU controller is available
                    let cpu_controller_available = std::path::Path::new("/sys/fs/cgroup/cpu").exists() ||
                        std::path::Path::new("/sys/fs/cgroup/cpu,cpuacct").exists();
                    
                    // Check if cgroups v2 is available (which also supports throttling)
                    let cgroups_v2_available = std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists();
                    
                    cpu_controller_available || cgroups_v2_available
                }
                
                #[cfg(not(target_os = "linux"))]
                {
                    false
                }
            },
            "macos" => {
                // Check for resource throttling capabilities on macOS
                #[cfg(target_os = "macos")]
                {
                    // macOS doesn't have standard throttling capabilities like Linux cgroups
                    // but resource limits can be applied with setrlimit
                    true
                }
                
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            },
            _ => false,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::path::PathBuf;

    #[test]
    fn test_create_resource_monitor() {
        let monitor = ResourceMonitor::new();
        assert!(!monitor.is_monitoring);
    }

    #[test]
    fn test_set_monitor_interval() {
        let mut monitor = ResourceMonitor::new();
        monitor.set_monitor_interval(Duration::from_secs(10));
        assert_eq!(monitor.monitor_interval, Duration::from_secs(10));
    }

    #[test]
    fn test_enable_disable_monitoring() {
        let mut monitor = ResourceMonitor::new();
        monitor.enable_monitoring();
        assert!(monitor.is_monitoring);
        monitor.disable_monitoring();
        assert!(!monitor.is_monitoring);
    }

    #[test]
    fn test_register_unregister_process() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let monitor = ResourceMonitor::new();
            let process_id = Uuid::new_v4();
            let result = monitor.register_process(process_id, 1234, &PathBuf::from("/path/to/executable")).await;
            assert!(result.is_ok());
            
            let result = monitor.unregister_process(process_id).await;
            assert!(result.is_ok());
            
            let result = monitor.unregister_process(process_id).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_resource_limits() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let monitor = ResourceMonitor::new();
            let process_id = Uuid::new_v4();
            
            // Try to set limits for non-registered process
            let result = monitor.set_resource_limits(process_id, ResourceLimits::default()).await;
            assert!(result.is_err());
            
            // Register process and try again
            let _ = monitor.register_process(process_id, 1234, &PathBuf::from("/path/to/executable")).await;
            let result = monitor.set_resource_limits(process_id, ResourceLimits::default()).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_platform_name() {
        let platform = ResourceMonitor::get_platform_name();
        println!("Platform name from test: {}", platform);
        assert!(!platform.is_empty());
        
        // Also test the expected platform for this OS
        #[cfg(target_os = "windows")]
        assert_eq!(platform, "windows");
        
        #[cfg(target_os = "linux")]
        assert_eq!(platform, "linux");
        
        #[cfg(target_os = "macos")]
        assert_eq!(platform, "macos");
    }
} 