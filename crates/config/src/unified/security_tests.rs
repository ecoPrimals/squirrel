// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for security configuration types

use super::*;

#[test]
fn test_auth_mode_variants() {
    let modes = vec![
        AuthMode::None,
        AuthMode::ApiKey,
        AuthMode::Jwt,
        AuthMode::MTls,
        AuthMode::Combined,
    ];
    
    for mode in modes {
        let cloned = mode.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_auth_mode_partial_eq() {
    assert_eq!(AuthMode::ApiKey, AuthMode::ApiKey);
    assert_ne!(AuthMode::ApiKey, AuthMode::Jwt);
}

#[test]
fn test_auth_mode_serialization() {
    let mode = AuthMode::ApiKey;
    let serialized = serde_json::to_string(&mode).expect("Failed to serialize");
    assert_eq!(serialized, "\"apikey\"");
}

#[test]
fn test_auth_mode_deserialization() {
    let json = "\"jwt\"";
    let mode: AuthMode = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(mode, AuthMode::Jwt);
}

#[test]
fn test_auth_mode_all_variants() {
    let variants = vec![
        (AuthMode::None, "none"),
        (AuthMode::ApiKey, "apikey"),
        (AuthMode::Jwt, "jwt"),
        (AuthMode::MTls, "mtls"),
        (AuthMode::Combined, "combined"),
    ];
    
    for (mode, expected) in variants {
        let serialized = serde_json::to_string(&mode).expect("test: should succeed");
        assert!(serialized.contains(expected));
    }
}

#[test]
fn test_jwt_secret_new_valid() {
    let secret = JwtSecret::new("my-very-long-secret-key-that-is-secure").expect("Failed");
    let debug_str = format!("{:?}", secret);
    assert!(debug_str.contains("REDACTED"));
}

#[test]
fn test_jwt_secret_new_empty() {
    let result = JwtSecret::new("");
    assert!(result.is_err());
}

#[test]
fn test_jwt_secret_new_too_short() {
    let result = JwtSecret::new("short");
    assert!(result.is_err());
}

#[test]
fn test_jwt_secret_get() {
    let secret_str = "my-very-long-secret-key-that-is-secure";
    let secret = JwtSecret::new(secret_str).expect("test: should succeed");
    assert_eq!(secret.get(), secret_str);
}

#[test]
fn test_jwt_secret_clone() {
    let secret = JwtSecret::new("my-very-long-secret-key-that-is-secure").expect("test: should succeed");
    let cloned = secret.clone();
    assert_eq!(cloned.get(), secret.get());
}

#[test]
fn test_jwt_secret_debug() {
    let secret = JwtSecret::new("my-very-long-secret-key-that-is-secure").expect("test: should succeed");
    let debug_str = format!("{:?}", secret);
    assert!(debug_str.contains("REDACTED"));
    assert!(!debug_str.contains("secret-key"));
}

#[test]
fn test_jwt_secret_error_empty() {
    let err = JwtSecretError::Empty;
    let msg = format!("{}", err);
    assert!(msg.contains("empty"));
}

#[test]
fn test_jwt_secret_error_too_short() {
    let err = JwtSecretError::TooShort { length: 5, min_length: 32 };
    let msg = format!("{}", err);
    assert!(msg.contains("5"));
    assert!(msg.contains("32"));
}

#[test]
fn test_api_key_new_valid() {
    let key = ApiKey::new("valid-api-key-123456").expect("Failed");
    let debug_str = format!("{:?}", key);
    assert!(debug_str.contains("REDACTED"));
}

#[test]
fn test_api_key_new_empty() {
    let result = ApiKey::new("");
    assert!(result.is_err());
}

#[test]
fn test_api_key_new_too_short() {
    let result = ApiKey::new("short");
    assert!(result.is_err());
}

#[test]
fn test_api_key_get() {
    let key_str = "valid-api-key-123456";
    let key = ApiKey::new(key_str).expect("test: should succeed");
    assert_eq!(key.get(), key_str);
}

#[test]
fn test_api_key_clone() {
    let key = ApiKey::new("valid-api-key-123456").expect("test: should succeed");
    let cloned = key.clone();
    assert_eq!(cloned.get(), key.get());
}

#[test]
fn test_api_key_debug() {
    let key = ApiKey::new("valid-api-key-123456").expect("test: should succeed");
    let debug_str = format!("{:?}", key);
    assert!(debug_str.contains("REDACTED"));
    assert!(!debug_str.contains("api-key"));
}

#[test]
fn test_api_key_error_empty() {
    let err = ApiKeyError::Empty;
    let msg = format!("{}", err);
    assert!(msg.contains("empty"));
}

#[test]
fn test_api_key_error_too_short() {
    let err = ApiKeyError::TooShort { length: 3, min_length: 16 };
    let msg = format!("{}", err);
    assert!(msg.contains("3"));
    assert!(msg.contains("16"));
}

#[test]
fn test_security_config_default() {
    let config = SecurityConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SecurityConfig"));
}

#[test]
fn test_security_config_clone() {
    let config = SecurityConfig::default();
    let cloned = config.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_security_config_serialization() {
    let config = SecurityConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let _deserialized: SecurityConfig = serde_json::from_str(&serialized).expect("Failed to deserialize");
}

#[test]
fn test_auth_mode_copy() {
    let mode = AuthMode::Jwt;
    let copied = mode;
    assert_eq!(mode, copied);
}

#[test]
fn test_jwt_secret_min_length() {
    let secret = "a".repeat(32);
    let result = JwtSecret::new(&secret);
    assert!(result.is_ok());
}

#[test]
fn test_api_key_min_length() {
    let key = "a".repeat(16);
    let result = ApiKey::new(&key);
    assert!(result.is_ok());
}

