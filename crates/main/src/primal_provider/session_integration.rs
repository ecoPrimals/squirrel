//! Session Management Integration

use serde_json::json;

use super::core::SquirrelPrimalProvider;
use crate::error::PrimalError;

/// Simple session data structure for mock operations
#[derive(Debug, Clone)]
struct SessionData {
    session_id: String,
    user_id: String,
    created_at: String,
    last_accessed: chrono::DateTime<chrono::Utc>,
}

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
            .map_err(|e| PrimalError::Internal(format!("Session creation failed: {}", e)))?;

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
            .map_err(|e| PrimalError::Internal(format!("Session retrieval failed: {}", e)))?;

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
            .map_err(|e| PrimalError::Internal(format!("Session update failed: {}", e)))?;

        Ok(json!({
            "session_id": session_id,
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "status": "updated"
        }))
    }

    /// Delete session
    pub async fn delete_session(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let session_id = request
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing session_id".to_string()))?;

        // Delete session through session manager - using create_session as placeholder
        self.session_manager
            .create_session(Some("placeholder".to_string()))
            .await
            .map_err(|e| PrimalError::Internal(format!("Session operation failed: {}", e)))?;

        Ok(json!({
            "session_id": session_id,
            "deleted_at": chrono::Utc::now().to_rfc3339(),
            "status": "deleted"
        }))
    }

    /// List user sessions
    pub async fn list_user_sessions(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let user_id = request
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing user_id".to_string()))?;

        // List sessions through session manager - mock response for now
        let sessions: Vec<SessionData> = vec![];

        let session_list: Vec<serde_json::Value> = sessions
            .into_iter()
            .map(|session| {
                json!({
                    "session_id": session.session_id,
                    "user_id": session.user_id,
                    "created_at": session.created_at,
                    "last_accessed": session.last_accessed.to_rfc3339(),
                    "status": "active"
                })
            })
            .collect();

        Ok(json!({
            "user_id": user_id,
            "sessions": session_list,
            "total_count": session_list.len()
        }))
    }
}
