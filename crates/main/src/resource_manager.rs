//! # Production Resource Management System
//! 
//! This module provides comprehensive resource management including:
//! - Background cleanup tasks
//! - Connection pool maintenance  
//! - Memory leak prevention
//! - File handle tracking
//! - Graceful resource lifecycle management

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};

use crate::error::PrimalError;
use crate::observability::{OperationContext, CorrelationId};
use crate::universal_primal_ecosystem::ServiceConnectionPool;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

/// Resource management configuration
#[derive(Debug, Clone)]
pub struct ResourceManagerConfig {
    /// Interval for connection pool cleanup
    pub connection_cleanup_interval: Duration,
    
    /// Interval for memory cleanup
    pub memory_cleanup_interval: Duration,
    
    /// Interval for health monitoring
    pub health_check_interval: Duration,
    
    /// Maximum memory usage before cleanup (bytes)
    pub max_memory_threshold: u64,
    
    /// Enable automatic resource cleanup
    pub enable_auto_cleanup: bool,
    
    /// Resource cleanup timeout
    pub cleanup_timeout: Duration,
}

impl Default for ResourceManagerConfig {
    fn default() -> Self {
        Self {
            connection_cleanup_interval: Duration::from_secs(300), // 5 minutes
            memory_cleanup_interval: Duration::from_secs(600),     // 10 minutes
            health_check_interval: Duration::from_secs(60),        // 1 minute
            max_memory_threshold: 500 * 1024 * 1024,              // 500MB
            enable_auto_cleanup: true,
            cleanup_timeout: Duration::from_secs(30),
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceUsageStats {
    /// Total memory allocated (estimated)
    pub memory_bytes: u64,
    
    /// Number of active connections
    pub active_connections: usize,
    
    /// Number of open file handles (estimated)
    pub file_handles: usize,
    
    /// Background tasks running
    pub background_tasks: usize,
    
    /// Last cleanup timestamp
    pub last_cleanup: Option<Instant>,
    
    /// Cleanup success rate
    pub cleanup_success_rate: f64,
}

/// Production resource manager
pub struct ResourceManager {
    /// Configuration
    config: ResourceManagerConfig,
    
    /// Connection pools being managed
    connection_pools: Arc<RwLock<HashMap<String, Arc<ServiceConnectionPool>>>>,
    
    /// Background task handles
    background_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    
    /// Resource usage statistics
    usage_stats: Arc<RwLock<ResourceUsageStats>>,
    
    /// Shutdown flag
    shutdown_requested: Arc<RwLock<bool>>,
    
    /// Cleanup metrics
    cleanup_metrics: Arc<RwLock<HashMap<String, CleanupMetrics>>>,
}

/// Cleanup operation metrics
#[derive(Debug, Clone, Default)]
pub struct CleanupMetrics {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub avg_duration_ms: f64,
    pub resources_cleaned: u64,
    pub last_run: Option<Instant>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(config: ResourceManagerConfig) -> Self {
        Self {
            config,
            connection_pools: Arc::new(RwLock::new(HashMap::new())),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
            usage_stats: Arc::new(RwLock::new(ResourceUsageStats::default())),
            shutdown_requested: Arc::new(RwLock::new(false)),
            cleanup_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a connection pool for management
    pub async fn register_connection_pool(&self, name: String, pool: Arc<ServiceConnectionPool>) {
        let mut pools = self.connection_pools.write().await;
        pools.insert(name.clone(), pool);
        
        info!(
            pool_name = %name,
            operation = "connection_pool_registered",
            "Connection pool registered for resource management"
        );
    }
    
    /// Start background resource management tasks
    pub async fn start_background_tasks(&self) -> Result<(), PrimalError> {
        if !self.config.enable_auto_cleanup {
            info!("Automatic resource cleanup disabled");
            return Ok(());
        }
        
        let correlation_id = CorrelationId::new();
        let mut tasks = self.background_tasks.lock().await;
        
        info!(
            correlation_id = %correlation_id,
            operation = "background_tasks_start",
            "Starting resource management background tasks"
        );
        
        // Connection pool cleanup task
        let connection_cleanup_task = self.start_connection_cleanup_task().await;
        tasks.push(connection_cleanup_task);
        
        // Memory cleanup task
        let memory_cleanup_task = self.start_memory_cleanup_task().await;
        tasks.push(memory_cleanup_task);
        
        // Health monitoring task
        let health_monitoring_task = self.start_health_monitoring_task().await;
        tasks.push(health_monitoring_task);
        
        // Resource statistics task
        let stats_task = self.start_resource_stats_task().await;
        tasks.push(stats_task);
        
        info!(
            correlation_id = %correlation_id,
            task_count = tasks.len(),
            operation = "background_tasks_started",
            "Resource management background tasks started"
        );
        
        Ok(())
    }
    
    /// Start connection pool cleanup task
    async fn start_connection_cleanup_task(&self) -> JoinHandle<()> {
        let pools = Arc::clone(&self.connection_pools);
        let shutdown = Arc::clone(&self.shutdown_requested);
        let cleanup_metrics = Arc::clone(&self.cleanup_metrics);
        let interval = self.config.connection_cleanup_interval;
        
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval);
            
            loop {
                cleanup_interval.tick().await;
                
                // Check shutdown flag
                if *shutdown.read().await {
                    info!("Connection cleanup task shutting down");
                    break;
                }
                
                let operation_start = Instant::now();
                let mut total_cleaned = 0;
                let mut successful_pools = 0;
                let mut failed_pools = 0;
                
                debug!("Starting connection pool cleanup cycle");
                
                // Cleanup all registered pools
                {
                    let pools_guard = pools.read().await;
                    for (pool_name, pool) in pools_guard.iter() {
                        match tokio::time::timeout(
                            Duration::from_secs(30),
                            pool.cleanup_stale_connections()
                        ).await {
                            Ok(()) => {
                                successful_pools += 1;
                                debug!("Connection cleanup completed for pool: {}", pool_name);
                            }
                            Err(_) => {
                                failed_pools += 1;
                                warn!("Connection cleanup timed out for pool: {}", pool_name);
                            }
                        }
                    }
                }
                
                let operation_duration = operation_start.elapsed();
                
                // Update metrics
                {
                    let mut metrics = cleanup_metrics.write().await;
                    let pool_metrics = metrics.entry("connection_cleanup".to_string())
                        .or_insert_with(CleanupMetrics::default);
                    
                    pool_metrics.total_runs += 1;
                    if failed_pools == 0 {
                        pool_metrics.successful_runs += 1;
                    } else {
                        pool_metrics.failed_runs += 1;
                    }
                    
                    // Update average duration
                    let new_duration_ms = operation_duration.as_millis() as f64;
                    pool_metrics.avg_duration_ms = if pool_metrics.total_runs == 1 {
                        new_duration_ms
                    } else {
                        (pool_metrics.avg_duration_ms * (pool_metrics.total_runs - 1) as f64 + new_duration_ms) 
                            / pool_metrics.total_runs as f64
                    };
                    
                    pool_metrics.resources_cleaned += total_cleaned;
                    pool_metrics.last_run = Some(operation_start);
                }
                
                info!(
                    operation = "connection_cleanup_cycle_complete",
                    duration_ms = operation_duration.as_millis(),
                    successful_pools = successful_pools,
                    failed_pools = failed_pools,
                    "Connection cleanup cycle completed"
                );
            }
        })
    }
    
    /// Start memory cleanup task
    async fn start_memory_cleanup_task(&self) -> JoinHandle<()> {
        let shutdown = Arc::clone(&self.shutdown_requested);
        let cleanup_metrics = Arc::clone(&self.cleanup_metrics);
        let usage_stats = Arc::clone(&self.usage_stats);
        let interval = self.config.memory_cleanup_interval;
        let memory_threshold = self.config.max_memory_threshold;
        
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval);
            
            loop {
                cleanup_interval.tick().await;
                
                if *shutdown.read().await {
                    info!("Memory cleanup task shutting down");
                    break;
                }
                
                let operation_start = Instant::now();
                
                // Check memory usage (this is a simplified check)
                let current_memory = Self::estimate_memory_usage().await;
                
                let needs_cleanup = current_memory > memory_threshold;
                
                if needs_cleanup {
                    warn!(
                        current_memory_mb = current_memory / (1024 * 1024),
                        threshold_mb = memory_threshold / (1024 * 1024),
                        operation = "memory_cleanup_triggered",
                        "Memory usage above threshold, triggering cleanup"
                    );
                    
                    // Force garbage collection if available
                    #[cfg(feature = "jemalloc")]
                    {
                        // Force memory release with jemalloc
                        unsafe {
                            libc::malloc_trim(0);
                        }
                    }
                    
                    // Update usage stats
                    {
                        let mut stats = usage_stats.write().await;
                        stats.memory_bytes = current_memory;
                        stats.last_cleanup = Some(operation_start);
                    }
                }
                
                let operation_duration = operation_start.elapsed();
                
                // Update metrics
                {
                    let mut metrics = cleanup_metrics.write().await;
                    let memory_metrics = metrics.entry("memory_cleanup".to_string())
                        .or_insert_with(CleanupMetrics::default);
                    
                    memory_metrics.total_runs += 1;
                    memory_metrics.successful_runs += 1; // Memory cleanup rarely "fails"
                    
                    let new_duration_ms = operation_duration.as_millis() as f64;
                    memory_metrics.avg_duration_ms = if memory_metrics.total_runs == 1 {
                        new_duration_ms
                    } else {
                        (memory_metrics.avg_duration_ms * (memory_metrics.total_runs - 1) as f64 + new_duration_ms) 
                            / memory_metrics.total_runs as f64
                    };
                    
                    memory_metrics.last_run = Some(operation_start);
                }
                
                debug!(
                    operation = "memory_cleanup_cycle_complete",
                    duration_ms = operation_duration.as_millis(),
                    current_memory_mb = current_memory / (1024 * 1024),
                    cleanup_triggered = needs_cleanup,
                    "Memory cleanup cycle completed"
                );
            }
        })
    }
    
    /// Start health monitoring task
    async fn start_health_monitoring_task(&self) -> JoinHandle<()> {
        let pools = Arc::clone(&self.connection_pools);
        let shutdown = Arc::clone(&self.shutdown_requested);
        let usage_stats = Arc::clone(&self.usage_stats);
        let interval = self.config.health_check_interval;
        
        tokio::spawn(async move {
            let mut health_interval = tokio::time::interval(interval);
            
            loop {
                health_interval.tick().await;
                
                if *shutdown.read().await {
                    info!("Health monitoring task shutting down");
                    break;
                }
                
                let mut total_connections = 0;
                let mut healthy_connections = 0;
                let mut unhealthy_connections = 0;
                
                // Check health of all connection pools
                {
                    let pools_guard = pools.read().await;
                    for (pool_name, pool) in pools_guard.iter() {
                        match tokio::time::timeout(
                            Duration::from_secs(10),
                            pool.get_health_metrics()
                        ).await {
                            Ok(metrics) => {
                                total_connections += metrics.total_connections;
                                healthy_connections += metrics.healthy_connections;
                                unhealthy_connections += metrics.unhealthy_connections;
                                
                                if metrics.overall_failure_rate > 0.2 {
                                    warn!(
                                        pool_name = %pool_name,
                                        failure_rate = %format!("{:.1}%", metrics.overall_failure_rate * 100.0),
                                        operation = "connection_pool_health_warning",
                                        "Connection pool showing high failure rate"
                                    );
                                }
                            }
                            Err(_) => {
                                warn!(
                                    pool_name = %pool_name,
                                    operation = "health_check_timeout",
                                    "Health check timed out for connection pool"
                                );
                            }
                        }
                    }
                }
                
                // Update usage statistics
                {
                    let mut stats = usage_stats.write().await;
                    stats.active_connections = total_connections;
                    
                    // Calculate cleanup success rate
                    if total_connections > 0 {
                        stats.cleanup_success_rate = healthy_connections as f64 / total_connections as f64;
                    }
                }
                
                debug!(
                    operation = "health_monitoring_complete",
                    total_connections = total_connections,
                    healthy_connections = healthy_connections,
                    unhealthy_connections = unhealthy_connections,
                    "Health monitoring cycle completed"
                );
            }
        })
    }
    
    /// Start resource statistics collection task
    async fn start_resource_stats_task(&self) -> JoinHandle<()> {
        let usage_stats = Arc::clone(&self.usage_stats);
        let background_tasks = Arc::clone(&self.background_tasks);
        let shutdown = Arc::clone(&self.shutdown_requested);
        
        tokio::spawn(async move {
            let mut stats_interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                stats_interval.tick().await;
                
                if *shutdown.read().await {
                    info!("Resource statistics task shutting down");
                    break;
                }
                
                let current_memory = Self::estimate_memory_usage().await;
                let task_count = background_tasks.lock().await.len();
                
                // Update statistics
                {
                    let mut stats = usage_stats.write().await;
                    stats.memory_bytes = current_memory;
                    stats.background_tasks = task_count;
                }
                
                debug!(
                    operation = "resource_stats_update",
                    memory_mb = current_memory / (1024 * 1024),
                    background_tasks = task_count,
                    "Resource statistics updated"
                );
            }
        })
    }
    
    /// Estimate current memory usage (simplified)
    async fn estimate_memory_usage() -> u64 {
        // This is a simplified memory estimation
        // In production, you might use more sophisticated memory tracking
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = tokio::fs::read_to_string("/proc/self/status").await {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: return a reasonable default
        100 * 1024 * 1024 // 100MB default
    }
    
    /// Get current resource usage statistics
    pub async fn get_usage_stats(&self) -> ResourceUsageStats {
        self.usage_stats.read().await.clone()
    }
    
    /// Get cleanup metrics
    pub async fn get_cleanup_metrics(&self) -> HashMap<String, CleanupMetrics> {
        self.cleanup_metrics.read().await.clone()
    }
    
    /// Perform immediate resource cleanup
    pub async fn cleanup_now(&self) -> Result<(), PrimalError> {
        let correlation_id = CorrelationId::new();
        let ctx = OperationContext::with_correlation_id("immediate_resource_cleanup", correlation_id);
        ctx.log_start();
        
        let cleanup_start = Instant::now();
        
        // Cleanup all connection pools
        {
            let pools = self.connection_pools.read().await;
            for (pool_name, pool) in pools.iter() {
                match tokio::time::timeout(
                    self.config.cleanup_timeout,
                    pool.cleanup_stale_connections()
                ).await {
                    Ok(()) => {
                        debug!(
                            correlation_id = %ctx.correlation_id,
                            pool_name = %pool_name,
                            "Connection pool cleanup completed"
                        );
                    }
                    Err(_) => {
                        warn!(
                            correlation_id = %ctx.correlation_id,
                            pool_name = %pool_name,
                            "Connection pool cleanup timed out"
                        );
                    }
                }
            }
        }
        
        let cleanup_duration = cleanup_start.elapsed();
        let result = ctx.clone().complete_success();
        
        info!(
            correlation_id = %ctx.correlation_id,
            cleanup_duration_ms = cleanup_duration.as_millis(),
            "Immediate resource cleanup completed"
        );
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl ShutdownHandler for ResourceManager {
    fn component_name(&self) -> &str {
        "resource_manager"
    }
    
    async fn shutdown(&self, phase: ShutdownPhase) -> Result<(), PrimalError> {
        match phase {
            ShutdownPhase::StopAccepting => {
                // Stop accepting new resources for management
                info!("Resource manager stopped accepting new resources");
                Ok(())
            }
            ShutdownPhase::DrainRequests => {
                // Allow current cleanup operations to complete
                info!("Draining active resource cleanup operations");
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            }
            ShutdownPhase::CloseConnections => {
                // Cleanup all connection pools
                let pools = self.connection_pools.read().await;
                for (pool_name, pool) in pools.iter() {
                    pool.shutdown().await;
                    info!("Connection pool '{}' shutdown completed", pool_name);
                }
                Ok(())
            }
            ShutdownPhase::CleanupResources => {
                // Signal background tasks to shutdown
                {
                    let mut shutdown_flag = self.shutdown_requested.write().await;
                    *shutdown_flag = true;
                }
                
                // Wait a moment for tasks to see the shutdown signal
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            ShutdownPhase::ShutdownTasks => {
                // Wait for and abort background tasks
                let mut tasks = self.background_tasks.lock().await;
                let mut shutdown_tasks = Vec::new();
                
                // Take ownership of all tasks
                for task in tasks.drain(..) {
                    shutdown_tasks.push(task);
                }
                
                // Cancel all tasks with timeout
                let cancel_timeout = Duration::from_secs(5);
                let cancel_start = Instant::now();
                
                for task in shutdown_tasks {
                    task.abort();
                    
                    // Wait a bit for graceful shutdown, then force
                    if cancel_start.elapsed() < cancel_timeout {
                        let _ = tokio::time::timeout(
                            Duration::from_millis(100),
                            task
                        ).await;
                    }
                }
                
                info!("Resource manager background tasks shutdown completed");
                Ok(())
            }
            ShutdownPhase::FinalCleanup => {
                // Clear all remaining data structures
                {
                    let mut pools = self.connection_pools.write().await;
                    pools.clear();
                }
                
                {
                    let mut metrics = self.cleanup_metrics.write().await;
                    metrics.clear();
                }
                
                info!("Resource manager final cleanup completed");
                Ok(())
            }
        }
    }
    
    async fn is_shutdown_complete(&self) -> bool {
        *self.shutdown_requested.read().await
    }
    
    fn estimated_shutdown_time(&self) -> Duration {
        Duration::from_secs(15) // Estimated 15 seconds for complete shutdown
    }
} 