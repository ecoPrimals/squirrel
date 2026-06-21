// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context domain JSON-RPC handlers: `context.create`, `context.update`, `context.summarize`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use tracing::info;

impl JsonRpcServer {
    /// Sync the live context session count to the metrics collector.
    async fn sync_context_session_count(&self) {
        if let Some(mc) = &self.metrics_collector {
            let count = self.context_manager.list_sessions().await.len() as u64;
            mc.set_context_session_count(count);
        }
    }

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

        let state = self
            .context_manager
            .get_context_state(&session_id)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to create context: {e}"),
                data: None,
            })?;

        self.sync_context_session_count().await;

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

        let mut state = self
            .context_manager
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

        self.context_manager
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

        let state = self
            .context_manager
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

    #[tokio::test]
    async fn context_create_update_summarize_roundtrip() {
        let server = JsonRpcServer::new("/tmp/sq-ctx-roundtrip.sock".to_string());

        // Create
        let created = server
            .handle_context_create(Some(serde_json::json!({ "session_id": "rt-1" })))
            .await
            .expect("create should succeed");
        assert_eq!(created["id"], "rt-1");

        // Update — mutates the same shared ContextManager
        let updated = server
            .handle_context_update(Some(serde_json::json!({
                "id": "rt-1",
                "data": { "key": "value" }
            })))
            .await
            .expect("update should succeed");
        assert_eq!(updated["id"], "rt-1");
        assert!(
            updated["version"]
                .as_u64()
                .expect("version must be present")
                > 0
        );

        // Summarize — reads from the same shared state
        let summary = server
            .handle_context_summarize(Some(serde_json::json!({ "id": "rt-1" })))
            .await
            .expect("summarize should succeed");
        assert_eq!(summary["id"], "rt-1");
        assert_eq!(summary["data"]["key"], "value");
    }

    #[tokio::test]
    async fn context_create_generates_uuid_when_no_session_id() {
        let server = JsonRpcServer::new("/tmp/sq-ctx-uuid.sock".to_string());
        let created = server
            .handle_context_create(None)
            .await
            .expect("create with None params");
        let id = created["id"].as_str().expect("id must be string");
        assert!(
            uuid::Uuid::parse_str(id).is_ok(),
            "auto-generated id should be valid UUID"
        );
    }
}
