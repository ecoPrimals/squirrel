// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Context domain JSON-RPC handlers: `context.create`, `context.update`, `context.summarize`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use tracing::info;

impl JsonRpcServer {
    /// Handle `context.create` — create a new context session.
    ///
    /// Stores context state in-memory via `ContextManager`.  When NestGate
    /// is discovered at runtime, persistence will be delegated automatically.
    pub(crate) async fn handle_context_create(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.create request");

        let session_id = params
            .as_ref()
            .and_then(|p| p.get("session_id"))
            .and_then(|v| v.as_str())
            .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToString::to_string);

        let metadata = params
            .as_ref()
            .and_then(|p| p.get("metadata"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let manager = squirrel_context::ContextManager::new();

        let state = manager
            .get_context_state(&session_id)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to create context: {e}"),
                data: None,
            })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "metadata": metadata,
        }))
    }

    /// Handle `context.update` — update an existing context with new data.
    pub(crate) async fn handle_context_update(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.update request");

        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'id' parameter".to_string(),
                data: None,
            })?;

        let data = params
            .get("data")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let manager = squirrel_context::ContextManager::new();

        let mut state = manager
            .get_context_state(id)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Context not found: {e}"),
                data: None,
            })?;

        state.data = data;
        state.version += 1;
        state.last_modified = std::time::SystemTime::now();

        manager
            .update_context_state(id, state.clone())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to update context: {e}"),
                data: None,
            })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Handle `context.summarize` — summarize a context session.
    pub(crate) async fn handle_context_summarize(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("context.summarize request");

        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'id' parameter".to_string(),
                data: None,
            })?;

        let manager = squirrel_context::ContextManager::new();

        let state = manager
            .get_context_state(id)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Context not found: {e}"),
                data: None,
            })?;

        Ok(serde_json::json!({
            "id": state.id,
            "version": state.version,
            "summary": format!("Context {} (v{}) with {} metadata keys",
                state.id, state.version, state.metadata.len()),
            "data": state.data,
            "synchronized": state.synchronized,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use crate::rpc::jsonrpc_server::error_codes;

    #[tokio::test]
    async fn context_update_missing_id_errors() {
        let server = JsonRpcServer::new("/tmp/sq-ctx-missing-id.sock".to_string());
        let err = server
            .handle_context_update(Some(serde_json::json!({ "data": {} })))
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn context_summarize_missing_params_errors() {
        let server = JsonRpcServer::new("/tmp/sq-ctx-sum-no-params.sock".to_string());
        let err = server.handle_context_summarize(None).await.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }
}
