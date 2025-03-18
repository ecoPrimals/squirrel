//! Security module for MCP (Machine Context Protocol)
//! 
//! This module provides authentication, authorization, and encryption services
//! for secure communication between MCP components.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use ring::aead::{self, BoundKey, Nonce, NonceSequence, UnboundKey, AES_256_GCM};
use serde::{Serialize, Deserialize};
use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::mcp::types::{SecurityLevel, EncryptionFormat};
use crate::mcp::error::types::{MCPError, SecurityError};
use crate::mcp::error::Result;
use crate::error::SquirrelError;

/// Role-Based Access Control (RBAC) implementation
pub mod rbac;
pub use rbac::{Role, Permission, Action, RBACManager};

/// Length of credential values in bytes
#[allow(dead_code)]
const CREDENTIAL_LEN: usize = 32;

/// Length of salt values in bytes
#[allow(dead_code)]
const SALT_LEN: usize = 16;

/// Length of nonce values in bytes
const NONCE_LEN: usize = 12;

/// Length of encryption keys in bytes
const KEY_LEN: usize = 32;

/// Token validity duration in seconds (1 hour)
const TOKEN_VALIDITY: i64 = 3600;

/// Configuration for the security manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Minimum security level required for authentication
    pub min_security_level: SecurityLevel,
    /// Format used for encryption
    pub encryption_format: EncryptionFormat,
    /// Token validity duration in seconds
    pub token_validity: i64,
    /// Maximum number of failed authentication attempts
    pub max_auth_attempts: u32,
    /// Default roles assigned to new users
    pub default_roles: Vec<Role>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            min_security_level: SecurityLevel::Standard,
            encryption_format: EncryptionFormat::Aes256Gcm,
            token_validity: TOKEN_VALIDITY,
            max_auth_attempts: 3,
            default_roles: vec![],
        }
    }
}

/// Manages security operations including authentication, authorization, and encryption
#[derive(Debug)]
pub struct SecurityManager {
    /// Security configuration settings
    config: SecurityConfig,
    /// Thread-safe security state storage
    state: Arc<RwLock<SecurityState>>,
    /// Manager for encryption keys
    key_manager: KeyManager,
    /// Role-based access control manager
    rbac_manager: Arc<RwLock<RBACManager>>,
}

/// Internal security state
#[derive(Debug, Default)]
struct SecurityState {
    /// Currently active sessions
    active_sessions: Vec<Session>,
    /// Map of authentication attempts by client ID
    auth_attempts: HashMap<String, AuthAttempt>,
}

/// Authentication attempt tracking
#[derive(Debug, Clone)]
struct AuthAttempt {
    /// Number of failed attempts
    count: u32,
    /// Timestamp of last attempt
    last_attempt: DateTime<Utc>,
}

/// Key management for encryption operations
#[derive(Debug, Default)]
struct KeyManager {
    /// Master encryption key
    #[allow(dead_code)]
    master_key: [u8; KEY_LEN],
    /// Map of session keys by session ID
    session_keys: Arc<RwLock<HashMap<String, SessionKey>>>,
}

impl KeyManager {
    /// Creates a new key manager with a random master key
    fn new() -> Self {
        let mut master_key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut master_key);
        
