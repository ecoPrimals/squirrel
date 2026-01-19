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

// Legacy HTTP API modules REMOVED - Squirrel uses Unix sockets + JSON-RPC + tarpc!
// Modern idiomatic Rust: capability-based discovery, no HTTP frameworks
// pub mod ai;           // DELETED: Use capability_ai instead
// mod ecosystem;        // DELETED: Use capability discovery
// mod health;           // DELETED: Unix sockets don't need HTTP health checks
// mod management;       // DELETED: Use JSON-RPC management
// mod metrics;          // DELETED: Monitoring via Unix sockets
// mod server;           // DELETED: No HTTP server needed
// mod service_mesh;     // DELETED: Capability discovery handles this
mod types;

// Legacy HTTP API re-exports REMOVED
// Use crates/main/src/rpc/ for JSON-RPC + tarpc instead!
// pub use ai::{ai_routes, provider_routes, ActionRegistry, AiRouter}; // DELETED
// pub use server::ApiServer; // DELETED
pub use types::*; // Keep types for backward compat (for now)
