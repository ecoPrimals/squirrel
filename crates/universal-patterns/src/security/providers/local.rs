// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Local fallback security provider.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Once;

use super::super::context::SecurityContext;
use super::super::errors::SecurityError;
use super::types::{
    HealthStatus, SecurityCapability, SecurityHealth, SecurityRequest, SecurityResponse,
    SecurityServiceConfig, SecurityServiceInfo, TrustLevel, UniversalSecurityService,
};
use crate::config::AuthMethod;

const CRYPTO_DELEGATION_MESSAGE: &str =
    "Local crypto operations require security capability provider (IPC not available)";

static CRYPTO_DELEGATION_WARN: Once = Once::new();

fn warn_crypto_delegation_required() {
    CRYPTO_DELEGATION_WARN.call_once(|| {
        tracing::warn!(
            "{CRYPTO_DELEGATION_MESSAGE}; configure a security capability provider via IPC"
        );
    });
}

fn crypto_delegation_error() -> SecurityError {
    warn_crypto_delegation_required();
    SecurityError::Other(CRYPTO_DELEGATION_MESSAGE.to_string())
}

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

impl UniversalSecurityService for LocalSecurityProvider {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        vec![
            SecurityCapability::Authentication {
                methods: vec![
                    AuthMethod::None,
                    AuthMethod::Token {
                        token_file: std::env::var(
                            universal_constants::env_vars::security::TOKEN_FILE,
                        )
                        .map_or_else(
                            |_| {
                                universal_constants::network::get_socket_dir()
                                    .join("security.token")
                            },
                            std::path::PathBuf::from,
                        ),
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
        Ok(SecurityResponse::success(
            request.request_id,
            format!(
                "Local fallback: {:?} (no external provider)",
                request.operation
            ),
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
impl crate::security::traits::UniversalSecurityProvider for LocalSecurityProvider {
    async fn authenticate(
        &self,
        _credentials: &crate::traits::Credentials,
    ) -> Result<crate::traits::AuthResult, SecurityError> {
        Err(crypto_delegation_error())
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

    async fn encrypt(&self, _data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        Err(crypto_delegation_error())
    }

    async fn decrypt(&self, _encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        Err(crypto_delegation_error())
    }

    async fn sign(&self, _data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        Err(crypto_delegation_error())
    }

    async fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool, SecurityError> {
        Err(crypto_delegation_error())
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
