// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Session Management Integration

use serde_json::json;

use super::core::SquirrelPrimalProvider;
use crate::error::PrimalError;

/// Session Operations functionality
pub struct SessionOperations;

impl SquirrelPrimalProvider {
    /// Create a new session
    pub async fn create_session(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Extract session creation parameters
        let user_id = request
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing user_id".to_string()))?;

        // Create session through session manager
        let session_id = self
            .session_manager
            .create_session(Some(user_id.to_string()))
            .await
            .map_err(|e| PrimalError::Internal(format!("Session creation failed: {e}")))?;

        Ok(json!({
            "session_id": session_id,
            "user_id": user_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "status": "active"
        }))
    }

    /// Retrieve session information
    pub async fn get_session(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let session_id = request
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing session_id".to_string()))?;

        // Retrieve session from session manager
        let session = self
            .session_manager
            .get_session_metadata(session_id)
            .await
            .map_err(|e| PrimalError::Internal(format!("Session retrieval failed: {e}")))?;

        Ok(json!({
            "session_id": session_id,
            "user_id": session.client_info.unwrap_or_else(|| "unknown".to_string()),
            "created_at": session.created_at.to_rfc3339(),
            "last_accessed": session.last_activity.to_rfc3339(),
            "status": "active",
            "metadata": session.capabilities
        }))
    }

    /// Update session
    pub async fn update_session(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let session_id = request
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing session_id".to_string()))?;

        let metadata = request
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Convert JSON Value to HashMap<String, serde_json::Value>
        let metadata_map = if let serde_json::Value::Object(map) = metadata {
            map.into_iter().collect()
        } else {
            std::collections::HashMap::new()
        };

        // Update session through session manager
        self.session_manager
            .update_session_data(session_id, metadata_map)
            .await
            .map_err(|e| PrimalError::Internal(format!("Session update failed: {e}")))?;

        Ok(json!({
            "session_id": session_id,
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "status": "updated"
        }))
    }

    /// Delete (terminate) a session
    pub async fn delete_session(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let session_id = request
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing session_id".to_string()))?;

        // Terminate session via session manager
        self.session_manager.terminate_session(session_id).await?;

        Ok(json!({
            "session_id": session_id,
            "deleted_at": chrono::Utc::now().to_rfc3339(),
            "status": "deleted"
        }))
    }

    /// List user sessions
    ///
    /// Note: The session manager trait supports per-session operations.
    /// Listing requires passing known session IDs from the caller.
    pub async fn list_user_sessions(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let user_id = request
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing user_id".to_string()))?;

        // Extract optional session IDs to check
        let session_ids: Vec<String> = request
            .get("session_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Probe each requested session for its metadata
        let mut active_sessions = Vec::new();
        for sid in &session_ids {
            if let Ok(metadata) = self.session_manager.get_session_metadata(sid).await {
                active_sessions.push(json!({
                    "session_id": sid,
                    "created_at": metadata.created_at.to_rfc3339(),
                    "last_activity": metadata.last_activity.to_rfc3339(),
                    "capabilities": metadata.capabilities,
                }));
            }
        }

        Ok(json!({
            "user_id": user_id,
            "sessions": active_sessions,
            "total_count": active_sessions.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::primal_provider::SquirrelPrimalProvider;
    use crate::universal_adapter_v2::UniversalAdapterV2;

    async fn provider() -> SquirrelPrimalProvider {
        let adapter = UniversalAdapterV2::awaken().await.expect("adapter");
        let mc = std::sync::Arc::new(crate::monitoring::metrics::MetricsCollector::new());
        let em = std::sync::Arc::new(crate::ecosystem::EcosystemManager::new(
            crate::ecosystem::config::EcosystemConfig::default(),
            mc,
        ));
        let sessions = std::sync::Arc::new(crate::session::SessionManagerImpl::new(
            crate::session::SessionConfig::default(),
        )) as std::sync::Arc<dyn crate::session::SessionManager>;
        SquirrelPrimalProvider::new(
            "sess-test".to_string(),
            squirrel_mcp_config::EcosystemConfig::default(),
            adapter,
            em,
            sessions,
        )
    }

    #[tokio::test]
    async fn create_get_update_delete_session_roundtrip() {
        let p = provider().await;
        let created = p
            .create_session(serde_json::json!({ "user_id": "u1" }))
            .await
            .expect("create");
        let sid = created["session_id"].as_str().expect("sid");
        assert_eq!(created["user_id"], "u1");

        let meta = p
            .get_session(serde_json::json!({ "session_id": sid }))
            .await
            .expect("get");
        assert_eq!(meta["session_id"], sid);

        let updated = p
            .update_session(serde_json::json!({
                "session_id": sid,
                "metadata": { "k": "v" }
            }))
            .await
            .expect("update");
        assert_eq!(updated["status"], "updated");

        let deleted = p
            .delete_session(serde_json::json!({ "session_id": sid }))
            .await
            .expect("delete");
        assert_eq!(deleted["status"], "deleted");
    }

    #[tokio::test]
    async fn validation_errors_missing_fields() {
        let p = provider().await;
        let e = p.create_session(serde_json::json!({})).await.unwrap_err();
        assert!(matches!(e, crate::error::PrimalError::ValidationError(_)));

        let e2 = p.get_session(serde_json::json!({})).await.unwrap_err();
        assert!(matches!(e2, crate::error::PrimalError::ValidationError(_)));
    }

    #[tokio::test]
    async fn list_user_sessions() {
        let p = provider().await;
        let created = p
            .create_session(serde_json::json!({ "user_id": "u2" }))
            .await
            .expect("should succeed");
        let sid = created["session_id"].as_str().expect("should succeed");

        let listed = p
            .list_user_sessions(serde_json::json!({
                "user_id": "u2",
                "session_ids": [sid]
            }))
            .await
            .expect("list");
        assert_eq!(listed["user_id"], "u2");
        assert_eq!(listed["total_count"], 1);
    }
}
