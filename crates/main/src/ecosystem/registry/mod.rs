//! Ecosystem registry manager modules
//!
//! This module contains the decomposed ecosystem registry manager functionality.

pub mod config;
pub mod discovery;
// health removed - HTTP-based health checks
pub mod metrics;
pub mod types;

// Re-export all public types and functions
pub use config::*;
pub use discovery::DiscoveryOps;
pub use health::HealthMonitor;
pub use metrics::{MetricsOps, ServiceStats};
pub use types::*;
