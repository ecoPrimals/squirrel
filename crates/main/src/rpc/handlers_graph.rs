// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Graph domain JSON-RPC handlers — `graph.parse`, `graph.validate`.
//!
//! Enables primalSpring and biomeOS to send BYOB deployment graphs to
//! Squirrel for parsing and structural validation. Squirrel can then
//! introspect whether it participates in the graph and what capabilities
//! are required.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use crate::orchestration::deploy_graph::NicheDeployGraph;
use serde_json::Value;
use tracing::debug;

impl JsonRpcServer {
    /// Handle `graph.parse` — parse a BYOB deployment graph from TOML.
    ///
    /// Accepts `{ "graph_toml": "<toml string>" }` and returns the parsed
    /// graph structure as JSON, or an error if parsing fails.
    pub(crate) async fn handle_graph_parse(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        debug!("graph.parse request");

        let graph_toml = params
            .as_ref()
            .and_then(|p| p.get("graph_toml"))
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "missing required parameter 'graph_toml'".into(),
                data: None,
            })?;

        let graph = NicheDeployGraph::from_toml(graph_toml).map_err(|e| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: format!("graph parse error: {e}"),
            data: None,
        })?;

        serde_json::to_value(&graph).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("serialization error: {e}"),
            data: None,
        })
    }

    /// Handle `graph.validate` — structurally validate a BYOB deployment graph.
    ///
    /// Returns `{ "valid": bool, "issues": [...], "node_count": N,
    /// "required_count": N, "includes_squirrel": bool }`.
    pub(crate) async fn handle_graph_validate(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        debug!("graph.validate request");

        let graph_toml = params
            .as_ref()
            .and_then(|p| p.get("graph_toml"))
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "missing required parameter 'graph_toml'".into(),
                data: None,
            })?;

        let graph = NicheDeployGraph::from_toml(graph_toml).map_err(|e| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: format!("graph parse error: {e}"),
            data: None,
        })?;

        let issues = graph.structural_issues();

        Ok(serde_json::json!({
            "valid": issues.is_empty(),
            "issues": issues,
            "name": graph.graph.name,
            "node_count": graph.graph.node.len(),
            "required_count": graph.required_count(),
            "includes_squirrel": graph.includes_squirrel(),
            "coordination": graph.graph.coordination,
        }))
    }
}
