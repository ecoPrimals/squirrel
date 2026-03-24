// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem Integration Module
//!
//! This module provides pure service discovery and communication for the ecoPrimals ecosystem.
//! Each primal is completely standalone and communicates through standardized APIs.
//!
//! ## Architecture Principles
//! - Pure capability-based service discovery
//! - No hard dependencies between primals
//! - Standardized HTTP/REST API communication
//! - Each primal can function independently
//! - Dynamic service registration and health monitoring

pub mod config;
pub mod manager;
pub mod registration;
pub mod status;
pub mod types;

#[cfg(test)]
mod ecosystem_manager_test;
#[cfg(test)]
mod manager_tests;
#[cfg(test)]
mod mod_tests;

pub mod registry;

// Re-export public API - registration before types so EcosystemPrimalType is in scope for types
pub use config::EcosystemConfig;
pub use manager::EcosystemManager;
pub use manager::initialize_ecosystem_integration;
pub use registration::EcosystemServiceRegistration;
pub use registry::*;
pub use status::*;
pub use types::{
    CapabilityIdentifier, EcosystemPrimalType, HealthCheckConfig, ResourceRequirements,
    ResourceSpec, SecurityConfig, ServiceCapabilities, ServiceEndpoints, capabilities,
};
