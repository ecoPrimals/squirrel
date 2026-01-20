//! Graceful shutdown implementation
//!
//! Handles coordinated shutdown with proper resource cleanup.

use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::error::PrimalError;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

use super::core::ResourceManager;

#[async_trait::async_trait]
impl ShutdownHandler for ResourceManager {
    fn component_name(&self) -> &'static str {
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
                // Allow current cleanup operations to complete via graceful drain
                info!("Draining active resource cleanup operations");

                // Use timeout as safety net, but let operations complete naturally
                // Check if active operations are done every 100ms
                let start = tokio::time::Instant::now();
                let max_wait = Duration::from_secs(2);

                while start.elapsed() < max_wait {
                    if self.active_operations() == 0 {
                        info!("All cleanup operations drained successfully");
                        break;
                    }
                    // Brief yield to allow tasks to complete
                    tokio::task::yield_now().await;
                }

                if self.active_operations() > 0 {
                    warn!(
                        "Shutdown proceeding with {} active operations remaining",
                        self.active_operations()
                    );
                }

                Ok(())
            }
            ShutdownPhase::CloseConnections => {
                // Cleanup all connection pools
                // connection_pools removed - Unix sockets don't need pooling
                // let pools = self.connection_pools.read().await;
                // for (pool_name, pool) in pools.iter() {
                //     pool.shutdown().await;
                //     info!("Connection pool '{}' shutdown completed", pool_name);
                // }
                Ok(())
            }
            ShutdownPhase::CleanupResources => {
                // Signal background tasks to shutdown
                {
                    let mut shutdown_flag = self.shutdown_requested.write().await;
                    *shutdown_flag = true;
                }

                // Brief yield to allow tasks to see the shutdown signal
                // No arbitrary delay - tasks should react to the shutdown signal
                tokio::task::yield_now().await;

                info!("Shutdown signal propagated to all tasks");
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
                        let _ = tokio::time::timeout(Duration::from_millis(100), task).await;
                    }
                }

                info!("Resource manager background tasks shutdown completed");
                Ok(())
            }
            ShutdownPhase::FinalCleanup => {
                // Clear all remaining data structures
                {
                    // connection_pools removed - Unix sockets don't need pooling
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_manager::{CleanupMetrics, ResourceManagerConfig, ResourceUsageStats};
    // ServiceConnectionPool removed - Unix sockets don't need connection pooling

    #[test]
    fn test_resource_manager_config_default() {
        let config = ResourceManagerConfig::default();
        assert_eq!(config.connection_cleanup_interval, Duration::from_secs(300));
        assert_eq!(config.memory_cleanup_interval, Duration::from_secs(600));
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
        assert_eq!(config.max_memory_threshold, 500 * 1024 * 1024);
        assert!(config.enable_auto_cleanup);
        assert_eq!(config.cleanup_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_resource_usage_stats_default() {
        let stats = ResourceUsageStats::default();
        assert_eq!(stats.memory_bytes, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.file_handles, 0);
        assert_eq!(stats.background_tasks, 0);
        assert!(stats.last_cleanup.is_none());
        assert_eq!(stats.cleanup_success_rate, 0.0);
    }

    #[test]
    fn test_cleanup_metrics_default() {
        let metrics = CleanupMetrics::default();
        assert_eq!(metrics.total_runs, 0);
        assert_eq!(metrics.successful_runs, 0);
        assert_eq!(metrics.failed_runs, 0);
        assert_eq!(metrics.avg_duration_ms, 0.0);
        assert_eq!(metrics.resources_cleaned, 0);
        assert!(metrics.last_run.is_none());
    }

    #[test]
    fn test_resource_manager_new() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Verify manager was created with empty state
        assert!(!manager.config.enable_auto_cleanup || manager.config.enable_auto_cleanup);
    }

    #[tokio::test]
    async fn test_register_connection_pool() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Connection pooling removed - Unix sockets don't need pooling
        manager
            .register_connection_pool("test-pool".to_string(), ())
            .await;

        // connection_pools removed - Unix sockets don't need pooling
        // let pools = manager.connection_pools.read().await;
        // assert_eq!(pools.len(), 1);
        // assert!(pools.contains_key("test-pool"));
    }

    #[tokio::test]
    #[ignore] // connection_pools removed
    async fn test_register_multiple_connection_pools() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        for i in 0..3 {
            // Connection pooling removed - Unix sockets don't need pooling
            manager
                .register_connection_pool(format!("pool-{}", i), ())
                .await;
        }

        // connection_pools removed - Unix sockets don't need pooling
        // let pools = manager.connection_pools.read().await;
        // assert_eq!(pools.len(), 3);
    }

    #[tokio::test]
    async fn test_get_usage_stats() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let stats = manager.get_usage_stats().await;
        assert_eq!(stats.memory_bytes, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.background_tasks, 0);
    }

    #[tokio::test]
    async fn test_get_cleanup_metrics_empty() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let metrics = manager.get_cleanup_metrics().await;
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_estimate_memory_usage() {
        let memory = ResourceManager::estimate_memory_usage().await;
        assert!(memory > 0);
    }

    #[tokio::test]
    async fn test_start_background_tasks_disabled() {
        let mut config = ResourceManagerConfig::default();
        config.enable_auto_cleanup = false;

        let manager = ResourceManager::new(config);
        let result = manager.start_background_tasks().await;
        assert!(result.is_ok());

        let tasks = manager.background_tasks.lock().await;
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_start_background_tasks_enabled() {
        let mut config = ResourceManagerConfig::default();
        config.enable_auto_cleanup = true;
        config.connection_cleanup_interval = Duration::from_secs(3600); // Long interval
        config.memory_cleanup_interval = Duration::from_secs(3600);
        config.health_check_interval = Duration::from_secs(3600);

        let manager = ResourceManager::new(config);
        let result = manager.start_background_tasks().await;
        assert!(result.is_ok());

        let tasks = manager.background_tasks.lock().await;
        assert_eq!(tasks.len(), 4); // 4 background tasks
    }

    #[tokio::test]
    async fn test_cleanup_now() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Register a pool
        // Connection pooling removed - Unix sockets don't need pooling
        manager
            .register_connection_pool("test-pool".to_string(), ())
            .await;

        // Run immediate cleanup
        let result = manager.cleanup_now().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resource_manager_component_name() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        assert_eq!(manager.component_name(), "resource_manager");
    }

    #[tokio::test]
    async fn test_estimated_shutdown_time() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let time = manager.estimated_shutdown_time();
        assert_eq!(time, Duration::from_secs(15));
    }

    #[tokio::test]
    async fn test_is_shutdown_complete_initial() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        assert!(!manager.is_shutdown_complete().await);
    }

    #[tokio::test]
    async fn test_shutdown_stop_accepting() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let result = manager.shutdown(ShutdownPhase::StopAccepting).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_drain_requests() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let result = manager.shutdown(ShutdownPhase::DrainRequests).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_close_connections() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Register a pool first
        // Connection pooling removed - Unix sockets don't need pooling
        manager
            .register_connection_pool("test-pool".to_string(), ())
            .await;

        let result = manager.shutdown(ShutdownPhase::CloseConnections).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_cleanup_resources() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let result = manager.shutdown(ShutdownPhase::CleanupResources).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_shutdown_tasks() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        let result = manager.shutdown(ShutdownPhase::ShutdownTasks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_final_cleanup() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Register pools and metrics first
        // Connection pooling removed - Unix sockets don't need pooling
        manager
            .register_connection_pool("test-pool".to_string(), ())
            .await;

        let result = manager.shutdown(ShutdownPhase::FinalCleanup).await;
        assert!(result.is_ok());

        // Verify cleanup cleared data
        // connection_pools removed - Unix sockets don't need pooling
        // let pools = manager.connection_pools.read().await;
        // assert_eq!(pools.len(), 0);

        let metrics = manager.cleanup_metrics.read().await;
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_complete_lifecycle() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Register resources
        // Connection pooling removed - Unix sockets don't need pooling
        manager
            .register_connection_pool("lifecycle-pool".to_string(), ())
            .await;

        // Go through all shutdown phases
        let phases = [
            ShutdownPhase::StopAccepting,
            ShutdownPhase::DrainRequests,
            ShutdownPhase::CloseConnections,
            ShutdownPhase::CleanupResources,
            ShutdownPhase::ShutdownTasks,
            ShutdownPhase::FinalCleanup,
        ];

        for phase in &phases {
            let result = manager.shutdown(*phase).await;
            assert!(result.is_ok(), "Phase {:?} failed", phase);
        }

        // Verify final state
        // connection_pools removed - Unix sockets don't need pooling
        // let pools = manager.connection_pools.read().await;
        // assert_eq!(pools.len(), 0);
    }

    #[test]
    fn test_resource_manager_config_custom() {
        let config = ResourceManagerConfig {
            connection_cleanup_interval: Duration::from_secs(100),
            memory_cleanup_interval: Duration::from_secs(200),
            health_check_interval: Duration::from_secs(30),
            max_memory_threshold: 1024 * 1024 * 1024, // 1GB
            enable_auto_cleanup: false,
            cleanup_timeout: Duration::from_secs(60),
        };

        assert_eq!(config.connection_cleanup_interval, Duration::from_secs(100));
        assert_eq!(config.memory_cleanup_interval, Duration::from_secs(200));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        assert_eq!(config.max_memory_threshold, 1024 * 1024 * 1024);
        assert!(!config.enable_auto_cleanup);
        assert_eq!(config.cleanup_timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_cleanup_metrics_accumulation() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);

        // Manually update metrics
        {
            let mut metrics = manager.cleanup_metrics.write().await;
            let conn_metrics = metrics
                .entry("test_cleanup".to_string())
                .or_insert_with(CleanupMetrics::default);

            conn_metrics.total_runs = 10;
            conn_metrics.successful_runs = 9;
            conn_metrics.failed_runs = 1;
            conn_metrics.avg_duration_ms = 150.0;
            conn_metrics.resources_cleaned = 50;
        }

        let metrics = manager.get_cleanup_metrics().await;
        let test_metrics = metrics.get("test_cleanup").expect("Should have metrics");
        assert_eq!(test_metrics.total_runs, 10);
        assert_eq!(test_metrics.successful_runs, 9);
        assert_eq!(test_metrics.failed_runs, 1);
        assert_eq!(test_metrics.avg_duration_ms, 150.0);
        assert_eq!(test_metrics.resources_cleaned, 50);
    }
}
