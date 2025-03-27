//! Resource metrics collection for system monitoring
//! 
//! Tracks system resource usage including:
//! - Memory usage per team
//! - Thread memory usage
//! - Storage usage
//! - Network bandwidth

use squirrel_core::error::{Result, SquirrelError};
use crate::metrics::{Metric, MetricCollector, MetricType};
// Import all the required trait extensions for sysinfo
use sysinfo::{SystemExt, ProcessExt, NetworkExt, CpuExt, PidExt, DiskExt, RefreshKind, NetworksExt, System, Process, ProcessStatus};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::metrics::performance::PerformanceCollectorAdapter;
use chrono;
use tracing::{debug, error, warn, info};
use anyhow::{anyhow};
use tokio::time::{sleep, Duration};
use serde_json::{json, Value};
use std::process::Command;
use std::str::FromStr;
use std::io::Read;
use std::fs::File;
use std::time::Instant;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::metrics::types::{
    CpuMetrics, DiskMetrics, MemoryMetrics, MetricsCollectorFactory,
    MetricsError, NetworkMetrics, ResourceMetricsCollector as ResourceMetricsCollectorTrait
};

/// Information about a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Number of threads
    pub thread_count: u32,
    /// Disk read bytes
    pub disk_read_bytes: u64,
    /// Disk write bytes
    pub disk_write_bytes: u64,
    /// Process status
    pub status: String,
}

/// Represents resource metrics for a team, including memory, storage, network, CPU, and process information.
/// 
/// This struct aggregates various system resource metrics associated with a specific team,
/// providing a comprehensive view of the team's resource utilization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceMetrics {
    /// Team identifier
    pub team_id: String,
    /// Memory usage in bytes.
    pub memory_usage: f64,
    /// Storage usage in bytes.
    pub storage_usage: f64,
    /// Network bandwidth usage in bits per second.
    pub network_bandwidth: f64,
    /// Number of active threads.
    pub thread_count: u32,
    /// Disk I/O statistics in bytes per second.
    pub disk_io: f64,
    /// CPU usage as a percentage (0.0 to 100.0).
    pub cpu_usage: f64,
    /// List of processes owned by the team.
    pub processes: Vec<ProcessInfo>,
    /// Timestamp of the metrics
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Labels for the metrics
    pub labels: HashMap<String, String>,
}

/// Disk I/O statistics
#[derive(Debug, Clone)]
pub struct DiskIOStats {
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Read operations per second
    pub reads_per_sec: f64,
    /// Write operations per second
    pub writes_per_sec: f64,
}

impl Default for DiskIOStats {
    fn default() -> Self {
        Self {
            bytes_read: 0,
            bytes_written: 0,
            reads_per_sec: 0.0,
            writes_per_sec: 0.0,
        }
    }
}

/// Resource metrics collector that monitors system and team resource usage
#[derive(Debug)]
pub struct ResourceMetricsCollector {
    /// System information collector
    system: Arc<RwLock<System>>,
    /// Team resource metrics
    metrics: Arc<RwLock<Vec<Metric>>>,
    /// Team workspace paths
    team_paths: Arc<RwLock<HashMap<String, PathBuf>>>,
    /// Previous disk I/O measurements
    prev_disk_io: Arc<RwLock<HashMap<String, DiskIOStats>>>,
    /// Performance collector adapter
    #[allow(dead_code)]
    performance_collector: Option<Arc<PerformanceCollectorAdapter>>,
    /// Configuration
    config: ResourceConfig,
}

