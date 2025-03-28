use crate::security::{
    SecurityManagerImpl, SecurityManager, Credentials, Session, 
    EnhancedRBACManager, PolicyManager, PolicyType, EnforcementLevel,
    SecurityPolicy, PolicyContext, PolicyEvaluationResult, 
    PasswordPolicyEvaluator, RateLimitPolicyEvaluator, SessionPolicyEvaluator,
    Role, Permission, Action, PermissionScope, PermissionContext
};
use crate::error::{Result, MCPError};
use crate::types::{SecurityLevel, EncryptionFormat};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use chrono::Utc;

#[tokio::test]
async fn test_security_manager_integration() -> Result<()> {
    // Create the main security components
    let rbac_manager = Arc::new(EnhancedRBACManager::new());
    let policy_manager = Arc::new(PolicyManager::new(true));
    
    // Register policy evaluators
    policy_manager.add_evaluator(Arc::new(PasswordPolicyEvaluator::new())).await?;
    policy_manager.add_evaluator(Arc::new(RateLimitPolicyEvaluator::new())).await?;
    policy_manager.add_evaluator(Arc::new(SessionPolicyEvaluator::new())).await?;
    
    // Create security manager with components
    let mut security = SecurityManagerImpl::with_components(
        rbac_manager, 
        policy_manager,
        crate::security::encryption::create_encryption_manager()
    );
    
    // Set up roles and permissions
    let mut admin_permissions = HashSet::new();
    admin_permissions.insert(Permission {
        id: "admin-read".to_string(),
        name: "Admin Read Access".to_string(),
        resource: "secure-resource".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    });
    admin_permissions.insert(Permission {
        id: "admin-write".to_string(),
        name: "Admin Write Access".to_string(),
        resource: "secure-resource".to_string(),
        action: Action::Write,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    });
    
    let admin_role = security.create_role(
        "admin".to_string(),
        Some("Administrator role".to_string()),
        admin_permissions,
        HashSet::new()
    ).await?;
    
    let mut user_permissions = HashSet::new();
    user_permissions.insert(Permission {
        id: "user-read".to_string(),
        name: "User Read Access".to_string(),
        resource: "secure-resource".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::Self_,
        conditions: Vec::new(),
    });
    
    let user_role = security.create_role(
        "user".to_string(),
        Some("Basic user role".to_string()),
        user_permissions,
        HashSet::new()
    ).await?;
    
    // Assign roles to users
    security.assign_role("admin-user".to_string(), admin_role.id.clone()).await?;
    security.assign_role("normal-user".to_string(), user_role.id.clone()).await?;
    
    // Set up security policies
    let mut password_settings = HashMap::new();
    password_settings.insert("min_length".to_string(), "8".to_string());
    password_settings.insert("require_uppercase".to_string(), "true".to_string());
    password_settings.insert("require_lowercase".to_string(), "true".to_string());
    password_settings.insert("require_digit".to_string(), "true".to_string());
    password_settings.insert("require_special".to_string(), "true".to_string());
    
    let password_policy = SecurityPolicy {
        id: "password-policy".to_string(),
        name: "Password Strength Policy".to_string(),
        description: Some("Password requirements for MCP system".to_string()),
        policy_type: PolicyType::Password,
        enforcement_level: EnforcementLevel::Enforced,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: password_settings,
        required_permissions: HashSet::new(),
        security_level: SecurityLevel::Standard,
        enabled: true,
    };
    
    security.add_policy(password_policy).await?;
    
    let mut session_settings = HashMap::new();
    session_settings.insert("max_session_length_minutes".to_string(), "60".to_string());
    session_settings.insert("inactivity_timeout_minutes".to_string(), "15".to_string());
    
    let session_policy = SecurityPolicy {
        id: "session-policy".to_string(),
        name: "Session Security Policy".to_string(),
        description: Some("Session timeout and validation rules".to_string()),
        policy_type: PolicyType::Session,
        enforcement_level: EnforcementLevel::Critical,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: session_settings,
        required_permissions: HashSet::new(),
        security_level: SecurityLevel::Standard,
        enabled: true,
    };
    
    security.add_policy(session_policy).await?;
    
    // Configure encryption for a session
    security.set_session_encryption_format("admin-session".to_string(), EncryptionFormat::Aes256Gcm);
    security.set_session_encryption_format("user-session".to_string(), EncryptionFormat::ChaCha20Poly1305);
    
    // Test permissions verification
    let admin_perm = Permission {
        id: "admin-write".to_string(),
        name: "Admin Write Access".to_string(),
        resource: "secure-resource".to_string(),
        action: Action::Write,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };
    
    assert!(security.has_permission("admin-user", &admin_perm).await);
    assert!(!security.has_permission("normal-user", &admin_perm).await);
    
    // Test policy evaluation
    let mut context = PolicyContext::default();
    let mut request_info = HashMap::new();
    request_info.insert("password".to_string(), "Password1!".to_string());
    context.request_info = request_info;
    
    let result = security.evaluate_policy("password-policy", &context).await?;
    assert!(matches!(result, PolicyEvaluationResult::Passed));
    
    // Test encryption/decryption for different sessions
    let admin_data = serde_json::json!({
        "user_id": "admin-user",
        "role": "admin",
        "sensitive_data": "This is sensitive admin information"
    });
    
    let user_data = serde_json::json!({
        "user_id": "normal-user",
        "role": "user",
        "sensitive_data": "This is sensitive user information"
    });
    
    // Encrypt and decrypt admin data
    let admin_encrypted = security.encrypt("admin-session", &admin_data, None).await?;
    let admin_decrypted = security.decrypt("admin-session", &admin_encrypted, None).await?;
    assert_eq!(admin_data, admin_decrypted);
    
    // Encrypt and decrypt user data
    let user_encrypted = security.encrypt("user-session", &user_data, None).await?;
    let user_decrypted = security.decrypt("user-session", &user_encrypted, None).await?;
    assert_eq!(user_data, user_decrypted);
    
    // Verify that encryption formats are different
    assert_ne!(admin_encrypted, user_encrypted);
    
    // Test session invalidation through policy
    let mut session_context = PolicyContext::default();
    session_context.session_token = Some("expired-session".to_string());
    
    // Set last_access_time to more than the timeout period ago
    let timeout_mins = 15;
    let three_hours_ago = Utc::now() - chrono::Duration::minutes(timeout_mins + 180);
    
    let mut session_info = HashMap::new();
    session_info.insert("last_access_time".to_string(), three_hours_ago.to_rfc3339());
    session_context.request_info = session_info;
    
    let result = security.evaluate_policy("session-policy", &session_context).await;
    assert!(result.is_ok());
    let evaluation_result = result.unwrap();
    assert!(matches!(evaluation_result, PolicyEvaluationResult::Violation(_)));
    
    // Generate encryption keys
    let aes_key = security.generate_encryption_key(EncryptionFormat::Aes256Gcm).await?;
    let chacha_key = security.generate_encryption_key(EncryptionFormat::ChaCha20Poly1305).await?;
    
    assert_eq!(aes_key.len(), 32); // AES-256 uses 32-byte keys
    assert_eq!(chacha_key.len(), 32); // ChaCha20-Poly1305 uses 32-byte keys
    
    Ok(())
}

