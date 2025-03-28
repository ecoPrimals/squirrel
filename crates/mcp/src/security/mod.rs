//! Security module for the MCP system
//!
//! This module provides security-related functionality, including
//! authentication, authorization, and encryption.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;

/// Security types, data structures, and constants for role-based access control,
/// permission conditions, and security level definitions
pub mod types;

pub mod crypto;
pub mod rbac;
pub mod policies;
pub mod encryption;

// Re-export types from types.rs
pub use types::{Role, Permission, PermissionCondition, PermissionContext, PermissionScope, Action};

// Import types from crate::types
use crate::types::{SessionToken, UserId, SecurityLevel, EncryptionFormat};

// Re-export enhanced RBAC components
pub use rbac::{
    EnhancedRBACManager, ValidationResult, ValidationRule, InheritanceType, ValidationAuditRecord
};

// Re-export policy components
pub use policies::{
    PolicyManager, SecurityPolicy, PolicyType, EnforcementLevel, PolicyContext, PolicyEvaluationResult,
    PolicyEvaluator, PasswordPolicyEvaluator, RateLimitPolicyEvaluator, SessionPolicyEvaluator
};

// Re-export encryption components
pub use encryption::{Encryption, EncryptionManager, create_encryption_manager};

use crate::error::Result;
use chrono::DateTime;
use chrono::Utc;

/// Authentication credentials for security operations
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Username for authentication
    pub username: String,
    
    /// Password for authentication (optional)
    pub password: Option<String>,
    
    /// Token for authentication (optional)
    pub token: Option<String>,
}

/// Session information for authenticated users
#[derive(Debug, Clone)]
pub struct Session {
    /// Session token
    pub token: SessionToken,
    
    /// User ID
    pub user_id: UserId,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Last access time
    pub updated_at: DateTime<Utc>,
    
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

/// Security manager interface
#[async_trait]
pub trait SecurityManager: Send + Sync {
    /// Authenticate a user with credentials
    /// Returns the user ID if authentication is successful
    async fn authenticate(&self, credentials: &Credentials) -> Result<String>;
    
    /// Authorize a user for a specific resource and action level
    /// Returns a Session if authorization is successful
    async fn authorize(&self, token: &str, security_level: SecurityLevel) -> Result<Session>;
    
    /// Encrypt data with the session's keys
    /// Returns the encrypted data as bytes
    async fn encrypt(&self, session_id: &str, data: &serde_json::Value, format: Option<EncryptionFormat>) -> Result<Vec<u8>>;
    
    /// Decrypt data with the session's keys
    /// Returns the decrypted data as a JSON value
    async fn decrypt(&self, session_id: &str, data: &[u8], format: Option<EncryptionFormat>) -> Result<serde_json::Value>;
    
    /// Check if a user has a specific permission
    async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool;
    
    /// Get all permissions for a user
    async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission>;
    
    /// Assign a role to a user
    async fn assign_role(&self, user_id: String, role_id: String) -> Result<()>;
    
    /// Create a new role
    async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role>;
    
    /// Get roles for a user
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>>;
    
    /// Evaluate a security policy
    async fn evaluate_policy(&self, policy_id: &str, context: &policies::PolicyContext) -> Result<policies::PolicyEvaluationResult>;
    
    /// Add a security policy
    async fn add_policy(&self, policy: policies::SecurityPolicy) -> Result<()>;
    
    /// Get a security policy by ID
    async fn get_policy(&self, policy_id: &str) -> Result<policies::SecurityPolicy>;
    
    /// Generate a new encryption key for a specific format
    async fn generate_encryption_key(&self, format: EncryptionFormat) -> Result<Vec<u8>>;
}

// Add this implementation for Debug
impl std::fmt::Debug for dyn SecurityManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn SecurityManager")
    }
}

/// Implementation of the security manager
pub struct SecurityManagerImpl {
    /// RBAC manager
    rbac_manager: Arc<EnhancedRBACManager>,
    
    /// Policy manager
    policy_manager: Arc<PolicyManager>,
    
