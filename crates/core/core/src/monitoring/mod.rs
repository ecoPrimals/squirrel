// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Monitoring Abstraction for Squirrel MCP
//!
//! This module provides a monitoring abstraction layer that delegates to external
//! monitoring systems while maintaining sovereignty. It can work with:
//!
//! - **Songbird** - When available as the observability primal
//! - **Future monitoring primals** - Through extensible interfaces
//! - **Basic logging** - As a fallback when no monitoring system is available
//!
//! ## Architecture Principles
//!
//! 1. **Delegation over Implementation** - Never implement monitoring directly
//! 2. **Graceful Degradation** - Continue operating without monitoring
//! 3. **Primal Agnostic** - Work with any monitoring system
//! 4. **Extensible** - Support new monitoring systems without core changes

#![expect(dead_code, reason = "Monitoring infrastructure awaiting activation")]

mod config;
mod fallback;
mod service;
mod songbird;
mod types;

pub use config::{FallbackConfig, MonitoringConfig, SongbirdConfig};
pub use fallback::FallbackLogger;
pub use service::MonitoringService;
pub use songbird::SongbirdProvider;
pub use types::{
    Metric, MetricValue, MonitoringCapability, MonitoringEvent, MonitoringProvider,
    MonitoringStatus, PerformanceMetrics, ProviderStatus, TimeFrame,
};

#[cfg(test)]
#[path = "../monitoring_tests.rs"]
mod monitoring_tests;
