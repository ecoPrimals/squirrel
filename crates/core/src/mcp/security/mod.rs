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

use crate::mcp::{MCPError, Result, SecurityError};
use crate::mcp::types::{SecurityLevel, EncryptionFormat};

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
    master_key: [u8; KEY_LEN],
    /// Map of session keys by session ID
    session_keys: Arc<RwLock<HashMap<String, SessionKey>>>,
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
    /// Creates a new security manager with the given configuration.
    /// 
    /// # Errors
    /// Returns an error if initialization fails
    pub async fn new(config: SecurityConfig) -> Result<Arc<Self>> {
        let mut key_manager = KeyManager::default();
        OsRng.fill_bytes(&mut key_manager.master_key);

        let manager = Arc::new(Self {
            config,
            state: Arc::new(RwLock::new(SecurityState::default())),
            key_manager,
            rbac_manager: Arc::new(RwLock::new(RBACManager::new())),
        });

        // Initialize default roles
        {
            let mut rbac = manager.rbac_manager.write().await;
            for role in &manager.config.default_roles {
                rbac.create_role(
                    role.name.clone(),
                    role.description.clone(),
                    role.permissions.clone(),
                    role.parent_roles.clone(),
                )?;
            }
        }

        Ok(manager)
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
                if let Err(e) = self.assign_role(credentials.client_id.clone(), role_id.as_str()).await {
                    tracing::warn!("Failed to assign role {}: {}", role_id, e);
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

    /// Assigns a role to a user.
    /// 
    /// # Errors
    /// Returns an error if the role assignment fails
    pub async fn assign_role(&self, user_id: String, role_id: &str) -> Result<()> {
        let mut rbac = self.rbac_manager.write().await;
        rbac.assign_role(user_id, role_id)
    }

    /// Checks if a user has a specific permission
    pub async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let rbac = self.rbac_manager.read().await;
        rbac.has_permission(user_id, permission)
    }

    /// Gets all permissions assigned to a user
    pub async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let rbac = self.rbac_manager.read().await;
        rbac.get_user_permissions(user_id)
    }

    /// Creates a new role with the specified parameters.
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Role name is already taken
    /// - Parent roles don't exist
    pub async fn create_role(&self, name: String, description: Option<String>, permissions: HashSet<Permission>, parent_roles: HashSet<String>) -> Result<Role> {
        let mut rbac = self.rbac_manager.write().await;
        rbac.create_role(name, description, permissions, parent_roles)
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
        
        // Create user role with read permission
        let mut user_permissions = HashSet::new();
        user_permissions.insert(Permission {
            id: Uuid::new_v4().to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        });

        let user_role = Role {
            id: Uuid::new_v4().to_string(),
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions.clone(),
            parent_roles: HashSet::new(),
        };

        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();

        // Get the role ID from the RBAC manager
        let role_id = security.rbac_manager.read().await.get_role_by_name("user").unwrap().id.clone();

        // Authenticate with role request
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![role_id]),
        };

        // Use the token to ensure it's not marked as unused
        let _token = security.authenticate(&credentials).await.unwrap();

        // Check permissions
        let read_permission = Permission {
            id: Uuid::new_v4().to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };

        assert!(security.has_permission(&credentials.client_id, &read_permission).await);
    }

    #[tokio::test]
    async fn test_authentication_with_roles() {
        let mut config = SecurityConfig::default();
        
        // Create default user role
        let mut user_permissions = HashSet::new();
        user_permissions.insert(Permission {
            id: Uuid::new_v4().to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        });

        let user_role = Role {
            id: Uuid::new_v4().to_string(),
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions,
            parent_roles: HashSet::new(),
        };

        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();

        // Get the role ID from the RBAC manager
        let role_id = security.rbac_manager.read().await.get_role_by_name("user").unwrap().id.clone();

        // Authenticate with role request
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![role_id]),
        };

        // Use the token to ensure it's not marked as unused
        let _token = security.authenticate(&credentials).await.unwrap();

        // Verify role assignment
        let read_permission = Permission {
            id: Uuid::new_v4().to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };

        assert!(security.has_permission(&credentials.client_id, &read_permission).await);
    }

    #[tokio::test]
    async fn test_authorization_with_permission() {
        let mut config = SecurityConfig::default();
        
        // Create user role with read permission
        let mut user_permissions = HashSet::new();
        let read_permission = Permission {
            id: Uuid::new_v4().to_string(),
            name: "read".to_string(),
            resource: "document".to_string(),
            action: Action::Read,
        };
        user_permissions.insert(read_permission.clone());

        let user_role = Role {
            id: Uuid::new_v4().to_string(),
            name: "user".to_string(),
            description: Some("Basic user".to_string()),
            permissions: user_permissions,
            parent_roles: HashSet::new(),
        };

        config.default_roles.push(user_role.clone());
        let security = SecurityManager::new(config).await.unwrap();

        // Get the role ID from the RBAC manager
        let role_id = security.rbac_manager.read().await.get_role_by_name("user").unwrap().id.clone();

        // Authenticate user
        let credentials = Credentials {
            client_id: "test_user".to_string(),
            client_secret: "secret".to_string(),
            security_level: SecurityLevel::Standard,
            requested_roles: Some(vec![role_id]),
        };

        let token = security.authenticate(&credentials).await.unwrap();

        // Test authorization with permission
        assert!(security.authorize(&token, SecurityLevel::Standard, Some(&read_permission)).await.is_ok());

        // Test authorization with invalid permission
        let write_permission = Permission {
            id: Uuid::new_v4().to_string(),
            name: "write".to_string(),
            resource: "document".to_string(),
            action: Action::Create,
        };

        assert!(security.authorize(&token, SecurityLevel::Standard, Some(&write_permission)).await.is_err());
    }
} 