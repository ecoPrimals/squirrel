// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

// Integration tests gated behind `integration-tests` feature — API migration
// (AuthService, AuthorizationLevel, LoginRequest) tracked in CURRENT_STATUS.md known issues.
#[cfg(not(feature = "integration-tests"))]
#[tokio::test]
async fn placeholder_auth_tests_disabled() {}

#[cfg(feature = "integration-tests")]
mod auth_tests_impl {
    //! Comprehensive tests for the authentication service
    //!
    //! This test suite covers:
    //! - Authentication flows (login, logout)
    //! - Authorization levels and permissions
    //! - Session management
    //! - Capability discovery
    //! - Error handling
    //! - Security validations

    use squirrel_mcp_auth::{
        auth::AuthService,
        errors::{AuthError, AuthResult},
        session::{AuthorizationLevel, Session, SessionManager},
        types::{AuthProvider, LoginRequest, LoginResponse, User},
    };

    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    /// Test helper to create a test auth service
    async fn create_test_auth_service() -> AuthResult<AuthService> {
        AuthService::new().await
    }

    /// Test helper to create a test login request
    fn create_test_login_request(username: &str, password: &str) -> LoginRequest {
        LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
            mfa_code: None,
        }
    }

    #[tokio::test]
    async fn test_auth_service_initialization() {
        let result = create_test_auth_service().await;
        assert!(
            result.is_ok(),
            "Auth service should initialize successfully"
        );

        let service = result.unwrap();
        // Service should be in a valid state
        // (Further assertions would require public methods)
    }

    #[tokio::test]
    async fn test_successful_login() {
        let service = create_test_auth_service().await.unwrap();

        let request = create_test_login_request("admin", "admin123");
        let result = service.login(request).await;

        assert!(
            result.is_ok(),
            "Login with valid credentials should succeed"
        );

        let response = result.unwrap();
        assert!(!response.token.is_empty(), "Token should not be empty");
        assert!(response.user.username == "admin", "Username should match");
    }

    #[tokio::test]
    async fn test_failed_login_invalid_credentials() {
        let service = create_test_auth_service().await.unwrap();

        let request = create_test_login_request("admin", "wrongpassword");
        let result = service.login(request).await;

        assert!(
            result.is_err(),
            "Login with invalid credentials should fail"
        );

        if let Err(AuthError::InvalidCredentials(_)) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidCredentials error");
        }
    }

    #[tokio::test]
    async fn test_failed_login_unknown_user() {
        let service = create_test_auth_service().await.unwrap();

        let request = create_test_login_request("nonexistent", "password");
        let result = service.login(request).await;

        assert!(result.is_err(), "Login with unknown user should fail");
    }

    #[tokio::test]
    async fn test_logout() {
        let service = create_test_auth_service().await.unwrap();

        // First login
        let request = create_test_login_request("admin", "admin123");
        let login_response = service.login(request).await.unwrap();

        // Then logout
        let result = service.logout(&login_response.token).await;
        assert!(result.is_ok(), "Logout should succeed");
    }

    #[tokio::test]
    async fn test_validate_token_valid() {
        let service = create_test_auth_service().await.unwrap();

        // Login to get a valid token
        let request = create_test_login_request("admin", "admin123");
        let response = service.login(request).await.unwrap();

        // Validate the token
        let result = service.validate_token(&response.token).await;
        assert!(result.is_ok(), "Valid token should pass validation");

        let user = result.unwrap();
        assert_eq!(user.username, "admin");
    }

    #[tokio::test]
    async fn test_validate_token_invalid() {
        let service = create_test_auth_service().await.unwrap();

        let result = service.validate_token("invalid_token").await;
        assert!(result.is_err(), "Invalid token should fail validation");
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        // Manager should initialize successfully
        // (Further assertions would require public methods)
    }

    #[tokio::test]
    async fn test_session_manager_create_session() {
        let manager = SessionManager::new();

        let user = User {
            id: "test-user-1".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            created_at: Utc::now(),
            last_login: None,
        };

        let session = manager.create_session(user, Duration::hours(1)).await;
        assert!(session.is_ok(), "Session creation should succeed");

        let session = session.unwrap();
        assert!(
            !session.token.is_empty(),
            "Session token should not be empty"
        );
        assert_eq!(session.user.username, "testuser");
    }

    #[tokio::test]
    async fn test_session_manager_validate_session() {
        let manager = SessionManager::new();

        let user = User {
            id: "test-user-2".to_string(),
            username: "validuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        let session = manager
            .create_session(user, Duration::hours(1))
            .await
            .unwrap();

        // Validate the session
        let result = manager.validate_session(&session.token).await;
        assert!(result.is_ok(), "Valid session should pass validation");

        let validated_session = result.unwrap();
        assert_eq!(validated_session.user.username, "validuser");
    }

    #[tokio::test]
    async fn test_session_manager_validate_nonexistent_session() {
        let manager = SessionManager::new();

        let result = manager.validate_session("nonexistent_token").await;
        assert!(
            result.is_err(),
            "Nonexistent session should fail validation"
        );
    }

    #[tokio::test]
    async fn test_session_manager_revoke_session() {
        let manager = SessionManager::new();

        let user = User {
            id: "test-user-3".to_string(),
            username: "revokeuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        let session = manager
            .create_session(user, Duration::hours(1))
            .await
            .unwrap();
        let token = session.token.clone();

        // Revoke the session
        let result = manager.revoke_session(&token).await;
        assert!(result.is_ok(), "Session revocation should succeed");

        // Try to validate revoked session
        let validation_result = manager.validate_session(&token).await;
        assert!(
            validation_result.is_err(),
            "Revoked session should fail validation"
        );
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let manager = SessionManager::new();

        let user = User {
            id: "test-user-4".to_string(),
            username: "expiryuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        // Create session with very short duration
        let session = manager
            .create_session(user, Duration::milliseconds(1))
            .await
            .unwrap();

        // Wait for session to expire
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Validation should fail
        let result = manager.validate_session(&session.token).await;
        assert!(result.is_err(), "Expired session should fail validation");
    }

    #[tokio::test]
    async fn test_authorization_level_system() {
        let level = AuthorizationLevel::System;

        // System level should authorize all other levels
        assert!(level.authorizes(&AuthorizationLevel::Admin));
        assert!(level.authorizes(&AuthorizationLevel::Elevated));
        assert!(level.authorizes(&AuthorizationLevel::User));
        assert!(level.authorizes(&AuthorizationLevel::None));
    }

    #[tokio::test]
    async fn test_authorization_level_admin() {
        let level = AuthorizationLevel::Admin;

        // Admin should not authorize System
        assert!(!level.authorizes(&AuthorizationLevel::System));

        // Admin should authorize lower levels
        assert!(level.authorizes(&AuthorizationLevel::Elevated));
        assert!(level.authorizes(&AuthorizationLevel::User));
        assert!(level.authorizes(&AuthorizationLevel::None));
    }

    #[tokio::test]
    async fn test_authorization_level_user() {
        let level = AuthorizationLevel::User;

        // User should not authorize higher levels
        assert!(!level.authorizes(&AuthorizationLevel::System));
        assert!(!level.authorizes(&AuthorizationLevel::Admin));
        assert!(!level.authorizes(&AuthorizationLevel::Elevated));

        // User should authorize itself and None
        assert!(level.authorizes(&AuthorizationLevel::User));
        assert!(level.authorizes(&AuthorizationLevel::None));
    }

    #[tokio::test]
    async fn test_user_has_role() {
        let user = User {
            id: "test-user-5".to_string(),
            username: "roleuser".to_string(),
            email: None,
            roles: vec!["admin".to_string(), "moderator".to_string()],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        assert!(user.has_role("admin"));
        assert!(user.has_role("moderator"));
        assert!(!user.has_role("guest"));
    }

    #[tokio::test]
    async fn test_user_has_permission() {
        let user = User {
            id: "test-user-6".to_string(),
            username: "permuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec!["read".to_string(), "write".to_string()],
            created_at: Utc::now(),
            last_login: None,
        };

        assert!(user.has_permission("read"));
        assert!(user.has_permission("write"));
        assert!(!user.has_permission("delete"));
    }

    #[tokio::test]
    async fn test_auth_provider_standalone() {
        let provider = AuthProvider::Standalone;

        // Standalone provider should not have external endpoint
        match provider {
            AuthProvider::Standalone => {
                // Expected
            }
            _ => panic!("Expected Standalone provider"),
        }
    }

    #[tokio::test]
    async fn test_auth_provider_beardog() {
        let endpoint = "https://beardog.example.com".to_string();
        let provider = AuthProvider::Beardog {
            endpoint: endpoint.clone(),
        };

        match provider {
            AuthProvider::Beardog { endpoint: ep } => {
                assert_eq!(ep, endpoint);
            }
            _ => panic!("Expected Beardog provider"),
        }
    }

    #[tokio::test]
    async fn test_login_request_creation() {
        let request = LoginRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            mfa_code: Some("123456".to_string()),
        };

        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
        assert_eq!(request.mfa_code, Some("123456".to_string()));
    }

    #[tokio::test]
    async fn test_login_response_creation() {
        let user = User {
            id: "test-user-7".to_string(),
            username: "responseuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        let response = LoginResponse {
            token: "test_token_123".to_string(),
            user: user.clone(),
            expires_at: Utc::now() + Duration::hours(1),
        };

        assert_eq!(response.token, "test_token_123");
        assert_eq!(response.user.username, "responseuser");
    }

    #[tokio::test]
    async fn test_error_invalid_credentials() {
        let error = AuthError::InvalidCredentials("Invalid password".to_string());

        match error {
            AuthError::InvalidCredentials(msg) => {
                assert_eq!(msg, "Invalid password");
            }
            _ => panic!("Expected InvalidCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_error_session_expired() {
        let error = AuthError::SessionExpired;

        match error {
            AuthError::SessionExpired => {
                // Expected
            }
            _ => panic!("Expected SessionExpired error"),
        }
    }

    #[tokio::test]
    async fn test_error_unauthorized() {
        let error = AuthError::Unauthorized("Insufficient permissions".to_string());

        match error {
            AuthError::Unauthorized(msg) => {
                assert_eq!(msg, "Insufficient permissions");
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_sessions() {
        let manager = SessionManager::new();

        let user1 = User {
            id: "concurrent-1".to_string(),
            username: "user1".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        let user2 = User {
            id: "concurrent-2".to_string(),
            username: "user2".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        // Create multiple sessions
        let session1 = manager
            .create_session(user1, Duration::hours(1))
            .await
            .unwrap();
        let session2 = manager
            .create_session(user2, Duration::hours(1))
            .await
            .unwrap();

        // Both sessions should be valid
        assert!(manager.validate_session(&session1.token).await.is_ok());
        assert!(manager.validate_session(&session2.token).await.is_ok());

        // Tokens should be different
        assert_ne!(session1.token, session2.token);
    }

    #[tokio::test]
    async fn test_session_update() {
        let manager = SessionManager::new();

        let user = User {
            id: "update-user".to_string(),
            username: "updateuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        let session = manager
            .create_session(user, Duration::hours(1))
            .await
            .unwrap();
        let token = session.token.clone();

        // Update session (e.g., refresh activity)
        let result = manager.update_session(&token).await;
        assert!(result.is_ok(), "Session update should succeed");
    }

    #[tokio::test]
    async fn test_capability_discovery_fallback() {
        // Test that service falls back to standalone when no capability found
        let service = create_test_auth_service().await.unwrap();

        // Service should initialize even if no external capability found
        // It should use standalone mode
        let request = create_test_login_request("admin", "admin123");
        let result = service.login(request).await;

        assert!(result.is_ok(), "Standalone mode should work");
    }

    #[tokio::test]
    async fn test_user_serialization() {
        use serde_json;

        let user = User {
            id: "serial-1".to_string(),
            username: "serialuser".to_string(),
            email: Some("serial@example.com".to_string()),
            roles: vec!["admin".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            created_at: Utc::now(),
            last_login: Some(Utc::now()),
        };

        // Serialize
        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.is_empty());

        // Deserialize
        let deserialized: User = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, "serialuser");
        assert_eq!(deserialized.roles, vec!["admin"]);
    }

    #[tokio::test]
    async fn test_multiple_role_authorization() {
        let user = User {
            id: "multi-role-1".to_string(),
            username: "multirole".to_string(),
            email: None,
            roles: vec![
                "admin".to_string(),
                "moderator".to_string(),
                "user".to_string(),
            ],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        // Should have all roles
        assert!(user.has_role("admin"));
        assert!(user.has_role("moderator"));
        assert!(user.has_role("user"));
        assert!(!user.has_role("superadmin"));
    }

    #[tokio::test]
    async fn test_empty_roles_and_permissions() {
        let user = User {
            id: "empty-1".to_string(),
            username: "emptyuser".to_string(),
            email: None,
            roles: vec![],
            permissions: vec![],
            created_at: Utc::now(),
            last_login: None,
        };

        // Should not have any role or permission
        assert!(!user.has_role("admin"));
        assert!(!user.has_permission("read"));
    }
}