impl ResourceMetricsCollector {
    /// Create a new resource metrics collector with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            team_paths: Arc::new(RwLock::new(HashMap::new())),
            prev_disk_io: Arc::new(RwLock::new(HashMap::new())),
            performance_collector: None,
            config: ResourceConfig::default(),
        }
    }

    /// Creates a new resource metrics collector with the specified configuration
    #[must_use]
    pub fn with_config(config: ResourceConfig) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            team_paths: Arc::new(RwLock::new(HashMap::new())),
            prev_disk_io: Arc::new(RwLock::new(HashMap::new())),
            performance_collector: None,
            config,
        }
    }

    /// Creates a new resource metrics collector with dependencies
    #[must_use]
    pub fn with_dependencies(
        config: ResourceConfig,
        performance_collector: Option<Arc<PerformanceCollectorAdapter>>,
    ) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            team_paths: Arc::new(RwLock::new(HashMap::new())),
            prev_disk_io: Arc::new(RwLock::new(HashMap::new())),
            performance_collector,
            config,
        }
    }

    /// Update resource metrics for all teams
    pub async fn update_metrics(&mut self) -> Result<()> {
        self.system.write().await.refresh_all();

        let team_paths = self.team_paths.read().await;
        let mut prev_disk_io = self.prev_disk_io.write().await;
        
        // Update metrics for each team
        for (team_name, team_path) in team_paths.iter() {
            // Get processes for this team
            let team_processes = self.get_team_processes_locked(team_name).await;
            
            // Calculate aggregate team metrics
            let memory_usage = team_processes.iter()
                .map(|p| p.memory_usage as f64)
                .sum::<f64>();

            let thread_count: u32 = team_processes.iter()
                .map(|p| p.thread_count)
                .sum();

            // Update storage usage if team path is configured
            let storage_usage = self.calculate_storage_usage_locked(team_path).await;
            
            // Update disk I/O statistics
            let current_io = self.calculate_disk_io_locked(team_path).await;
            let _prev_io = prev_disk_io.get(team_name).cloned().unwrap_or_default();
            
            // Calculate disk I/O changes
            let disk_io = current_io.writes_per_sec + current_io.reads_per_sec;
            
            // Update the previous I/O stats for next time
            prev_disk_io.insert(team_name.clone(), current_io);

            // Calculate CPU usage
            let cpu_usage = self.calculate_cpu_usage_locked(team_name).await;
            
            // Calculate network bandwidth
            let network_bandwidth = self.calculate_network_bandwidth_locked(team_name).await;

            // Update metrics
            let mut team_metric = Metric::with_optional_labels(
                format!("resource.team.{team_name}"),
                memory_usage,
                MetricType::Gauge,
                Some(HashMap::new()),
            );
            team_metric.labels.insert("team".to_string(), team_name.clone());
            team_metric.labels.insert("type".to_string(), "resource".to_string());
            team_metric.labels.insert("storage_usage".to_string(), storage_usage.to_string());
            team_metric.labels.insert("thread_count".to_string(), thread_count.to_string());
            team_metric.labels.insert("disk_io".to_string(), disk_io.to_string());
            team_metric.labels.insert("cpu_usage".to_string(), cpu_usage.to_string());
            team_metric.labels.insert("network_bandwidth".to_string(), network_bandwidth.to_string());
            team_metric.labels.insert("process_count".to_string(), team_processes.len().to_string());
            
            self.metrics.write().await.push(team_metric);
        }
        Ok(())
    }

    /// Get processes related to a specific team
    async fn get_team_processes_locked(&self, team_name: &str) -> Vec<ProcessInfo> {
        let system = self.system.read().await;
        let mut result = Vec::new();
        
        // Iterate over all processes
        for process in system.processes().values() {
            if Self::is_team_process(process, team_name) {
                result.push(Self::collect_process_info(process));
            }
        }
        
        result
    }

    /// Helper method for storage usage that works with RwLockReadGuard
    async fn calculate_storage_usage_locked(&self, team_path: &Path) -> f64 {
        let system = self.system.read().await;
        Self::calculate_storage_usage(system, team_path)
    }

    /// Helper method for disk IO that works with RwLockReadGuard
    async fn calculate_disk_io_locked(&self, team_path: &Path) -> DiskIOStats {
        let system = self.system.read().await;
        Self::calculate_disk_io(system, team_path)
    }

    /// Helper method for CPU usage that works with RwLockReadGuard
    async fn calculate_cpu_usage_locked(&self, team_name: &str) -> f64 {
        let system = self.system.read().await;
        Self::calculate_cpu_usage(system, team_name)
    }

    /// Helper method for network bandwidth that works with RwLockReadGuard
    async fn calculate_network_bandwidth_locked(&self, _team_name: &str) -> f64 {
        // Create a persistent System instance to ensure networks remain valid
        let mut system = System::new();
        system.refresh_networks();
        let networks = system.networks();
        
        // Calculate total bandwidth across all network interfaces
        let bandwidth: f64 = networks.iter()
            .map(|(_, network)| {
                let received = network.total_received() as f64;
                let transmitted = network.total_transmitted() as f64;
                received + transmitted
            })
            .sum();
        
        bandwidth
    }

    /// Calculate storage usage for a team's workspace
    fn calculate_storage_usage(system: &System, path: &Path) -> f64 {
        // First try our calculation method
        match Self::calculate_dir_size(path) {
            Ok(size) => size as f64,
            Err(_) => {
                // If that fails, use the UI team's method
                let disks = system.disks();
                
                // Calculate storage usage for the given path
                if let Ok(metadata) = std::fs::metadata(path) {
                    let total_space = metadata.len() as f64;
                    let free_space = disks.iter()
                        .filter(|disk| Path::new(disk.mount_point()).starts_with(path))
                        .map(|disk| disk.available_space() as f64)
                        .sum::<f64>();

                    if total_space > 0.0 {
                        (total_space - free_space) / total_space
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
        }
    }

    /// Calculate disk I/O statistics for a team's workspace
    fn calculate_disk_io(system: &System, path: &Path) -> DiskIOStats {
        // Create a fresh Disks instance with refreshed data
        let disks = system.disks();
        
        DiskIOStats {
            bytes_read: 0,
            bytes_written: 0,
            reads_per_sec: 0.0,
            writes_per_sec: 0.0,
        }
    }

    /// Calculate the total size of a directory using a non-recursive approach
    #[allow(dead_code)]
    fn calculate_dir_size(start_path: &Path) -> std::io::Result<u64> {
        let mut total_size = 0u64;
        let mut dirs_to_visit = vec![start_path.to_path_buf()];
        
        while let Some(path) = dirs_to_visit.pop() {
            if path.is_dir() {
                for entry in std::fs::read_dir(&path)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        total_size += entry.metadata()?.len();
                    } else if path.is_dir() {
                        dirs_to_visit.push(path);
                    }
                }
            }
        }
        
        Ok(total_size)
    }

    /// Get processes belonging to a team
    fn get_team_processes<'a>(
        system: &'a System,
        team_name: &str,
    ) -> Vec<&'a Process> {
        system.processes()
            .values()
            .filter(|p| Self::is_team_process(p, team_name))
            .collect()
    }

    /// Check if a process belongs to a team
    fn is_team_process(process: &Process, team_name: &str) -> bool {
        // This is a simplistic implementation based on name matching
        // In a real-world implementation, this would use more sophisticated
        // techniques like checking environment variables, cgroups, etc.
        let process_name = process.name().to_lowercase();
        process_name.contains(&team_name.to_lowercase())
    }

    /// Collects information about a process and converts it into a ProcessInfo struct
    /// 
    /// This function extracts various metrics from a system process including CPU usage,
    /// memory usage, thread count, disk I/O, and status information.
    /// 
    /// # Arguments
    /// 
    /// * `process` - Reference to a Process object to collect information from
    /// 
    /// # Returns
    /// 
    /// A ProcessInfo struct containing the collected metrics
    fn collect_process_info(process: &Process) -> ProcessInfo {
        // Determine status
        let status = match process.status() {
            ProcessStatus::Run => "Running",
            ProcessStatus::Sleep => "Sleeping",
            ProcessStatus::Stop => "Stopped",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Idle => "Idle",
            _ => "Unknown",
        };
        
        // Set a default thread count (sysinfo v0.30 doesn't have a direct thread count method)
        let thread_count = 1; // Default to 1 thread per process

        ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            thread_count,
            disk_read_bytes: process.disk_usage().read_bytes,
            disk_write_bytes: process.disk_usage().written_bytes,
            status: status.to_string(),
        }
    }

    /// Get current resource metrics for a team
    ///
    /// Retrieves the most recently collected resource metrics for the specified team.
    /// This method searches through the metrics collection for metrics associated with
    /// the given team name and converts them into a TeamResourceMetrics object.
    ///
    /// # Parameters
    ///
    /// * `team_name` - The name of the team for which to retrieve metrics
    ///
    /// # Returns
    ///
    /// Returns `Some(TeamResourceMetrics)` if metrics for the specified team are found,
    /// or `None` if no metrics are available for that team. The returned metrics include
    /// memory usage, storage usage, network bandwidth, CPU usage, and other resource statistics.
    pub async fn get_team_metrics(&self, team_name: &str) -> Option<TeamResourceMetrics> {
        let metrics_guard = self.metrics.read().await;
        let metrics = crate::metrics::read_guard_to_vec(&metrics_guard);
        
        // Find the metric with the matching team name and convert it to TeamResourceMetrics
        for metric in metrics {
            if metric.name == team_name {
                if let Some(team_label) = metric.labels.get("team") {
                    if team_label == team_name {
                        // Create a TeamResourceMetrics from the metric data
                        return Some(TeamResourceMetrics {
                            team_id: team_name.to_string(),
                            memory_usage: metric.value,
                            storage_usage: metric.labels.get("storage_usage")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(0.0),
                            network_bandwidth: metric.labels.get("network_bandwidth")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(0.0),
                            thread_count: metric.labels.get("thread_count")
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(0),
                            disk_io: metric.labels.get("disk_io")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(0.0),
                            cpu_usage: metric.labels.get("cpu_usage")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(0.0),
                            processes: Vec::new(),
                            timestamp: chrono::Utc::now(),
                            labels: metric.labels.clone(),
                        });
                    }
                }
            }
        }
        
        None
    }

    /// Register a new team for resource tracking
    pub async fn register_team(&self, team_name: String, workspace_path: PathBuf) {
        let mut team_paths = self.team_paths.write().await;
        team_paths.insert(team_name.clone(), workspace_path);
        
        // Initialize metrics for the team
        let mut metrics = self.metrics.write().await;
        let mut labels = HashMap::new();
        labels.insert("team".to_string(), team_name.clone());
        labels.insert("type".to_string(), "resource".to_string());
        
        metrics.push(Metric::with_optional_labels(
            team_name,
            0.0, // Initial memory usage
            MetricType::Gauge,
            Some(labels),
        ));
    }

    /// Start periodic metrics collection
    ///
    /// # Panics
    ///
    /// This function panics if the Tokio runtime cannot be created when attempting to spawn 
    /// the background metrics collection task. This can happen if there are system resource 
    /// constraints or if there are issues initializing the runtime.
    pub async fn start_collection(&self) {
        let mut collector = self.clone();
        std::thread::spawn(move || {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(async move {
                        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                        loop {
                            interval.tick().await;
                            if let Err(e) = collector.update_metrics().await {
                                eprintln!("Error updating metrics: {e}");
                            }
                        }
                    });
                },
                Err(e) => {
                    eprintln!("Failed to create Tokio runtime for metrics collection: {e}");
                }
            }
        });
    }

    /// Collect system resource metrics
    pub fn collect_system_metrics(&self) -> Result<TeamResourceMetrics> {
        // Create a new System and refresh all components
        let mut system = System::new_all();
        system.refresh_all();
        
        // Calculate CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        // Calculate memory usage
        let memory_usage = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        
        // Calculate storage usage - using System's disks
        let storage_usage = {
            let disks = system.disks();
            if disks.is_empty() {
                0.0
            } else {
                disks.iter()
                    .map(|disk| {
                        let total = disk.total_space();
                        let available = disk.available_space();
                        if total == 0 {
                            0.0
                        } else {
                            ((total - available) as f64 / total as f64) * 100.0
                        }
                    })
                    .fold(0.0, |acc, x| acc + x) / disks.len() as f64
            }
        };
        
        // Collect network bandwidth (simplified)
        let network_bandwidth = {
            // Get network information
            let system = System::new();
            let networks = system.networks();
            networks.iter()
                .map(|(_, network)| (network.total_received() + network.total_transmitted()) as f64)
                .sum::<f64>()
        };

        // Calculate thread count safely
        let thread_count: u32 = system.processes()
            .values()
            .map(|_| 1u32) // Just count each process as having 1 thread
            .sum();
        
        // Calculate disk I/O
        let disk_io = Self::calculate_disk_io(&system, &std::env::current_dir()?);
        
        // Return system metrics as a TeamResourceMetrics
        Ok(TeamResourceMetrics {
            team_id: "system".to_string(),
            memory_usage,
            storage_usage,
            network_bandwidth,
            thread_count,
            disk_io: disk_io.bytes_read as f64,
            cpu_usage: f64::from(cpu_usage), // Convert f32 to f64
            processes: Vec::new(), // Don't include all system processes
            timestamp: chrono::Utc::now(),
            labels: HashMap::new(),
        })
    }

    /// Calculate CPU usage for a team
    fn calculate_cpu_usage(system: &System, team_name: &str) -> f64 {
        let team_processes = Self::get_team_processes(system, team_name);
        team_processes.iter()
            .map(|p| f64::from(p.cpu_usage()))
            .sum::<f64>()
    }

    /// Gets disk usage for a specific path
    #[must_use] pub fn get_disk_usage(&self, path: &Path) -> Option<f64> {
        // Get system disk information
        let system = System::new();
        let disk_info = system.disks()
            .iter()
            .find(|disk| path.starts_with(disk.mount_point()));
        
        disk_info.map(|disk| {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            ((total - available) / total) * 100.0
        })
    }

    /// Gets disk space for a specific path (used, total)
    #[must_use] pub fn get_disk_space(&self, path: &Path) -> Option<(u64, u64)> {
        // Get system disk information
        let system = System::new();
        let disk_info = system.disks()
            .iter()
            .find(|disk| path.starts_with(disk.mount_point()));
        
        disk_info.map(|disk| {
            (disk.total_space(), disk.available_space())
        })
    }

    /// Collect team metrics
    pub async fn collect_team_metrics(&self) -> Result<HashMap<String, TeamResourceMetrics>> {
        // Refresh system information
        self.system.write().await.refresh_all();
        
        let team_paths = self.team_paths.read().await;
        let mut results = HashMap::new();
        
        for (team_name, path) in team_paths.iter() {
            let team_processes = self.get_team_processes_locked(team_name).await;
            
            // Calculate aggregate memory usage
            let memory_usage = team_processes.iter()
                .map(|p| p.memory_usage as f64)
                .sum::<f64>();
            
            let storage_usage = self.calculate_storage_usage_locked(path).await;
            
            // Calculate network bandwidth
            let network_bandwidth = self.calculate_network_bandwidth_locked(team_name).await;
            
            let thread_count = team_processes.iter()
                .map(|p| p.thread_count)
                .sum();
            
            let disk_io = self.calculate_disk_io_locked(path).await;
            
            let cpu_usage = self.calculate_cpu_usage_locked(team_name).await;
            
            results.insert(team_name.clone(), TeamResourceMetrics {
                team_id: team_name.clone(),
                memory_usage,
                storage_usage,
                network_bandwidth,
                thread_count,
                disk_io: disk_io.reads_per_sec + disk_io.writes_per_sec,
                cpu_usage,
                processes: team_processes,
                timestamp: chrono::Utc::now(),
                labels: HashMap::new(),
            });
        }
        
        Ok(results)
    }
}

