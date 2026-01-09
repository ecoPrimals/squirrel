//! REST API server for ecosystem integration
//!
//! This module provides a comprehensive HTTP API following ecosystem standards,
//! with full capability-based discovery and no hardcoded primal dependencies.
//!
//! # Architecture
//!
//! The API is organized into cohesive modules:
//! - `health`: Health checks and readiness probes
//! - `ecosystem`: Service discovery and primal integration  
//! - `metrics`: Performance metrics and monitoring
//! - `songbird`: Service mesh integration
//! - `management`: Administrative operations
//! - `types`: Shared request/response types
//! - `server`: Core server implementation
//!
//! # Usage
//!
//! ```rust,no_run
//! use squirrel::api::ApiServer;
//! use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
//! use squirrel::shutdown::ShutdownManager;
//! use squirrel::MetricsCollector;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let port = 9010;
//! let config = EcosystemConfig::default();
//! let metrics = Arc::new(MetricsCollector::new());
//! let ecosystem = Arc::new(EcosystemManager::new(config, metrics.clone()));
//! let shutdown = Arc::new(ShutdownManager::new());
//!
//! let api_server = ApiServer::new(port, ecosystem, metrics, shutdown);
//! api_server.start().await?;
//! # Ok(())
//! # }
//! ```

pub mod ai;
mod ecosystem;
mod health;
mod management;
mod metrics;
mod server;
mod songbird;
mod types;

// Re-export main types
pub use ai::{ai_routes, provider_routes, ActionRegistry, AiRouter};
pub use server::ApiServer;
pub use types::*;
