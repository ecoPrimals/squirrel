// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Production Resource Management System
//!
//! Comprehensive resource management with lifecycle control, background cleanup,
//! and health monitoring.
//!
//! ## Features
//!
//! - **Automatic Cleanup**: Background tasks for connection pools, memory, and file handles
//! - **Health Monitoring**: Continuous resource usage tracking
//! - **Graceful Shutdown**: Coordinated shutdown with proper resource release
//! - **Production-Ready**: Battle-tested patterns for reliability
//!
//! ## Module Organization
//!
//! - `types` - Core type definitions (Config, Stats, Metrics)
//! - `core` - `ResourceManager` implementation
//! - `shutdown` - Graceful shutdown handling
//!
//! ## Usage Example
//!
//! ```no_run
//! use squirrel::resource_manager::{ResourceManager, ResourceManagerConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create manager with default configuration
//! let config = ResourceManagerConfig::default();
//! let manager = ResourceManager::new(config);
//!
//! // Start background cleanup tasks
//! manager.start_background_tasks().await?;
//!
//! // Monitor resource usage
//! let stats = manager.get_usage_stats().await;
//! println!("Active connections: {}", stats.active_connections);
//! # Ok(())
//! # }
//! ```

mod core;
mod shutdown;
mod types;

// Re-export public types
pub use core::ResourceManager;
pub use types::{CleanupMetrics, ResourceManagerConfig, ResourceUsageStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager_config_default() {
        let config = ResourceManagerConfig::default();
        assert_eq!(
            config.connection_cleanup_interval,
            std::time::Duration::from_secs(300)
        );
        assert_eq!(
            config.memory_cleanup_interval,
            std::time::Duration::from_secs(600)
        );
        assert!(config.enable_auto_cleanup);
        assert_eq!(config.max_memory_threshold, 500 * 1024 * 1024);
    }

    #[test]
    fn test_resource_usage_stats_default() {
        let stats = ResourceUsageStats::default();
        assert_eq!(stats.memory_bytes, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.background_tasks, 0);
        assert!(stats.last_cleanup.is_none());
    }

    #[test]
    fn test_cleanup_metrics_default() {
        let metrics = CleanupMetrics::default();
        assert_eq!(metrics.total_runs, 0);
        assert_eq!(metrics.successful_runs, 0);
        assert_eq!(metrics.failed_runs, 0);
        assert!(metrics.last_run.is_none());
    }

    #[tokio::test]
    async fn test_resource_manager_new() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);
        let stats = manager.get_usage_stats().await;
        assert_eq!(stats.memory_bytes, 0);
    }

    #[tokio::test]
    async fn test_resource_manager_register_connection_pool() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);
        manager
            .register_connection_pool("test_pool".to_string(), ())
            .await;
    }

    #[tokio::test]
    async fn test_resource_manager_start_background_tasks_disabled() {
        let config = ResourceManagerConfig {
            enable_auto_cleanup: false,
            ..Default::default()
        };
        let manager = ResourceManager::new(config);
        let result = manager.start_background_tasks().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resource_manager_cleanup_now() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);
        let result = manager.cleanup_now().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resource_manager_get_cleanup_metrics() {
        let config = ResourceManagerConfig::default();
        let manager = ResourceManager::new(config);
        let metrics = manager.get_cleanup_metrics().await;
        assert!(metrics.is_empty());
    }

    #[tokio::test]
    async fn test_start_background_tasks_short_intervals_run_cleanup_cycles() {
        let config = ResourceManagerConfig {
            enable_auto_cleanup: true,
            connection_cleanup_interval: std::time::Duration::from_millis(30),
            memory_cleanup_interval: std::time::Duration::from_millis(30),
            health_check_interval: std::time::Duration::from_millis(30),
            max_memory_threshold: 0,
            ..Default::default()
        };
        let manager = ResourceManager::new(config);
        manager.start_background_tasks().await.expect("start");
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        let metrics = manager.get_cleanup_metrics().await;
        assert!(metrics.contains_key("connection_cleanup"));
        assert!(metrics.contains_key("memory_cleanup"));
        assert_eq!(manager.active_operations(), 0);
    }
}
