// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Test helper functions for creating provider instances
//!
//! Provides reusable test utilities for setting up SquirrelPrimalProvider
//! with proper mock dependencies.

// NOTE: These helpers are currently disabled due to type resolution issues with EcosystemConfig.
// They are unused in the current test suite. Re-enable when needed by fixing the type ambiguity
// between squirrel::ecosystem::EcosystemConfig and squirrel_mcp_config::EcosystemConfig.

/*
use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
use squirrel::monitoring::metrics::MetricsCollector;
use squirrel::primal_provider::SquirrelPrimalProvider;
use squirrel::session::{SessionConfig, SessionManagerImpl};
use squirrel::shutdown::ShutdownManager;
use squirrel::universal_adapter::{UniversalAdapter, UniversalAdapterConfig};
use std::sync::Arc;

/// Create a test instance of SquirrelPrimalProvider with mock dependencies
#[allow(dead_code)]
pub fn create_test_provider() -> SquirrelPrimalProvider {
    let instance_id = "test-instance".to_string();
    let config = EcosystemConfig::default();
    let metrics_collector = Arc::new(MetricsCollector::new());
    let ecosystem_manager = Arc::new(EcosystemManager::new(config.clone(), metrics_collector.clone()));
    let shutdown_manager = Arc::new(ShutdownManager::new());
    let adapter_config = UniversalAdapterConfig::default();
    let universal_adapter = UniversalAdapter::new(
        adapter_config,
        ecosystem_manager.clone(),
        metrics_collector,
        shutdown_manager,
    );
    let session_manager = Arc::new(SessionManagerImpl::new(SessionConfig::default()));

    SquirrelPrimalProvider::new(
        instance_id,
        config,
        universal_adapter,
        ecosystem_manager,
        session_manager,
    )
}

/// Create a test provider with custom configuration
#[allow(dead_code)]
pub fn create_test_provider_with_config(config: EcosystemConfig) -> SquirrelPrimalProvider {
    let instance_id = "test-instance".to_string();
    let metrics_collector = Arc::new(MetricsCollector::new());
    let ecosystem_manager = Arc::new(EcosystemManager::new(config.clone(), metrics_collector.clone()));
    let shutdown_manager = Arc::new(ShutdownManager::new());
    let adapter_config = UniversalAdapterConfig::default();
    let universal_adapter = UniversalAdapter::new(
        adapter_config,
        ecosystem_manager.clone(),
        metrics_collector,
        shutdown_manager,
    );
    let session_manager = Arc::new(SessionManagerImpl::new(SessionConfig::default()));

    SquirrelPrimalProvider::new(
        instance_id,
        config,
        universal_adapter,
        ecosystem_manager,
        session_manager,
    )
}

/// Create a test provider with custom instance ID
#[allow(dead_code)]
pub fn create_test_provider_with_id(instance_id: String) -> SquirrelPrimalProvider {
    let config = EcosystemConfig::default();
    let metrics_collector = Arc::new(MetricsCollector::new());
    let ecosystem_manager = Arc::new(EcosystemManager::new(config.clone(), metrics_collector.clone()));
    let shutdown_manager = Arc::new(ShutdownManager::new());
    let adapter_config = UniversalAdapterConfig::default();
    let universal_adapter = UniversalAdapter::new(
        adapter_config,
        ecosystem_manager.clone(),
        metrics_collector,
        shutdown_manager,
    );
    let session_manager = Arc::new(SessionManagerImpl::new(SessionConfig::default()));

    SquirrelPrimalProvider::new(
        instance_id,
        config,
        universal_adapter,
        ecosystem_manager,
        session_manager,
    )
}
*/
