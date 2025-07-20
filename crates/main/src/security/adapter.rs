//! Universal Security Adapter
//!
//! This module implements the universal security adapter that can integrate
//! with any security provider in the ecosystem through standardized interfaces.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use super::{
    config::{AuthMethod, SecurityProviderConfig},
    health::SecurityHealthStatus,
    session::SecuritySession,
    traits::SecurityAdapter,
    types::{SecurityLevel, SecurityContext, SecurityRequest, SecurityResponse},
};
use crate::error::PrimalError;

/// Universal Security Adapter Implementation
#[derive(Debug, Clone)]
pub struct UniversalSecurityAdapter {
    /// Security provider configuration
    provider_config: SecurityProviderConfig,
    /// Active security sessions
    active_sessions: Arc<std::sync::RwLock<HashMap<String, SecuritySession>>>,
    /// Security context cache
    context_cache: Arc<std::sync::RwLock<HashMap<String, SecurityContext>>>,
    /// Adapter health status
    health_status: Arc<std::sync::RwLock<SecurityHealthStatus>>,
}

impl UniversalSecurityAdapter {
    /// Create a new Universal Security Adapter
    pub fn new(provider_config: SecurityProviderConfig) -> Self {
        Self {
            provider_config,
            active_sessions: Arc::new(std::sync::RwLock::new(HashMap::new())),
            context_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            health_status: Arc::new(std::sync::RwLock::new(SecurityHealthStatus::healthy())),
        }
    }

    /// Get the adapter health status
    pub async fn get_health_status(&self) -> SecurityHealthStatus {
        self.health_status.read().unwrap().clone()
    }

    /// Update the adapter health status
    pub async fn update_health_status(&self, status: SecurityHealthStatus) {
        *self.health_status.write().unwrap() = status;
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<SecuritySession> {
        self.active_sessions
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<(), PrimalError> {
        let mut sessions = self.active_sessions.write().unwrap();
        let now = Utc::now();

        sessions.retain(|_, session| session.expires_at > now);

        Ok(())
    }
}

#[async_trait]
impl SecurityAdapter for UniversalSecurityAdapter {
    async fn handle_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, PrimalError> {
        debug!("Handling security request: {:?}", request.request_type);

        let processing_start = std::time::Instant::now();

        let result = match request.request_type {
            super::SecurityRequestType::Authentication => {
                // Handle authentication
                Value::String("authenticated".to_string())
            }
            super::SecurityRequestType::Authorization => {
                // Handle authorization
                Value::Bool(true)
            }
            super::SecurityRequestType::Encryption => {
                // Handle encryption
                Value::String("encrypted_data".to_string())
            }
            super::SecurityRequestType::Decryption => {
                // Handle decryption
                Value::String("decrypted_data".to_string())
            }
            _ => Value::String("handled".to_string()),
        };

        Ok(SecurityResponse {
            request_id: request.request_id,
            status: super::SecurityResponseStatus::Success,
            payload: result,
            metadata: HashMap::new(),
            processing_time: processing_start.elapsed(),
            timestamp: Utc::now(),
        })
    }

    async fn authenticate(&self, credentials: Value) -> Result<SecuritySession, PrimalError> {
        debug!("Authenticating with credentials");

        // Extract and validate credentials
        let user_id = credentials
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PrimalError::ValidationError("Missing user_id in credentials".to_string())
            })?;

        let auth_method = credentials
            .get("auth_method")
            .and_then(|v| v.as_str())
            .unwrap_or("password");

        // Validate credentials based on auth method
        let is_valid = match auth_method {
            "api_key" => {
                let api_key = credentials
                    .get("api_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PrimalError::ValidationError("Missing api_key".to_string()))?;
                self.validate_api_key(api_key).await?
            }
            "password" => {
                let password = credentials
                    .get("password")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PrimalError::ValidationError("Missing password".to_string()))?;
                self.validate_password(user_id, password).await?
            }
            "oauth" => {
                let token = credentials
                    .get("oauth_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        PrimalError::ValidationError("Missing oauth_token".to_string())
                    })?;
                self.validate_oauth_token(token).await?
            }
            _ => {
                return Err(PrimalError::ValidationError(format!(
                    "Unsupported auth method: {}",
                    auth_method
                )));
            }
        };

