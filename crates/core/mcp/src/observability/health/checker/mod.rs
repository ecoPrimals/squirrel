// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Health Checker System
//!
//! This module provides comprehensive health checking functionality for monitoring
//! the status of system components. It includes health check definitions, execution,
//! scheduling, reporting, and integration with alerting systems.
//!
//! ## Architecture
//!
//! The health checker system is organized into focused modules:
//! - `types`: Health check definitions and configuration structures
//! - `execution`: Health check execution and status update logic
//! - `core`: Main HealthChecker struct and core functionality
//! - `scheduler`: Automatic health check scheduling and execution
//! - `reporting`: Report generation and status event creation
//! - `integration`: Alerting integration and utility functions
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use squirrel_mcp::observability::health::checker::{
//!     HealthChecker, HealthCheckResult, HealthStatus
//! };
//!
//! // Create a health checker
//! let checker = Arc::new(HealthChecker::new());
//! checker.initialize_async().await?;
//!
//! // Register a health check
//! checker.register_health_check(
//!     "database",
//!     "db_connection",
//!     "Database Connection Check",
//!     Box::new(|| HealthCheckResult::healthy_with_message("DB is responsive")),
//! )?;
//!
//! // Execute all checks
//! let results = checker.execute_all()?;
//!
//! // Get overall system status
//! let status = checker.overall_status()?;
//! ```

// Module declarations
pub mod types;
pub mod execution;
pub mod core;
pub mod scheduler;
pub mod reporting;
pub mod integration;

// Re-exports for backward compatibility and convenient access
pub use types::{
    HealthCheck, HealthCheckFn, HealthCheckerConfig
};

pub use core::HealthChecker;

pub use execution::{
    run_health_check, execute_health_check_sync, 
    update_component_health_with_result, calculate_system_status
};

pub use scheduler::{
    start_scheduler, execute_due_health_checks, execute_component_checks,
    execute_with_runtime
};

pub use reporting::{
    get_json_report, create_status_event_sync, create_status_event,
    generate_comprehensive_report, get_system_health_status, get_system_health_sync
};

pub use integration::{
    connect_health_to_alerting, create_standard_health_checks,
    create_connectivity_check, create_database_check, create_service_dependency_check,
    monitor_health_checker, create_web_service_checks
};

// Re-export commonly used types from parent modules for convenience
pub use crate::observability::health::types::{HealthStatus, HealthCheckType};
pub use crate::observability::health::result::HealthCheckResult;
pub use crate::observability::health::component::ComponentHealth;
pub use crate::observability::health::event::{HealthStatusEvent, HealthStatusReport};
pub use crate::observability::health::subscription::{
    HealthStatusSubscriber, HealthStatusSubscriberNonBlocking
};
pub use crate::observability::health::HealthReport; 