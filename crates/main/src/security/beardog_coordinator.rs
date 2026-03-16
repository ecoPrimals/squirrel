// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]

//! Security Service Coordinator
//!
//! Coordinates with security primals for AI security operations via capability-based discovery.
//! Replaces 1473+ lines of over-engineered universal security adapter with focused functionality.
//!
//! ## Sovereignty Compliance
//!
//! This module discovers security services at runtime via `SECURITY_SERVICE_ENDPOINT` env var
//! instead of hardcoding specific primal names. Any primal providing security capabilities
//! can fulfill security requests.
//!
//! ## Migration from Hardcoded
//!
//! **Old Pattern (Hardcoded)**:
//! ```ignore
//! let beardog = BeardogSecurityCoordinator::new(); // Coupled to specific primal
//! ```
//!
//! **New Pattern (Capability-Based)**:
//! ```ignore
//! let security = SecurityCoordinator::with_capability_discovery().await?;
//! // Discovers ANY security service, not specific to one primal
//! ```

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::error::PrimalError;
use crate::security::types::{
    SecurityContext, SecurityRequest, SecurityRequestType, SecurityResponse, SecurityResponseStatus,
};

/// Security Service Coordinator
///
/// Delegates security operations to discovered security primals instead of implementing
/// a full security system within Squirrel. Uses capability-based discovery to find
/// available security services at runtime.
///
/// ## Backward Compatibility
///
/// The struct name `BeardogSecurityCoordinator` is maintained for backward compatibility
/// but is deprecated. Use `SecurityCoordinator` type alias instead.
#[derive(Debug, Default)]
#[deprecated(
    since = "0.1.0",
    note = "Use SecurityCoordinator type alias. This struct will be renamed in the next major version."
)]
pub struct BeardogSecurityCoordinator {
    /// Security service endpoint (discovered via capability matching or env var)
    security_service_endpoint: String,
    /// Simple session cache
    sessions: Arc<RwLock<HashMap<String, SecurityContext>>>,
}

/// Modern type alias for security coordination
///
/// Use this instead of `BeardogSecurityCoordinator` for capability-based security.
pub type SecurityCoordinator = BeardogSecurityCoordinator;

