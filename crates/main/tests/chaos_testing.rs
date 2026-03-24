// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unnecessary_wraps,
    dead_code,
    clippy::similar_names,
    clippy::match_same_arms,
    clippy::equatable_if_let,
    clippy::too_many_lines,
    clippy::future_not_send,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::match_wildcard_for_single_variants,
    clippy::option_if_let_else,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self
)] // Test code: explicit unwrap/expect and local lint noise
//! # Chaos Testing Suite
//!
//! This test binary provides chaos engineering tests to validate system resilience
//! under adverse conditions. See the `chaos` module for test categories and structure.

mod chaos;
