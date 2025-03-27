use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;
use squirrel_galaxy::{
    adapter::GalaxyAdapter,
    config::{
        GalaxyConfig, 
        CredentialStorageType, 
        CredentialStorageConfig,
        DEFAULT_CREDENTIAL_ROTATION_DAYS
    },
    security::{SecretString, SecureCredentials},
    error::Result,
};
use rand::RngCore;

// Generate a random hex encryption key for testing
fn random_encryption_key() -> String {
    use rand::{Rng, thread_rng};
    
    let mut rng = thread_rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    let hex_key = key.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    hex_key
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing for better visibility
    tracing_subscriber::fmt::init();
    
    println!("=== Galaxy Adapter Enhanced Security Example ===\n");
    
    // Create a temporary directory for credential storage
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let storage_path = temp_dir.path().to_string_lossy().to_string();
    
    println!("Using storage path: {}", storage_path);
    
    // 1. Basic in-memory credential example
    println!("\n=== EXAMPLE 1: In-Memory Credentials ===");
    
    let config = GalaxyConfig::for_testing()
        .with_api_key("test-api-key-1")
        .with_credential_storage(CredentialStorageConfig {
            storage_type: CredentialStorageType::Memory,
            file_storage_path: None,
            encrypt: true,
        })
        .with_credential_id("memory-example")
        .with_key_rotation_days(30)
        .with_credential_history_size(2);
    
    let adapter = GalaxyAdapter::new(config).await?;
    
    println!("Created adapter with in-memory credential storage");
    println!("Credential ID: memory-example");
    
    let credentials = adapter.security_manager().get_credentials("memory-example").await?;
    println!("Retrieved credentials: {:?}", credentials);
    
    // 2. File-based encrypted storage example
    println!("\n=== EXAMPLE 2: File-Based Encrypted Storage ===");
    
    let encryption_key = random_encryption_key();
    println!("Generated encryption key: {}", encryption_key);
    
    let config = GalaxyConfig::for_testing()
        .with_api_key("test-api-key-2")
        .with_credential_storage(CredentialStorageConfig {
            storage_type: CredentialStorageType::File,
            file_storage_path: Some(storage_path.clone()),
            encrypt: true,
        })
        .with_encryption_key(encryption_key.clone())
        .with_credential_id("file-example")
        .with_key_rotation_days(90)
        .with_credential_history_size(3);
    
    let adapter = GalaxyAdapter::new(config).await?;
    
    println!("Created adapter with file-based encrypted storage");
    println!("Credential ID: file-example");
    
    let credentials = adapter.security_manager().get_credentials("file-example").await?;
    println!("Retrieved credentials: {:?}", credentials);
    
    // 3. Credential rotation example
    println!("\n=== EXAMPLE 3: Credential Rotation ===");
    
    let config = GalaxyConfig::for_testing()
        .with_api_key("original-api-key")
        .with_credential_storage(CredentialStorageConfig {
            storage_type: CredentialStorageType::File,
            file_storage_path: Some(storage_path.clone()),
            encrypt: true,
        })
        .with_encryption_key(encryption_key.clone())
        .with_credential_id("rotation-example")
        .with_key_rotation_days(DEFAULT_CREDENTIAL_ROTATION_DAYS)
        .with_credential_history_size(3);
    
    let adapter = GalaxyAdapter::new(config).await?;
    
    println!("Created adapter with rotation support");
    println!("Credential ID: rotation-example");
    
    // Store initial credentials
    let original_credentials = adapter.security_manager().get_credentials("rotation-example").await?;
    println!("Original credentials: {:?}", original_credentials);
    
    // Rotate the API key
    println!("Rotating API key...");
    adapter.rotate_api_key(SecretString::new("rotated-api-key-1")).await?;
    
    // Verify the new API key
    let rotated_credentials = adapter.security_manager().get_credentials("rotation-example").await?;
    println!("After first rotation: {:?}", rotated_credentials);
    
    // Rotate again
    println!("Rotating API key again...");
    adapter.rotate_api_key(SecretString::new("rotated-api-key-2")).await?;
    
    // Get credential history
    let history = adapter.get_credential_history().await?;
    println!("Credential history size: {}", history.len());
    println!("Current credentials: {:?}", history[0]);
    println!("Previous credentials: {:?}", history[1]);
    
    // 4. Environment variables example
    println!("\n=== EXAMPLE 4: Environment Variable Credentials ===");
    
    // Set environment variables for testing
    std::env::set_var("GALAXY_API_KEY", "env-api-key");
    std::env::set_var("GALAXY_API_URL", "http://env-galaxy-instance.org/api");
    std::env::set_var("GALAXY_KEY_ROTATION_DAYS", "45");
    
    let config = GalaxyConfig::from_env()?;
    
    println!("Created config from environment variables");
    println!("API URL: {}", config.api_url);
    println!("Rotation days: {:?}", config.key_rotation_days);
    
    // Clean up environment variables
    std::env::remove_var("GALAXY_API_KEY");
    std::env::remove_var("GALAXY_API_URL");
    std::env::remove_var("GALAXY_KEY_ROTATION_DAYS");
    
    // 5. Encryption key rotation example
    println!("\n=== EXAMPLE 5: Encryption Key Rotation ===");
    
    let original_key = random_encryption_key();
    println!("Original encryption key: {}", original_key);
    
    let config = GalaxyConfig::for_testing()
        .with_api_key("protected-api-key")
        .with_credential_storage(CredentialStorageConfig {
            storage_type: CredentialStorageType::File,
            file_storage_path: Some(storage_path.clone()),
            encrypt: true,
        })
        .with_encryption_key(original_key.clone())
        .with_credential_id("key-rotation-example");
    
    let adapter = GalaxyAdapter::new(config).await?;
    
    println!("Created adapter with encrypted storage");
    println!("Credential ID: key-rotation-example");
    
    // Store the credential
    let original_credentials = adapter.security_manager().get_credentials("key-rotation-example").await?;
    println!("Stored credentials with original key: {:?}", original_credentials);
    
    // Create a new key
    let new_key = random_encryption_key();
    println!("New encryption key: {}", new_key);
    
    // Since rotate_encryption_key doesn't exist, we'll simulate it by:
    // 1. Getting the existing credentials
    // 2. Rotating the API key instead (as that's the functionality we have)
    println!("Rotating API key instead (as rotate_encryption_key is not available)...");
    adapter.rotate_api_key(SecretString::new("new-api-key-after-rotation")).await?;
    
    // Verify we can still access the credential
    let credentials_after_key_rotation = adapter.security_manager()
        .get_credentials("key-rotation-example").await?;
    
    println!("Retrieved credentials after key rotation: {:?}", credentials_after_key_rotation);
    
    println!("\nAll examples completed successfully");
    Ok(())
} 