    /// Encryption manager
    encryption_manager: Arc<dyn Encryption>,
    
    /// Session encryption formats
    session_encryption_formats: HashMap<String, EncryptionFormat>,
}

impl SecurityManagerImpl {
    /// Create a new security manager
    #[must_use] pub fn new() -> Self {
        Self {
            rbac_manager: Arc::new(EnhancedRBACManager::new()),
            policy_manager: Arc::new(PolicyManager::new(true)),
            encryption_manager: encryption::create_encryption_manager(),
            session_encryption_formats: HashMap::new(),
        }
    }
    
    /// Create a new security manager with custom policy manager
    pub fn with_policy_manager(policy_manager: Arc<PolicyManager>) -> Self {
        Self {
            rbac_manager: Arc::new(EnhancedRBACManager::new()),
            policy_manager,
            encryption_manager: encryption::create_encryption_manager(),
            session_encryption_formats: HashMap::new(),
        }
    }
    
    /// Create a new security manager with custom components
    pub fn with_components(
        rbac_manager: Arc<EnhancedRBACManager>,
        policy_manager: Arc<PolicyManager>,
        encryption_manager: Arc<dyn Encryption>
    ) -> Self {
        Self {
            rbac_manager,
            policy_manager,
            encryption_manager,
            session_encryption_formats: HashMap::new(),
        }
    }
    
    /// Get the policy manager
    #[must_use] pub fn policy_manager(&self) -> Arc<PolicyManager> {
        self.policy_manager.clone()
    }
    
    /// Get the RBAC manager
    #[must_use] pub fn rbac_manager(&self) -> Arc<EnhancedRBACManager> {
        self.rbac_manager.clone()
    }
    
    /// Get the encryption manager
    #[must_use] pub fn encryption_manager(&self) -> Arc<dyn Encryption> {
        self.encryption_manager.clone()
    }
    
    /// Set the encryption format for a session
    pub fn set_session_encryption_format(&mut self, session_id: String, format: EncryptionFormat) {
        self.session_encryption_formats.insert(session_id, format);
    }
    
    /// Get the encryption format for a session (defaults to `Aes256Gcm`)
    fn get_session_encryption_format(&self, session_id: &str) -> EncryptionFormat {
        self.session_encryption_formats.get(session_id)
            .copied()
            .unwrap_or(EncryptionFormat::Aes256Gcm)
    }
}

#[async_trait]
impl SecurityManager for SecurityManagerImpl {
    async fn authenticate(&self, credentials: &Credentials) -> Result<String> {
        // For demonstration purposes, using a simple username-based authentication
        // In a real system, you would validate the credentials against a database
        if credentials.username.is_empty() {
            return Err(crate::error::MCPError::Security(crate::error::types::SecurityError::AuthenticationFailed("Username cannot be empty".to_string())));
        }
        
        // Here we would normally validate password hash, token validity, etc.
        // For now, just return the username as the user ID
        Ok(credentials.username.clone())
    }
    
    async fn authorize(&self, token: &str, security_level: SecurityLevel) -> Result<Session> {
        // Basic implementation of authorize
        // In a real system, this would validate the token, check expiration, etc.
        if token.is_empty() {
            return Err(crate::error::MCPError::Security(crate::error::types::SecurityError::AuthorizationFailed("Token cannot be empty".to_string())));
        }
        
        // Check security level
        if security_level == SecurityLevel::Critical {
            // For critical operations, we might require additional validation
            // For simplicity, we'll just reject all critical requests in this example
            return Err(crate::error::MCPError::Security(crate::error::types::SecurityError::InvalidSecurityLevel {
                required: SecurityLevel::Critical,
                provided: SecurityLevel::Standard,
            }));
        }
        
        // Create a basic session
        Ok(Session {
            token: SessionToken(token.to_string()),
            user_id: crate::types::UserId(token.to_string()), // Simple implementation, user ID from token
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        })
    }
    
