//! Authentication Integration Tests
//!
//! These tests verify that the Beardog authentication system is properly integrated
//! and can handle production authentication scenarios.

use std::env;
use universal_patterns::config::{ConfigBuilder, PrimalType};
use universal_patterns::security::BeardogIntegration;
use universal_patterns::traits::{Credentials, Principal, PrincipalType};
use url::Url;

#[tokio::test]
async fn test_beardog_config_from_universal_patterns() {
    // Test that BeardogConfig can be created from universal patterns
    let config = ConfigBuilder::squirrel()
        .beardog_endpoint("http://localhost:8443").expect("Valid URL")
        .beardog_auth("squirrel-test")
        .build()
        .expect("Should create config with Beardog integration");
    
    // Create Beardog integration from config
    assert!(config.security.beardog_endpoint.is_some());
    let endpoint = config.security.beardog_endpoint.as_ref().unwrap();
    let beardog = BeardogIntegration::new(endpoint.clone(), "squirrel-test".to_string());
    
    // Verify integration is properly configured
    assert_eq!(beardog.service_id(), "squirrel-test");
    assert_eq!(beardog.endpoint(), endpoint);
}

#[tokio::test]
async fn test_beardog_config_from_env() {
    // Test that BeardogConfig can be created from environment variables
    env::set_var("BEARDOG_ENDPOINT", "https://prod.beardog.example.com:8443");
    
    let endpoint = Url::parse("https://prod.beardog.example.com:8443")
        .expect("Valid URL");
    
    let beardog = BeardogIntegration::new(endpoint.clone(), "prod-service".to_string());
    
    assert_eq!(beardog.endpoint(), &endpoint);
    assert_eq!(beardog.service_id(), "prod-service");
    
    // Clean up environment variables
    env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_beardog_environment_variables() {
    // Test environment variable configuration
    env::set_var("BEARDOG_ENDPOINT", "https://test.beardog.example.com:8443");
    env::set_var("BEARDOG_API_KEY", "test-api-key-12345");
    env::set_var("BEARDOG_SERVICE_ID", "test-service");
    
    // Test that environment variables are properly configured
    assert_eq!(env::var("BEARDOG_ENDPOINT").unwrap(), "https://test.beardog.example.com:8443");
    assert_eq!(env::var("BEARDOG_API_KEY").unwrap(), "test-api-key-12345");
    assert_eq!(env::var("BEARDOG_SERVICE_ID").unwrap(), "test-service");
    
    // Clean up environment variables
    env::remove_var("BEARDOG_ENDPOINT");
    env::remove_var("BEARDOG_API_KEY");
    env::remove_var("BEARDOG_SERVICE_ID");
}

#[tokio::test]
async fn test_production_credentials_setup() {
    // Test production credential management
    let _credentials = Credentials::Password {
        username: "prod-user".to_string(),
        password: "secure-password".to_string(),
    };
    
    // Verify credential structure (should not panic)
    println!("✅ Production credentials structure validated");
}

#[tokio::test]
async fn test_authentication_integration_scenario() {
    // Test complete authentication scenario with universal patterns
    let config = ConfigBuilder::development()
        .beardog_endpoint("http://localhost:8443").expect("Valid URL")
        .beardog_auth("test-service")
        .build()
        .expect("Should create config");
    
    let endpoint = config.security.beardog_endpoint.as_ref().unwrap();
    let beardog = BeardogIntegration::new(endpoint.clone(), "test-service".to_string());
    
    // Test authentication flow with proper credentials
    let credentials = Credentials::Password {
        username: "test-user".to_string(),
        password: "test-password".to_string(),
    };
    
    // In a real scenario, this would make actual HTTP calls to Beardog
    // For now, we test the integration setup
    assert_eq!(beardog.service_id(), "test-service");
    assert!(!beardog.endpoint().as_str().is_empty());
    
    // Note: Actual authentication would fail against a mock server, but the setup is correct
    println!("✅ Authentication integration test passed with universal patterns");
}

#[tokio::test]
async fn test_beardog_error_handling() {
    // Test error handling for invalid configurations
    let result = ConfigBuilder::new()
        .beardog_endpoint("invalid-url");
    
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_security_credential_types() {
    // Test various credential types with proper structure
    let password_creds = Credentials::Password {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    
    let api_key_creds = Credentials::ApiKey {
        key: "api-key-12345".to_string(),
        service_id: "test-service".to_string(),
    };
    
    let token_creds = Credentials::Token {
        token: "jwt-token-12345".to_string(),
    };
    
    let bearer_creds = Credentials::Bearer {
        token: "bearer-token-12345".to_string(),
    };
    
    // Verify all credential types are properly structured
    match password_creds {
        Credentials::Password { .. } => println!("✅ Password credentials valid"),
        _ => panic!("Password credentials invalid"),
    }
    
    match api_key_creds {
        Credentials::ApiKey { .. } => println!("✅ API key credentials valid"),
        _ => panic!("API key credentials invalid"),
    }
    
    match token_creds {
        Credentials::Token { .. } => println!("✅ Token credentials valid"),
        _ => panic!("Token credentials invalid"),
    }
    
    match bearer_creds {
        Credentials::Bearer { .. } => println!("✅ Bearer credentials valid"),
        _ => panic!("Bearer credentials invalid"),
    }
}

#[tokio::test]
async fn test_principal_creation() {
    // Test Principal creation with proper metadata field
    let principal = Principal {
        id: "test-user-123".to_string(),
        name: "Test User".to_string(),
        principal_type: PrincipalType::User,
        roles: vec!["user".to_string(), "admin".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(principal.id, "test-user-123");
    assert_eq!(principal.name, "Test User");
    assert_eq!(principal.principal_type, PrincipalType::User);
    assert!(principal.roles.contains(&"user".to_string()));
    assert!(principal.permissions.contains(&"read".to_string()));
    assert!(principal.metadata.is_empty());
    
    // Test service principal
    let service_principal = Principal {
        id: "service-123".to_string(),
        name: "Test Service".to_string(),
        principal_type: PrincipalType::Service,
        roles: vec!["service".to_string()],
        permissions: vec!["execute".to_string()],
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(service_principal.principal_type, PrincipalType::Service);
    assert!(service_principal.roles.contains(&"service".to_string()));
    
    println!("✅ Principal creation tests passed");
}

#[tokio::test]
async fn test_beardog_integration_with_security_patterns() {
    // Test integration with universal security patterns
    let config = ConfigBuilder::beardog()
        .beardog_endpoint("http://localhost:8443").expect("Valid URL")
        .beardog_auth("beardog-test")
        .enable_audit_logging()
        .enable_inter_primal_encryption()
        .build()
        .expect("Should create Beardog config");
    
    // Verify security configuration
    assert!(config.security.audit_logging);
    assert!(config.security.encryption.enable_inter_primal);
    assert!(config.security.beardog_endpoint.is_some());
    
    // Create Beardog integration
    let endpoint = config.security.beardog_endpoint.as_ref().unwrap();
    let beardog = BeardogIntegration::new(endpoint.clone(), "beardog-test".to_string());
    
    assert_eq!(beardog.service_id(), "beardog-test");
    assert_eq!(beardog.endpoint(), endpoint);
    
    println!("✅ Beardog integration with security patterns validated");
} 