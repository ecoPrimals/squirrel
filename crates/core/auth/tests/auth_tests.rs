// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration tests for MCP auth (`integration-tests` feature).

// Integration tests gated behind `integration-tests` feature — requires http-auth for AuthService, which depends on reqwest/network access.
#[cfg(not(feature = "integration-tests"))]
#[tokio::test]
async fn placeholder_auth_tests_disabled() {}

#[cfg(feature = "integration-tests")]
mod auth_tests_impl {
    #![expect(
        clippy::expect_used,
        reason = "Integration auth tests use expect on auth/session Result paths"
    )]

    use squirrel_mcp_auth::{
        auth::AuthService,
        errors::{AuthError, AuthResult},
        session::SessionManager,
        types::{AuthProvider, LoginRequest, Permission, Session, User},
    };

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    // ====================================================================
    // AuthService
    // ====================================================================

    async fn create_test_auth_service() -> AuthResult<AuthService> {
        AuthService::new().await
    }

    #[tokio::test]
    async fn test_auth_service_initialization() {
        let result = create_test_auth_service().await;
        assert!(
            result.is_ok(),
            "Auth service should initialize successfully"
        );
    }

    #[tokio::test]
    async fn test_successful_standalone_login() {
        let service = create_test_auth_service().await.expect("should succeed");
        let request = LoginRequest::new("admin", "admin123");
        let result = service.authenticate(request).await;

        assert!(
            result.is_ok(),
            "Login with valid credentials should succeed"
        );
        let response = result.expect("should succeed");
        assert!(response.success, "Response should indicate success");
        assert!(
            response.session_token.is_some(),
            "Session token should be present"
        );
        assert!(
            response.user_context.is_some(),
            "User context should be present"
        );
        let ctx = response.user_context.clone().expect("should succeed");
        assert_eq!(ctx.username, "admin");
    }

    #[tokio::test]
    async fn test_failed_login_invalid_credentials() {
        let service = create_test_auth_service().await.expect("should succeed");
        let request = LoginRequest::new("admin", "wrongpassword");
        let response = service.authenticate(request).await.expect("should succeed");

        assert!(
            !response.success,
            "Login with invalid credentials should fail"
        );
        assert!(response.error_message.is_some());
    }

    #[tokio::test]
    async fn test_failed_login_unknown_user() {
        let service = create_test_auth_service().await.expect("should succeed");
        let request = LoginRequest::new("nonexistent", "password");
        let response = service.authenticate(request).await.expect("should succeed");

        assert!(!response.success, "Login with unknown user should fail");
    }

    #[tokio::test]
    async fn test_logout() {
        let service = create_test_auth_service().await.expect("should succeed");
        let request = LoginRequest::new("admin", "admin123");
        let login_response = service.authenticate(request).await.expect("should succeed");
        assert!(login_response.success);

        let token = login_response
            .session_token
            .clone()
            .expect("should succeed");
        let result = service.logout(&token).await;
        assert!(result.is_ok(), "Logout should succeed");
    }

    #[tokio::test]
    async fn test_validate_session_valid() {
        let service = create_test_auth_service().await.expect("should succeed");
        let request = LoginRequest::new("admin", "admin123");
        let response = service.authenticate(request).await.expect("should succeed");
        assert!(response.success);

        let token = response.session_token.clone().expect("should succeed");
        let result = service.validate_session(&token).await;
        assert!(result.is_ok(), "Valid session should pass validation");

        let ctx = result.expect("should succeed");
        assert!(ctx.is_some(), "Should return an auth context");
        assert_eq!(ctx.expect("should succeed").username, "admin");
    }

    #[tokio::test]
    async fn test_validate_session_invalid() {
        let service = create_test_auth_service().await.expect("should succeed");
        let fake_uuid = Uuid::new_v4().to_string();
        let result = service.validate_session(&fake_uuid).await;
        assert!(result.is_ok());
        assert!(
            result.expect("should succeed").is_none(),
            "Invalid session should return None"
        );
    }

    #[tokio::test]
    async fn test_capability_discovery_fallback() {
        let service = create_test_auth_service().await.expect("should succeed");
        let provider = service.get_auth_provider();
        assert!(
            matches!(provider, AuthProvider::Standalone),
            "Without network, should fall back to standalone"
        );
    }

    // ====================================================================
    // SessionManager
    // ====================================================================

    #[tokio::test]
    async fn test_session_manager_create_and_get() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let session_id = session.id;

        manager
            .create_session(session)
            .await
            .expect("should succeed");

        let retrieved = manager
            .get_session(&session_id)
            .await
            .expect("should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("should succeed").user_id, user_id);
    }

    #[tokio::test]
    async fn test_session_manager_validate_nonexistent() {
        let manager = SessionManager::new();
        let fake_id = Uuid::new_v4();
        let result = manager.get_session(&fake_id).await.expect("should succeed");
        assert!(result.is_none(), "Nonexistent session should return None");
    }

    #[tokio::test]
    async fn test_session_manager_invalidate() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let session_id = session.id;

        manager
            .create_session(session)
            .await
            .expect("should succeed");
        let result = manager
            .invalidate_session(&session_id)
            .await
            .expect("should succeed");
        assert!(result, "Session invalidation should succeed");

        let retrieved = manager
            .get_session(&session_id)
            .await
            .expect("should succeed");
        assert!(
            retrieved.is_none(),
            "Invalidated session should not be returned"
        );
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        session.expires_at = Utc::now() - Duration::hours(1);
        let session_id = session.id;

        manager
            .create_session(session)
            .await
            .expect("should succeed");
        let retrieved = manager
            .get_session(&session_id)
            .await
            .expect("should succeed");
        assert!(
            retrieved.is_none(),
            "Expired session should not be returned"
        );
    }

    #[tokio::test]
    async fn test_session_touch() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let session_id = session.id;

        manager
            .create_session(session)
            .await
            .expect("should succeed");
        let result = manager
            .touch_session(&session_id)
            .await
            .expect("should succeed");
        assert!(result, "Session touch should succeed");
    }

    // ====================================================================
    // User and Permission
    // ====================================================================

    #[tokio::test]
    async fn test_user_has_role() {
        let mut user = User::new("roleuser", "role@test.local");
        user.roles = vec!["admin".to_string(), "moderator".to_string()];

        assert!(user.has_role("admin"));
        assert!(user.has_role("moderator"));
        assert!(!user.has_role("guest"));
    }

    #[tokio::test]
    async fn test_user_has_permission() {
        let mut user = User::new("permuser", "perm@test.local");
        user.permissions = vec![
            Permission::new("mcp", "read"),
            Permission::new("api", "write"),
        ];

        assert!(user.has_permission(&Permission::new("mcp", "read")));
        assert!(user.has_permission(&Permission::new("api", "write")));
        assert!(!user.has_permission(&Permission::new("admin", "delete")));
    }

    #[tokio::test]
    async fn test_empty_roles_and_permissions() {
        let user = User::new("emptyuser", "empty@test.local");
        assert!(!user.has_role("admin"));
        assert!(!user.has_permission(&Permission::new("mcp", "read")));
    }

    // ====================================================================
    // AuthProvider
    // ====================================================================

    #[tokio::test]
    async fn test_auth_provider_standalone() {
        let provider = AuthProvider::Standalone;
        assert!(matches!(provider, AuthProvider::Standalone));
    }

    #[tokio::test]
    async fn test_auth_provider_default_is_standalone() {
        let provider = AuthProvider::default();
        assert!(matches!(provider, AuthProvider::Standalone));
    }

    // ====================================================================
    // LoginRequest
    // ====================================================================

    #[tokio::test]
    async fn test_login_request_creation() {
        let request = LoginRequest::new("testuser", "testpass");
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
        assert!(request.additional_factors.is_none());
    }

    #[tokio::test]
    async fn test_login_request_with_factors() {
        let factors = serde_json::json!({"totp": "123456"});
        let request = LoginRequest::new("testuser", "testpass").with_factors(factors.clone());
        assert_eq!(request.additional_factors, Some(factors));
    }

    // ====================================================================
    // AuthError
    // ====================================================================

    #[tokio::test]
    async fn test_error_authentication_failed() {
        let error = AuthError::authentication_failed("Invalid password");
        assert!(matches!(error, AuthError::AuthenticationFailed { .. }));
        assert!(error.to_string().contains("Invalid password"));
    }

    #[tokio::test]
    async fn test_error_token_expired() {
        let error = AuthError::TokenExpired;
        assert_eq!(error.to_string(), "Token has expired");
    }

    #[tokio::test]
    async fn test_error_authorization() {
        let error = AuthError::authorization_error("Insufficient permissions");
        assert!(matches!(error, AuthError::Authorization { .. }));
    }

    // ====================================================================
    // Concurrent sessions
    // ====================================================================

    #[tokio::test]
    async fn test_concurrent_sessions() {
        let manager = SessionManager::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let s1 = Session::new(user1, Duration::hours(1), AuthProvider::Standalone);
        let s2 = Session::new(user2, Duration::hours(1), AuthProvider::Standalone);
        let id1 = s1.id;
        let id2 = s2.id;

        manager.create_session(s1).await.expect("should succeed");
        manager.create_session(s2).await.expect("should succeed");

        assert!(
            manager
                .get_session(&id1)
                .await
                .expect("should succeed")
                .is_some()
        );
        assert!(
            manager
                .get_session(&id2)
                .await
                .expect("should succeed")
                .is_some()
        );
        assert_ne!(id1, id2);
    }

    // ====================================================================
    // User serialization round-trip
    // ====================================================================

    #[tokio::test]
    async fn test_user_serialization() {
        let mut user = User::new("serialuser", "serial@example.com");
        user.roles = vec!["admin".to_string()];
        user.permissions = vec![
            Permission::new("mcp", "read"),
            Permission::new("api", "write"),
        ];

        let json = serde_json::to_string(&user).expect("should succeed");
        assert!(!json.is_empty());

        let deserialized: User = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.username, "serialuser");
        assert_eq!(deserialized.roles, vec!["admin"]);
        assert_eq!(deserialized.permissions.len(), 2);
    }
}
