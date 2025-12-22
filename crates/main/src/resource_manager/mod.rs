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
//! - [`types`] - Core type definitions (Config, Stats, Metrics)
//! - [`core`] - ResourceManager implementation
//! - [`shutdown`] - Graceful shutdown handling
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
