//! Primal health status types.

use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Primal health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but operational
    Degraded {
        /// List of issues causing degradation
        issues: Vec<String>,
    },
    /// Primal is unhealthy and not operational
    Unhealthy {
        /// Reason why the primal is unhealthy
        reason: String,
    },
}
