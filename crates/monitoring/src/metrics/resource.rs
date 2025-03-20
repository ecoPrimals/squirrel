//! Resource metrics collection for system monitoring
//! 
//! Tracks system resource usage including:
//! - Memory usage per team
//! - Thread memory usage
//! - Storage usage
//! - Network bandwidth

use squirrel_core::error::{Result, SquirrelError};
use crate::metrics::{Metric, MetricCollector, MetricType};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use sysinfo::{System, Process, Disks, Networks, ProcessStatus};
use async_trait::async_trait;
use crate::metrics::performance::PerformanceCollectorAdapter;
use chrono;

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
        Self::calculate_storage_usage(&system, team_path)
    }

    /// Helper method for disk IO that works with RwLockReadGuard
    async fn calculate_disk_io_locked(&self, team_path: &Path) -> DiskIOStats {
        let system = self.system.read().await;
        Self::calculate_disk_io(&system, team_path)
    }

    /// Helper method for CPU usage that works with RwLockReadGuard
    async fn calculate_cpu_usage_locked(&self, team_name: &str) -> f64 {
        let system = self.system.read().await;
        Self::calculate_cpu_usage(&system, team_name)
    }

    /// Helper method for network bandwidth that works with RwLockReadGuard
    async fn calculate_network_bandwidth_locked(&self, _team_name: &str) -> f64 {
        // In sysinfo 0.30, Networks needs to be created freshly
        let networks = Networks::new_with_refreshed_list();
        
        // Calculate total bandwidth across all network interfaces
        let network_bandwidth = networks.iter()
            .map(|(_, network)| {
                let received = network.received() as f64;
                let transmitted = network.transmitted() as f64;
                received + transmitted
            })
            .fold(0.0, |acc, x| acc + x);
            
        network_bandwidth
    }

    /// Calculate storage usage for a team's workspace
    fn calculate_storage_usage(_system: &System, path: &Path) -> f64 {
        // Create a fresh Disks instance with refreshed data
        let disks = Disks::new_with_refreshed_list();
        
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

    /// Calculate disk I/O statistics for a team's workspace
    fn calculate_disk_io(_system: &System, path: &Path) -> DiskIOStats {
        // Create a fresh Disks instance with refreshed data
        let disks = Disks::new_with_refreshed_list();
        
        // Calculate disk I/O for the given path
        let disk_io = disks.iter()
            .filter(|disk| Path::new(disk.mount_point()).starts_with(path))
            .fold(DiskIOStats::default(), |mut acc, disk| {
                // In sysinfo 0.30, disks don't provide direct read/write bytes
                // We'll use available/total space as a proxy
                let total = disk.total_space();
                let available = disk.available_space();
                acc.bytes_read += total - available;
                acc.bytes_written += 0; // Not directly available
                acc
            });

        disk_io
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
        let status = match process.status() {
            ProcessStatus::Run => "running",
            ProcessStatus::Sleep => "sleeping",
            ProcessStatus::Stop => "stopped",
            ProcessStatus::Zombie => "zombie",
            ProcessStatus::Tracing => "tracing",
            ProcessStatus::Dead => "dead",
            ProcessStatus::Idle => "idle",
            _ => "unknown",
        };

        // Get the thread count
        let thread_count = match process.thread_kind() {
            Some(_) => 1u32,
            None => 0u32,
        };

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
        
        // Calculate storage usage
        let storage_usage = {
            // In sysinfo 0.30, system.disks() doesn't exist, so we need to create a fresh Disks instance
            let disks = Disks::new_with_refreshed_list();
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
        };
        
        // Collect network bandwidth (simplified)
        let network_bandwidth = {
            // In sysinfo 0.30, system.networks() doesn't exist, so we need to create a fresh Networks instance
            let networks = Networks::new_with_refreshed_list();
            networks.iter()
                .map(|(_, network)| (network.received() + network.transmitted()) as f64)
                .fold(0.0, |acc, x| acc + x)
        };

        // Calculate thread count safely
        let thread_count: u32 = system.processes()
            .values()
            .map(|process| {
                // In sysinfo 0.30, thread_kind() returns an Option<ThreadKind>
                // We just want to count threads, so we'll return 1 if it has a thread_kind
                match process.thread_kind() {
                    Some(_) => 1u32, // If it's a thread, count it as 1
                    None => 0u32,    // If it's not a thread, count as 0
                }
            })
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
        // Create a new disks instance to get fresh disk data
        let disks = sysinfo::Disks::new_with_refreshed_list();
        
        if let Some(disk) = disks.iter().find(|d| path.starts_with(d.mount_point())) {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            Some(used as f64 / total as f64 * 100.0)
        } else {
            None
        }
    }

    /// Gets disk space for a specific path (used, total)
    #[must_use] pub fn get_disk_space(&self, path: &Path) -> Option<(u64, u64)> {
        // Create a new disks instance to get fresh disk data
        let disks = sysinfo::Disks::new_with_refreshed_list();
        
        if let Some(disk) = disks.iter().find(|d| path.starts_with(d.mount_point())) {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            Some((used, total))
        } else {
            None
        }
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
#[derive(Debug, Clone)]
pub struct ResourceMetricsCollectorFactory {
    /// Configuration for creating collectors
    config: ResourceConfig,
}

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

impl Default for ResourceMetricsCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new resource metrics collector adapter
#[must_use]
pub fn create_collector_adapter() -> Arc<ResourceMetricsCollectorAdapter> {
    ResourceMetricsCollectorFactory::new().create_collector_adapter()
}

/// Create a new resource metrics collector adapter with a specific collector
#[must_use]
pub fn create_collector_adapter_with_collector(
    collector: Arc<ResourceMetricsCollector>
) -> Arc<ResourceMetricsCollectorAdapter> {
    Arc::new(ResourceMetricsCollectorAdapter::with_collector(collector))
}

/// Adapter for the Resource Metrics Collector to support dependency injection
#[derive(Debug)]
pub struct ResourceMetricsCollectorAdapter {
    /// The inner collector instance
    inner: Option<Arc<ResourceMetricsCollector>>,
}

impl Default for ResourceMetricsCollectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMetricsCollectorAdapter {
    /// Create a new adapter with no inner collector
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }

    /// Create a new adapter with a specific collector
    #[must_use] pub fn with_collector(collector: Arc<ResourceMetricsCollector>) -> Self {
        Self {
            inner: Some(collector),
        }
    }

    /// Check if the adapter has a valid inner collector
    #[must_use] pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// Get resource metrics from the collector
    pub async fn get_team_metrics(&self) -> Result<HashMap<String, TeamResourceMetrics>> {
        if let Some(collector) = &self.inner {
            collector.collect_team_metrics().await
        } else {
            Err(SquirrelError::Generic("ResourceMetricsCollectorAdapter not initialized".to_string()))
        }
    }

    /// Get system metrics from the collector
    pub fn get_system_metrics(&self) -> Result<TeamResourceMetrics> {
        if let Some(collector) = &self.inner {
            collector.collect_system_metrics()
        } else {
            Err(SquirrelError::Generic("ResourceMetricsCollectorAdapter not initialized".to_string()))
        }
    }

    /// Register a team for resource metrics collection
    ///
    /// # Arguments
    /// * `team_name` - The name of the team
    /// * `workspace_path` - Path to the team's workspace
    pub async fn register_team(&self, team_name: String, workspace_path: PathBuf) {
        if let Some(collector) = &self.inner {
            collector.register_team(team_name, workspace_path).await;
        }
        // If not initialized, silently fail - can't register team
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metric(metric).await
        } else {
            Ok(())
        }
    }
}

impl Clone for ResourceMetricsCollectorAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
impl MetricCollector for ResourceMetricsCollectorAdapter {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        if let Some(collector) = &self.inner {
            collector.collect_metrics().await
        } else {
            Ok(Vec::new())
        }
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.record_metric(metric).await
        } else {
            Ok(())
        }
    }

    async fn start(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.start().await
        } else {
            Ok(())
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(collector) = &self.inner {
            collector.stop().await
        } else {
            Ok(())
        }
    }
}

impl TeamResourceMetrics {
    /// Create a team metrics object from a system
    #[allow(unused)]
    #[must_use] pub fn new(system: &System) -> Self {
        // Calculate CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        // Calculate memory usage
        let memory_usage = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        
        // Calculate storage usage - using fresh disks since system.disks() is not available
        let storage_usage = {
            let disks = Disks::new_with_refreshed_list();
            if disks.len() == 0 {
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
        
        // Calculate network bandwidth (simplified)
        let network_bandwidth = {
            // In sysinfo 0.30, system.networks() doesn't exist, so we need to create a fresh Networks instance
            let networks = Networks::new_with_refreshed_list();
            networks.iter()
                .map(|(_, network)| (network.received() + network.transmitted()) as f64)
                .fold(0.0, |acc, x| acc + x)
        };
        
        // Get thread count from all processes
        let thread_count: u32 = system.processes().iter()
            .map(|(_, process)| {
                // In sysinfo 0.30, thread_kind() returns an Option<ThreadKind>
                // We just want to count threads, so we'll return 1 if it has a thread_kind
                match process.thread_kind() {
                    Some(_) => 1u32, // If it's a thread, count it as 1
                    None => 0u32,    // If it's not a thread, count as 0
                }
            })
            .sum();
            
        // Calculate disk I/O (simplified)
        let disk_io = 0.0; // In a real system, this would track disk read/write rates
        
        // Collect process information
        let processes = system.processes().iter()
            .take(10) // Limit to top 10 processes by CPU
            .map(|(_, process)| Self::collect_process_info(process))
            .collect();
        
        Self {
            team_id: "system".to_string(),
            memory_usage,
            storage_usage,
            network_bandwidth,
            thread_count,
            disk_io,
            cpu_usage: f64::from(cpu_usage),
            processes,
            timestamp: chrono::Utc::now(),
            labels: HashMap::new(),
        }
    }
    
    /// Collects detailed information about a process into a ProcessInfo struct
    /// 
    /// # Arguments
    /// 
    /// * `process` - A reference to the Process object to extract information from
    /// 
    /// # Returns
    /// 
    /// Returns a ProcessInfo struct with CPU, memory, disk, and status information
    fn collect_process_info(process: &Process) -> ProcessInfo {
        let status = match process.status() {
            ProcessStatus::Run => "running",
            ProcessStatus::Sleep => "sleeping",
            ProcessStatus::Zombie => "zombie",
            ProcessStatus::Stop => "stopped",
            ProcessStatus::Idle => "idle",
            _ => "unknown",
        };

        // In sysinfo 0.30, process.thread_count() doesn't exist
        // Instead, use a fixed value of 1 as older versions of sysinfo handled threads differently
        // In a real implementation, we would track this differently
        let thread_count = 1;
        
        // Get disk usage from process
        let disk_usage = process.disk_usage();
        
        ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            thread_count,
            disk_read_bytes: disk_usage.read_bytes,
            disk_write_bytes: disk_usage.written_bytes,
            status: status.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_metrics_collector() {
        // Create a collector
        let collector = ResourceMetricsCollector::new();
        
        // Manually add some test metrics
        let cpu_metric = Metric::new(
            "test_cpu_usage".to_string(),
            50.0,
            MetricType::Gauge,
            HashMap::new(),
        );
        
        let memory_metric = Metric::new(
            "test_memory_usage".to_string(),
            1024.0,
            MetricType::Gauge,
            HashMap::new(),
        );
        
        // Add metrics manually
        {
            let mut metrics = collector.metrics.write().await;
            metrics.push(cpu_metric);
            metrics.push(memory_metric);
        }
        
        // Now get the metrics
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty());
        
        // Verify we have at least the metrics we added
        assert!(metrics.len() >= 2);
    }

    #[tokio::test]
    async fn test_resource_metrics_collector_with_dependencies() {
        let config = ResourceConfig::default();
        let factory = ResourceMetricsCollectorFactory::with_config(config);
        let collector = factory.create_collector();
        
        // Register a test team
        collector.register_team(
            "test_team".to_string(),
            std::env::current_dir().unwrap()
        ).await;
        
        // Get metrics
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty());
        
        // Get team metrics
        let team_metrics = collector.get_team_metrics("test_team").await;
        assert!(team_metrics.is_some());
    }