        Self {
            master_key,
            session_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Represents an authenticated user session
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique identifier for the session
    pub id: String,
    /// Authentication token used for subsequent requests
    pub token: String,
    /// Identifier of the client that owns this session
    pub client_id: String,
    /// Security level assigned to this session
    pub security_level: SecurityLevel,
    /// Timestamp when the session was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the session will expire
    pub expires_at: DateTime<Utc>,
}

/// Session-specific encryption key
#[derive(Debug, Clone)]
struct SessionKey {
    /// Encryption key bytes
    key: [u8; KEY_LEN],
    /// Creation timestamp
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    /// Expiration timestamp
    expires_at: DateTime<Utc>,
}

/// Nonce generator for encryption operations
#[derive(Debug)]
struct NonceGen {
    /// Current nonce value
    nonce: [u8; NONCE_LEN],
}

impl NonceGen {
    /// Creates a new nonce generator with the given initial nonce.
    const fn new(nonce: [u8; NONCE_LEN]) -> Self {
        Self { nonce }
    }
}

impl NonceSequence for NonceGen {
    fn advance(&mut self) -> std::result::Result<Nonce, ring::error::Unspecified> {
        Ok(Nonce::assume_unique_for_key(self.nonce))
    }
}

impl SecurityManager {
    /// Creates a new security manager
    pub async fn new(config: SecurityConfig) -> Result<Arc<Self>> {
        // Create key manager
        let key_manager = KeyManager::new();
        
        // Create RBAC manager
        let mut rbac_manager = RBACManager::new();
        
        // Initialize default roles
        for role in &config.default_roles {
            // Create role with parent relationships
            let parent_roles = role.parent_roles.clone();
            let permissions = role.permissions.clone();
            
            let role_result = rbac_manager.create_role_with_id(
                role.id.clone(),
                role.name.clone(),
                role.description.clone(),
                permissions,
                parent_roles,
            );
            
            // Convert error if needed
            if let Err(e) = role_result {
                return Err(match e {
                    crate::error::SquirrelError::Security(msg) => 
                        MCPError::Security(SecurityError::InvalidRole(msg)),
                    _ => MCPError::Security(SecurityError::InvalidRole(format!("{}", e))),
                });
            }
        }
        
        let state = SecurityState {
            active_sessions: Vec::new(),
            auth_attempts: HashMap::new(),
        };
        
        Ok(Arc::new(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            key_manager,
            rbac_manager: Arc::new(RwLock::new(rbac_manager)),
        }))
    }

    /// Authenticates a client using the provided credentials.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Authentication attempts are exceeded
    /// - Credentials are invalid
    /// - Session creation fails
    pub async fn authenticate(&self, credentials: &Credentials) -> Result<String> {
        // Check auth attempts
        self.check_auth_attempts(credentials).await?;

        // Verify credentials
        if !Self::verify_credentials(credentials) {
            self.record_failed_attempt(credentials).await?;
            return Err(MCPError::Security(SecurityError::AuthenticationFailed(
                format!("Too many failed attempts for client {}", credentials.client_id)
            )));
        }

        // Create session
        let session = self.create_session(credentials).await?;
        
        // Generate session key
        self.generate_session_key(&session.id).await?;

        // Assign requested roles if provided
        if let Some(roles) = &credentials.requested_roles {
            for role_id in roles {
                if let Err(e) = self.assign_role(credentials.client_id.clone(), role_id.to_string()).await {
                    tracing::warn!("Failed to assign role: {}", e);
                }
            }
        }

        Ok(session.token)
    }

    /// Authorizes a session for the required security level and permission.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Session is not found
    /// - Session has expired
    /// - Security level is insufficient
    /// - Required permission is not granted
    pub async fn authorize(&self, token: &str, required_level: SecurityLevel, required_permission: Option<&Permission>) -> Result<Session> {
        let session = {
            let state = self.state.read().await;
            state
                .active_sessions
                .iter()
                .find(|s| s.token == token)
                .cloned()
                .ok_or_else(|| MCPError::Security(SecurityError::InvalidToken("Session token not found".to_string())))?
        };

        // Verify session hasn't expired
        if session.expires_at < Utc::now() {
            return Err(MCPError::Security(SecurityError::TokenExpired));
        }

        // Verify security level
        if session.security_level < required_level {
            return Err(MCPError::Security(SecurityError::InvalidSecurityLevel {
                required: required_level,
                provided: session.security_level,
            }));
        }

        // Verify permission if required
        if let Some(permission) = required_permission {
            let rbac = self.rbac_manager.read().await;
            if !rbac.has_permission(&session.client_id, permission) {
                return Err(MCPError::Security(SecurityError::AuthorizationFailed(
                    format!("Missing required permission: {permission:?}")
                )));
            }
        }

        Ok(session)
    }

    /// Encrypts data using the session's encryption key.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Session key is not found
    /// - Encryption operation fails
    pub async fn encrypt(&self, session_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_session_key(session_id).await?;
        
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        let unbound_key = UnboundKey::new(&AES_256_GCM, &key.key)
            .map_err(|_| MCPError::Security(SecurityError::EncryptionFailed("Invalid key".into())))?;

        let nonce_gen = NonceGen::new(nonce);
        let mut sealing_key = aead::SealingKey::new(unbound_key, nonce_gen);
        
        let mut in_out = data.to_vec();
        let tag = sealing_key.seal_in_place_separate_tag(aead::Aad::empty(), &mut in_out)
            .map_err(|_| MCPError::Security(SecurityError::EncryptionFailed("Encryption failed".into())))?;

        let mut result = Vec::with_capacity(NONCE_LEN + in_out.len() + AES_256_GCM.tag_len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&in_out);
        result.extend_from_slice(tag.as_ref());

        Ok(result)
    }

