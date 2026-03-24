// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::AuthService;
use crate::errors::AuthError;
use crate::session::{Session, SessionManager};
use crate::types::Permission;
use crate::types::{AuthProvider, LoginRequest, LoginResponse, SecurityCapabilityInfo, User};
use chrono::Duration;
use std::collections::HashMap;

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
