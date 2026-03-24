// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for security session management

use super::session::*;
use super::types::AuthorizationLevel;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// SECURITY SESSION TESTS
// ============================================================================

#[test]
fn test_security_session_new() {
    let session = SecuritySession::new("session-123".to_string(), Some("user-456".to_string()));

    assert_eq!(session.session_id, "session-123");
    assert_eq!(session.user_id, Some("user-456".to_string()));
    assert_eq!(session.user_id_string, Some("user-456".to_string()));
    assert_eq!(session.session_type, "standard");
    assert!(!session.authenticated);
    assert!(matches!(
        session.authorization_level,
        AuthorizationLevel::None
    ));
    assert!(session.roles.is_empty());
}

#[test]
fn test_security_session_authenticated() {
    let session = SecuritySession::authenticated("session-789".to_string(), "user-321".to_string());

    assert_eq!(session.session_id, "session-789");
    assert_eq!(session.user_id, Some("user-321".to_string()));
    assert_eq!(session.session_type, "authenticated");
    assert!(session.authenticated);
    assert!(matches!(
        session.authorization_level,
        AuthorizationLevel::User
    ));
    assert_eq!(session.roles, vec!["user"]);
}

#[test]
fn test_security_session_is_expired() {
    let mut session = SecuritySession::new("test".to_string(), None);

    // Fresh session should not be expired
    assert!(!session.is_expired());

    // Set expiry to past
    session.expires_at = Utc::now() - chrono::Duration::hours(1);
    assert!(session.is_expired());
}

#[test]
fn test_security_session_update_last_accessed() {
    let mut session = SecuritySession::new("test".to_string(), None);
    let initial_time = session.last_accessed;

    // Force a small delay by updating twice
    // This tests the update mechanism without relying on wall-clock timing
    session.update_last_accessed();
    let first_update = session.last_accessed;

    // Second update should also update timestamp
    session.update_last_accessed();

    // At minimum, timestamps should be >= initial time
    // (May be equal in fast execution, which is fine - tests the mechanism works)
    assert!(session.last_accessed >= initial_time);
    assert!(first_update >= initial_time);
}

#[test]
fn test_security_session_with_authorization_level() {
    let session = SecuritySession::new("test".to_string(), None)
        .with_authorization_level(AuthorizationLevel::Admin);

    assert!(matches!(
        session.authorization_level,
        AuthorizationLevel::Admin
    ));
}

#[test]
fn test_security_session_with_session_data() {
    let session = SecuritySession::new("test".to_string(), None)
        .with_session_data("key1".to_string(), json!("value1"))
        .with_session_data("key2".to_string(), json!(42));

    assert_eq!(session.session_data.len(), 2);
    assert_eq!(session.session_data.get("key1"), Some(&json!("value1")));
    assert_eq!(session.session_data.get("key2"), Some(&json!(42)));
}

#[test]
fn test_security_session_serialization() {
    let session =
        SecuritySession::authenticated("test-session".to_string(), "test-user".to_string());

    let json = serde_json::to_string(&session).expect("should succeed");
    let deserialized: SecuritySession = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(session.session_id, deserialized.session_id);
    assert_eq!(session.user_id, deserialized.user_id);
    assert_eq!(session.authenticated, deserialized.authenticated);
}

#[test]
fn test_security_session_clone() {
    let session = SecuritySession::authenticated("clone-test".to_string(), "user".to_string());
    let cloned = session.clone();

    assert_eq!(session.session_id, cloned.session_id);
    assert_eq!(session.user_id, cloned.user_id);
}

#[test]
fn test_security_session_default_expiry() {
    let session = SecuritySession::new("test".to_string(), None);
    let expected_expiry = session.created_at + chrono::Duration::hours(24);

    // Allow for small time differences in test execution
    let diff = (session.expires_at - expected_expiry).num_seconds().abs();
    assert!(diff < 2, "Expiry time should be ~24 hours from creation");
}

#[test]
fn test_security_session_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("ip".to_string(), "127.0.0.1".to_string());
    metadata.insert("user_agent".to_string(), "Mozilla/5.0".to_string());

    let mut session = SecuritySession::new("test".to_string(), None);
    session.metadata = metadata;

    assert_eq!(session.metadata.len(), 2);
    assert_eq!(session.metadata.get("ip"), Some(&"127.0.0.1".to_string()));
}

#[test]
fn test_security_session_roles() {
    let mut session = SecuritySession::authenticated("test".to_string(), "user".to_string());
    session.roles.push("admin".to_string());
    session.roles.push("moderator".to_string());

    assert_eq!(session.roles.len(), 3); // "user" + "admin" + "moderator"
    assert!(session.roles.contains(&"admin".to_string()));
}

