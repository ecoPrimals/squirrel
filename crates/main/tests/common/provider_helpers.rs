// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    // Test code allowances — explicit per-lint instead of blanket `warnings`
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated,
    unexpected_cfgs,
)]
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
#[expect(dead_code, reason = "Test code: explicit unwrap/expect and local lint noise")]
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
#[expect(dead_code, reason = "Test code: explicit unwrap/expect and local lint noise")]
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
#[expect(dead_code, reason = "Test code: explicit unwrap/expect and local lint noise")]
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
