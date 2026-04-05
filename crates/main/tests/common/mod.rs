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
//! Common test utilities
//!
//! ## Modern Concurrent Testing
//!
//! This module provides utilities for truly robust, concurrent tests.
//! **Test issues ARE production issues** - no sleeps, no race conditions.

pub mod async_test_utils;
pub mod concurrent_helpers;
pub mod mock_providers;
pub mod provider_factory;
pub mod provider_helpers;
pub mod test_utils;

// Re-export modern async utilities (PREFERRED)
pub use async_test_utils::{
    create_notification, retry_until_success, retry_until_success_async, wait_for, wait_for_all,
    wait_for_any, wait_for_async, with_timeout, Notifier, TimeoutError, Waiter,
};

// Note: assert_eventually and assert_eventually_async are macros defined in async_test_utils.rs
// They are exported via #[macro_export] and available at the crate root

// Re-export modern provider factory (NEW - for test modernization)
pub use provider_factory::{
    create_test_provider, create_test_provider_with_config, ProviderFactory,
};

// Re-export commonly used utilities
pub use test_utils::{
    quick_test_duration, retry_with_backoff, run_concurrent, test_duration, with_test_timeout,
};
