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

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use crate::rpc::jsonrpc_server::error_codes;

    const VALID_GRAPH_TOML: &str = r#"
[graph]
name = "test_graph"
coordination = "Sequential"

[[graph.node]]
name = "squirrel"
binary = "squirrel"
order = 1
required = true
by_capability = "ai"
health_method = "health.check"

[[graph.node]]
name = "nestgate"
binary = "nestgate"
order = 2
health_method = "health.check"
"#;

    #[tokio::test]
    async fn graph_parse_missing_params_errors() {
        let server = JsonRpcServer::new("/tmp/sq-graph-no-params.sock".to_string());
        let err = server.handle_graph_parse(None).await.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("graph_toml"));
    }

    #[tokio::test]
    async fn graph_parse_invalid_toml_errors() {
        let server = JsonRpcServer::new("/tmp/sq-graph-bad-toml.sock".to_string());
        let err = server
            .handle_graph_parse(Some(serde_json::json!({ "graph_toml": "{{not toml" })))
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("parse error"));
    }

    #[tokio::test]
    async fn graph_parse_valid_returns_json() {
        let server = JsonRpcServer::new("/tmp/sq-graph-ok.sock".to_string());
        let result = server
            .handle_graph_parse(Some(serde_json::json!({ "graph_toml": VALID_GRAPH_TOML })))
            .await
            .expect("should parse");
        assert!(
            result.get("graph").is_some(),
            "parsed graph must have 'graph' key"
        );
    }

    #[tokio::test]
    async fn graph_validate_missing_params_errors() {
        let server = JsonRpcServer::new("/tmp/sq-graph-val-no-params.sock".to_string());
        let err = server.handle_graph_validate(None).await.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn graph_validate_returns_structure() {
        let server = JsonRpcServer::new("/tmp/sq-graph-val-ok.sock".to_string());
        let result = server
            .handle_graph_validate(Some(serde_json::json!({ "graph_toml": VALID_GRAPH_TOML })))
            .await
            .expect("should validate");
        assert_eq!(result["valid"], true);
        assert_eq!(result["name"], "test_graph");
        assert_eq!(result["node_count"], 2);
        assert_eq!(result["required_count"], 1);
        assert_eq!(result["includes_squirrel"], true);
        assert_eq!(result["coordination"], "Sequential");
    }

    #[tokio::test]
    async fn graph_validate_detects_issues_in_empty_graph() {
        let server = JsonRpcServer::new("/tmp/sq-graph-val-empty.sock".to_string());
        let empty_graph = r#"
[graph]
name = "empty"
"#;
        let result = server
            .handle_graph_validate(Some(serde_json::json!({ "graph_toml": empty_graph })))
            .await
            .expect("should validate even if empty");
        assert_eq!(result["valid"], false, "empty graph should have issues");
        assert!(
            !result["issues"]
                .as_array()
                .expect("issues array")
                .is_empty()
        );
        assert_eq!(result["includes_squirrel"], false);
    }
}
