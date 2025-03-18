//! Resource metrics collection for system monitoring
//! 
//! Tracks system resource usage including:
//! - Memory usage per team
//! - Thread memory usage
//! - Storage usage
//! - Network bandwidth

use crate::error::{Result, SquirrelError};
use crate::monitoring::metrics::{Metric, MetricCollector, MetricType};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, ProcessExt, Process, DiskExt, NetworkExt, PidExt, CpuExt, NetworksExt};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

/// Resource usage metrics for a team.
/// 
/// This struct tracks various resource usage metrics for a specific team,
/// including memory, storage, network, and process information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceMetrics {
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
pub struct ResourceMetricsCollector {
    /// System information collector
    system: System,
    /// Team resource metrics
    metrics: Arc<RwLock<Vec<Metric>>>,
    /// Team workspace paths
    team_paths: Arc<RwLock<HashMap<String, PathBuf>>>,
    /// Previous disk I/O measurements
    prev_disk_io: Arc<RwLock<HashMap<String, DiskIOStats>>>,
    /// Performance collector adapter
    performance_collector: Option<Arc<PerformanceCollectorAdapter>>,
    /// Configuration
    config: ResourceConfig,
}

impl ResourceMetricsCollector {
    /// Create a new resource metrics collector with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ResourceConfig::default())
    }

    /// Creates a new resource metrics collector with the specified configuration
    #[must_use]
    pub fn with_config(config: ResourceConfig) -> Self {
        Self {
            system: System::new_all(),
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
            system: System::new_all(),
            metrics: Arc::new(RwLock::new(Vec::new())),
            team_paths: Arc::new(RwLock::new(HashMap::new())),
            prev_disk_io: Arc::new(RwLock::new(HashMap::new())),
            performance_collector,
            config,
        }
    }

    /// Update resource metrics for all teams
    pub async fn update_metrics(&mut self) -> Result<()> {
        self.system.refresh_all();

        let mut metrics = self.metrics.write().await;
        let team_paths = self.team_paths.read().await;
        let mut prev_disk_io = self.prev_disk_io.write().await;
        
        // Update metrics for each team
        for (team_name, team_path) in team_paths.iter() {
            // Get processes for this team
            let team_processes = Self::get_team_processes(&self.system, team_name);
            
            // Collect process information
            let process_info: Vec<ProcessInfo> = team_processes.iter()
                .map(|p| Self::collect_process_info(p))
                .collect();
            
            // Calculate total memory usage
            let memory_usage = process_info.iter()
                .map(|p| p.memory_usage)
                .sum::<f64>();

            // Calculate thread count
            let thread_count = Self::calculate_thread_count(&self.system, team_name);

            // Update storage usage if team path is configured
            let storage_usage = Self::calculate_storage_usage(&self.system, team_path);
            
            // Update disk I/O statistics
            let current_io = Self::calculate_disk_io(&self.system, team_path);
            let prev_io = prev_disk_io.get(team_name).cloned().unwrap_or_default();
            
            // Calculate rates
            let time_diff = 60.0; // Collection interval in seconds
            let disk_io = (current_io.bytes_read - prev_io.bytes_read) as f64 / time_diff;
            
            // Store current I/O stats for next update
            prev_disk_io.insert(team_name.clone(), current_io);

            // Calculate CPU usage
            let cpu_usage = Self::calculate_cpu_usage(&self.system, team_name);
            
            // Calculate network bandwidth
            let network_bandwidth = Self::calculate_network_bandwidth(&self.system, team_name);

            // Update metrics
            let mut team_metric = Metric::new(
                team_name.to_string(),
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
            team_metric.labels.insert("process_count".to_string(), process_info.len().to_string());
            
            metrics.push(team_metric);
        }
        Ok(())
    }

    /// Calculate storage usage for a team's workspace
    fn calculate_storage_usage(system: &System, path: &Path) -> f64 {
        let mut total_usage = 0.0;

        // Get disk containing the path
        if let Some(disk) = system.disks().iter().find(|d| path.starts_with(d.mount_point())) {
            // Get available space
            let _total_space = disk.total_space();
            let _available_space = disk.available_space();
            
            // Calculate used space for the team's directory
            if let Ok(dir_size) = Self::calculate_dir_size(path) {
                total_usage = dir_size as f64;
            }
        }

        total_usage
    }

    /// Calculate disk I/O statistics for a team's workspace
    fn calculate_disk_io(system: &System, path: &Path) -> DiskIOStats {
        let mut stats = DiskIOStats::default();

        // Get disk containing the path
        if let Some(disk) = system.disks().iter().find(|d| path.starts_with(d.mount_point())) {
            stats.bytes_read = disk.total_space() - disk.available_space();
            // Note: sysinfo doesn't provide direct disk I/O stats
            // In a production system, we would use platform-specific APIs
            // or tools like iostat for more accurate I/O statistics
        }

        stats
    }

    /// Calculate the total size of a directory using a non-recursive approach
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
        // Improved team process detection
        let process_name = process.name().to_lowercase();
        let team_name = team_name.to_lowercase();

        // Check process name
        if process_name.contains(&team_name) {
            return true;
        }

        // Check environment variables - fix the environ method usage
        let env = process.environ();
        for var in env {
            if var.contains(&team_name) {
                return true;
            }
        }

        // Check command line arguments
        if let Some(cmd) = process.cmd().first() {
            if cmd.to_lowercase().contains(&team_name) {
                return true;
            }
        }

        false
    }

    /// Calculate thread count for a team
    fn calculate_thread_count(system: &System, team_name: &str) -> u32 {
        // Count the number of processes for the team as a simple approximation
        u32::try_from(system.processes()
            .iter()
            .filter(|(_, p)| p.name().contains(team_name))
            .count())
            .unwrap_or(0)
    }

    /// Get current resource metrics for a team
    pub async fn get_team_metrics(&self, team_name: &str) -> Option<TeamResourceMetrics> {
        let metrics = self.metrics.read().await;
        // Find the metric with the matching team name and convert it to TeamResourceMetrics
        for metric in metrics.iter() {
            if metric.name == team_name {
                if let Some(team_label) = metric.labels.get("team") {
                    if team_label == team_name {
                        // Create a TeamResourceMetrics from the metric data
                        return Some(TeamResourceMetrics {
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
        
        metrics.push(Metric::new(
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
    /// This function panics if the Tokio runtime cannot be created
    pub async fn start_collection(&self) {
        let mut collector = self.clone();
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                    loop {
                        interval.tick().await;
                        if let Err(e) = collector.update_metrics().await {
                            eprintln!("Error updating metrics: {e}");
                        }
                    }
                });
        });
    }

    /// Collect process information
    fn collect_process_info(process: &Process) -> ProcessInfo {
        ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cpu_usage: f64::from(process.cpu_usage()),
            memory_usage: process.memory() as f64,
            disk_read: process.disk_usage().read_bytes,
            disk_write: process.disk_usage().written_bytes,
        }
    }

    /// Collect system metrics
    pub fn collect_system_metrics(&self) -> Result<ResourceMetrics> {
        let mut system = System::new_all();
        system.refresh_all();
        
        Ok(ResourceMetrics::new(&system))
    }

    /// Calculate CPU usage for a team
    fn calculate_cpu_usage(system: &System, team_name: &str) -> f64 {
        let team_processes = Self::get_team_processes(system, team_name);
        team_processes.iter()
            .map(|p| f64::from(p.cpu_usage()))
            .sum::<f64>()
    }

    /// Calculate network bandwidth for a team
    fn calculate_network_bandwidth(system: &System, team_name: &str) -> f64 {
        // Simple implementation - in a real system, you would track network usage per process
        let team_processes = Self::get_team_processes(system, team_name);
        if team_processes.is_empty() {
            return 0.0;
        }
        
        // Sum up network usage from all network interfaces
        let mut total_bandwidth = 0.0;
        for (_, network) in system.networks() {
            total_bandwidth += (network.received() + network.transmitted()) as f64;
        }
        
        // Distribute bandwidth proportionally to the number of team processes
        let process_ratio = team_processes.len() as f64 / system.processes().len() as f64;
        total_bandwidth * process_ratio
    }
}

impl Clone for ResourceMetricsCollector {
    fn clone(&self) -> Self {
        Self {
            system: System::new_all(),  // Create a new System instance instead of cloning
            metrics: self.metrics.clone(),
            team_paths: self.team_paths.clone(),
            prev_disk_io: self.prev_disk_io.clone(),
            performance_collector: self.performance_collector.clone(),
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
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 60,
            history_size: 100,
        }
    }
}

/// Factory for creating and managing resource metrics collector instances
#[derive(Debug, Clone)]
pub struct ResourceMetricsCollectorFactory {
    /// Configuration for creating collectors
    config: ResourceConfig,
}

impl ResourceMetricsCollectorFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ResourceConfig::default(),
        }
    }

    /// Creates a new factory with the specified configuration
    #[must_use]
    pub const fn with_config(config: ResourceConfig) -> Self {
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

    /// Gets the global collector instance, initializing it if necessary
    pub async fn get_global_collector(&self) -> Result<Arc<ResourceMetricsCollector>> {
        if let Some(collector) = RESOURCE_COLLECTOR.get() {
            Ok(collector.clone())
        } else {
            // Create performance collector adapter
            let performance_collector = match crate::monitoring::metrics::performance::create_collector_adapter().await {
                Ok(adapter) => Some(Arc::new(adapter)),
                Err(_) => None,
            };

            // Create collector with dependencies
            let collector = self.create_collector_with_dependencies(performance_collector);
            
            // Initialize the collector
            match RESOURCE_COLLECTOR.set(collector.clone()) {
                Ok(_) => {
                    // Start collection if enabled
                    if self.config.enabled {
                        collector.start_collection().await;
                    }
                    Ok(collector)
                }
                Err(_) => Err(anyhow::anyhow!("Failed to set global resource collector")),
            }
        }
    }
}

impl Default for ResourceMetricsCollectorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory for creating resource metrics collectors
static FACTORY: OnceLock<ResourceMetricsCollectorFactory> = OnceLock::new();

/// Initialize the resource metrics collector factory
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory(config: Option<ResourceConfig>) -> Result<()> {
    let factory = match config {
        Some(cfg) => ResourceMetricsCollectorFactory::with_config(cfg),
        None => ResourceMetricsCollectorFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::metric("Resource metrics collector factory already initialized"))?;
    Ok(())
}

/// Get the resource metrics collector factory
#[must_use]
pub fn get_factory() -> Option<ResourceMetricsCollectorFactory> {
    FACTORY.get().cloned()
}

/// Get or create the resource metrics collector factory
#[must_use]
pub fn ensure_factory() -> ResourceMetricsCollectorFactory {
    FACTORY.get_or_init(ResourceMetricsCollectorFactory::new).clone()
}

// Module initialization
static RESOURCE_COLLECTOR: tokio::sync::OnceCell<Arc<ResourceMetricsCollector>> = 
    tokio::sync::OnceCell::const_new();

/// Initializes the resource metrics collector with the given configuration
///
/// # Arguments
/// * `config` - Optional configuration for the collector
///
/// # Errors
/// Returns an error if the collector cannot be initialized
pub async fn initialize(config: Option<ResourceConfig>) -> Result<Arc<ResourceMetricsCollector>> {
    // Initialize factory with config
    initialize_factory(config)?;

    // Get factory and create collector with dependencies
    let factory = ensure_factory();
    let collector = factory.get_global_collector().await?;

    // Initialize global collector
    match RESOURCE_COLLECTOR.set(collector.clone()) {
        Ok(_) => {
            // Start collection if enabled
            if factory.config.enabled {
                collector.start_collection().await;
            }
            Ok(collector)
        }
        Err(_) => Err(anyhow::anyhow!("Failed to set global resource collector")),
    }
}

/// Get resource metrics for a team
///
/// # Parameters
/// * `team_name` - The name of the team to get metrics for
///
/// # Returns
/// * `Option<TeamResourceMetrics>` - The team resource metrics, if available
pub async fn get_team_metrics(team_name: &str) -> Option<TeamResourceMetrics> {
    if let Some(collector) = RESOURCE_COLLECTOR.get() {
        collector.get_team_metrics(team_name).await
    } else {
        // Try to initialize on-demand
        match ensure_factory().get_global_collector().await {
            Ok(collector) => collector.get_team_metrics(team_name).await,
            Err(_) => None,
        }
    }
}

/// Register a new team for resource tracking
///
/// # Panics
///
/// Panics if the resource collector is not initialized
pub async fn register_team(team_name: String, workspace_path: PathBuf) {
    RESOURCE_COLLECTOR
        .get()
        .expect("Resource collector not initialized")
        .register_team(team_name, workspace_path)
        .await;
}

#[async_trait]
impl MetricCollector for ResourceMetricsCollector {
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let system_metrics = self.collect_system_metrics()?;
        let mut result = Vec::new();
        
        // Add CPU usage
        let mut labels = HashMap::new();
        labels.insert("resource".to_string(), "cpu".to_string());
        result.push(Metric::new(
            "system.cpu.usage".to_string(),
            system_metrics.cpu_usage,
            MetricType::Gauge,
            Some(labels),
        ));
        
        // Add memory usage
        let mut labels = HashMap::new();
        labels.insert("resource".to_string(), "memory".to_string());
        result.push(Metric::new(
            "system.memory.usage".to_string(),
            system_metrics.memory_usage,
            MetricType::Gauge,
            Some(labels),
        ));
        
        // Add disk usage
        let mut labels = HashMap::new();
        labels.insert("resource".to_string(), "disk".to_string());
        result.push(Metric::new(
            "system.disk.usage".to_string(),
            system_metrics.disk_usage,
            MetricType::Gauge,
            Some(labels),
        ));
        
        // Add network metrics
        let mut labels = HashMap::new();
        labels.insert("direction".to_string(), "rx".to_string());
        result.push(Metric::new(
            "system.network.traffic".to_string(),
            system_metrics.network_rx as f64,
            MetricType::Counter,
            Some(labels),
        ));
        
        let mut labels = HashMap::new();
        labels.insert("direction".to_string(), "tx".to_string());
        result.push(Metric::new(
            "system.network.traffic".to_string(),
            system_metrics.network_tx as f64,
            MetricType::Counter,
            Some(labels),
        ));
        
        Ok(result)
    }

    async fn record_metric(&self, metric: Metric) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        Ok(())
    }
}