    #[tokio::test]
    async fn test_resource_metrics_collector_adapter() {
        let factory = ResourceMetricsCollectorFactory::new();
        let adapter = factory.create_collector_adapter();
        
        // Register a test team
        adapter.register_team(
            "test_team".to_string(),
            std::env::current_dir().unwrap()
        ).await;
        
        // Get metrics
        let metrics = adapter.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty());
        
        // Get team metrics
        let team_metrics = adapter.get_team_metrics().await;
        assert!(team_metrics.is_ok());
    }
}

#[async_trait]
impl MetricCollector for ResourceMetricsCollector {
    async fn record_metric(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric.clone());
        Ok(())
    }

    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        // Update system information before collecting metrics
        self.system.write().await.refresh_all();
        
        // Generate at least some basic system metrics
        let cpu_metric = Metric::new(
            "system_cpu_usage".to_string(),
            f64::from(self.system.read().await.global_cpu_info().cpu_usage()),
            MetricType::Gauge,
            HashMap::new(),
        );
        
        let memory_metric = Metric::new(
            "system_memory_usage".to_string(),
            self.system.read().await.used_memory() as f64,
            MetricType::Gauge,
            HashMap::new(),
        );
        
        // Record the new metrics
        let mut metrics = self.metrics.write().await;
        metrics.push(cpu_metric.clone());
        metrics.push(memory_metric.clone());
        
        // Return all metrics
        Ok(crate::metrics::write_guard_to_vec(&metrics))
    }

    async fn start(&self) -> Result<()> {
        // Implementation depends on the existing code
        tracing::info!("Starting ResourceMetricsCollector");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // Implementation depends on the existing code
        tracing::info!("Stopping ResourceMetricsCollector");
        Ok(())
    }
}

/// Collects resource metrics from the system
/// Returns a vector of resource metrics
pub async fn collect_resource_metrics() -> Result<Vec<Metric>> {
    let adapter = create_collector_adapter();
    adapter.collect_metrics().await
} 