impl Clone for ResourceMetricsCollector {
    fn clone(&self) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            team_paths: Arc::new(RwLock::new(HashMap::new())),
            prev_disk_io: Arc::new(RwLock::new(HashMap::new())),
            performance_collector: None,
            config: self.config.clone(),
        }
    }
}

impl Default for ResourceMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for resource metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Whether to enable resource metrics collection
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval: u64,
    /// Maximum history size
    pub history_size: usize,
    /// Max process age to track in seconds
    pub max_process_age: u64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 60,
            history_size: 100,
            max_process_age: 300,
        }
    }
}

/// Factory for creating resource metrics collectors
pub struct ResourceMetricsCollectorFactory;

impl ResourceMetricsCollectorFactory {
    /// Creates a new factory with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: ResourceConfig::default(),
        }
    }

    /// Creates a new factory with the specified configuration
    #[must_use] pub const fn with_config(config: ResourceConfig) -> Self {
        Self { config }
    }

    /// Creates a new collector instance with dependency injection
    ///
    /// # Arguments
    /// * `performance_collector` - Optional performance collector adapter
    ///
    /// # Returns
    /// A new ResourceMetricsCollector instance wrapped in an Arc
    #[must_use]
    pub fn create_collector_with_dependencies(
        &self,
        performance_collector: Option<Arc<PerformanceCollectorAdapter>>,
    ) -> Arc<ResourceMetricsCollector> {
        Arc::new(ResourceMetricsCollector::with_dependencies(
            self.config.clone(),
            performance_collector,
        ))
    }

    /// Creates a new collector instance with the default configuration
    #[must_use]
    pub fn create_collector(&self) -> Arc<ResourceMetricsCollector> {
        self.create_collector_with_dependencies(None)
    }

    /// Creates a new collector adapter
    #[must_use]
    pub fn create_collector_adapter(&self) -> Arc<ResourceMetricsCollectorAdapter> {
        let collector = self.create_collector();
        Arc::new(ResourceMetricsCollectorAdapter::with_collector(collector))
    }
}

