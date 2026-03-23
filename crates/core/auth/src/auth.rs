// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Modern authentication service with capability discovery and standalone fallback
//!
//! Uses universal adapter pattern for capability discovery - no hardcoded primal dependencies.
//! Discovers any primal providing security/auth capabilities through network effects.

use crate::errors::{AuthError, AuthResult};
use crate::session::{Session, SessionManager};
use crate::types::{
    AuthContext, AuthProvider, LoginRequest, LoginResponse, SecurityCapabilityInfo, User,
};

use chrono::Duration;
#[cfg(feature = "http-auth")]
use reqwest::Client;
use serde_json::json;
// Removed: use squirrel_mcp_config::get_service_endpoints;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Modern authentication service supporting capability discovery and standalone fallback
#[derive(Debug)]
pub struct AuthService {
    /// HTTP client for external auth requests
    #[cfg(feature = "http-auth")]
    client: Client,
    /// Session manager for handling user sessions
    session_manager: SessionManager,
    /// Current authentication provider configuration
    auth_provider: AuthProvider,
    /// In-memory user store (for standalone mode)
    users: HashMap<String, User>,
}

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
        users.insert("admin".to_string(), Self::create_default_admin_user());
        users.insert("user".to_string(), Self::create_default_user());

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
                    endpoint: security_endpoint.clone(),
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

    /// Authenticate user with discovered security capability
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if network I/O, HTTP, JSON parsing, or session storage fails.
    pub async fn authenticate(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        match &self.auth_provider {
            AuthProvider::SecurityCapability {
                endpoint,
                capability_info,
                ..
            } => {
                let endpoint_clone = endpoint.clone();
                let capability_clone = capability_info.clone();
                self.authenticate_with_security_capability(
                    request,
                    &endpoint_clone,
                    &capability_clone,
                )
                .await
            }
            AuthProvider::Standalone => self.authenticate_standalone(request).await,
            AuthProvider::Development => self.authenticate_development(request).await,
        }
    }

    /// Authenticate using any discovered security capability - completely generic
    async fn authenticate_with_security_capability(
        &self,
        request: LoginRequest,
        endpoint: &str,
        capability_info: &SecurityCapabilityInfo,
    ) -> AuthResult<LoginResponse> {
        // Use generic auth API patterns that work across primals
        let auth_url = format!("{}/api/auth/login", endpoint.trim_end_matches('/'));

        let payload = json!({
            "username": request.username,
            "password": request.password,
            "additional_factors": request.additional_factors,
            "client_info": {
                "primal": "squirrel",
                "version": "v1"
            }
        });

        debug!(
            "Authenticating with security capability at: {} (type: {})",
            auth_url, capability_info.primal_type
        );

        let response = self.client.post(&auth_url).json(&payload).send().await?;

        if response.status().is_success() {
            let auth_data: serde_json::Value = response.json().await?;

            // Parse generic security response
            let user = Self::parse_security_user_response(&auth_data)?;
            let session_duration = Duration::hours(8); // Default 8-hour session
            let session = Session::new(user.id, session_duration, self.auth_provider.clone());

            let auth_context = AuthContext::new(
                &user,
                session.id,
                session.expires_at,
                self.auth_provider.clone(),
            );

            // Store session and get needed values before moving
            let session_id = session.id;
            let expires_at = session.expires_at;
            self.session_manager.create_session(session).await?;

            Ok(LoginResponse {
                success: true,
                user_context: Some(auth_context),
                session_token: Some(session_id.to_string()),
                expires_at: Some(expires_at),
                error_message: None,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some(format!(
                    "Security capability authentication failed: {error_text}"
                )),
            })
        }
    }

    /// Standalone authentication (failsafe fallback)
    async fn authenticate_standalone(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in standalone mode");

        // Simple credential check for demo/fallback
        if let Some(user) = self.users.get(&request.username) {
            // In a real implementation, you'd hash and compare passwords properly
            if Self::verify_password(&request.password, &request.username) {
                let session_duration = Duration::hours(8);
                let session = Session::new(user.id, session_duration, AuthProvider::Standalone);

                let auth_context = AuthContext::new(
                    user,
                    session.id,
                    session.expires_at,
                    AuthProvider::Standalone,
                );

                // Store session and get needed values before moving
                let session_id = session.id;
                let expires_at = session.expires_at;
                self.session_manager.create_session(session).await?;

                Ok(LoginResponse {
                    success: true,
                    user_context: Some(auth_context),
                    session_token: Some(session_id.to_string()),
                    expires_at: Some(expires_at),
                    error_message: None,
                })
            } else {
                Ok(LoginResponse {
                    success: false,
                    user_context: None,
                    session_token: None,
                    expires_at: None,
                    error_message: Some("Invalid credentials".to_string()),
                })
            }
        } else {
            Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some("User not found".to_string()),
            })
        }
    }

    /// Development authentication (always succeeds for testing)
    async fn authenticate_development(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in development mode");

        let user = User::new(&request.username, format!("{}@dev.local", request.username));
        let session_duration = Duration::hours(24); // Long session for dev
        let session = Session::new(user.id, session_duration, AuthProvider::Development);

        let auth_context = AuthContext::new(
            &user,
            session.id,
            session.expires_at,
            AuthProvider::Development,
        );

        let session_id = session.id;
        let expires_at = session.expires_at;
        self.session_manager.create_session(session).await?;

        Ok(LoginResponse {
            success: true,
            user_context: Some(auth_context),
            session_token: Some(session_id.to_string()),
            expires_at: Some(expires_at),
            error_message: None,
        })
    }

    /// Validate a session token
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token is malformed or session lookup fails.
    pub async fn validate_session(&self, session_token: &str) -> AuthResult<Option<AuthContext>> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        if let Some(session) = self.session_manager.get_session(&session_id).await? {
            if session.is_expired() || !session.is_active {
                return Ok(None);
            }

            // Get user information based on auth provider
            if let Some(user) = self.get_user_by_id(&session.user_id)? {
                let auth_context = AuthContext::new(
                    &user,
                    session.id,
                    session.expires_at,
                    session.auth_provider.clone(),
                );
                Ok(Some(auth_context))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Invalidate a session
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token cannot be parsed or invalidation fails.
    pub async fn logout(&self, session_token: &str) -> AuthResult<bool> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        self.session_manager.invalidate_session(&session_id).await
    }

    /// Get current authentication provider
    pub const fn get_auth_provider(&self) -> &AuthProvider {
        &self.auth_provider
    }

    // Helper methods

    fn parse_security_user_response(data: &serde_json::Value) -> AuthResult<User> {
        let username = data["username"]
            .as_str()
            .or_else(|| data["user"]["username"].as_str())
            .ok_or_else(|| AuthError::internal_error("Missing username in security response"))?;

        let default_email = format!("{username}@security.local");
        let email = data["email"]
            .as_str()
            .or_else(|| data["user"]["email"].as_str())
            .unwrap_or(&default_email);

        let mut user = User::new(username, email);

        // Parse roles if available from any security provider format
        if let Some(roles) = data["roles"]
            .as_array()
            .or_else(|| data["user"]["roles"].as_array())
        {
            for role in roles {
                if let Some(role_str) = role.as_str() {
                    user.roles.push(role_str.to_string());
                }
            }
        }

        Ok(user)
    }

    fn verify_password(password: &str, username: &str) -> bool {
        // Simple fallback verification - in production use proper hashing
        match username {
            "admin" => password == "admin123",
            "user" => password == "user123",
            _ => false,
        }
    }

    fn get_user_by_id(&self, user_id: &Uuid) -> AuthResult<Option<User>> {
        // In standalone mode, find user by ID
        for user in self.users.values() {
            if user.id == *user_id {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }

    fn create_default_admin_user() -> User {
        let mut user = User::new("admin", "admin@squirrel.local");
        user.roles.push("admin".to_string());
        user.roles.push("user".to_string());
        user
    }

    fn create_default_user() -> User {
        let mut user = User::new("user", "user@squirrel.local");
        user.roles.push("user".to_string());
        user
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::Permission;
    use chrono::Duration;

    #[test]
    fn test_auth_service_new_standalone_fallback() {
        // Point to non-existent endpoint so discovery fails and we get Standalone
        let result = temp_env::with_vars(
            [
                ("SECURITY_SERVICE_ENDPOINT", Some("http://127.0.0.1:19999")),
                ("SECURITY_AUTHENTICATION_PORT", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(AuthService::new())
            },
        );
        let service = result.expect("AuthService::new should succeed");
        assert!(matches!(
            service.get_auth_provider(),
            AuthProvider::Standalone
        ));
    }

    #[tokio::test]
    async fn test_authenticate_standalone_admin_success() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        users.insert(
            "admin".to_string(),
            User::new("admin", "admin@squirrel.local"),
        );
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let request = LoginRequest::new("admin", "admin123");
        let response = service.authenticate(request).await.unwrap();
        assert!(response.success);
        assert!(response.user_context.is_some());
        assert!(response.session_token.is_some());
        assert!(response.error_message.is_none());
    }

    #[tokio::test]
    async fn test_authenticate_standalone_wrong_password() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        users.insert(
            "admin".to_string(),
            User::new("admin", "admin@squirrel.local"),
        );
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let request = LoginRequest::new("admin", "wrong");
        let response = service.authenticate(request).await.unwrap();
        assert!(!response.success);
        assert!(response.user_context.is_none());
        assert_eq!(
            response.error_message.as_deref(),
            Some("Invalid credentials")
        );
    }

    #[tokio::test]
    async fn test_authenticate_standalone_user_not_found() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let request = LoginRequest::new("unknown", "any");
        let response = service.authenticate(request).await.unwrap();
        assert!(!response.success);
        assert_eq!(response.error_message.as_deref(), Some("User not found"));
    }

    #[tokio::test]
    async fn test_authenticate_standalone_user_success() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        users.insert("user".to_string(), User::new("user", "user@squirrel.local"));
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let request = LoginRequest::new("user", "user123");
        let response = service.authenticate(request).await.unwrap();
        assert!(response.success);
        assert!(response.user_context.is_some());
    }

    #[tokio::test]
    async fn test_authenticate_development_always_succeeds() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Development, users);

        let request = LoginRequest::new("devuser", "any-password");
        let response = service.authenticate(request).await.unwrap();
        assert!(response.success);
        assert!(response.user_context.is_some());
        assert_eq!(response.user_context.as_ref().unwrap().username, "devuser");
    }

    #[tokio::test]
    async fn test_validate_session_valid() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        let user = User::new("admin", "admin@squirrel.local");
        users.insert("admin".to_string(), user.clone());
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let request = LoginRequest::new("admin", "admin123");
        let login_response = service.authenticate(request).await.unwrap();
        let session_token = login_response.session_token.clone().unwrap();

        let validated = service.validate_session(&session_token).await.unwrap();
        assert!(validated.is_some());
        assert_eq!(validated.unwrap().username, "admin");
    }

    #[tokio::test]
    async fn test_validate_session_invalid_uuid() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let result = service.validate_session("not-a-valid-uuid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_session_unknown_session() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let fake_uuid = uuid::Uuid::new_v4().to_string();
        let validated = service.validate_session(&fake_uuid).await.unwrap();
        assert!(validated.is_none());
    }

    #[tokio::test]
    async fn test_logout_valid_session() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        users.insert(
            "admin".to_string(),
            User::new("admin", "admin@squirrel.local"),
        );
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let login_response = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .unwrap();
        let session_token = login_response.session_token.clone().unwrap();

        let result = service.logout(&session_token).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_logout_invalid_uuid() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let result = service.logout("not-a-uuid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_auth_provider() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Development, users);
        assert!(matches!(
            service.get_auth_provider(),
            AuthProvider::Development
        ));
    }

    #[test]
    fn test_parse_security_capability_with_auth() {
        let body = r#"{"auth": true, "primal_id": "security-primal"}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert_eq!(info.primal_type, "security-primal");
        assert!(info.supports_auth);
        assert_eq!(info.api_version, "v1");
    }

    #[test]
    fn test_parse_security_capability_with_security() {
        let body = r#"{"security": true, "session": true}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert_eq!(info.primal_type, "discovered");
        assert!(info.supports_sessions);
        assert_eq!(info.api_version, "v1");
    }

    #[test]
    fn test_parse_security_capability_with_token() {
        let body = r#"{"token": "supported"}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert!(info.supports_sessions);
    }

    #[test]
    fn test_parse_security_capability_no_capabilities() {
        let body = r#"{"foo": "bar", "baz": 123}"#;
        let result = AuthService::parse_security_capability_for_test(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_security_capability_empty_body() {
        let result = AuthService::parse_security_capability_for_test("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_security_capability_primal_id_from_nested() {
        let body = r#"{"user": {"primal_id": "nested"}}"#;
        let result = AuthService::parse_security_capability_for_test(body);
        assert!(result.is_err()); // no auth/security/session keywords
    }

    #[test]
    fn test_parse_security_capability_authentication_keyword() {
        let body = r#"{"authentication": "enabled"}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert!(info.supports_auth);
    }

    #[test]
    fn login_request_constructs_with_additional_factors() {
        let factors = serde_json::json!({"mfa": "123456"});
        let req = LoginRequest::new("alice", "secret").with_factors(factors.clone());
        assert_eq!(req.username, "alice");
        assert_eq!(req.password, "secret");
        assert_eq!(req.additional_factors, Some(factors));
    }

    #[test]
    fn security_capability_info_constructs_with_expected_fields() {
        let info = SecurityCapabilityInfo {
            primal_type: "test-primal".to_string(),
            supports_auth: true,
            supports_sessions: false,
            api_version: "v2".to_string(),
        };
        assert_eq!(info.primal_type, "test-primal");
        assert!(info.supports_auth);
        assert!(!info.supports_sessions);
        assert_eq!(info.api_version, "v2");
    }

    #[test]
    fn auth_provider_default_is_standalone() {
        assert_eq!(AuthProvider::default(), AuthProvider::Standalone);
    }

    #[test]
    fn auth_provider_security_capability_holds_endpoint_and_metadata() {
        let cap = SecurityCapabilityInfo {
            primal_type: "p".to_string(),
            supports_auth: true,
            supports_sessions: true,
            api_version: "v1".to_string(),
        };
        let provider = AuthProvider::SecurityCapability {
            endpoint: "https://auth.example:8443".to_string(),
            discovery_method: "unit_test".to_string(),
            capability_info: cap.clone(),
        };
        match provider {
            AuthProvider::SecurityCapability {
                endpoint,
                discovery_method,
                capability_info,
            } => {
                assert_eq!(endpoint, "https://auth.example:8443");
                assert_eq!(discovery_method, "unit_test");
                assert_eq!(capability_info, cap);
            }
            _ => panic!("expected SecurityCapability variant"),
        }
    }

    #[test]
    fn permission_matches_requires_resource_action_and_scope() {
        let read_mcp = Permission::new("mcp", "read");
        let read_mcp_scope = Permission::with_scope("mcp", "read", "tenant-a");
        assert!(read_mcp.matches(&read_mcp));
        assert!(!read_mcp.matches(&read_mcp_scope));
        assert!(!read_mcp.matches(&Permission::new("mcp", "write")));
    }

    #[test]
    fn user_has_role_and_permission_checks() {
        let mut user = User::new("ops", "ops@example.com");
        user.roles.push("admin".to_string());
        let perm = Permission::new("api", "read");
        user.permissions.push(perm.clone());
        assert!(user.has_role("admin"));
        assert!(!user.has_role("guest"));
        assert!(user.has_permission(&Permission::new("api", "read")));
        assert!(!user.has_permission(&Permission::new("api", "write")));
        assert!(user.has_permission(&perm));
    }

    #[tokio::test]
    async fn auth_service_standalone_success_propagates_user_roles_to_context() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        let mut admin = User::new("admin", "admin@squirrel.local");
        admin.roles.push("admin".to_string());
        admin.roles.push("user".to_string());
        users.insert("admin".to_string(), admin);
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let response = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .unwrap();
        assert!(response.success);
        let ctx = response
            .user_context
            .as_ref()
            .expect("expected auth context");
        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("user"));
        assert!(!ctx.has_role("superuser"));
    }

    #[tokio::test]
    async fn auth_service_development_authentication_sets_username_and_empty_roles() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Development, users);
        let response = service
            .authenticate(LoginRequest::new("devuser", "ignored"))
            .await
            .unwrap();
        let ctx = response
            .user_context
            .as_ref()
            .expect("expected auth context");
        assert_eq!(ctx.username, "devuser");
        assert!(ctx.roles.is_empty());
    }

    #[tokio::test]
    async fn auth_service_validate_session_returns_none_after_logout() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        users.insert(
            "admin".to_string(),
            User::new("admin", "admin@squirrel.local"),
        );
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);

        let login = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .unwrap();
        let token = login.session_token.clone().expect("expected session token");
        assert!(service.validate_session(&token).await.unwrap().is_some());
        assert!(service.logout(&token).await.unwrap());
        assert!(service.validate_session(&token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn auth_service_validate_session_returns_none_for_expired_session() {
        let session_manager = SessionManager::new();
        let mut users = HashMap::new();
        let user = User::new("ghost", "ghost@example.com");
        let user_id = user.id;
        users.insert("ghost".to_string(), user);

        let expired = Session::new(user_id, Duration::hours(-1), AuthProvider::Standalone);
        let token = expired.id.to_string();
        session_manager.create_session(expired).await.unwrap();

        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);
        let validated = service.validate_session(&token).await.unwrap();
        assert!(validated.is_none());
    }

    #[tokio::test]
    async fn auth_service_validate_session_invalid_token_maps_to_token_error() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);
        let err = service
            .validate_session("clearly-not-a-uuid")
            .await
            .expect_err("expected token parse error");
        let msg = format!("{err}");
        assert!(
            msg.contains("parse") || msg.contains("Token error"),
            "unexpected message: {msg}"
        );
    }

    #[tokio::test]
    async fn auth_service_logout_invalid_token_returns_token_error() {
        let session_manager = SessionManager::new();
        let users = HashMap::new();
        let service = AuthService::for_testing(session_manager, AuthProvider::Standalone, users);
        let err = service
            .logout("also-not-a-uuid")
            .await
            .expect_err("expected token parse error");
        let msg = format!("{err}");
        assert!(
            msg.contains("parse") || msg.contains("Token error"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn parse_security_capability_accepts_secure_keyword_without_auth_flag() {
        let body = r#"{"secure": true}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert!(!info.supports_auth);
        assert!(!info.supports_sessions);
        assert_eq!(info.primal_type, "discovered");
    }

    #[test]
    fn parse_security_capability_accepts_session_keyword() {
        let body = r#"{"session": "ok"}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert!(info.supports_sessions);
    }

    #[test]
    fn parse_security_capability_primal_id_from_json_root() {
        let body = r#"{"auth": true, "primal_id": "alpha"}"#;
        let info = AuthService::parse_security_capability_for_test(body).unwrap();
        assert_eq!(info.primal_type, "alpha");
    }

    #[test]
    fn login_response_constructs_success_and_failure_shapes() {
        let ok = LoginResponse {
            success: true,
            user_context: None,
            session_token: Some("tok".to_string()),
            expires_at: None,
            error_message: None,
        };
        assert!(ok.success);
        assert!(ok.error_message.is_none());

        let fail = LoginResponse {
            success: false,
            user_context: None,
            session_token: None,
            expires_at: None,
            error_message: Some("nope".to_string()),
        };
        assert!(!fail.success);
        assert_eq!(fail.error_message.as_deref(), Some("nope"));
    }

    #[test]
    fn auth_error_token_error_display_matches_expected_format() {
        let err = AuthError::token_error("parse", "bad uuid");
        assert_eq!(format!("{err}"), "Token error in parse: bad uuid");
    }
}
