/*!
 * Security module tests.
 * 
 * This file contains tests for the security module of the Galaxy adapter.
 */

use crate::security::{
    SecretString, 
    SecureCredentials, 
    CredentialStorage,
    MemoryStorage,
    FileStorage,
    SecurityManager,
    RotationPolicy,
    helpers,
};
use std::sync::Arc;
use tempfile::tempdir;
use std::env;

#[test]
fn test_secret_string_basic() {
    // Test creation
    let secret = SecretString::new("test-secret");
    assert!(!secret.is_empty());
    assert_eq!(secret.len(), 11);
    
    // Test debug output (should be redacted)
    let debug_str = format!("{:?}", secret);
    assert_eq!(debug_str, "[REDACTED]");
    
    // Test display output (should be redacted)
    let display_str = format!("{}", secret);
    assert_eq!(display_str, "[REDACTED]");
    
    // Test expose method
    assert_eq!(secret.expose(), "test-secret");
    
    // Test equality
    let secret2 = SecretString::new("test-secret");
    assert_eq!(secret, secret2);
    
    let secret3 = SecretString::new("other-secret");
    assert_ne!(secret, secret3);
}

#[test]
fn test_secret_string_empty() {
    let empty = SecretString::empty();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_secure_credentials_with_api_key() {
    let api_key = SecretString::new("test-api-key");
    let creds = SecureCredentials::with_api_key(api_key);
    
    // Verify API key is stored correctly
    assert!(!creds.is_empty());
    assert!(!creds.is_expired());
    assert!(creds.api_key().is_some());
    assert_eq!(creds.api_key().unwrap().expose(), "test-api-key");
    assert!(creds.email().is_none());
    assert!(creds.password().is_none());
    
    // Check debug output
    let debug_str = format!("{:?}", creds);
    assert!(!debug_str.contains("test-api-key"));
}

#[test]
fn test_secure_credentials_with_email_password() {
    let email = "test@example.com";
    let password = SecretString::new("test-password");
    let creds = SecureCredentials::with_email_password(email.to_string(), password);
    
    // Verify email/password are stored correctly
    assert!(!creds.is_empty());
    assert!(!creds.is_expired());
    assert!(creds.api_key().is_none());
    assert!(creds.email().is_some());
    assert_eq!(creds.email().unwrap(), email);
    assert!(creds.password().is_some());
    assert_eq!(creds.password().unwrap().expose(), "test-password");
    
    // Check debug output
    let debug_str = format!("{:?}", creds);
    assert!(!debug_str.contains("test-password"));
}

#[test]
fn test_secure_credentials_expiration() {
    let api_key = SecretString::new("test-api-key");
    let mut creds = SecureCredentials::with_api_key(api_key);
    
    // Not expired by default
    assert!(!creds.is_expired());
    
    // Set expiration in the past
    let past = time::OffsetDateTime::now_utc() - time::Duration::days(1);
    creds = creds.with_expiration(past);
    assert!(creds.is_expired());
    
    // Set expiration in the future
    let future = time::OffsetDateTime::now_utc() + time::Duration::days(1);
    creds = creds.with_expiration(future);
    assert!(!creds.is_expired());
}

#[tokio::test]
async fn test_memory_storage() {
    let storage = MemoryStorage::new();
    let creds = SecureCredentials::with_api_key(SecretString::new("test-key"));
    
    // Store credentials
    storage.store("test-id", creds.clone()).await.unwrap();
    
    // Retrieve credentials
    let retrieved = storage.get("test-id").await.unwrap();
    assert_eq!(
        retrieved.api_key().unwrap().expose(), 
        creds.api_key().unwrap().expose()
    );
    
    // List credentials
    let ids = storage.list().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "test-id");
    
    // Delete credentials
    storage.delete("test-id").await.unwrap();
    let result = storage.get("test-id").await;
    assert!(result.is_err());
    
    // Clear storage
    storage.store("id1", creds.clone()).await.unwrap();
    storage.store("id2", creds.clone()).await.unwrap();
    storage.clear().await.unwrap();
    assert_eq!(storage.list().await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_file_storage() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path().to_str().unwrap().to_string();
    let storage = FileStorage::with_random_key(base_path);
    let creds = SecureCredentials::with_api_key(SecretString::new("test-key"));
    
    // Store credentials
    storage.store("test-id", creds.clone()).await.unwrap();
    
    // Retrieve credentials
    let retrieved = storage.get("test-id").await.unwrap();
    assert_eq!(
        retrieved.api_key().unwrap().expose(), 
        creds.api_key().unwrap().expose()
    );
    
    // List credentials
    let ids = storage.list().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"test-id".to_string()));
    
    // Delete credentials
    storage.delete("test-id").await.unwrap();
    let result = storage.get("test-id").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_security_manager() {
    let storage = Arc::new(MemoryStorage::new());
    let manager = SecurityManager::with_storage(storage.clone())
        .allow_environment_variables(true)
        .with_rotation_policy(RotationPolicy::default());
    
    let creds = SecureCredentials::with_api_key(SecretString::new("test-key"));
    
    // Store credentials
    manager.store_credentials("test-id", creds.clone()).await.unwrap();
    
    // Get credentials
    let retrieved = manager.get_credentials("test-id").await.unwrap();
    assert_eq!(
        retrieved.api_key().unwrap().expose(), 
        creds.api_key().unwrap().expose()
    );
    
    // Test validation
    assert!(manager.validate_credentials(&creds).await.unwrap());
    
    // Test empty credentials validation
    let empty = SecureCredentials::empty();
    assert!(manager.validate_credentials(&empty).await.is_err());
    
    // Test credential rotation
    let new_creds = SecureCredentials::with_api_key(SecretString::new("new-key"));
    manager.rotate_credentials("test-id", new_creds.clone()).await.unwrap();
    
    let rotated = manager.get_credentials("test-id").await.unwrap();
    assert_eq!(
        rotated.api_key().unwrap().expose(), 
        new_creds.api_key().unwrap().expose()
    );
}

