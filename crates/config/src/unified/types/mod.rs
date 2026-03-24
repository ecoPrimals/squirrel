// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Unified Configuration Types
//!
//! This module defines the canonical configuration structure for the entire
//! Squirrel ecosystem. All configuration should flow through [`SquirrelUnifiedConfig`].

mod defaults;
mod definitions;
mod impls;
mod service;

pub use definitions::{
    AiProvidersConfig, CircuitBreakerConfig, DatabaseBackend, DatabaseConfig, FeatureFlags,
    LoadBalancingConfig, LoadBalancingStrategy, McpConfig, MonitoringConfig, NetworkConfig,
    ProviderConfig, SecurityConfig, ServiceMeshConfig, ServiceRegistryType, SquirrelUnifiedConfig,
    SystemConfig,
};
