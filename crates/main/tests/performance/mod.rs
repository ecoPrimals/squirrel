// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Performance testing module

pub mod load_testing;

pub use load_testing::{LoadTestConfig, LoadTestEngine, LoadTestResults};