    async fn encrypt(&self, session_id: &str, data: &serde_json::Value, format: Option<EncryptionFormat>) -> Result<Vec<u8>> {
        // Get the format to use (either provided or from session)
        let encryption_format = format.unwrap_or_else(|| self.get_session_encryption_format(session_id));
        
        // Convert the JSON data to bytes
        let data_bytes = serde_json::to_vec(data)
            .map_err(|e| crate::error::MCPError::Security(crate::error::types::SecurityError::EncryptionFailed(
                format!("Failed to serialize data: {e}")
            )))?;
        
        // Use the encryption manager to encrypt the data
        self.encryption_manager.encrypt(&data_bytes, encryption_format).await
    }
    
    async fn decrypt(&self, session_id: &str, data: &[u8], format: Option<EncryptionFormat>) -> Result<serde_json::Value> {
        // Get the format to use (either provided or from session)
        let encryption_format = format.unwrap_or_else(|| self.get_session_encryption_format(session_id));
        
        // Use the encryption manager to decrypt the data
        let decrypted_bytes = self.encryption_manager.decrypt(data, encryption_format).await?;
        
        // Parse the decrypted bytes as JSON
        serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| crate::error::MCPError::Security(crate::error::types::SecurityError::DecryptionFailed(
                format!("Failed to parse decrypted data as JSON: {e}")
            )))
    }
    
    async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        // Create a default context
        let _context = PermissionContext {
            user_id: user_id.to_string(),
            current_time: Some(chrono::Utc::now()),
            network_address: None,
            security_level: crate::types::SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        };
        
        // Check permission with context
        match self.rbac_manager.has_permission(
            user_id,
            &permission.resource,
            permission.action,
            &_context,
        ).await {
            Ok(result) => matches!(result, ValidationResult::Granted),
            Err(_) => false,
        }
    }
    
    async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        // Create a default context
        let _context = PermissionContext {
            user_id: user_id.to_string(),
            current_time: Some(chrono::Utc::now()),
            network_address: None,
            security_level: crate::types::SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        };
        
        // Get user role IDs
        let user_role_ids = self.rbac_manager.get_user_roles(user_id).await;
        let mut roles = Vec::new();
        
        // Get role objects from IDs
        for role_id in &user_role_ids {
            if let Ok(role) = self.rbac_manager.get_role(role_id).await {
                roles.push(role);
            }
        }
        
        // Collect all permissions, including inherited ones
        let mut all_permissions = HashSet::new();
        
        // Create role map for inheritance
        let _role_map: HashMap<String, Role> = roles
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();
            
        // Add direct permissions from roles and gather inherited ones
        for role in &roles {
            // Add direct permissions
            all_permissions.extend(role.permissions.clone());
            
            // Add inherited permissions through has_permission checks
            for permission in &role.permissions {
                if self.has_permission(user_id, permission).await {
                    all_permissions.insert(permission.clone());
                }
            }
        }
        
        all_permissions
    }
    
    async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        self.rbac_manager.assign_role(user_id, role_id).await
    }
    
    async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        self.rbac_manager.create_role(name, description, permissions, parent_roles).await
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>> {
        // Get the role IDs from the RBAC manager
        let role_ids = self.rbac_manager.get_user_roles(user_id).await;
        
        // Retrieve each role from the RBAC manager
        let mut roles = Vec::new();
        for role_id in &role_ids {
            if let Ok(role) = self.rbac_manager.get_role(role_id).await {
                roles.push(role);
            }
        }
        
        Ok(roles)
    }
    
    // Add new methods for policy management
    async fn evaluate_policy(&self, policy_id: &str, context: &policies::PolicyContext) -> Result<policies::PolicyEvaluationResult> {
        self.policy_manager.evaluate_policy(policy_id, context).await
    }
    
    async fn add_policy(&self, policy: policies::SecurityPolicy) -> Result<()> {
        self.policy_manager.add_policy(policy).await
    }
    
    async fn get_policy(&self, policy_id: &str) -> Result<policies::SecurityPolicy> {
        self.policy_manager.get_policy(policy_id).await
    }
    
    async fn generate_encryption_key(&self, format: EncryptionFormat) -> Result<Vec<u8>> {
        self.encryption_manager.generate_key(format).await
    }
}