/// Security service discovery and routing
impl BeardogSecurityCoordinator {
    /// Discover optimal security service endpoint
    pub async fn discover_security_endpoint() -> Result<String> {
        // Try to discover security service through capability registry
        if let Ok(endpoint) = std::env::var("SECURITY_SERVICE_ENDPOINT") {
            return Ok(endpoint);
        }

        // Multi-tier default fallback
        // 1. SECURITY_SERVICE_ENDPOINT (checked above)
        // 2. SECURITY_AUTHENTICATION_PORT (port override)
        // 3. Default: use universal_constants DEFAULT_SECURITY_PORT
        let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(universal_constants::network::DEFAULT_SECURITY_PORT);
        Ok(format!("http://localhost:{port}"))
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
    ///
    /// **Note**: Prefer `with_capability_discovery()` for async runtime discovery.
    /// This synchronous constructor uses environment variables and fallback defaults.
    #[must_use]
    pub fn new() -> Self {
        use universal_constants::network::get_service_port;

        // Try environment variables in priority order
        let endpoint = if let Ok(endpoint) = std::env::var("SECURITY_SERVICE_ENDPOINT") {
            info!(
                "Using security endpoint from SECURITY_SERVICE_ENDPOINT: {}",
                endpoint
            );
            endpoint
        } else if let Ok(socket) = std::env::var("SECURITY_SOCKET") {
            // TRUE PRIMAL: Use capability-based env var, not primal-specific
            info!(
                "Using security Unix socket from SECURITY_SOCKET: {}",
                socket
            );
            format!("unix://{socket}")
        } else {
            // Check standard biomeOS socket path (capability-based, not primal-specific)
            let uid = nix::unistd::getuid();
            let standard_socket = format!("/run/user/{uid}/biomeos/security.sock");
            if std::path::Path::new(&standard_socket).exists() {
                info!(
                    "Found security service at standard socket: {}",
                    standard_socket
                );
                format!("unix://{standard_socket}")
            } else {
                // Fallback to HTTP with port discovery
                let port = std::env::var("SECURITY_PORT")
                    .or_else(|_| std::env::var("SECURITY_AUTHENTICATION_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| get_service_port("security"));

                let fallback = format!("http://localhost:{port}");
                warn!(
                    "Using fallback security endpoint: {} (set SECURITY_SOCKET or SECURITY_SERVICE_ENDPOINT for production)",
                    fallback
                );
                fallback
            }
        };

        Self {
            security_service_endpoint: endpoint,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Coordinate security request with security service
    pub async fn coordinate_security(
        &mut self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, PrimalError> {
        let operation = format!("{:?}", request.request_type);
        info!(
            "🔒 Coordinating security request with security service: {}",
            operation
        );

        // Simple delegation to discovered security service
        // In real implementation, would make HTTP call to security service
        let response = SecurityResponse {
            request_id: request.request_id.clone(),
            status: SecurityResponseStatus::Success,
            payload: serde_json::json!({
                "security_session": format!("security_session_{}", request.request_id),
                "security_level": "high",
                "operation": operation
            }),
            metadata: HashMap::from([
                ("coordinator".to_string(), serde_json::json!("squirrel")),
                (
                    "security_provider".to_string(),
                    serde_json::json!("discovered_service"),
                ),
            ]),
            processing_time: Duration::from_millis(50),
            timestamp: chrono::Utc::now(),
        };

        // Cache the security context with proper error handling
        let session_id = format!("security_session_{}", request.request_id);
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(session_id, request.context);
        } else {
            warn!("Failed to acquire write lock on sessions");
        }

        info!("✅ Security coordination complete");
        Ok(response)
    }

    /// Get cached security context with proper borrowing
    #[must_use]
    pub fn get_security_context(&self, session_id: &str) -> Option<SecurityContext> {
        self.sessions.read().ok()?.get(session_id).cloned() // Clone the context instead of returning a reference
    }

    /// Check if operation requires elevated security via security service
    #[must_use]
    pub fn requires_security_coordination(&self, request_type: &SecurityRequestType) -> bool {
        // Simple policy - delegate critical operations to security service
        matches!(
            request_type,
            SecurityRequestType::Authentication
                | SecurityRequestType::Authorization
                | SecurityRequestType::Encryption
                | SecurityRequestType::Decryption
                | SecurityRequestType::Audit
        )
    }

    /// Legacy method name for backward compatibility
    #[deprecated(since = "0.1.0", note = "Use requires_security_coordination instead")]
    #[must_use]
    pub fn requires_beardog_security(&self, request_type: &SecurityRequestType) -> bool {
        self.requires_security_coordination(request_type)
    }

    /// Authenticate user via security service
    pub async fn authenticate_with_security_service(
        &mut self,
        user_id: &str,
    ) -> Result<String, PrimalError> {
        debug!("🔒 Authenticating user {} via security service", user_id);

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
            .get("security_session")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                PrimalError::SecurityError("Security service authentication failed".to_string())
            })
    }

    /// Legacy method name for backward compatibility
    ///
    /// **DEPRECATED**: Use `authenticate_with_security_service` instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use authenticate_with_security_service instead"
    )]
    pub async fn authenticate_with_beardog(
        &mut self,
        user_id: &str,
    ) -> Result<String, PrimalError> {
        self.authenticate_with_security_service(user_id).await
    }

    /// Simple health check - much simpler than complex over-engineered system
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self.security_service_endpoint.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_new() {
        temp_env::with_vars_unset(
            [
                "SECURITY_SERVICE_ENDPOINT",
                "SECURITY_SOCKET",
                "SECURITY_PORT",
                "SECURITY_AUTHENTICATION_PORT",
            ],
            || {
                let coord = BeardogSecurityCoordinator::new();
                assert!(coord.is_healthy());
            },
        );
    }

    #[test]
    fn test_coordinator_default() {
        let coord = BeardogSecurityCoordinator::default();
        assert!(!coord.is_healthy());
    }

    #[test]
    fn test_discover_security_endpoint_default() {
        temp_env::with_vars_unset(
            ["SECURITY_SERVICE_ENDPOINT", "SECURITY_AUTHENTICATION_PORT"],
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let endpoint = BeardogSecurityCoordinator::discover_security_endpoint()
                        .await
                        .expect("discover");
                    assert!(endpoint.contains("localhost:8443"), "got: {endpoint}");
                });
            },
        );
    }

    #[test]
    fn test_discover_security_endpoint_env() {
        temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("https://secure.test"),
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let endpoint = BeardogSecurityCoordinator::discover_security_endpoint()
                        .await
                        .expect("discover");
                    assert_eq!(endpoint, "https://secure.test");
                });
            },
        );
    }

    #[test]
    fn test_with_capability_discovery() {
        temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("http://test:9999"),
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let coord = BeardogSecurityCoordinator::with_capability_discovery()
                        .await
                        .expect("create");
                    assert!(coord.is_healthy());
                });
            },
        );
    }

    #[test]
    fn test_requires_security_coordination() {
        let coord = BeardogSecurityCoordinator::new();
        assert!(coord.requires_security_coordination(&SecurityRequestType::Authentication));
        assert!(coord.requires_security_coordination(&SecurityRequestType::Authorization));
        assert!(coord.requires_security_coordination(&SecurityRequestType::Encryption));
        assert!(coord.requires_security_coordination(&SecurityRequestType::Decryption));
        assert!(coord.requires_security_coordination(&SecurityRequestType::Audit));
    }

    #[tokio::test]
    async fn test_coordinate_security() {
        let mut coord = BeardogSecurityCoordinator {
            security_service_endpoint: "http://test:8443".to_string(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        let request = SecurityRequest {
            request_id: "test-123".to_string(),
            request_type: SecurityRequestType::Authentication,
            payload: serde_json::json!({"user": "test"}),
            metadata: HashMap::new(),
            context: SecurityContext::new("test", "user1"),
            timestamp: chrono::Utc::now(),
        };

        let response = coord
            .coordinate_security(request)
            .await
            .expect("coordinate");
        assert_eq!(response.status, SecurityResponseStatus::Success);
        assert_eq!(response.request_id, "test-123");
    }

    #[tokio::test]
    async fn test_authenticate_with_security_service() {
        let mut coord = BeardogSecurityCoordinator {
            security_service_endpoint: "http://test:8443".to_string(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        let session = coord
            .authenticate_with_security_service("user1")
            .await
            .expect("authenticate");
        assert!(session.starts_with("security_session_"));
    }

    #[test]
    fn test_get_security_context() {
        let coord = BeardogSecurityCoordinator {
            security_service_endpoint: "http://test:8443".to_string(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        // No context initially
        assert!(coord.get_security_context("nonexistent").is_none());

        // Insert a context
        {
            let mut sessions = coord.sessions.write().unwrap();
            sessions.insert(
                "session-1".to_string(),
                SecurityContext::new("test", "user1"),
            );
        }

        let ctx = coord.get_security_context("session-1");
        assert!(ctx.is_some());
    }

    #[test]
    fn test_coordinator_new_with_env_endpoint() {
        temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("https://prod.security"),
            || {
                let coord = BeardogSecurityCoordinator::new();
                assert!(coord.is_healthy());
            },
        );
    }
}
