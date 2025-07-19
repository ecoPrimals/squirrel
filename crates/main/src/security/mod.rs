//! Universal Security Adapter for Ecosystem Integration
//!
//! This module implements universal security patterns that enable Squirrel to integrate
//! with any security primal in the ecosystem through standardized interfaces.

pub mod adapter;
pub mod config;
pub mod health;
pub mod metrics;
pub mod policy;
pub mod session;
pub mod traits;
pub mod types;

// Re-export the main types and traits for convenience
pub use adapter::UniversalSecurityAdapter;
pub use config::*;
pub use health::SecurityHealthStatus;
pub use metrics::SecurityMetrics;
pub use policy::*;
pub use session::*;
pub use traits::SecurityAdapter;
pub use types::*;

// Re-export commonly used types from std and external crates
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use std::time::Duration;
pub use tracing::{debug, info, warn};

use crate::error::PrimalError;
use crate::universal::*;