    /// Decrypts data using the session's encryption key.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Session key is not found
    /// - Decryption operation fails
    pub async fn decrypt(&self, session_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < NONCE_LEN + AES_256_GCM.tag_len() {
            return Err(MCPError::Security(SecurityError::DecryptionFailed("Invalid data length".into())));
        }

        let key = self.get_session_key(session_id).await?;
        let nonce = &data[..NONCE_LEN];
        
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key.key)
            .map_err(|_| MCPError::Security(SecurityError::DecryptionFailed("Invalid key".into())))?;

        let nonce_gen = NonceGen::new(nonce.try_into().map_err(|_| MCPError::Security(SecurityError::DecryptionFailed("Invalid nonce".into())))?);
        let mut opening_key = aead::OpeningKey::new(unbound_key, nonce_gen);

        let mut in_out = data[NONCE_LEN..].to_vec();
        opening_key.open_in_place(aead::Aad::empty(), &mut in_out)
            .map_err(|_| MCPError::Security(SecurityError::DecryptionFailed("Decryption failed".into())))?;

        Ok(in_out[..in_out.len() - AES_256_GCM.tag_len()].to_vec())
    }

    /// Checks authentication attempts for rate limiting.
    async fn check_auth_attempts(&self, credentials: &Credentials) -> Result<()> {
        let attempt = {
            let state = self.state.read().await;
            state.auth_attempts.get(&credentials.client_id).cloned()
        };

        if let Some(attempt) = attempt {
            if attempt.count >= self.config.max_auth_attempts {
                return Err(MCPError::Security(SecurityError::AuthenticationFailed(
                    format!("Too many failed attempts for client {}", credentials.client_id)
                )));
            }
        }

        Ok(())
    }

    /// Records a failed authentication attempt.
    async fn record_failed_attempt(&self, credentials: &Credentials) -> Result<()> {
        let mut state = self.state.write().await;
        {
            let attempt = state.auth_attempts
                .entry(credentials.client_id.clone())
                .or_insert_with(|| AuthAttempt {
                    count: 0,
                    last_attempt: Utc::now(),
                });
            attempt.count += 1;
            attempt.last_attempt = Utc::now();
        }
        Ok(())
    }

    /// Creates a new session for authenticated credentials.
    async fn create_session(&self, credentials: &Credentials) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            token: Uuid::new_v4().to_string(),
            client_id: credentials.client_id.clone(),
            security_level: credentials.security_level,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(TOKEN_VALIDITY),
        };

        self.state.write().await.active_sessions.push(session.clone());

        Ok(session)
    }

    /// Generates a new session encryption key.
    async fn generate_session_key(&self, session_id: &str) -> Result<()> {
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);

        let session_key = SessionKey {
            key,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(TOKEN_VALIDITY),
        };

        self.key_manager.session_keys.write().await
            .insert(session_id.to_string(), session_key);

        Ok(())
    }

    /// Retrieves the encryption key for a session.
    async fn get_session_key(&self, session_id: &str) -> Result<SessionKey> {
        let keys = self.key_manager.session_keys.read().await;
        keys.get(session_id)
            .cloned()
            .ok_or_else(|| MCPError::Security(SecurityError::InvalidToken("Session token not found".to_string())))
    }

    /// Removes expired sessions and session keys.
    /// 
    /// # Errors
    /// Returns an error if the cleanup operation fails
    pub async fn cleanup_expired_sessions(&self) -> Result<()> {
        let now = Utc::now();

        {
            let mut state = self.state.write().await;
            state.active_sessions.retain(|session| session.expires_at > now);
        }

        {
            let mut keys = self.key_manager.session_keys.write().await;
            keys.retain(|_, key| key.expires_at > now);
        }

        Ok(())
    }

    /// Assigns a role to a user
    pub async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        let mut rbac_manager = self.rbac_manager.write().await;
        rbac_manager.assign_role(user_id, role_id)
            .map_err(|e| match e {
                SquirrelError::Security(msg) => MCPError::Security(SecurityError::InvalidCredentials(msg)),
                _ => MCPError::Security(SecurityError::InvalidCredentials(format!("{}", e))),
            })
    }

    /// Assigns a role to a user by name
    pub async fn assign_role_by_name(&self, user_id: String, role_name: String) -> Result<()> {
        let mut rbac_manager = self.rbac_manager.write().await;
        rbac_manager.assign_role_by_name(user_id, role_name)
            .map_err(|e| match e {
                SquirrelError::Security(msg) => MCPError::Security(SecurityError::InvalidCredentials(msg)),
                _ => MCPError::Security(SecurityError::InvalidCredentials(format!("{}", e))),
            })
    }

    /// Checks if a user has a specific permission
    pub async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        // Use a read lock since we're not modifying the RBAC manager
        let rbac_manager = self.rbac_manager.read().await;
        rbac_manager.has_permission(user_id, permission)
    }

    /// Gets all permissions for a user
    pub async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let rbac_manager = self.rbac_manager.read().await;
        rbac_manager.get_user_permissions(user_id)
    }

    /// Creates a new role
    pub async fn create_role(
        &self, 
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        let mut rbac_manager = self.rbac_manager.write().await;
        rbac_manager.create_role(name, description, permissions, parent_roles)
            .map_err(|e| match e {
                SquirrelError::Security(msg) => MCPError::Security(SecurityError::InvalidRole(msg)),
                _ => MCPError::Security(SecurityError::InvalidRole(format!("{}", e))),
            })
    }

    /// Creates a role with a specific ID (useful for testing)
    pub async fn create_role_with_id(
        &self, 
        id: String,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        let mut rbac_manager = self.rbac_manager.write().await;
        rbac_manager.create_role_with_id(id, name, description, permissions, parent_roles)
            .map_err(|e| match e {
                SquirrelError::Security(msg) => MCPError::Security(SecurityError::InvalidRole(msg)),
                _ => MCPError::Security(SecurityError::InvalidRole(format!("{}", e))),
            })
    }

    /// Gets a role by ID
    pub async fn get_role_by_id(&self, id: &str) -> Option<Role> {
        let rbac_manager = self.rbac_manager.read().await;
        rbac_manager.get_role_by_id(id).cloned()
    }

    /// Gets a role by name
    pub async fn get_role_by_name(&self, name: &str) -> Option<Role> {
        let rbac_manager = self.rbac_manager.read().await;
        rbac_manager.get_role_by_name(name).cloned()
    }

    /// Verifies user credentials.
    /// 
    /// For testing purposes, this always returns true.
    /// In a real implementation, this would verify against a user database.
    #[must_use]
    const fn verify_credentials(_credentials: &Credentials) -> bool {
        true
    }
}

