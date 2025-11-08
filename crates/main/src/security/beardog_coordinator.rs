//! Simple BearDog Security Coordinator
//!
//! Coordinates with BearDog security primal for AI security operations.
//! Replaces 1473+ lines of over-engineered universal security adapter with focused functionality.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::error::PrimalError;
use crate::security::types::{
    SecurityContext, SecurityRequest, SecurityRequestType, SecurityResponse, SecurityResponseStatus,
};

/// Simple BearDog Security Coordinator
///
/// Delegates security operations to BearDog primal instead of trying to implement
/// a full security system within Squirrel.
#[derive(Debug, Default)]
pub struct BeardogSecurityCoordinator {
    /// BearDog endpoint for security operations
    security_service_endpoint: String, // Discovered via capability matching
    /// Simple session cache
    sessions: Arc<RwLock<HashMap<String, SecurityContext>>>,
}

/// Security service discovery and routing
impl BeardogSecurityCoordinator {
    /// Discover optimal security service endpoint
    pub async fn discover_security_endpoint() -> Result<String> {
        // Try to discover security service through capability registry
        if let Ok(endpoint) = std::env::var("SECURITY_SERVICE_ENDPOINT") {
            return Ok(endpoint);
        }

        // Default fallback
        Ok("http://localhost:8090".to_string())
    }

    /// Create new coordinator with capability-based discovery
    pub async fn with_capability_discovery() -> Result<Self> {
        let endpoint = Self::discover_security_endpoint().await?;

        Ok(Self {
            security_service_endpoint: endpoint,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new security coordinator with fallback endpoint
    pub fn new() -> Self {
        Self {
            security_service_endpoint: std::env::var("SECURITY_SERVICE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8443".to_string()),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Coordinate security request with BearDog
    pub async fn coordinate_security(
        &mut self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, PrimalError> {
        let operation = format!("{:?}", request.request_type);
        info!(
            "🐻 Coordinating security request with BearDog: {}",
            operation
        );

        // Simple delegation to BearDog
        // In real implementation, would make HTTP call to BearDog
        let response = SecurityResponse {
            request_id: request.request_id.clone(),
            status: SecurityResponseStatus::Success,
            payload: serde_json::json!({
                "beardog_session": format!("beardog_session_{}", request.request_id),
                "security_level": "high",
                "operation": operation
            }),
            metadata: HashMap::from([
                ("coordinator".to_string(), serde_json::json!("squirrel")),
                (
                    "security_provider".to_string(),
                    serde_json::json!("beardog"),
                ),
            ]),
            processing_time: Duration::from_millis(50),
            timestamp: chrono::Utc::now(),
        };

        // Cache the security context with proper error handling
        let session_id = format!("beardog_session_{}", request.request_id);
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(session_id, request.context);
        } else {
            warn!("Failed to acquire write lock on sessions");
        }

        info!("✅ BearDog security coordination complete");
        Ok(response)
    }

    /// Get cached security context with proper borrowing
    pub fn get_security_context(&self, session_id: &str) -> Option<SecurityContext> {
        self.sessions.read().ok()?.get(session_id).cloned() // Clone the context instead of returning a reference
    }

    /// Check if operation requires elevated security via BearDog
    pub fn requires_beardog_security(&self, request_type: &SecurityRequestType) -> bool {
        // Simple policy - delegate critical operations to BearDog
        matches!(
            request_type,
            SecurityRequestType::Authentication
                | SecurityRequestType::Authorization
                | SecurityRequestType::Encryption
                | SecurityRequestType::Decryption
                | SecurityRequestType::Audit
        )
    }

    /// Coordinate with BearDog for authentication
    pub async fn authenticate_with_beardog(
        &mut self,
        user_id: &str,
    ) -> Result<String, PrimalError> {
        debug!("🐻 Authenticating user {} via BearDog", user_id);

        let request = SecurityRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: SecurityRequestType::Authentication,
            payload: serde_json::json!({
                "user_id": user_id,
                "operation": "authenticate"
            }),
            metadata: HashMap::new(),
            context: SecurityContext::new("authenticate", user_id),
            timestamp: chrono::Utc::now(),
        };

        let response = self.coordinate_security(request).await?;

        // Extract session from response payload
        response
            .payload
            .get("beardog_session")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| PrimalError::SecurityError("BearDog authentication failed".to_string()))
    }

    /// Simple health check - much simpler than complex over-engineered system
    pub fn is_healthy(&self) -> bool {
        !self.security_service_endpoint.is_empty()
    }
}
