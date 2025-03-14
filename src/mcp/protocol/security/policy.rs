use ring::{aead, rand};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::mpsc;
use anyhow::Result;
use crate::mcp::protocol::{MCPMessage, SecurityLevel, SecurityMetadata};

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Token error: {0}")]
    Token(String),

    #[error("Key error: {0}")]
    Key(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Permission error: {0}")]
    Permission(String),

    #[error("Key rotation error: {0}")]
    KeyRotation(String),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Ring(#[from] ring::error::Unspecified),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub key_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: String,
    pub iv: Vec<u8>,
    pub tag: Vec<u8>,
}

#[derive(Debug)]
pub struct SecurityManager {
    keys: TokioRwLock<KeyManager>,
    tokens: TokioRwLock<TokenManager>,
    permissions: TokioRwLock<PermissionManager>,
    claims: Arc<TokioRwLock<HashMap<String, Claims>>>,
}

#[derive(Debug)]
struct KeyManager {
    current_key: aead::LessSafeKey,
    key_id: String,
    last_rotation: DateTime<Utc>,
    keys: HashMap<String, KeyInfo>,
}

impl Default for KeyManager {
    fn default() -> Self {
        Self {
            current_key: aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, &[0u8; 32]).unwrap()),
            key_id: Uuid::new_v4().to_string(),
            last_rotation: Utc::now(),
            keys: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct TokenManager {
    active_tokens: HashMap<String, TokenInfo>,
    jwt_key: String,
    tokens: HashMap<String, TokenInfo>,
}

impl Default for TokenManager {
    fn default() -> Self {
        Self {
            active_tokens: HashMap::new(),
            jwt_key: Uuid::new_v4().to_string(),
            tokens: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct TokenInfo {
    user_id: String,
    expires_at: DateTime<Utc>,
    roles: Vec<String>,
}

#[derive(Debug)]
struct PermissionManager {
    role_permissions: HashMap<String, Vec<String>>,
    permissions: HashMap<String, PermissionInfo>,
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self {
            role_permissions: HashMap::new(),
            permissions: HashMap::new(),
        }
    }
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            keys: TokioRwLock::new(KeyManager::default()),
            tokens: TokioRwLock::new(TokenManager::default()),
            permissions: TokioRwLock::new(PermissionManager::default()),
            claims: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    #[instrument(skip(self, data))]
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let keys = self.keys.read().await;
        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]); // Use a proper nonce in production

        let mut encrypted_data = data.to_vec();
        keys.current_key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut encrypted_data)
            .map_err(|e| SecurityError::Encryption(format!("Encryption failed: {:?}", e)))?;

        Ok(encrypted_data)
    }

    #[instrument(skip(self, data))]
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let keys = self.keys.read().await;
        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]); // Use a proper nonce in production

        let mut decrypted_data = data.to_vec();
        keys.current_key
            .open_in_place(nonce, aead::Aad::empty(), &mut decrypted_data)
            .map_err(|e| SecurityError::Decryption(format!("Decryption failed: {:?}", e)))?;

        Ok(decrypted_data)
    }

    #[instrument(skip(self))]
    pub async fn create_token(&self, user_id: &str, roles: Vec<String>) -> Result<String, SecurityError> {
        let tokens = self.tokens.read().await;
        let expiration = Utc::now() + Duration::hours(1);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(tokens.jwt_key.as_bytes()),
        )?;

        Ok(token)
    }

    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> Result<bool, SecurityError> {
        let claims = self.verify_token(token).await?;
        // Add validation logic here
        Ok(!claims.is_expired())
    }

    #[instrument(skip(self))]
    pub async fn check_permission(&self, user_roles: &[String], required_permission: &str) -> Result<bool, SecurityError> {
        let permissions = self.permissions.read().await;
        
        for role in user_roles {
            if let Some(role_permissions) = permissions.role_permissions.get(role) {
                if role_permissions.contains(&required_permission.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[instrument(skip(self))]
    pub async fn rotate_keys(&self) -> Result<(), SecurityError> {
        let mut keys = self.keys.write().await;
        
        // Generate new key
        let rng = rand::SystemRandom::new();
        let key_bytes: [u8; 32] = rand::generate(&rng)?.expose();
        let new_key = aead::LessSafeKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
                .map_err(|_| SecurityError::Key("Failed to create encryption key".into()))?
        );

        // Update key information
        keys.current_key = new_key;
        keys.key_id = Uuid::new_v4().to_string();
        keys.last_rotation = Utc::now();

        info!("Encryption keys rotated successfully");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn add_role_permission(&self, role: &str, permission: &str) -> Result<(), SecurityError> {
        let mut permissions = self.permissions.write().await;
        
        permissions.role_permissions
            .entry(role.to_string())
            .or_insert_with(Vec::new)
            .push(permission.to_string());

        info!(role = role, permission = permission, "Added permission to role");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn remove_role_permission(&self, role: &str, permission: &str) -> Result<(), SecurityError> {
        let mut permissions = self.permissions.write().await;
        
        if let Some(role_permissions) = permissions.role_permissions.get_mut(role) {
            role_permissions.retain(|p| p != permission);
            info!(role = role, permission = permission, "Removed permission from role");
        }

        Ok(())
    }

    pub async fn validate_encryption(&self, encryption: &EncryptionInfo) -> Result<bool, SecurityError> {
        // Validate encryption parameters
        if encryption.algorithm.is_empty() {
            return Ok(false);
        }
        if encryption.key_id.is_empty() {
            return Ok(false);
        }
        // Add more validation as needed
        Ok(true)
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims, SecurityError> {
        let tokens = self.tokens.read().await;
        
        let validation = Validation::default();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(tokens.jwt_key.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub async fn verify_message(&self, message: &MCPMessage) -> Result<(), SecurityError> {
        if let Some(security) = &message.security {
            // Verify security token
            let claims = self.verify_token(security).await?;
            
            // Store claims for future reference
            let mut claims_map = self.claims.write().await;
            claims_map.insert(message.id.clone(), claims);
        }
        Ok(())
    }

    pub async fn get_claims(&self, message_id: &str) -> Option<Claims> {
        let claims = self.claims.read().await;
        claims.get(message_id).cloned()
    }

    #[instrument(skip(self, user_id))]
    pub async fn generate_token(&self, user_id: &str) -> Result<String, SecurityError> {
        let tokens = self.tokens.read().await;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(tokens.jwt_key.as_bytes()),
        )
        .map_err(|e| SecurityError::Token(format!("Token generation failed: {:?}", e)))?;

        Ok(token)
    }

    #[instrument(skip(self, role, permission))]
    pub async fn add_permission(&self, role: &str, permission: &str) -> Result<(), SecurityError> {
        let mut permissions = self.permissions.write().await;
        let role_permissions = permissions.role_permissions
            .entry(role.to_string())
            .or_insert_with(Vec::new);
        
        if !role_permissions.contains(&permission.to_string()) {
            role_permissions.push(permission.to_string());
        }

        Ok(())
    }

    #[instrument(skip(self, role, permission))]
    pub async fn has_permission(&self, role: &str, permission: &str) -> Result<bool, SecurityError> {
        let permissions = self.permissions.read().await;
        Ok(permissions.role_permissions
            .get(role)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false))
    }

    #[instrument(skip(self))]
    pub async fn rotate_key(&self) -> Result<(), SecurityError> {
        let mut keys = self.keys.write().await;
        let new_key = aead::LessSafeKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, &[0u8; 32]).unwrap()
        );
        
        keys.current_key = new_key;
        keys.key_id = Uuid::new_v4().to_string();
        keys.last_rotation = Utc::now();

        Ok(())
    }
}

impl Clone for SecurityManager {
    fn clone(&self) -> Self {
        Self {
            keys: TokioRwLock::new(KeyManager::default()),
            tokens: TokioRwLock::new(TokenManager::default()),
            permissions: TokioRwLock::new(PermissionManager::default()),
            claims: self.claims.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_creation_and_validation() {
        let manager = SecurityManager::new();
        let roles = vec!["user".to_string()];
        let token = manager.create_token("test_user", roles).await.unwrap();
        let claims = manager.validate_token(&token).await.unwrap();
        assert_eq!(claims.sub, "test_user");
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let manager = SecurityManager::new();
        let data = b"test data";
        let encrypted = manager.encrypt(data).await.unwrap();
        let decrypted = manager.decrypt(&encrypted).await.unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let manager = SecurityManager::new();
        manager.add_role_permission("admin", "read").await.unwrap();
        let has_permission = manager.check_permission(&["admin".to_string()], "read").await.unwrap();
        assert!(has_permission);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let manager = SecurityManager::new();
        assert!(manager.rotate_keys().await.is_ok());
    }

    #[tokio::test]
    async fn test_security_verification() {
        let manager = SecurityManager::new();
        let message = MCPMessage {
            id: "test_id".to_string(),
            version: ProtocolVersion::default(),
            message_type: MessageType::Request,
            target: Some("test_target".to_string()),
            source: Some("test_source".to_string()),
            payload: Value::Null,
            metadata: HashMap::new(),
            security: Some("test_token".to_string()),
            correlation_id: None,
        };

        manager.verify_message(&message).await.unwrap();
        let claims = manager.get_claims("test_id").await.unwrap();
        assert_eq!(claims.sub, "test_user");
    }
} 