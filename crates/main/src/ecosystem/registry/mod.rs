// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem registry manager modules
//!
//! This module contains the decomposed ecosystem registry manager functionality.

pub mod config;
pub mod discovery;
// health removed - HTTP-based health checks
pub mod metrics;
pub mod types;

#[cfg(test)]
mod discovery_error_tests;
#[cfg(test)]
mod metrics_tests;

// Re-export all public types and functions
pub use config::*;
pub use discovery::DiscoveryOps;
// HealthMonitor removed - HTTP-based health checks
pub use metrics::{MetricsOps, ServiceStats};
pub use types::*;
