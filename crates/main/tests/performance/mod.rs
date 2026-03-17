// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Performance testing module

pub mod load_testing;

pub use load_testing::{LoadTestConfig, LoadTestEngine, LoadTestResults};

