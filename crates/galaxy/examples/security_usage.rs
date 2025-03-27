/// Security module usage example
///
/// This example demonstrates how to use the Galaxy security module
/// for secure credential management, including:
/// 
/// 1. Creating and storing secure credentials
/// 2. Configuring the security manager with different storage backends
/// 3. Implementing credential rotation
/// 4. Using encrypted storage

use squirrel_galaxy::security::{
    SecretString, 
    SecureCredentials, 
    SecurityManager, 
    RotationPolicy,
    storage::{CredentialStorage, FileStorage, MemoryStorage, EncryptedStorage},
};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use rand::RngCore;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Galaxy Security Module Example");
    println!("==============================\n");
    
    // Example 1: Basic credential management
    println!("Example 1: Basic Credential Management");
    println!("--------------------------------------");
    
    // Create a security manager with in-memory storage
    let security_manager = SecurityManager::new()
        .allow_environment_variables(true);
    
    // Create secure credentials with API key
    let api_key = SecretString::new("galaxy-api-key-1234");
    let credentials = SecureCredentials::with_api_key(api_key);
    
    // Store the credentials
    println!("Storing credentials...");
    security_manager.store_credentials("my-galaxy-instance", credentials).await?;
    
    // Retrieve the credentials
    println!("Retrieving credentials...");
    let retrieved = security_manager.get_credentials("my-galaxy-instance").await?;
    
    // Use the credentials (note: in a real app, we'd never print the API key)
    println!("Using credentials:");
    if let Some(key) = retrieved.api_key() {
        println!("  API Key: {}", key.expose());
    }
    println!();
    
    // Example 2: File-based encrypted storage
    println!("Example 2: File-based Encrypted Storage");
    println!("--------------------------------------");
    
    // Create temporary directory for credentials
    let temp_dir = tempfile::tempdir()?;
    let storage_path = temp_dir.path().to_string_lossy().to_string();
    println!("Storage path: {}", storage_path);
    
    // Create a random encryption key
    let mut encryption_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut encryption_key);
    
    // Create file storage
    let file_storage = Arc::new(FileStorage::new(storage_path.clone(), encryption_key));
    println!("Created encrypted file storage");
    
    // Create a security manager with file storage
    let file_security_manager = SecurityManager::with_storage(file_storage.clone());
    
    // Store credentials
    println!("Storing credentials in encrypted storage...");
    let file_credentials = SecureCredentials::with_api_key(
        SecretString::new("galaxy-file-storage-key")
    );
    file_security_manager.store_credentials("galaxy-file", file_credentials).await?;
    
    // Retrieve credentials
    println!("Retrieving credentials from encrypted storage...");
    let file_retrieved = file_security_manager.get_credentials("galaxy-file").await?;
    println!("  API Key: {}", file_retrieved.api_key().unwrap().expose());
    println!();
    
    // Example 3: Credential rotation
    println!("Example 3: Credential Rotation");
    println!("-----------------------------");
    
    // Create a security manager with rotation policy
    let rotation_manager = SecurityManager::with_storage(Arc::new(MemoryStorage::new()))
        .with_rotation_policy(RotationPolicy {
            frequency_days: 30,
            auto_rotate: true,
            history_size: 3,
            update_dependents: false,
        });
    
    // Store initial credentials
    println!("Storing initial credentials...");
    let initial_credentials = SecureCredentials::with_api_key(
        SecretString::new("initial-galaxy-key")
    );
    rotation_manager.store_credentials("rotated-galaxy", initial_credentials).await?;
    
    // Rotate credentials
    println!("Rotating credentials...");
    let rotated_credentials = SecureCredentials::with_api_key(
        SecretString::new("rotated-galaxy-key")
    );
    rotation_manager.rotate_credentials("rotated-galaxy", rotated_credentials).await?;
    
    // Retrieve current credentials
    let current = rotation_manager.get_credentials("rotated-galaxy").await?;
    println!("Current API key: {}", current.api_key().unwrap().expose());
    
    // Retrieve credential history
    let history = rotation_manager.get_credential_history("rotated-galaxy").await?;
    println!("Credential history count: {}", history.len());
    println!("Previous API key: {}", history[0].api_key().unwrap().expose());
    println!();
    
    // Example 4: Encryption key rotation
    println!("Example 4: Encryption Key Rotation");
    println!("--------------------------------");
    
    // Create file storage with a known key
    let original_key = [1u8; 32];
    let encrypted_storage = Arc::new(FileStorage::new(
        format!("{}/key_rotation", storage_path),
        original_key
    ));
    
    // Store a credential
    println!("Storing credential with original encryption key...");
    let secret = SecureCredentials::with_api_key(
        SecretString::new("protected-by-encryption")
    );
    encrypted_storage.store("encrypted-galaxy", secret).await?;
    
    // Generate a new encryption key
    println!("Generating new encryption key...");
    let new_key = [2u8; 32];
    
    // Rotate the encryption key
    println!("Rotating encryption key...");
    let storage_ref = encrypted_storage.as_ref() as &dyn EncryptedStorage;
    storage_ref.rotate_encryption_key(new_key).await?;
    
    // Verify we can still access the credential
    let rotated_secret = encrypted_storage.get("encrypted-galaxy").await?;
    println!("Retrieved credential after key rotation: {}", 
        rotated_secret.api_key().unwrap().expose());
    
    // Check encryption status
    let status = storage_ref.encryption_status();
    println!("Encryption algorithm: {}", status.algorithm);
    println!("Key strength: {} bits", status.key_strength);
    println!("Last rotation: {:?}", status.last_rotation);
    
    println!("\nExamples completed successfully!");
    Ok(())
} 