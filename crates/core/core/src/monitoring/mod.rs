// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Monitoring Abstraction for Squirrel MCP
//!
//! This module provides a monitoring abstraction layer that delegates to external
//! monitoring systems while maintaining sovereignty. It can work with:
//!
//! - **Monitoring service** - When available via capability discovery
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
mod monitoring_provider;
mod service;
mod types;

pub use config::{FallbackConfig, MonitoringConfig, MonitoringServiceConfig};
pub use fallback::FallbackLogger;
pub use monitoring_provider::{MonitoringProviderImpl, MonitoringServiceProvider};
pub use service::MonitoringService;
pub use types::{
    Metric, MetricValue, MonitoringCapability, MonitoringEvent, MonitoringProvider,
    MonitoringStatus, PerformanceMetrics, ProviderStatus, TimeFrame,
};

#[cfg(test)]
#[path = "../monitoring_tests.rs"]
mod monitoring_tests;
