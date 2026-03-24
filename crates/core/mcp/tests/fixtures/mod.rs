// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Test Fixtures for MCP Core Testing
//!
//! This module provides reusable test fixtures, mocks, and helpers
//! for testing MCP functionality.

pub mod mock_transport;
pub mod test_utils;

pub use mock_transport::*;
pub use test_utils::*;

