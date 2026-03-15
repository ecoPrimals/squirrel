// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health monitoring module
//!
//! This module provides comprehensive health monitoring capabilities for all ecosystem components.
//!
//! # Module Structure
//!
//! - `types` - Core types and trait definitions
//! - `monitor` - Health monitor implementation
//! - `tests` - Test suite

pub mod monitor;
pub mod types;

pub use monitor::HealthMonitor;
pub use types::{ComponentHealth, HealthSnapshot, MonitoringHealthCheckConfig};

use super::HealthState;
