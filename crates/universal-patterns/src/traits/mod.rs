// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal primal traits and interfaces
//!
//! This module provides the core traits and types for universal primal patterns,
//! designed for full compatibility with songbird's orchestration system.

// Submodules
mod auth;
mod capabilities;
mod context;
mod dependencies;
mod endpoints;
mod errors;
mod health_status;
mod metrics;
mod ports;
mod primal;
mod primal_info;
mod provider;
mod requests;
mod responses;

// Re-export traits
pub use primal::Primal;
pub use provider::PrimalProvider;

// Re-export types from primal_info
pub use primal_info::{PrimalInfo, PrimalState, PrimalType};

// Re-export types from health_status
pub use health_status::{HealthDetail, HealthState, HealthStatus, PrimalHealth};

// Re-export types from metrics
pub use metrics::MetricValue;

// Re-export types from errors
pub use errors::{PrimalError, PrimalResult};

// Re-export types from context
pub use context::{NetworkLocation, PrimalContext, SecurityLevel};

// Re-export types from ports
pub use ports::{DynamicPortInfo, PortStatus, PortType};

// Re-export types from capabilities
pub use capabilities::PrimalCapability;

// Re-export types from dependencies
pub use dependencies::PrimalDependency;

// Re-export types from endpoints
pub use endpoints::PrimalEndpoints;

// Re-export types from requests
pub use requests::{PrimalRequest, PrimalRequestType};

// Re-export types from responses
pub use responses::{PrimalResponse, PrimalResponseType};

// Re-export types from auth
pub use auth::{AuthResult, Credentials, Principal, PrincipalType};
