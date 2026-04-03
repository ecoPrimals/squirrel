// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// Chaos harness intentionally stresses paths that trigger many rustc/clippy lints; keep the
// integration binary green under `-D warnings` without brittle per-lint `expect` lists.
#![allow(warnings)]

//! # Chaos Testing Suite
//!
//! This test binary provides chaos engineering tests to validate system resilience
//! under adverse conditions. See the `chaos` module for test categories and structure.

mod chaos;