/// Information about a running process.
/// 
/// This struct contains various metrics about a process's resource usage,
/// including CPU, memory, and disk I/O statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process identifier.
    pub pid: u32,
    /// Name of the process.
    pub name: String,
    /// CPU usage as a percentage (0.0 to 100.0).
    pub cpu_usage: f64,
    /// Memory usage in bytes.
    pub memory_usage: f64,
    /// Total bytes read from disk.
    pub disk_read: u64,
    /// Total bytes written to disk.
    pub disk_write: u64,
}

impl From<&Process> for ProcessInfo {
    fn from(process: &Process) -> Self {
        Self {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            cpu_usage: f64::from(process.cpu_usage()),
            memory_usage: process.memory() as f64,
            disk_read: process.disk_usage().read_bytes,
            disk_write: process.disk_usage().written_bytes,
        }
    }
}

/// System-wide resource usage metrics.
/// 
/// This struct provides an overview of system resource utilization,
/// including CPU, memory, disk, and network usage.
pub struct ResourceMetrics {
    /// CPU usage as a percentage (0.0 to 100.0).
    pub cpu_usage: f64,
    /// Memory usage in bytes.
    pub memory_usage: f64,
    /// Disk space usage in bytes.
    pub disk_usage: f64,
    /// Total bytes received over network interfaces.
    pub network_rx: u64,
    /// Total bytes transmitted over network interfaces.
    pub network_tx: u64,
    /// List of running processes and their resource usage.
    pub processes: Vec<ProcessInfo>,
}

