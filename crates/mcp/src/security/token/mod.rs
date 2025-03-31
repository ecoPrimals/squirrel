//! Token management and credential types.

use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use crate::error::{Result, SecurityError};
use crate::security::identity::UserId;
use uuid::Uuid;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};

/// Represents user credentials, typically username and password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String, // Note: In a real system, this should be handled securely (e.g., hashed)
    // Add other fields if needed, e.g., domain, tenant ID
}

/// Represents an authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token: String, // The actual token string (e.g., JWT)
    pub user_id: UserId, // Link to the user
    pub roles: Vec<String>, // Roles associated with the token
    pub expires_at: Option<DateTime<Utc>>, // Token expiry time
    pub issued_at: DateTime<Utc>,
    // Add other relevant fields (e.g., issuer, audience, scopes)
}

/// JWT Claims structure used for token generation and validation
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Issued at timestamp
    iat: i64,
    /// Expiration timestamp
    exp: i64,
    /// Roles assigned to the user
    roles: Vec<String>,
    /// Unique token identifier
    jti: String,
}

/// Wrapper type for Session Tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String);

/// Wrapper type for Authentication Tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthToken(pub String);

/// Trait for managing authentication tokens.
#[async_trait]
pub trait TokenManager: Send + Sync {
    /// Generate a token for a user with specified roles
    async fn generate_token(&self, user_id: &UserId, roles: &[String]) -> Result<Token>;
    
    /// Validate a token string and return the parsed token
    async fn validate_token(&self, token_str: &str) -> Result<Token>;
    
    /// Revoke a token
    async fn revoke_token(&self, token: &Token) -> Result<()>;
}

/// Default implementation for TokenManager.
pub struct DefaultTokenManager {
    key_storage: Arc<dyn crate::security::KeyStorage>,
    crypto_provider: Arc<dyn crate::security::CryptoProvider>,
    /// In-memory store of revoked tokens (by JTI)
    revoked_tokens: RwLock<HashMap<String, DateTime<Utc>>>,
}

impl DefaultTokenManager {
    /// Create a new DefaultTokenManager
    pub fn new(
        key_storage: Arc<dyn crate::security::KeyStorage>,
        crypto_provider: Arc<dyn crate::security::CryptoProvider>,
    ) -> Self {
        Self {
            key_storage,
            crypto_provider,
            revoked_tokens: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get the signing key for JWT tokens
    async fn get_signing_key(&self) -> Result<Vec<u8>> {
        // Try to get the existing key, or generate a new one if it doesn't exist
        let key_result = self.key_storage.get_latest_key(crate::security::key_storage::KeyPurpose::TokenSigning).await;
        
        match key_result {
            Ok((_, key)) => Ok(key),
            Err(_) => {
                // Generate a new key for token signing
                let new_key = self.crypto_provider.generate_hmac_key();
                let key_clone = new_key.clone(); // Clone the key before moving it
                self.key_storage.store_key(
                    new_key,
                    crate::security::key_storage::KeyPurpose::TokenSigning
                ).await?;
                Ok(key_clone)
            }
        }
    }
    
    /// Check if a token has been revoked by its JTI
    fn is_token_revoked(&self, jti: &str) -> bool {
        let revoked_tokens = self.revoked_tokens.read().unwrap();
        revoked_tokens.contains_key(jti)
    }
    
    /// Clean up expired revoked tokens
    fn cleanup_revoked_tokens(&self) {
        let now = Utc::now();
        let mut revoked_tokens = self.revoked_tokens.write().unwrap();
        revoked_tokens.retain(|_, expiry| *expiry > now);
    }
}

#[async_trait]
impl TokenManager for DefaultTokenManager {
    async fn generate_token(&self, user_id: &UserId, roles: &[String]) -> Result<Token> {
        // Get the signing key
        let key = self.get_signing_key().await?;
        
        // Create token expiration (1 hour from now)
        let expiration = Utc::now() + chrono::Duration::hours(1);
        let issued_at = Utc::now();
        
        // Generate a unique token ID
        let token_id = Uuid::new_v4().to_string();
        
        // Create the claims
        let claims = Claims {
            sub: user_id.0.to_string(),
            iat: issued_at.timestamp(),
            exp: expiration.timestamp(),
            roles: roles.to_vec(),
            jti: token_id,
        };
        
        // Encode the JWT
        let token_string = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&key)
        ).map_err(|e| SecurityError::TokenGenerationFailed(e.to_string()))?;
        
        // Create the token
        let token = Token {
            token: token_string,
            user_id: user_id.clone(),
            roles: roles.to_vec(),
            expires_at: Some(expiration),
            issued_at,
        };
        
        Ok(token)
    }
    
    async fn validate_token(&self, token_str: &str) -> Result<Token> {
        // Get the signing key
        let key = self.get_signing_key().await?;
        
        // Validate token
        let token_data = decode::<Claims>(
            token_str,
            &DecodingKey::from_secret(&key),
            &Validation::new(Algorithm::HS256)
        ).map_err(|e| SecurityError::InvalidToken(e.to_string()))?;
        
        let claims = token_data.claims;
        
        // Check if token has been revoked
        if self.is_token_revoked(&claims.jti) {
            return Err(SecurityError::InvalidToken("Token has been revoked".to_string()).into());
        }
        
        // Parse the user ID
        let user_id = UserId(Uuid::parse_str(&claims.sub)
            .map_err(|e| SecurityError::InvalidToken(format!("Invalid user ID: {}", e)))?);
        
        // Create expiration datetime
        let expires_at = chrono::DateTime::<Utc>::from_timestamp(claims.exp, 0)
            .ok_or_else(|| SecurityError::InvalidToken("Invalid expiration time".to_string()))?;
        
        // Create issued_at datetime
        let issued_at = chrono::DateTime::<Utc>::from_timestamp(claims.iat, 0)
            .ok_or_else(|| SecurityError::InvalidToken("Invalid issued at time".to_string()))?;
        
        // Create the token
        let token = Token {
            token: token_str.to_string(),
            user_id,
            roles: claims.roles,
            expires_at: Some(expires_at),
            issued_at,
        };
        
        Ok(token)
    }
    
    async fn revoke_token(&self, token: &Token) -> Result<()> {
        // In a production implementation, we would:
        // 1. Parse the token to get its JTI
        // 2. Add it to a persistent revocation list
        
        // For demonstration, we'll parse the JWT to get the JTI
        let key = self.get_signing_key().await?;
        
        let token_data = decode::<Claims>(
            &token.token,
            &DecodingKey::from_secret(&key),
            &Validation::new(Algorithm::HS256)
        ).map_err(|e| SecurityError::InvalidToken(e.to_string()))?;
        
        let jti = token_data.claims.jti;
        
        // Add to revoked tokens with expiry time matching the token
        let expires_at = token.expires_at.unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));
        
        // Add to revoked tokens
        let mut revoked_tokens = self.revoked_tokens.write().unwrap();
        revoked_tokens.insert(jti, expires_at);
        
        // Periodically clean up expired tokens
        drop(revoked_tokens); // Release the lock before cleanup
        self.cleanup_revoked_tokens();
        
        Ok(())
    }
} 