#[test]
fn test_environment_variables() {
    // Set environment variables for testing
    env::set_var("TEST_API_KEY", "test-key-from-env");
    
    // Test direct from_env method
    let secret = SecretString::from_env("TEST_API_KEY").unwrap();
    assert_eq!(secret.expose(), "test-key-from-env");
    
    // Test helper function
    let secret2 = helpers::secure_string_from_env("TEST_API_KEY").unwrap();
    assert_eq!(secret2.expose(), "test-key-from-env");
    
    // Test missing environment variable
    let result = SecretString::from_env("NONEXISTENT_VAR");
    assert!(result.is_err());
    
    // Clean up
    env::remove_var("TEST_API_KEY");
}

#[test]
fn test_credentials_from_config() {
    // Test with API key
    let config = crate::config::GalaxyConfig::new("https://example.com/api")
        .with_api_key("test-api-key");
    
    let creds = crate::security::credentials::credentials_from_config(&config).unwrap();
    assert!(creds.api_key().is_some());
    assert_eq!(creds.api_key().unwrap().expose(), "test-api-key");
    
    // Test with email/password
    let config2 = crate::config::GalaxyConfig::new("https://example.com/api")
        .with_credentials("test@example.com", "test-password");
    
    let creds2 = crate::security::credentials::credentials_from_config(&config2).unwrap();
    assert!(creds2.email().is_some());
    assert_eq!(creds2.email().unwrap(), "test@example.com");
    assert!(creds2.password().is_some());
    assert_eq!(creds2.password().unwrap().expose(), "test-password");
    
    // Test with missing credentials
    let config3 = crate::config::GalaxyConfig::new("https://example.com/api");
    let result = crate::security::credentials::credentials_from_config(&config3);
    assert!(result.is_err());
}

#[test]
fn test_helper_functions() {
    // Test credentials_from_api_key
    let api_key = SecretString::new("test-api-key");
    let creds = helpers::credentials_from_api_key(api_key);
    assert!(creds.api_key().is_some());
    assert_eq!(creds.api_key().unwrap().expose(), "test-api-key");
    
    // Test credentials_from_email_password
    let email = "test@example.com".to_string();
    let password = SecretString::new("test-password");
    let creds2 = helpers::credentials_from_email_password(email.clone(), password);
    assert!(creds2.email().is_some());
    assert_eq!(creds2.email().unwrap(), email);
    assert!(creds2.password().is_some());
    assert_eq!(creds2.password().unwrap().expose(), "test-password");
} 