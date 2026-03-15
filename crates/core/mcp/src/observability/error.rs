// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the observability module

use thiserror::Error;

/// Errors that can occur in observability operations
#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Metrics error: {0}")]
    MetricsError(String),
    
    #[error("Tracing error: {0}")]
    TracingError(String),
    
    #[error("Logging error: {0}")]
    LoggingError(String),
    
    #[error("Health checking error: {0}")]
    HealthError(String),
    
    #[error("Alerting error: {0}")]
    AlertingError(String),
    
    #[error("Dashboard error: {0}")]
    DashboardError(String),
    
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("External system error: {0}")]
    External(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for observability operations
pub type ObservabilityResult<T> = std::result::Result<T, ObservabilityError>; 