// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Base error types and utilities.

use serde::{Deserialize, Serialize};

/// Security level placeholder for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Wire format error placeholder for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireFormatError {
    pub message: String,
}

impl std::fmt::Display for WireFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wire format error: {}", self.message)
    }
}

impl std::error::Error for WireFormatError {}

#[cfg(test)]
#[path = "base_types_tests.rs"]
mod tests;
