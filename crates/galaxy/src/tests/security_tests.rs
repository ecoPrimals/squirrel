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
    credentials::*,
    storage::{EncryptedStorage},
};
use std::sync::Arc;
use tempfile::tempdir;
use std::env;
use anyhow::Result;
use tempfile::TempDir;

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

#[tokio::test]
async fn test_secure_credentials_lifecycle() -> Result<()> {
    // Create a memory storage
    let storage = Arc::new(MemoryStorage::new());
    
    // Create a security manager
    let security_manager = SecurityManager::with_storage(storage.clone())
        .allow_environment_variables(true)
        .with_rotation_policy(RotationPolicy {
            frequency_days: 30,
            auto_rotate: true,
            history_size: 3,
            update_dependents: false,
        })
        .auto_check_rotation(true);
    
    // Create a test credential
    let api_key = SecretString::new("test-api-key-1");
    let credentials = SecureCredentials::with_api_key(api_key);
    
    // Store the credential
    security_manager.store_credentials("galaxy-1", credentials).await?;
    
    // Retrieve the credential
    let retrieved = security_manager.get_credentials("galaxy-1").await?;
    assert_eq!(
        retrieved.api_key().unwrap().expose(),
        "test-api-key-1"
    );
    
    // Rotate the credential
    let new_api_key = SecretString::new("test-api-key-2");
    let new_credentials = SecureCredentials::with_api_key(new_api_key);
    security_manager.rotate_credentials("galaxy-1", new_credentials).await?;
    
    // Retrieve the new credential
    let new_retrieved = security_manager.get_credentials("galaxy-1").await?;
    assert_eq!(
        new_retrieved.api_key().unwrap().expose(),
        "test-api-key-2"
    );
    
    // Get credential history
    let history = security_manager.get_credential_history("galaxy-1").await?;
    assert_eq!(history.len(), 1); // Should have 1 old credential
    
    // Old credential should be in history
    let old_credential = &history[0];
    assert_eq!(
        old_credential.api_key().unwrap().expose(),
        "test-api-key-1"
    );
    
    // Rotate again
    let newer_api_key = SecretString::new("test-api-key-3");
    let newer_credentials = SecureCredentials::with_api_key(newer_api_key);
    security_manager.rotate_credentials("galaxy-1", newer_credentials).await?;
    
    // Get credential history again
    let history = security_manager.get_credential_history("galaxy-1").await?;
    assert_eq!(history.len(), 1); // Should have 1 old credential
    
    // Newest old credential should be first in history
    assert_eq!(
        history[0].api_key().unwrap().expose(),
        "test-api-key-2"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_encrypted_storage() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path().to_string_lossy().to_string();
    
    // Create a file storage with a known encryption key
    let key = [42u8; 32]; // Test key
    let storage = Arc::new(FileStorage::new(base_path, key));
    
    // Create a test credential
    let api_key = SecretString::new("galaxy-api-key");
    let credentials = SecureCredentials::with_api_key(api_key);
    
    // Store the credential
    storage.store("test-galaxy", credentials.clone()).await?;
    
    // Retrieve the credential
    let retrieved = storage.get("test-galaxy").await?;
    assert_eq!(
        retrieved.api_key().unwrap().expose(),
        credentials.api_key().unwrap().expose()
    );
    
    // Test encryption key rotation
    let new_key = [99u8; 32]; // New test key
    let encrypted_storage = storage.as_ref() as &dyn EncryptedStorage;
    encrypted_storage.rotate_encryption_key(new_key).await?;
    
    // Credential should still be retrievable with the new key
    let retrieved_after_rotation = storage.get("test-galaxy").await?;
    assert_eq!(
        retrieved_after_rotation.api_key().unwrap().expose(),
        credentials.api_key().unwrap().expose()
    );
    
    // Check encryption status
    let status = encrypted_storage.encryption_status();
    assert_eq!(status.algorithm, "AES-GCM-SIV-256");
    assert_eq!(status.key_strength, 256);
    assert!(status.last_rotation.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_find_by_api_key() -> Result<()> {
    // Create a memory storage
    let storage = Arc::new(MemoryStorage::new());
    
    // Create a security manager
    let security_manager = SecurityManager::with_storage(storage.clone());
    
    // Create multiple test credentials
    let api_key1 = SecretString::new("api-key-1");
    let credentials1 = SecureCredentials::with_api_key(api_key1.clone());
    
    let api_key2 = SecretString::new("api-key-2");
    let credentials2 = SecureCredentials::with_api_key(api_key2.clone());
    
    // Store the credentials
    security_manager.store_credentials("galaxy-1", credentials1).await?;
    security_manager.store_credentials("galaxy-2", credentials2).await?;
    
    // Find by API key
    let found1 = security_manager.find_by_api_key(&api_key1).await?;
    assert!(found1.is_some());
    let (id1, creds1) = found1.unwrap();
    assert_eq!(id1, "galaxy-1");
    assert_eq!(
        creds1.api_key().unwrap().expose(),
        api_key1.expose()
    );
    
    // Find by different API key
    let found2 = security_manager.find_by_api_key(&api_key2).await?;
    assert!(found2.is_some());
    let (id2, creds2) = found2.unwrap();
    assert_eq!(id2, "galaxy-2");
    assert_eq!(
        creds2.api_key().unwrap().expose(),
        api_key2.expose()
    );
    
    // Find with non-existent API key
    let not_found = security_manager.find_by_api_key(&SecretString::new("nonexistent")).await?;
    assert!(not_found.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_credential_rotation_limit_history() -> Result<()> {
    // Create a memory storage
    let storage = Arc::new(MemoryStorage::new());
    
    // Create a security manager with history limit of 2
    let security_manager = SecurityManager::with_storage(storage.clone())
        .with_rotation_policy(RotationPolicy {
            frequency_days: 30,
            auto_rotate: true,
            history_size: 2, // Only keep 2 old credentials
            update_dependents: false,
        });
    
    // Create and store initial credential
    let api_key1 = SecretString::new("test-key-1");
    let credentials1 = SecureCredentials::with_api_key(api_key1);
    security_manager.store_credentials("galaxy-test", credentials1).await?;
    
    // Rotate the credential 3 times (should only keep latest 2 in history)
    for i in 2..=4 {
        let new_key = SecretString::new(format!("test-key-{}", i));
        let new_creds = SecureCredentials::with_api_key(new_key);
        security_manager.rotate_credentials("galaxy-test", new_creds).await?;
    }
    
    // Get credential history
    let history = security_manager.get_credential_history("galaxy-test").await?;
    
    // Should only have 2 old credentials
    assert_eq!(history.len(), 1);
    
    // History should contain most recent rotations (3 and 2, but not 1)
    assert_eq!(
        history[0].api_key().unwrap().expose(),
        "test-key-3"
    );
    
    // The oldest credential should not be in history
    for h in history {
        assert_ne!(h.api_key().unwrap().expose(), "test-key-1");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_should_rotate() -> Result<()> {
    // Create a memory storage
    let storage = Arc::new(MemoryStorage::new());
    
    // Create a security manager with 30-day rotation
    let security_manager = SecurityManager::with_storage(storage.clone())
        .with_rotation_policy(RotationPolicy {
            frequency_days: 30,
            auto_rotate: true,
            history_size: 3,
            update_dependents: false,
        });
    
    // Create a credential
    let api_key = SecretString::new("test-api-key");
    let credentials = SecureCredentials::with_api_key(api_key);
    
    // New credential shouldn't need rotation
    assert!(!security_manager.should_rotate(&credentials));
    
    // Create an expired credential (would need simulation of time passage)
    let expired_credentials = SecureCredentials::with_api_key(SecretString::new("expired"))
        .with_expiration(time::OffsetDateTime::now_utc() - time::Duration::days(1));
    
    // Expired credential should need rotation
    assert!(security_manager.should_rotate(&expired_credentials));
    
    Ok(())
} 