/// User credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Unique identifier for the client
    pub client_id: String,
    /// Secret used for authentication
    pub client_secret: String,
    /// Required security level
    pub security_level: SecurityLevel,
    /// Optional roles requested during authentication
    pub requested_roles: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::SecurityLevel;

    #[tokio::test]
    async fn test_authentication() {
        let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: None,
        };

        let token = security.authenticate(&credentials).await.unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_authorization() {
        let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::High,
            requested_roles: None,
        };

        let token = security.authenticate(&credentials).await.unwrap();
        assert!(security.authorize(&token, SecurityLevel::Standard, None).await.is_ok());
        assert!(security.authorize(&token, SecurityLevel::High, None).await.is_ok());
        assert!(security.authorize(&token, SecurityLevel::Maximum, None).await.is_err());
    }

    #[tokio::test]
    async fn test_encryption() {
        let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: None,
        };

        let token = security.authenticate(&credentials).await.unwrap();
        let session = security.authorize(&token, SecurityLevel::Standard, None).await.unwrap();

        let data = b"test data";
        let encrypted = security.encrypt(&session.id, data).await.unwrap();
        let decrypted = security.decrypt(&session.id, &encrypted).await.unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }

    #[tokio::test]
    async fn test_rbac_integration() {
        let mut config = SecurityConfig::default();
        
        // Create default user role with read permission
        let mut user_permissions = HashSet::new();
        let read_permission = Permission {
            id: "read-doc-1".to_string(), // Use a stable ID for testing
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };
        user_permissions.insert(read_permission.clone());
        
        let user_role = Role {
            id: "user-role-1".to_string(), // Use a stable ID for testing
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions,
            parent_roles: HashSet::new(),
        };
        
        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();
        
        // Authenticate with role request
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![user_role.id.clone()]),
        };
        
        // Use the token to ensure it's not marked as unused
        let _token = security.authenticate(&credentials).await.unwrap();
        
        // Check permissions
        assert!(security.has_permission(&credentials.client_id, &read_permission).await);
    }
    
    #[tokio::test]
    async fn test_authentication_with_roles() {
        let mut config = SecurityConfig::default();
        
        // Create user role with stable ID
        let mut user_permissions = HashSet::new();
        let read_permission = Permission {
            id: "read-doc-2".to_string(), // Use a stable ID for testing
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };
        user_permissions.insert(read_permission.clone());
        
        let user_role = Role {
            id: "user-role-2".to_string(), // Use a stable ID for testing
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions,
            parent_roles: HashSet::new(),
        };
        
        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();
        
        // Authenticate with role request
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![user_role.id.clone()]),
        };
        
        // Use the token to ensure it's not marked as unused
        let _token = security.authenticate(&credentials).await.unwrap();
        
        // Verify permission assignment
        assert!(security.has_permission(&credentials.client_id, &read_permission).await);
    }
    
    #[tokio::test]
    async fn test_authorization_with_permission() {
        let mut config = SecurityConfig::default();
        
        // Create user role with read permission and stable ID
        let mut user_permissions = HashSet::new();
        let read_permission = Permission {
            id: "read-doc-3".to_string(), // Use a stable ID for testing
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };
        user_permissions.insert(read_permission.clone());
        
        let user_role = Role {
            id: "user-role-3".to_string(), // Use a stable ID for testing
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions,
            parent_roles: HashSet::new(),
        };
        
        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();
        
        // Authenticate user
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![user_role.id.clone()]),
        };
        
        let token = security.authenticate(&credentials).await.unwrap();
        
        // Test authorization with permission
        assert!(security.authorize(&token, SecurityLevel::Standard, Some(&read_permission)).await.is_ok());
        
        // Test authorization with invalid permission
        let write_permission = Permission {
            id: "write-doc-3".to_string(),
            name: "write".to_string(),
            resource: "document".to_string(),
            action: Action::Create,
        };
        
        assert!(security.authorize(&token, SecurityLevel::Standard, Some(&write_permission)).await.is_err());
    }
    
    // Add a test for role inheritance with our refactored RBAC
    #[tokio::test]
    async fn test_role_inheritance_integration() {
        let config = SecurityConfig::default();
        let security = SecurityManager::new(config).await.unwrap();
        
        // Create base role with read permission
        let mut base_permissions = HashSet::new();
        let read_permission = Permission {
            id: "read-doc-4".to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };
        base_permissions.insert(read_permission.clone());
        
        let base_role = security.create_role_with_id(
            "reader-role-4".to_string(),
            "reader".to_string(),
            None,
            base_permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Create admin role with write permission, inheriting from base role
        let mut admin_permissions = HashSet::new();
        let write_permission = Permission {
            id: "write-doc-4".to_string(),
            name: "write".to_string(),
            resource: "document".to_string(),
            action: Action::Create,
        };
        admin_permissions.insert(write_permission.clone());
        
        let mut parent_roles = HashSet::new();
        parent_roles.insert(base_role.id.clone());
        
        let admin_role = security.create_role_with_id(
            "admin-role-4".to_string(),
            "admin".to_string(),
            None,
            admin_permissions,
            parent_roles,
        ).await.unwrap();
        
        // Assign admin role to user
        let user_id = "test_admin_user";
        security.assign_role(user_id.to_string(), admin_role.id.clone()).await.unwrap();
        
        // Check permissions
        assert!(security.has_permission(user_id, &read_permission).await);
        assert!(security.has_permission(user_id, &write_permission).await);
        
        // User should have access to both permissions
        let user_permissions = security.get_user_permissions(user_id).await;
        assert_eq!(user_permissions.len(), 2);
    }
} 