impl ResourceMetrics {
    /// Creates a new ResourceMetrics instance from system information.
    /// 
    /// # Arguments
    /// 
    /// * `system` - Reference to a System instance containing system information
    /// 
    /// # Returns
    /// 
    /// Returns a new ResourceMetrics instance with current system metrics.
    #[must_use] pub fn new(system: &System) -> Self {
        // Calculate CPU usage
        let cpu_usage = f64::from(system.global_cpu_info().cpu_usage());
        
        // Calculate memory usage
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 { used_memory / total_memory * 100.0 } else { 0.0 };
        
        // Calculate disk usage
        let mut total_space = 0;
        let mut used_space = 0;
        for disk in system.disks() {
            total_space += disk.total_space();
            used_space += disk.total_space() - disk.available_space();
        }
        let disk_usage = if total_space > 0 { (used_space as f64 / total_space as f64) * 100.0 } else { 0.0 };
        
        // Calculate network usage
        let network_rx: f64 = system.networks().iter().map(|(_, net)| net.received() as f64).sum();
        let network_tx: f64 = system.networks().iter().map(|(_, net)| net.transmitted() as f64).sum();
        
        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_rx: network_rx as u64,
            network_tx: network_tx as u64,
            processes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_metrics_collector() {
        let collector = ResourceMetricsCollector::new();
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(!metrics.is_empty());
    }
} 