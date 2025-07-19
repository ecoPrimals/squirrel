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
use tracing::{debug, info, warn};

use super::{
    types::SecurityLevel, AuthMethod, AuthorizationLevel, SecurityAdapter, SecurityContext,
    SecurityHealthStatus, SecurityProviderConfig, SecurityRequest, SecurityResponse,
    SecuritySession,
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

        // Create a new session
        let session = SecuritySession::new(
            format!("session_{}", uuid::Uuid::new_v4()),
            Some("default_user".to_string()),
        );

        // Store session
        self.active_sessions
            .write()
            .unwrap()
            .insert(session.session_id.clone(), session.clone());

        Ok(session)
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
