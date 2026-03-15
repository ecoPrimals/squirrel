// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Type conversion implementations for MCPError.

use super::{MCPError, WireFormatError};
use crate::error::protocol_err::ProtocolError;

// Add From implementations for various error types
impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<WireFormatError> for MCPError {
    fn from(err: WireFormatError) -> Self {
        MCPError::Protocol(ProtocolError::Wire(err.to_string()))
    }
}

// Implement From<String> for MCPError to handle cases where String is converted to MCPError
impl From<String> for MCPError {
    fn from(msg: String) -> Self {
        MCPError::General(msg)
    }
}

// Implement From<&str> for MCPError for convenience
impl From<&str> for MCPError {
    fn from(msg: &str) -> Self {
        MCPError::General(msg.to_string())
    }
}
