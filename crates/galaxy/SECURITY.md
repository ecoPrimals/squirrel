# Galaxy MCP Adapter Security Features

This document provides an overview of the security features implemented in the Galaxy MCP Adapter library, focusing on credential management, storage, and rotation.

## Table of Contents

- [Overview](#overview)
- [SecurityManager](#securitymanager)
- [Secure Credentials](#secure-credentials)
- [Credential Storage](#credential-storage)
- [Credential Rotation](#credential-rotation)
- [Encryption](#encryption)
- [Environment Variable Handling](#environment-variable-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Overview

The Galaxy MCP Adapter incorporates several security features to ensure that sensitive information such as API keys and login credentials are managed securely. Key security features include:

- Secure credential management and storage
- Credential rotation and history tracking
- Encryption of stored credentials
- Environment variable integration
- Secure memory handling for sensitive strings

## SecurityManager

The `SecurityManager` is the central component that coordinates all security operations:

```rust
let security_manager = SecurityManager::with_storage(storage)
    .allow_environment_variables(true)
    .with_rotation_policy(RotationPolicy {
        frequency_days: 90,
        auto_rotate: false,
        history_size: 3,
        update_dependents: false,
    })
    .auto_check_rotation(true);
```

### Features

- **Credential Storage**: Manages secure storage of credentials, either in-memory or file-based.
- **Rotation Policy**: Configurable policy for credential rotation, including frequency and history size.
- **Automatic Rotation Check**: Can automatically check if credentials need rotation.
- **Environment Variable Integration**: Optionally load credentials from environment variables.

## Secure Credentials

The `SecureCredentials` type stores authentication information securely:

```rust
// Create with API key
let credentials = SecureCredentials::with_api_key(SecretString::new("your-api-key"));

// Create with email and password
let credentials = SecureCredentials::with_email_password(
    "user@example.com",
    SecretString::new("your-password")
);
```

### Features

- **Secure Storage**: Credentials are stored securely in memory, minimizing exposure.
- **SecretString**: Passwords and API keys are wrapped in a `SecretString` that prevents accidental logging or exposure.
- **Credential Types**: Supports both API key authentication and email/password authentication.
- **Creation Timestamp**: Tracks when credentials were created for rotation purposes.
- **Validation**: Includes methods to validate credentials against the Galaxy API.

## Credential Storage

The adapter supports two types of credential storage:

1. **In-Memory Storage**: Credentials are stored only in memory and not persisted.
   ```rust
   let config = GalaxyConfig::default()
       .with_credential_storage(CredentialStorageConfig {
           storage_type: CredentialStorageType::Memory,
           file_storage_path: None,
           encrypt: true,
       });
   ```

2. **File-Based Storage**: Credentials are encrypted and stored in a specified directory.
   ```rust
   let config = GalaxyConfig::default()
       .with_credential_storage(CredentialStorageConfig {
           storage_type: CredentialStorageType::File,
           file_storage_path: Some("/path/to/secure/storage"),
           encrypt: true,
       })
       .with_encryption_key("your-hex-encoded-key");
   ```

### Features

- **Secure File Format**: File-based storage uses an encrypted format that protects credentials at rest.
- **Multiple Credential Sets**: Support for storing and retrieving multiple credential sets with unique IDs.
- **Credential History**: Stores a history of previous credentials to enable fallback if needed.

## Credential Rotation

The Galaxy Adapter supports credential rotation to enhance security:

```rust
// Rotate API key
adapter.rotate_api_key(SecretString::new("new-api-key")).await?;

// Get credential history
let history = adapter.get_credential_history().await?;
```

### Features

- **API Key Rotation**: Support for rotating API keys while maintaining service availability.
- **Rotation History**: Keeps a configurable history of previous credentials.
- **Rotation Policies**: Configure rotation frequency and automatic rotation checks.
- **Rotation Warnings**: Logs warnings when credentials should be rotated based on age.

## Encryption

Sensitive data is encrypted when stored on disk:

```rust
// Generate or provide an encryption key
let encryption_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

// Configure with encryption key
let config = GalaxyConfig::default()
    .with_encryption_key(encryption_key);
```

### Features

- **Encryption at Rest**: All sensitive data stored on disk is encrypted.
- **Key Rotation**: Support for rotating encryption keys without losing access to credentials.
- **Secure Key Handling**: Encryption keys are treated as sensitive data and protected in memory.

## Environment Variable Handling

The adapter can load configuration and credentials from environment variables:

```rust
// Load configuration from environment variables
let config = GalaxyConfig::from_env()?;

// Or explicitly enable environment variable support
let config = GalaxyConfig::default()
    .allow_env_credentials(true)
    .allow_env_vars(true);
```

### Environment Variables

- `GALAXY_API_URL`: Galaxy API URL
- `GALAXY_API_KEY`: API key for Galaxy authentication
- `GALAXY_EMAIL`: Email for Galaxy account
- `GALAXY_PASSWORD`: Password for Galaxy account
- `GALAXY_TIMEOUT`: API request timeout in seconds
- `GALAXY_STORAGE_PATH`: Path for credential storage
- `GALAXY_ENCRYPTION_KEY`: Encryption key for secure storage
- `GALAXY_KEY_ROTATION_DAYS`: Credential rotation period in days
- `GALAXY_AUTO_ROTATE_KEYS`: Whether to automatically rotate keys

## Best Practices

1. **Use API Keys**: Prefer API key authentication over email/password when possible.
2. **Enable Encryption**: Always use encryption for file-based storage.
3. **Rotate Credentials**: Regularly rotate API keys (recommended every 90 days).
4. **Secure Storage Path**: Store credentials in a directory with appropriate file system permissions.
5. **Encryption Key Management**: Store encryption keys securely, separate from the credentials.
6. **Environment Variables**: Use environment variables in production environments for better security.

## Examples

For code examples demonstrating these security features, see:

- [enhanced_security.rs](examples/enhanced_security.rs): Comprehensive example of all security features.
- [security_usage.rs](examples/security_usage.rs): Basic usage examples for the security module.

## Implementation Details

### Secure Memory Handling

The `SecretString` type provides secure memory handling for sensitive strings:

- Minimizes copies in memory
- Zeroes memory when dropped
- Prevents accidental logging or display
- Provides controlled access to the underlying string

### Encryption Implementation

Encryption uses XChaCha20-Poly1305 with the following properties:

- 256-bit encryption keys
- Authenticated encryption with associated data (AEAD)
- Unique nonces for each encryption operation
- Integrity verification to prevent tampering

### Error Handling

Security operations provide detailed error types to help diagnose issues:

- `SecurityError::EncryptionError`: Problems with encryption/decryption
- `SecurityError::StorageError`: Problems with credential storage
- `SecurityError::MissingCredentials`: Credentials not found
- `SecurityError::InvalidCredentials`: Credentials failed validation
- `SecurityError::RotationError`: Problems during credential rotation 