// Test the complete workflow with authentication, authorization, policy validation, and encryption
#[tokio::test]
async fn test_security_workflow() -> Result<()> {
    // Create the security manager
    let security = SecurityManagerImpl::new();
    
    // 1. Authentication
    let credentials = Credentials {
        username: "test_admin".to_string(),
        password: Some("SecurePassword123!".to_string()),
        token: None,
    };
    
    let user_id = security.authenticate(&credentials).await?;
    assert_eq!(user_id, "test_admin");
    
    // 2. Attempt to authorize (this is simplified in our implementation)
    let token = "admin-session-token";
    let result = security.authorize(token, SecurityLevel::Standard).await;
    
    // Our simple implementation rejects Critical level but accepts Standard
    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(session.token.0, token);
    
    // 3. Encrypt sensitive data
    let sensitive_data = serde_json::json!({
        "username": "test_admin",
        "account": "admin123",
        "privileges": ["read", "write", "execute"],
        "session_data": {
            "ip_address": "192.168.1.1",
            "user_agent": "Test Client 1.0",
            "login_time": "2024-09-13T10:00:00Z"
        }
    });
    
    // Use both encryption formats to test
    let aes_encrypted = security.encrypt(&session.token.0, &sensitive_data, Some(EncryptionFormat::Aes256Gcm)).await?;
    let chacha_encrypted = security.encrypt(&session.token.0, &sensitive_data, Some(EncryptionFormat::ChaCha20Poly1305)).await?;
    
    // Data should be encrypted differently
    assert_ne!(aes_encrypted, chacha_encrypted);
    
    // 4. Decrypt the data back
    let aes_decrypted = security.decrypt(&session.token.0, &aes_encrypted, Some(EncryptionFormat::Aes256Gcm)).await?;
    let chacha_decrypted = security.decrypt(&session.token.0, &chacha_encrypted, Some(EncryptionFormat::ChaCha20Poly1305)).await?;
    
    // Both should match the original data
    assert_eq!(sensitive_data, aes_decrypted);
    assert_eq!(sensitive_data, chacha_decrypted);
    
    Ok(())
} 