impl ResourceMetricsCollectorTrait for ResourceMetricsCollectorAdapter {
    fn collect_cpu_metrics(&mut self) -> Result<CpuMetrics, MetricsError> {
        if let Some(collector) = &self.inner {
            let system_metrics = collector.collect_system_metrics()?;
            
            Ok(CpuMetrics {
                usage_percentage: system_metrics.cpu_usage as f32,
                // You might want to add more detailed CPU metrics here
            })
        } else {
            Err(MetricsError::NotInitialized)
        }
    }
    
    fn collect_memory_metrics(&mut self) -> Result<MemoryMetrics, MetricsError> {
        if let Some(collector) = &self.inner {
            let system = System::new_all();
            
            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let available_memory = total_memory - used_memory;
            let usage_percentage = if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };
            
            Ok(MemoryMetrics {
                total_bytes: total_memory,
                used_bytes: used_memory,
                available_bytes: available_memory,
                usage_percentage,
            })
        } else {
            Err(MetricsError::NotInitialized)
        }
    }
    
    fn collect_disk_metrics(&mut self) -> Result<Vec<DiskMetrics>, MetricsError> {
        let mut disk_metrics = Vec::new();
        let system = System::new_all();
        
        for disk in system.disks() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percentage = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };
            
            disk_metrics.push(DiskMetrics {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: total_space,
                used_bytes: used_space,
                available_bytes: available_space,
                usage_percentage,
            });
        }
        
        Ok(disk_metrics)
    }
    
    fn collect_network_metrics(&mut self) -> Result<Vec<NetworkMetrics>, MetricsError> {
        let mut network_metrics = Vec::new();
        let system = System::new_all();
        
        for (interface_name, network) in system.networks() {
            network_metrics.push(NetworkMetrics {
                interface: interface_name.to_string(),
                received_bytes: network.received(),
                transmitted_bytes: network.transmitted(),
                // Add more network metrics as needed
            });
        }
        
        Ok(network_metrics)
    }
}