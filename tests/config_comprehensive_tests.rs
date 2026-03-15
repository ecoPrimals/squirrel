// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Additional configuration tests for comprehensive coverage

use squirrel_mcp_config::*;
use std::time::Duration;

#[tokio::test]
async fn test_config_defaults_are_sensible() {
    let config = SquirrelConfig::default();
    
    assert!(config.server.host == "0.0.0.0" || config.server.host == "127.0.0.1");
    assert!(config.server.port > 1024); // Non-privileged port
    assert!(config.timeouts.connection > Duration::from_secs(0));
    assert!(config.timeouts.request > Duration::from_secs(0));
}

#[tokio::test]
async fn test_config_validation_bounds() {
    // Test port bounds
    let mut config = SquirrelConfig::default();
    config.server.port = 0;
    assert!(config.validate().is_err());
    
    config.server.port = 65536;
    assert!(config.validate().is_err());
    
    config.server.port = 8080;
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_config_timeout_validation() {
    let mut config = SquirrelConfig::default();
    
    // Connection timeout should be positive
    config.timeouts.connection = Duration::from_secs(0);
    assert!(config.validate().is_err());
    
    config.timeouts.connection = Duration::from_secs(30);
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_config_serialization_roundtrip() {
    let original = SquirrelConfig::default();
    
    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();
    
    // Deserialize back
    let deserialized: SquirrelConfig = serde_json::from_str(&json).unwrap();
    
    // Verify equality
    assert_eq!(original.server.host, deserialized.server.host);
    assert_eq!(original.server.port, deserialized.server.port);
}

#[tokio::test]
async fn test_config_from_environment() {
    std::env::set_var("SQUIRREL_HOST", "192.168.1.1");
    std::env::set_var("SQUIRREL_PORT", "9090");
    
    let config = SquirrelConfig::from_env().unwrap();
    
    assert_eq!(config.server.host, "192.168.1.1");
    assert_eq!(config.server.port, 9090);
    
    // Cleanup
    std::env::remove_var("SQUIRREL_HOST");
    std::env::remove_var("SQUIRREL_PORT");
}

#[tokio::test]
async fn test_config_merge_priority() {
    let mut base = SquirrelConfig::default();
    base.server.port = 8080;
    
    let override_config = SquirrelConfig {
        server: ServerConfig {
            host: "10.0.0.1".to_string(),
            port: 9090,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let merged = base.merge(override_config);
    
    assert_eq!(merged.server.host, "10.0.0.1"); // From override
    assert_eq!(merged.server.port, 9090); // From override
}

#[tokio::test]
async fn test_config_partial_updates() {
    let mut config = SquirrelConfig::default();
    let original_port = config.server.port;
    
    config.server.host = "custom.example.com".to_string();
    
    assert_eq!(config.server.host, "custom.example.com");
    assert_eq!(config.server.port, original_port); // Unchanged
}

#[test]
fn test_config_builder_pattern() {
    let config = SquirrelConfig::builder()
        .host("test.local")
        .port(7070)
        .connection_timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    
    assert_eq!(config.server.host, "test.local");
    assert_eq!(config.server.port, 7070);
    assert_eq!(config.timeouts.connection, Duration::from_secs(15));
}

#[test]
fn test_config_security_settings() {
    let mut config = SquirrelConfig::default();
    
    config.security.tls_enabled = true;
    config.security.require_client_certs = true;
    
    assert!(config.security.tls_enabled);
    assert!(config.security.require_client_certs);
}

#[test]
fn test_config_logging_levels() {
    let config = SquirrelConfig::default();
    
    let valid_levels = vec!["trace", "debug", "info", "warn", "error"];
    assert!(valid_levels.contains(&config.logging.level.as_str()));
}