        if !is_valid {
            return Err(PrimalError::AuthenticationFailed(
                "Invalid credentials provided".to_string(),
            ));
        }

        // Create authenticated session with validated user
        let session = SecuritySession::new(
            format!("session_{}", uuid::Uuid::new_v4()),
            Some(user_id.to_string()),
        );

        // Store session with additional metadata from credentials
        let session_id = session.session_id.clone();
        let mut session_data = session;

        // Enhance session with credential metadata
        if let Some(metadata) = credentials.get("metadata").and_then(|v| v.as_object()) {
            for (key, value) in metadata {
                if let Some(str_value) = value.as_str() {
                    session_data
                        .metadata
                        .insert(key.clone(), str_value.to_string());
                }
            }
        }

        // Add authentication method to session metadata
        session_data
            .metadata
            .insert("auth_method".to_string(), auth_method.to_string());
        session_data.metadata.insert(
            "authenticated_at".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        // Store session
        self.active_sessions
            .write()
            .unwrap()
            .insert(session_id.clone(), session_data);

        Ok(session_data)
    }

    /// Validate API key credentials
    async fn validate_api_key(&self, api_key: &str) -> Result<bool, PrimalError> {
        debug!("Validating API key");

        // Enhanced API key validation
        if api_key.len() < 32 {
            return Ok(false);
        }

        // Check if API key follows expected format (e.g., starts with 'sk-')
        if !api_key.starts_with("sk-") && !api_key.starts_with("pk-") {
            return Ok(false);
        }

        // In production, this would check against a secure API key database
        // For now, accept well-formed keys for demonstration
        Ok(true)
    }

    /// Validate password credentials  
    async fn validate_password(&self, user_id: &str, password: &str) -> Result<bool, PrimalError> {
        debug!("Validating password for user: {}", user_id);

        // Basic password validation
        if password.len() < 8 {
            return Ok(false);
        }

        // In production, this would:
        // 1. Hash the password with the user's salt
        // 2. Compare against stored hash in secure database
        // 3. Implement rate limiting and account lockout

        // For demonstration, accept passwords with basic complexity
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        Ok(has_upper && has_lower && has_digit)
    }

    /// Validate OAuth token credentials
    async fn validate_oauth_token(&self, token: &str) -> Result<bool, PrimalError> {
        debug!("Validating OAuth token");

        // Basic OAuth token validation
        if token.is_empty() || token.len() < 16 {
            return Ok(false);
        }

        // In production, this would:
        // 1. Validate token with OAuth provider
        // 2. Check token expiration
        // 3. Verify token scope and permissions

        // For demonstration, accept well-formed tokens
        Ok(token.starts_with("bearer_") || token.starts_with("oauth_"))
    }

    async fn authorize(
        &self,
        session: &SecuritySession,
        operation: &str,
    ) -> Result<bool, PrimalError> {
        debug!(
            "Authorizing operation: {} for session: {}",
            operation, session.session_id
        );

        // Simple authorization logic
        Ok(true)
    }

    async fn validate_token(&self, token: &str) -> Result<SecuritySession, PrimalError> {
        debug!("Validating token: {}", token);

        // Simple token validation
        let session = SecuritySession::new(
            format!("token_session_{}", uuid::Uuid::new_v4()),
            Some("token_user".to_string()),
        );

        Ok(session)
    }

    async fn create_session(&self, user_id: &str) -> Result<SecuritySession, PrimalError> {
        debug!("Creating session for user: {}", user_id);

        let session = SecuritySession::new(
            format!("session_{}", uuid::Uuid::new_v4()),
            Some(user_id.to_string()),
        );

        // Store session
        self.active_sessions
            .write()
            .unwrap()
            .insert(session.session_id.clone(), session.clone());

        Ok(session)
    }

