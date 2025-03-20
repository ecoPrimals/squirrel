use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, Context};
use ring::{aead, rand};
use ring::rand::SecureRandom;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use uuid::Uuid;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, instrument};
use crate::mcp::error::{MCPError, SecurityErrorKind, ErrorContext, ErrorSeverity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiry: Duration,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub min_key_rotation: Duration,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AesGcm256,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct SecurityManager {
    config: SecurityConfig,
    keys: Arc<RwLock<KeyStore>>,
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expiry: DateTime<Utc>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub permissions: Vec<Permission>,
}

#[derive(Debug)]
struct KeyStore {
    current_key: aead::LessSafeKey,
    current_key_id: String,
    previous_keys: HashMap<String, aead::LessSafeKey>,
    last_rotation: DateTime<Utc>,
}

impl SecurityManager {
    #[instrument(skip(config))]
    pub async fn new(config: SecurityConfig) -> Result<Self, MCPError> {
        let rng = rand::SystemRandom::new();
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &rand::generate(&rng).unwrap().expose())
            .map_err(|_| MCPError::Security {
                kind: SecurityErrorKind::EncryptionFailed,
                context: ErrorContext::new("initialize_security", "security_manager")
                    .with_severity(ErrorSeverity::Critical)
                    .not_recoverable(),
                security_level: config.security_level,
            })?;

        let key_store = KeyStore {
            current_key: aead::LessSafeKey::new(key),
            current_key_id: uuid::Uuid::new_v4().to_string(),
            previous_keys: HashMap::new(),
            last_rotation: Utc::now(),
        };

        Ok(Self {
            config,
            keys: Arc::new(RwLock::new(key_store)),
            tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    #[instrument(skip(self, data))]
    pub async fn encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, String), MCPError> {
        let keys = self.keys.read().await;
        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]);
        let mut buffer = data.to_vec();

        keys.current_key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut buffer)
            .map_err(|_| MCPError::Security {
                kind: SecurityErrorKind::EncryptionFailed,
                context: ErrorContext::new("encrypt", "security_manager")
                    .with_severity(ErrorSeverity::High),
                security_level: self.config.security_level,
            })?;

        Ok((buffer, keys.current_key_id.clone()))
    }

    #[instrument(skip(self, encrypted_data))]
    pub async fn decrypt(&self, encrypted_data: &[u8], key_id: &str) -> Result<Vec<u8>, MCPError> {
        let keys = self.keys.read().await;
        let key = if key_id == keys.current_key_id {
            &keys.current_key
        } else {
            keys.previous_keys.get(key_id).ok_or_else(|| MCPError::Security {
                kind: SecurityErrorKind::DecryptionFailed,
                context: ErrorContext::new("decrypt", "security_manager")
                    .with_severity(ErrorSeverity::High),
                security_level: self.config.security_level,
            })?
        };

        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]);
        let mut buffer = encrypted_data.to_vec();

        key.open_in_place(nonce, aead::Aad::empty(), &mut buffer)
            .map_err(|_| MCPError::Security {
                kind: SecurityErrorKind::DecryptionFailed,
                context: ErrorContext::new("decrypt", "security_manager")
                    .with_severity(ErrorSeverity::High),
                security_level: self.config.security_level,
            })?;

        Ok(buffer)
    }

    #[instrument(skip(self))]
    pub async fn create_token(&self, user_id: &str, permissions: Vec<Permission>) -> Result<AuthToken, MCPError> {
        let expiry = Utc::now() + self.config.token_expiry;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: Utc::now().timestamp(),
            permissions: permissions.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        ).map_err(|_| MCPError::Security {
            kind: SecurityErrorKind::AuthenticationFailed,
            context: ErrorContext::new("create_token", "security_manager")
                .with_severity(ErrorSeverity::High),
            security_level: self.config.security_level,
        })?;

        let auth_token = AuthToken {
            token: token.clone(),
            expiry,
            permissions,
        };

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.clone(), auth_token.clone());

        Ok(auth_token)
    }

    #[instrument(skip(self))]
    pub async fn validate_token(&self, token: &str) -> Result<Claims, MCPError> {
        let tokens = self.tokens.read().await;
        if let Some(auth_token) = tokens.get(token) {
            if auth_token.expiry < Utc::now() {
                return Err(MCPError::Security {
                    kind: SecurityErrorKind::TokenExpired,
                    context: ErrorContext::new("validate_token", "security_manager")
                        .with_severity(ErrorSeverity::Medium),
                    security_level: self.config.security_level,
                });
            }
        }

        let validation = Validation::default();
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| MCPError::Security {
            kind: SecurityErrorKind::InvalidToken,
            context: ErrorContext::new("validate_token", "security_manager")
                .with_severity(ErrorSeverity::High),
            security_level: self.config.security_level,
        })
        .map(|token_data| token_data.claims)
    }

    #[instrument(skip(self))]
    pub async fn check_permission(&self, token: &str, required_permission: &Permission) -> Result<bool, MCPError> {
        let claims = self.validate_token(token).await?;
        
        Ok(claims.permissions.iter().any(|p| {
            p.resource == required_permission.resource && 
            matches!(
                (&p.action, &required_permission.action),
                (Action::Admin, _) |
                (Action::Write, Action::Read) |
                (a1, a2) if a1 == a2
            )
        }))
    }

    #[instrument(skip(self))]
    pub async fn rotate_keys(&self) -> Result<(), MCPError> {
        let mut keys = self.keys.write().await;
        
        if Utc::now() - keys.last_rotation < self.config.min_key_rotation {
            return Ok(());
        }

        let rng = rand::SystemRandom::new();
        let new_key = aead::UnboundKey::new(
            &aead::AES_256_GCM,
            &rand::generate(&rng).unwrap().expose(),
        ).map_err(|_| MCPError::Security {
            kind: SecurityErrorKind::EncryptionFailed,
            context: ErrorContext::new("rotate_keys", "security_manager")
                .with_severity(ErrorSeverity::High),
            security_level: self.config.security_level,
        })?;

        // Store current key in previous keys
        keys.previous_keys.insert(
            keys.current_key_id.clone(),
            std::mem::replace(&mut keys.current_key, aead::LessSafeKey::new(new_key)),
        );

        // Update key ID and rotation time
        keys.current_key_id = uuid::Uuid::new_v4().to_string();
        keys.last_rotation = Utc::now();

        // Clean up old keys
        keys.previous_keys.retain(|_, _| true); // TODO: Implement key cleanup strategy

        Ok(())
    }
} 