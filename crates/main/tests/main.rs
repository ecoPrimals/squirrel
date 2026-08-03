// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Single-binary integration test harness.
//!
//! All integration tests compile into **one** binary instead of 34 separate
//! ones. This eliminates ~33 redundant link cycles (~130 MB each) and cuts the
//! full test build from ~8 minutes to ~1–2 minutes.
//!
//! Individual test modules live under `integration/` so Cargo does not
//! auto-discover them as separate test targets.

#![allow(
    clippy::assertions_on_constants,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::equatable_if_let,
    clippy::expect_used,
    clippy::explicit_iter_loop,
    clippy::field_reassign_with_default,
    clippy::future_not_send,
    clippy::many_single_char_names,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::return_self_not_must_use,
    clippy::semicolon_if_nothing_returned,
    clippy::significant_drop_tightening,
    clippy::similar_names,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::unreadable_literal,
    clippy::unused_async,
    clippy::unused_self,
    clippy::unwrap_used,
    dead_code,
    deprecated,
    missing_docs,
    unused_imports,
    unused_variables,
)]

#[path = "integration/additional_coverage_tests.rs"]
mod additional_coverage_tests;
#[path = "integration/additional_error_coverage.rs"]
mod additional_error_coverage;
#[path = "integration/basic_test.rs"]
mod basic_test;
#[path = "integration/biomeos_integration_real.rs"]
mod biomeos_integration_real;
#[path = "integration/biomeos_integration_tests.rs"]
mod biomeos_integration_tests;
#[path = "integration/capability_based_discovery_tests.rs"]
mod capability_based_discovery_tests;
#[path = "integration/capability_discovery_error_tests.rs"]
mod capability_discovery_error_tests;
#[path = "integration/capability_discovery_tests.rs"]
mod capability_discovery_tests;
#[path = "integration/chaos/mod.rs"]
mod chaos;
#[path = "integration/concurrent_stress_tests.rs"]
mod concurrent_stress_tests;
#[path = "integration/concurrent_test_helpers.rs"]
mod concurrent_test_helpers;
#[path = "integration/context_management_integration_tests.rs"]
mod context_management_integration_tests;
#[path = "integration/critical_path_coverage_tests.rs"]
mod critical_path_coverage_tests;
#[path = "integration/cross_primal_ipc_tests.rs"]
mod cross_primal_ipc_tests;
#[path = "integration/discovery_tests.rs"]
mod discovery_tests;
#[path = "integration/e2e_comprehensive_workflow_tests.rs"]
mod e2e_comprehensive_workflow_tests;
#[path = "integration/ecosystem_integration_comprehensive.rs"]
mod ecosystem_integration_comprehensive;
#[path = "integration/ecosystem_types_tests.rs"]
mod ecosystem_types_tests;
#[path = "integration/enhanced_mcp_simple_test.rs"]
mod enhanced_mcp_simple_test;
#[path = "integration/inference_register_provider_tests.rs"]
mod inference_register_provider_tests;
#[path = "integration/integration_error_scenarios.rs"]
mod integration_error_scenarios;
#[path = "integration/integration_test.rs"]
mod integration_test;
#[path = "integration/mcp_core_minimal_test.rs"]
mod mcp_core_minimal_test;
#[path = "integration/mcp_core_only.rs"]
mod mcp_core_only;
#[path = "integration/mcp_core_tests.rs"]
mod mcp_core_tests;
#[path = "integration/mock_verification.rs"]
mod mock_verification;
#[path = "integration/observability_correlation_tests.rs"]
mod observability_correlation_tests;
#[path = "integration/proptest_roundtrip.rs"]
mod proptest_roundtrip;
#[path = "integration/security_input_validator_tests.rs"]
mod security_input_validator_tests;
#[path = "integration/security_rate_limiter_tests.rs"]
mod security_rate_limiter_tests;
#[path = "integration/service_discovery_error_paths.rs"]
mod service_discovery_error_paths;
#[path = "integration/storage_client_tests.rs"]
mod storage_client_tests;
#[path = "integration/universal_adapter_integration.rs"]
mod universal_adapter_integration;
#[path = "integration/universal_adapter_tests.rs"]
mod universal_adapter_tests;