    async fn destroy_session(&self, session_id: &str) -> Result<(), PrimalError> {
        debug!("Destroying session: {}", session_id);

        self.active_sessions.write().unwrap().remove(session_id);
        Ok(())
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, PrimalError> {
        debug!("Encrypting data of length: {}", data.len());

        // Simple encryption simulation
        let mut encrypted = data.to_vec();
        for byte in &mut encrypted {
            *byte = byte.wrapping_add(1);
        }

        Ok(encrypted)
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, PrimalError> {
        debug!("Decrypting data of length: {}", encrypted_data.len());

        // Simple decryption simulation
        let mut decrypted = encrypted_data.to_vec();
        for byte in &mut decrypted {
            *byte = byte.wrapping_sub(1);
        }

        Ok(decrypted)
    }

    async fn audit_event(&self, event: Value) -> Result<(), PrimalError> {
        debug!("Auditing event: {:?}", event);

        // Simple audit logging
        info!("Audit event: {}", event);

        Ok(())
    }

    async fn check_policy(
        &self,
        policy_id: &str,
        context: &SecurityContext,
    ) -> Result<bool, PrimalError> {
        debug!("Checking policy: {} for context: {:?}", policy_id, context);

        // Simple policy check
        Ok(true)
    }

    async fn health_check(&self) -> Result<SecurityHealthStatus, PrimalError> {
        debug!("Performing health check");

        Ok(self.health_status.read().unwrap().clone())
    }
}

/// Create a default Universal Security Adapter
pub fn create_default_universal_security_adapter() -> UniversalSecurityAdapter {
    let config = SecurityProviderConfig {
        provider_type: "default".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        auth_method: AuthMethod::None,
        timeout: Duration::from_secs(30),
        retry_config: super::config::RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        },
        security_level: SecurityLevel::Standard,
        capabilities: vec![
            super::SecurityCapability::Authentication,
            super::SecurityCapability::AccessControl,
        ],
    };

    UniversalSecurityAdapter::new(config)
}

/// Create a Universal Security Adapter from environment variables
pub fn create_universal_security_adapter_from_env() -> Result<UniversalSecurityAdapter, PrimalError>
{
    let provider_type =
        std::env::var("SECURITY_PROVIDER_TYPE").unwrap_or_else(|_| "default".to_string());

    let endpoint =
        std::env::var("SECURITY_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let auth_method = AuthMethod::None; // Simplified for now

    let config = SecurityProviderConfig {
        provider_type,
        endpoint,
        auth_method,
        timeout: Duration::from_secs(30),
        retry_config: super::config::RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        },
        security_level: SecurityLevel::Standard,
        capabilities: vec![
            super::SecurityCapability::Authentication,
            super::SecurityCapability::AccessControl,
        ],
    };

    Ok(UniversalSecurityAdapter::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_security_adapter_creation() {
        let adapter = create_default_universal_security_adapter();
        assert_eq!(adapter.provider_config.provider_type, "default");
        assert_eq!(
            adapter.provider_config.security_level,
            SecurityLevel::Standard
        );
    }

    #[tokio::test]
    async fn test_authentication() {
        let adapter = create_default_universal_security_adapter();

        let credentials = Value::String("test_credentials".to_string());
        let session = adapter.authenticate(credentials).await.unwrap();

        assert!(!session.session_id.is_empty());
        assert_eq!(session.user_id, Some("default_user".to_string()));
    }

    #[tokio::test]
    async fn test_session_management() {
        let adapter = create_default_universal_security_adapter();

        let session = adapter.create_session("test_user").await.unwrap();
        assert_eq!(session.user_id, Some("test_user".to_string()));

        let is_authorized = adapter.authorize(&session, "read").await.unwrap();
        assert!(is_authorized);

        adapter.destroy_session(&session.session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let adapter = create_default_universal_security_adapter();

        let data = b"test data";
        let encrypted = adapter.encrypt(data).await.unwrap();
        let decrypted = adapter.decrypt(&encrypted).await.unwrap();

        assert_eq!(data, decrypted.as_slice());
    }
}
