# MCP Security Module

The MCP Security Module provides a comprehensive security solution for the Machine Context Protocol system. It offers authentication, authorization, encryption, and security policy enforcement mechanisms to ensure secure communication and operation.

## Key Components

### 1. SecurityManager

The `SecurityManager` trait defines the core security operations including:

- Authentication: Verify user credentials
- Authorization: Validate user permissions for specific resources
- Encryption: Secure sensitive data
- RBAC: Manage roles and permissions
- Policy Management: Evaluate and enforce security policies

```rust
// Example usage (for documentation purposes, not executable doctests):
// let security = SecurityManagerImpl::new();
// 
// // Authenticate a user
// let user_id = security.authenticate(&credentials).await?;
// 
// // Encrypt sensitive data
// let encrypted = security.encrypt(session_id, &data, Some(EncryptionFormat::Aes256Gcm)).await?;
// 
// // Decrypt data
// let decrypted = security.decrypt(session_id, &encrypted, Some(EncryptionFormat::Aes256Gcm)).await?;
// 
// // Check permissions
// if security.has_permission(&user_id, &permission).await {
//     // User has permission
// }
```

### 2. Role-Based Access Control (RBAC)

The `EnhancedRBACManager` provides a sophisticated role-based access control system with features such as:

- Role hierarchy with inheritance
- Comprehensive permission validation
- Context-aware permission checking
- Permission caching for performance
- Permission condition evaluation

```rust
// Example for documentation only:
// // Create a role with permissions
// let admin_role = security.create_role(
//     "admin",
//     Some("Administrator role"),
//     admin_permissions,
//     HashSet::new()
// ).await?;
// 
// // Assign a role to a user
// security.assign_role("user123", role_id).await?;
```

### 3. Security Policies

The security policies module provides a flexible and extensible system for defining and enforcing security rules:

- Multiple policy types (Password, RateLimit, Session, Authentication, etc.)
- Configurable enforcement levels (Advisory, Warning, Enforced, Critical)
- Custom policy evaluators for specific security needs
- Policy lifecycle management

```rust
// Example for documentation only:
// // Create a password policy
// let password_policy = SecurityPolicy {
//     id: "password-policy",
//     name: "Password Strength Policy",
//     description: Some("Password requirements for MCP system"),
//     policy_type: PolicyType::Password,
//     enforcement_level: EnforcementLevel::Enforced,
//     // ... other fields
// };
// 
// // Add the policy
// security.add_policy(password_policy).await?;
// 
// // Evaluate a policy
// let result = security.evaluate_policy("password-policy", &context).await?;
```

### 4. Cryptography

The cryptography module provides robust encryption, signing, and hashing capabilities:

- AES-256-GCM authenticated encryption
- ChaCha20-Poly1305 authenticated encryption
- HMAC-SHA256 signing and verification
- Multiple hash algorithms (SHA-256, SHA-512, BLAKE3)
- Secure random key generation
- Base64 encoding utilities

```rust
// Direct usage of crypto functions (for documentation):
// use squirrel_mcp::security::crypto;
// 
// // Generate a key
// let key = crypto::generate_key(EncryptionFormat::Aes256Gcm)?;
// 
// // Encrypt data
// let encrypted = crypto::encrypt(data, &key, EncryptionFormat::Aes256Gcm)?;
// 
// // Sign data
// let signature = crypto::sign(data, &signing_key)?;
// 
// // Verify signature
// let is_valid = crypto::verify(data, &signature, &signing_key)?;
// 
// // Hash data
// let hash = crypto::hash(data, HashAlgorithm::Sha256)?;
```

### 5. Encryption Manager

The `EncryptionManager` provides a higher-level interface for encryption operations:

- Format-specific encryption
- Key management
- Session-specific encryption formats
- Automatic key generation

```rust
// Example for documentation only:
// // Create with default settings (AES-256-GCM)
// let encryption = create_encryption_manager();
// 
// // Create with specific format
// let encryption = create_encryption_manager_with_format(EncryptionFormat::ChaCha20Poly1305);
// 
// // Encrypt data
// let encrypted = encryption.encrypt(data, format).await?;
// 
// // Decrypt data
// let decrypted = encryption.decrypt(&encrypted, format).await?;
```

## Security Policies

### Password Policies

Password policies enforce strong password requirements:

- Minimum length
- Character type requirements (uppercase, lowercase, digits, special)
- Password expiration
- History enforcement

### Rate Limiting Policies

Rate limiting policies prevent abuse through request throttling:

- Request limits per time window
- User or IP-based rate limiting
- Graduated response (warnings, blocks)

### Session Policies

Session policies enforce secure session management:

- Maximum session duration
- Inactivity timeouts
- IP binding
- Device fingerprinting
- Concurrent session limits

## How to Extend

### Adding New Policy Types

1. Define a new policy type in the `PolicyType` enum
2. Create a policy evaluator that implements the `PolicyEvaluator` trait
3. Register your evaluator with the policy manager

```rust
// Example of extending with a custom policy evaluator:
// 
// use async_trait::async_trait;
// use squirrel_mcp::security::PolicyEvaluator;
// use squirrel_mcp::security::types::{SecurityPolicy, PolicyType, PolicyContext, PolicyEvaluationResult};
// use squirrel_mcp::error::Result;
// 
// pub struct CustomPolicyEvaluator {
//     id: String,
// }
// 
// #[async_trait]
// impl PolicyEvaluator for CustomPolicyEvaluator {
//     fn get_id(&self) -> &str {
//         &self.id
//     }
//     
//     fn get_supported_policy_types(&self) -> Vec<PolicyType> {
//         vec![PolicyType::Custom]
//     }
//     
//     async fn evaluate(&self, policy: &SecurityPolicy, context: &PolicyContext) -> Result<PolicyEvaluationResult> {
//         // Your evaluation logic here
//         Ok(PolicyEvaluationResult::default())
//     }
// }
```

### Adding New Encryption Formats

To add a new encryption format:

1. Add a new variant to the `EncryptionFormat` enum
2. Update the cryptography module to support the new format
3. Implement proper key generation, encryption, and decryption functions

## Best Practices

1. **Layered Security**: Combine multiple security mechanisms for defense in depth
2. **Policy Enforcement**: Use Enforced or Critical levels for security-critical policies
3. **Key Management**: Rotate encryption keys regularly and store securely
4. **Performance**: Use caching for frequently checked permissions
5. **Auditing**: Enable audit logging for security-sensitive operations
6. **Testing**: Regularly test security mechanisms with automated tests

## Performance Considerations

The security module is designed for high performance, with optimizations for:

- Permission checks: Cached for fast repeated access
- Encryption: Uses hardware acceleration when available
- Policy evaluation: Caches evaluation results for similar contexts

For very high-throughput applications, consider:

- Using ChaCha20-Poly1305 on platforms without AES hardware acceleration
- Minimizing unnecessary permission checks
- Tuning the enforcement level of non-critical policies

## Integration Testing

Comprehensive integration tests are provided to ensure all components work together correctly:

- `integration_test.rs`: Tests the interaction between different security components
- `performance_benchmark.rs`: Benchmarks key security operations

## Related Documentation

- [MCP Security Specification](../../../specs/mcp/MCP_SPECIFICATION.md)
- [RBAC Design Document](../../../docs/rbac-design.md)
- [Security Policies Guide](../../../docs/security-policies.md) 