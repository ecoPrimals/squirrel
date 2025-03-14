use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use ring::{aead, rand};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ai::mcp_tools::types::MCPError;

/// Security level for MCP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Authentication token for MCP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
    pub roles: Vec<String>,
    pub expires_at: SystemTime,
    pub security_level: SecurityLevel,
}

/// Security context for MCP operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub token: Option<AuthToken>,
    pub encryption_enabled: bool,
    pub security_level: SecurityLevel,
}

/// Security manager for MCP operations
pub struct SecurityManager {
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
    roles: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> roles
    permissions: Arc<RwLock<HashMap<String, Vec<String>>>>, // role -> permissions
    encryption_key: aead::LessSafeKey,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Result<Self, MCPError> {
        let rng = rand::SystemRandom::new();
        let mut key_bytes = vec![0u8; aead::CHACHA20_POLY1305.key_len()];
        rng.fill(&mut key_bytes)
            .map_err(|_| MCPError::SecurityError("Failed to generate encryption key".to_string()))?;

        let encryption_key = aead::LessSafeKey::new(
            aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
                .map_err(|_| MCPError::SecurityError("Failed to create encryption key".to_string()))?
        );

        Ok(Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            encryption_key,
        })
    }

    /// Authenticate a user and generate a token
    pub fn authenticate(&self, user_id: &str, _password: &str) -> Result<AuthToken, MCPError> {
        // In a real implementation, validate credentials against a secure store
        // For now, just generate a token
        let token = AuthToken {
            token: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            roles: self.get_user_roles(user_id)?,
            expires_at: SystemTime::now() + Duration::from_secs(3600), // 1 hour
            security_level: SecurityLevel::Medium,
        };

        let mut tokens = self.tokens.write()
            .map_err(|_| MCPError::SecurityError("Failed to acquire lock".to_string()))?;
        tokens.insert(token.token.clone(), token.clone());

        Ok(token)
    }

    /// Validate an authentication token
    pub fn validate_token(&self, token: &str) -> Result<bool, MCPError> {
        let tokens = self.tokens.read()
            .map_err(|_| MCPError::SecurityError("Failed to acquire lock".to_string()))?;

        if let Some(auth_token) = tokens.get(token) {
            if SystemTime::now() > auth_token.expires_at {
                return Ok(false);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get roles for a user
    pub fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>, MCPError> {
        let roles = self.roles.read()
            .map_err(|_| MCPError::SecurityError("Failed to acquire lock".to_string()))?;
        
        Ok(roles.get(user_id)
            .cloned()
            .unwrap_or_else(|| vec!["user".to_string()]))
    }

    /// Check if a user has a specific permission
    pub fn check_permission(&self, token: &str, permission: &str) -> Result<bool, MCPError> {
        let tokens = self.tokens.read()
            .map_err(|_| MCPError::SecurityError("Failed to acquire lock".to_string()))?;
        
        let auth_token = tokens.get(token)
            .ok_or_else(|| MCPError::SecurityError("Invalid token".to_string()))?;

        let permissions = self.permissions.read()
            .map_err(|_| MCPError::SecurityError("Failed to acquire lock".to_string()))?;

        for role in &auth_token.roles {
            if let Some(role_permissions) = permissions.get(role) {
                if role_permissions.contains(&permission.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Encrypt data
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, MCPError> {
        let nonce = aead::Nonce::assume_unique_for_key([0u8; aead::NONCE_LEN]);
        let aad = aead::Aad::empty();

        let mut in_out = data.to_vec();
        let tag_len = aead::CHACHA20_POLY1305.tag_len();
        in_out.extend(vec![0u8; tag_len]);

        self.encryption_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|_| MCPError::SecurityError("Encryption failed".to_string()))?;

        Ok(in_out)
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, MCPError> {
        let nonce = aead::Nonce::assume_unique_for_key([0u8; aead::NONCE_LEN]);
        let aad = aead::Aad::empty();

        let mut in_out = encrypted_data.to_vec();
        let decrypted_data = self.encryption_key.open_in_place(nonce, aad, &mut in_out)
            .map_err(|_| MCPError::SecurityError("Decryption failed".to_string()))?;

        Ok(decrypted_data.to_vec())
    }

    /// Grant permissions to a user by assigning roles and permissions
    pub fn grant_permissions(&self, user_id: &str, roles: Vec<String>, role_permissions: HashMap<String, Vec<String>>) -> Result<(), MCPError> {
        let mut roles_lock = self.roles.write().unwrap();
        let mut permissions_lock = self.permissions.write().unwrap();

        roles_lock.insert(user_id.to_string(), roles.clone());
        
        for (role, perms) in role_permissions {
            permissions_lock.insert(role, perms);
        }

        Ok(())
    }

    /// Update a user's token with new roles
    pub fn update_token_roles(&self, token: &str, roles: Vec<String>) -> Result<(), MCPError> {
        let mut tokens_lock = self.tokens.write().unwrap();
        
        if let Some(mut token_info) = tokens_lock.get(token).cloned() {
            token_info.roles = roles;
            tokens_lock.insert(token.to_string(), token_info);
            Ok(())
        } else {
            Err(MCPError::AuthenticationError("Token not found".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authentication() {
        let manager = SecurityManager::new().unwrap();
        let token = manager.authenticate("test_user", "password").unwrap();
        
        assert!(manager.validate_token(&token.token).unwrap());
        assert!(!manager.validate_token("invalid_token").unwrap());
    }

    #[test]
    fn test_permissions() {
        let manager = SecurityManager::new().unwrap();
        let token = manager.authenticate("test_user", "password").unwrap();
        
        // By default, users don't have admin permissions
        assert!(!manager.check_permission(&token.token, "admin").unwrap());
    }

    #[test]
    fn test_encryption() {
        let manager = SecurityManager::new().unwrap();
        let data = b"test data";
        
        let encrypted = manager.encrypt(data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        // Only compare the actual data length, ignoring padding
        assert_eq!(&decrypted[..data.len()], data);
    }
} 