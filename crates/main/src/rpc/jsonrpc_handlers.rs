// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC 2.0 utility helpers shared across domain handler modules.
//!
//! Domain handlers are split by semantic domain per wateringHole naming standard:
//! - `handlers_ai.rs` — `ai.*` methods
//! - `handlers_capability.rs` — `capability.*` methods
//! - `handlers_system.rs` — `system.*`, `discovery.*`, `lifecycle.*` methods
//! - `handlers_context.rs` — `context.*` methods
//! - `handlers_tool.rs` — `tool.*` methods

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use anyhow::Context;
use serde_json::Value;

impl JsonRpcServer {
    // -----------------------------------------------------------------------
    // Utility methods
    // -----------------------------------------------------------------------

    /// Parse parameters into expected type
    pub(crate) fn parse_params<T: serde::de::DeserializeOwned>(
        &self,
        params: Option<Value>,
    ) -> Result<T, JsonRpcError> {
        match params {
            Some(value) => serde_json::from_value(value)
                .context("Failed to deserialize JSON-RPC parameters")
                .map_err(|e| JsonRpcError {
                    code: error_codes::INVALID_PARAMS,
                    message: format!("Invalid parameters: {e}"),
                    data: None,
                }),
            None => Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing parameters".to_string(),
                data: None,
            }),
        }
    }

    /// Create method not found error
    pub(crate) fn method_not_found(&self, method: &str) -> JsonRpcError {
        JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    /// Create error response
    pub(crate) fn error_response(
        &self,
        id: Value,
        code: i32,
        message: &str,
    ) -> super::jsonrpc_server::JsonRpcResponse {
        super::jsonrpc_server::JsonRpcResponse {
            jsonrpc: std::sync::Arc::from("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }
}

#[cfg(test)]
#[path = "jsonrpc_handlers_tests.rs"]
mod tests;