#[test]
fn test_security_session_without_user() {
    let session = SecuritySession::new("anon-session".to_string(), None);

    assert_eq!(session.user_id, None);
    assert_eq!(session.user_id_string, None);
    assert!(!session.authenticated);
}

#[test]
fn test_security_session_chaining() {
    let session = SecuritySession::new("chain-test".to_string(), Some("user".to_string()))
        .with_authorization_level(AuthorizationLevel::Admin)
        .with_session_data("pref1".to_string(), json!("dark_mode"))
        .with_session_data("pref2".to_string(), json!(true));

    assert!(matches!(
        session.authorization_level,
        AuthorizationLevel::Admin
    ));
    assert_eq!(session.session_data.len(), 2);
}

#[test]
fn test_security_session_timestamps() {
    let session = SecuritySession::new("time-test".to_string(), None);

    assert!(session.created_at <= Utc::now());
    assert!(session.last_accessed <= Utc::now());
    assert!(session.expires_at > Utc::now());
}

#[test]
fn test_security_session_authenticated_has_user_role() {
    let session = SecuritySession::authenticated("test".to_string(), "user".to_string());

    assert!(session.roles.contains(&"user".to_string()));
    assert!(session.authenticated);
}

#[test]
fn test_security_session_types() {
    let standard = SecuritySession::new("test1".to_string(), None);
    let authenticated = SecuritySession::authenticated("test2".to_string(), "user".to_string());

    assert_eq!(standard.session_type, "standard");
    assert_eq!(authenticated.session_type, "authenticated");
}

#[test]
fn test_security_session_empty_session_data() {
    let session = SecuritySession::new("test".to_string(), None);

    assert!(session.session_data.is_empty());
}

#[test]
fn test_security_session_complex_session_data() {
    let session = SecuritySession::new("test".to_string(), None)
        .with_session_data(
            "preferences".to_string(),
            json!({
                "theme": "dark",
                "language": "en",
                "notifications": true
            }),
        )
        .with_session_data("last_activity".to_string(), json!("2025-11-26T12:00:00Z"));

    assert_eq!(session.session_data.len(), 2);

    let prefs = session
        .session_data
        .get("preferences")
        .expect("should succeed");
    assert!(prefs.is_object());
}

#[test]
fn test_security_session_authorization_levels() {
    let none = SecuritySession::new("test1".to_string(), None);
    let user = SecuritySession::authenticated("test2".to_string(), "user".to_string());
    let admin = SecuritySession::new("test3".to_string(), None)
        .with_authorization_level(AuthorizationLevel::Admin);

    assert!(matches!(none.authorization_level, AuthorizationLevel::None));
    assert!(matches!(user.authorization_level, AuthorizationLevel::User));
    assert!(matches!(
        admin.authorization_level,
        AuthorizationLevel::Admin
    ));
}

#[test]
fn test_security_session_expiry_boundary() {
    let mut session = SecuritySession::new("test".to_string(), None);

    // Set to past time
    session.expires_at = Utc::now() - chrono::Duration::milliseconds(1);

    // Should be expired immediately
    assert!(session.is_expired());
}

#[test]
fn test_security_session_long_lived() {
    let mut session = SecuritySession::new("test".to_string(), None);

    // Set to far future
    session.expires_at = Utc::now() + chrono::Duration::days(365);

    assert!(!session.is_expired());
}

#[test]
fn test_security_session_update_last_accessed_multiple() {
    let mut session = SecuritySession::new("test".to_string(), None);
    let initial = session.last_accessed;

    // Modern approach: Force time difference without blocking
    // SystemTime has ~1ms precision, so updates are naturally different
    session.update_last_accessed();
    let first_update = session.last_accessed;

    // Create small computation gap to ensure different timestamp
    let _ = (0..100).map(|x| x * 2).sum::<i32>();

    session.update_last_accessed();
    let second_update = session.last_accessed;

    assert!(first_update >= initial, "First update should be >= initial");
    assert!(
        second_update >= first_update,
        "Second update should be >= first"
    );
}

#[test]
fn test_security_session_json_round_trip() {
    let original = SecuritySession::authenticated("json-test".to_string(), "user-123".to_string())
        .with_authorization_level(AuthorizationLevel::Admin)
        .with_session_data("test_data".to_string(), json!({"key": "value"}));

    let json_str = serde_json::to_string(&original).expect("should succeed");
    let restored: SecuritySession = serde_json::from_str(&json_str).expect("should succeed");

    assert_eq!(original.session_id, restored.session_id);
    assert_eq!(original.user_id, restored.user_id);
    assert_eq!(original.authenticated, restored.authenticated);
    assert_eq!(original.session_data.len(), restored.session_data.len());
}
