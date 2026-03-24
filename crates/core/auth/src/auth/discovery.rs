// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability discovery and health probing against a configurable security endpoint.

use super::AuthService;
use crate::errors::{AuthError, AuthResult};
use crate::session::SessionManager;
#[cfg(all(test, feature = "http-auth"))]
use crate::types::User;
use crate::types::{AuthProvider, SecurityCapabilityInfo};
#[cfg(feature = "http-auth")]
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, info};

impl AuthService {
    /// Create a new authentication service with pure capability discovery
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the service cannot be constructed.
    pub async fn new() -> AuthResult<Self> {
        let client = Client::new();
        let session_manager = SessionManager::new();

        // Pure capability discovery - no hardcoded primal dependencies
        let auth_provider = Self::discover_security_capability(&client).await;

        info!(
            "Initialized auth service with provider: {:?}",
            auth_provider
        );

        // Initialize with some default users for standalone mode
        let mut users = HashMap::new();
        users.insert("admin".to_string(), super::default_admin_user());
        users.insert("user".to_string(), super::default_user());

        Ok(Self {
            client,
            session_manager,
            auth_provider,
            users,
        })
    }

    /// Discover security capability through universal adapter - no hardcoded primal knowledge
    async fn discover_security_capability(client: &Client) -> AuthProvider {
        // Try to discover ANY primal with security capabilities through universal adapter
        // Multi-tier security endpoint resolution
        let security_endpoint = std::env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8443); // Default security auth port
            format!("http://localhost:{port}")
        });

        debug!(
            "Attempting security capability discovery at: {}",
            security_endpoint
        );

        match Self::test_security_capability(client, &security_endpoint).await {
            Ok(capability_info) => {
                info!("Security capability discovered: {:?}", capability_info);
                AuthProvider::SecurityCapability {
                    endpoint: security_endpoint,
                    discovery_method: "universal_adapter_discovery".to_string(),
                    capability_info,
                }
            }
            Err(e) => {
                debug!(
                    "Security capability discovery failed: {}. Using standalone fallback",
                    e
                );
                AuthProvider::Standalone
            }
        }
    }

    /// Test any primal for security capability - completely generic
    async fn test_security_capability(
        client: &Client,
        endpoint: &str,
    ) -> AuthResult<SecurityCapabilityInfo> {
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));

        let response = client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            // Check for generic security capability indicators
            if let Ok(body) = response.text().await {
                let capability_info = Self::parse_security_capability(&body)?;
                Ok(capability_info)
            } else {
                Err(AuthError::network_error(
                    "capability_test",
                    "No response body",
                ))
            }
        } else {
            Err(AuthError::network_error(
                "capability_test",
                format!("HTTP {}", response.status()),
            ))
        }
    }

    /// Parse security capability information from any primal
    ///
    /// # TRUE PRIMAL Pattern
    ///
    /// Squirrel identifies capabilities by **what they can do** (auth, session,
    /// security), not by **who provides them**. The `provider_id` field is
    /// extracted from the JSON-RPC response's `primal_id` field if present,
    /// falling back to `"discovered"`.
    fn parse_security_capability(response_body: &str) -> AuthResult<SecurityCapabilityInfo> {
        // Look for generic security capability indicators (not primal-specific)
        let has_auth = response_body.contains("auth") || response_body.contains("authentication");
        let has_security = response_body.contains("security") || response_body.contains("secure");
        let has_session = response_body.contains("session") || response_body.contains("token");

        if has_auth || has_security || has_session {
            // Capability-based: extract provider_id from response if available
            let primal_type = serde_json::from_str::<serde_json::Value>(response_body)
                .ok()
                .and_then(|v| {
                    v.get("primal_id")
                        .and_then(|id| id.as_str().map(String::from))
                })
                .unwrap_or_else(|| "discovered".to_string());

            Ok(SecurityCapabilityInfo {
                primal_type,
                supports_auth: has_auth,
                supports_sessions: has_session,
                api_version: "v1".to_string(),
            })
        } else {
            Err(AuthError::authorization_error(
                "No security capabilities detected",
            ))
        }
    }

    /// Test helper: parse security capability from response body (for unit tests).
    ///
    /// # Errors
    ///
    /// Returns `AuthError` if the response body cannot be parsed as a security capability.
    #[cfg(all(test, feature = "http-auth"))]
    pub fn parse_security_capability_for_test(
        response_body: &str,
    ) -> AuthResult<SecurityCapabilityInfo> {
        Self::parse_security_capability(response_body)
    }

    /// Test helper: construct `AuthService` with explicit components (no network)
    #[cfg(all(test, feature = "http-auth"))]
    pub fn for_testing(
        session_manager: SessionManager,
        auth_provider: AuthProvider,
        users: HashMap<String, User>,
    ) -> Self {
        Self {
            client: Client::new(),
            session_manager,
            auth_provider,
            users,
        }
    }
}
