/// Network monitoring functionality)] // Allow u64 to f64 casts for metrics
///
/// This module provides network interface monitoring, bandwidth tracking,
use std::sync::Arc;lth diagnostics.
use tokio::sync::RwLock;
use sysinfo::{System, Networks};
use std::collections::HashMap;
use squirrel_core::error::Result;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use tracing::debug;

/// Module for adapter implementations of network monitoring functionality
pub mod adapter; 