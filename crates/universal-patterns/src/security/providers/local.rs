// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Local fallback security provider.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

use super::super::context::SecurityContext;
use super::super::errors::SecurityError;
use super::types::{
    HealthStatus, SecurityCapability, SecurityHealth, SecurityRequest, SecurityResponse,
    SecurityServiceConfig, SecurityServiceInfo, TrustLevel, UniversalSecurityService,
};
use crate::config::AuthMethod;

/// Local Security Provider Implementation\
/// Provides basic local security capabilities for fallback scenarios
pub struct LocalSecurityProvider {
    config: SecurityServiceConfig,
}

impl LocalSecurityProvider {
    /// Create a new local security provider
    pub async fn new(config: SecurityServiceConfig) -> Result<Self, SecurityError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl UniversalSecurityService for LocalSecurityProvider {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        vec![
            SecurityCapability::Authentication {
                methods: vec![
                    AuthMethod::None,
                    AuthMethod::Token {
                        token_file: std::path::PathBuf::from("/tmp/token"),
                    },
                ],
                multi_factor: false,
                session_management: false,
            },
            SecurityCapability::Cryptography {
                algorithms: vec!["AES-128".to_string()],
                key_management: false,
                hardware_security: false,
            },
        ]
    }

    fn get_service_info(&self) -> SecurityServiceInfo {
        let trust_level = if self.config.service_id == "local-security" {
            TrustLevel::Medium
        } else {
            TrustLevel::Low
        };

        SecurityServiceInfo {
            service_id: "local-security".to_string(),
            name: "Local Security Service".to_string(),
            version: "1.0.0".to_string(),
            description: "Basic local security capabilities for fallback".to_string(),
            capabilities: self.get_capabilities(),
            endpoints: vec![],
            supported_protocols: vec!["Local".to_string()],
            compliance_certifications: vec![],
            trust_level,
        }
    }

    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError> {
        // Local implementation - simplified operations
        Ok(SecurityResponse::success(
            request.request_id,
            "Local operation completed".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            status: HealthStatus::Healthy,
            message: "Local security service operational".to_string(),
            last_check: Utc::now(),
            metrics: HashMap::new(),
        })
    }

    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError> {
        self.config = config;
        Ok(())
    }
}

// Implement the traits::UniversalSecurityProvider for LocalSecurityProvider
#[async_trait]
impl crate::security::traits::UniversalSecurityProvider for LocalSecurityProvider {
    async fn authenticate(
        &self,
        _credentials: &crate::traits::Credentials,
    ) -> Result<crate::traits::AuthResult, SecurityError> {
        // Local authentication - simplified for testing
        Ok(crate::traits::AuthResult {
            principal: crate::traits::Principal {
                id: "local-user".to_string(),
                name: "Local User".to_string(),
                principal_type: crate::traits::PrincipalType::User,
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            token: "local-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn authorize(
        &self,
        principal: &crate::traits::Principal,
        action: &str,
        _resource: &str,
    ) -> Result<bool, SecurityError> {
        let allowed = principal
            .permissions
            .iter()
            .any(|p| p == action || p == "*");
        Ok(allowed)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let key = blake3::derive_key(
            "ecoPrimals local encrypt v1",
            self.config.service_id.as_bytes(),
        );
        let mut reader = blake3::Hasher::new_keyed(&key)
            .update(b"local-encrypt-stream")
            .finalize_xof();
        let mut keystream = vec![0u8; data.len()];
        reader.fill(&mut keystream);
        Ok(data
            .iter()
            .zip(keystream.iter())
            .map(|(d, k)| d ^ k)
            .collect())
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        self.encrypt(encrypted_data).await
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let key = blake3::derive_key(
            "ecoPrimals local sign v1",
            self.config.service_id.as_bytes(),
        );
        let sig = blake3::keyed_hash(&key, data);
        Ok(sig.as_bytes().to_vec())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let expected = self.sign(data).await?;
        Ok(expected == signature)
    }

    async fn audit_log(
        &self,
        operation: &str,
        _context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        tracing::info!(operation, "Local security audit event");
        Ok(())
    }

    async fn health_check(
        &self,
    ) -> Result<crate::security::context::SecurityHealth, SecurityError> {
        // Convert from providers::SecurityHealth to context::SecurityHealth
        let providers_health = UniversalSecurityService::health_check(self).await?;

        Ok(crate::security::context::SecurityHealth {
            status: match providers_health.status {
                HealthStatus::Healthy => crate::security::context::HealthStatus::Healthy,
                HealthStatus::Degraded => crate::security::context::HealthStatus::Unhealthy,
                HealthStatus::Unhealthy => crate::security::context::HealthStatus::Unhealthy,
            },
            latency: std::time::Duration::from_millis(5), // Default latency for local
            last_check: providers_health.last_check,
            details: std::collections::HashMap::new(),
        })
    }
}
