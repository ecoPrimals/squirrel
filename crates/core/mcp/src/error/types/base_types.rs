// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Base error types and utilities.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Security level placeholder for core MCP functionality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Low security level
    Low,
    /// Medium security level (default)
    #[default]
    Medium,
    /// High security level
    High,
    /// Critical security level
    Critical,
}

/// Wire format error placeholder for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("Wire format error: {message}")]
pub struct WireFormatError {
    /// Human-readable error message describing the wire format failure
    pub message: String,
}

#[cfg(test)]
#[path = "base_types_tests.rs"]
mod tests;