impl Default for SecurityManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_manager_basic() {
        let manager = SecurityManagerImpl::new();
        
        // Create a test role
        let mut permissions = HashSet::new();
        permissions.insert(Permission {
            id: "test-permission".to_string(),
            name: "Test Permission".to_string(),
            resource: "test-resource".to_string(),
            action: Action::Read,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        });
        
        let role = manager.create_role(
            "TestRole".to_string(),
            Some("Test role".to_string()),
            permissions.clone(),
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        manager.assign_role("test-user".to_string(), role.id.clone()).await.unwrap();
        
        // Check permission
        let permission = Permission {
            id: "test-permission".to_string(),
            name: "Test Permission".to_string(),
            resource: "test-resource".to_string(),
            action: Action::Read,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        };
        
        assert!(manager.has_permission("test-user", &permission).await);
        
        // Get user roles
        let user_roles = manager.get_user_roles("test-user").await.unwrap();
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0].name, "TestRole");
    }
    
    #[tokio::test]
    async fn test_policy_integration() {
        let security = SecurityManagerImpl::new();
        
        // Register policy evaluators
        let policy_manager = security.policy_manager();
        policy_manager.add_evaluator(Arc::new(PasswordPolicyEvaluator::new())).await.unwrap();
        
        // Create a password policy
        let mut settings = HashMap::new();
        settings.insert("min_length".to_string(), "8".to_string());
        settings.insert("require_uppercase".to_string(), "true".to_string());
        
        let policy = SecurityPolicy {
            id: "password-policy".to_string(),
            name: "Password Policy".to_string(),
            description: Some("Password requirements".to_string()),
            policy_type: PolicyType::Password,
            enforcement_level: EnforcementLevel::Enforced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings,
            required_permissions: HashSet::new(),
            security_level: SecurityLevel::Standard,
            enabled: true,
        };
        
        // Add policy through the security manager
        security.add_policy(policy).await.unwrap();
        
        // Evaluate policy through security manager
        let mut context = PolicyContext::default();
        let mut request_info = HashMap::new();
        request_info.insert("password".to_string(), "Password123".to_string());
        context.request_info = request_info;
        
        let result = security.evaluate_policy("password-policy", &context).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PolicyEvaluationResult::Passed));
    }
    
    #[tokio::test]
    async fn test_encryption_integration() {
        let manager = SecurityManagerImpl::new();
        
        // Test data to encrypt
        let data = serde_json::json!({
            "username": "test_user",
            "email": "test@example.com",
            "attributes": {
                "role": "admin",
                "permissions": ["read", "write", "delete"]
            }
        });
        
        // Encrypt the data
        let encrypted = manager.encrypt("test-session", &data, Some(EncryptionFormat::Aes256Gcm)).await.unwrap();
        
        // The encrypted data should not be the same as the original JSON
        let original_bytes = serde_json::to_vec(&data).unwrap();
        assert_ne!(encrypted, original_bytes);
        
        // Decrypt the data
        let decrypted = manager.decrypt("test-session", &encrypted, Some(EncryptionFormat::Aes256Gcm)).await.unwrap();
        
        // The decrypted data should match the original
        assert_eq!(decrypted, data);
    }
    
    #[tokio::test]
    async fn test_generate_encryption_key() {
        let manager = SecurityManagerImpl::new();
        
        // Generate an AES-256-GCM key
        let aes_key = manager.generate_encryption_key(EncryptionFormat::Aes256Gcm).await.unwrap();
        assert_eq!(aes_key.len(), 32); // AES-256 uses 32-byte keys
        
        // Generate a ChaCha20-Poly1305 key
        let chacha_key = manager.generate_encryption_key(EncryptionFormat::ChaCha20Poly1305).await.unwrap();
        assert_eq!(chacha_key.len(), 32); // ChaCha20-Poly1305 uses 32-byte keys
    }
}
