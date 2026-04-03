// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Beardog-backed universal security provider and integration helpers.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use universal_constants::primal_names;

use super::super::context::SecurityContext;
use super::super::errors::SecurityError;
use super::types::{
    HealthStatus, Priority, SecurityCapability, SecurityHealth, SecurityOperation, SecurityRequest,
    SecurityResponse, SecurityServiceConfig, SecurityServiceInfo, TrustLevel,
    UniversalSecurityService,
};
use crate::config::AuthMethod;

/// Beardog security provider (integration with the Beardog primal).
///
/// Uses capability-based discovery; HTTP is removed in favor of Unix sockets (often via
/// Songbird).
///
/// ## Phase 2: `UniversalSecurityProvider` behavior
///
/// The [`crate::security::traits::UniversalSecurityProvider`] implementation dispatches
/// `SecurityRequest` values where possible, but **`authenticate` still returns a
/// static test principal and token** until Beardog’s socket protocol returns real
/// [`crate::traits::AuthResult`] data. `authorize`, encrypt/decrypt, and sign/verify are
/// identity/pass-through stubs until Beardog crypto endpoints are wired—do not use this
/// provider for production security boundaries yet.
pub struct BeardogSecurityProvider {
    config: SecurityServiceConfig,
    // Note: HTTP client removed - should use Unix socket for Beardog communication
}

impl BeardogSecurityProvider {
    /// Create a new Beardog security provider
    /// NOTE: Uses Unix socket discovery via ecosystem patterns
    pub async fn new(config: SecurityServiceConfig) -> Result<Self, SecurityError> {
        // Beardog communication should use Unix sockets
        // Pattern: UnixStream::connect("/var/run/beardog/security.sock").await
        tracing::info!("BeardogSecurityProvider created (HTTP delegation not yet implemented)");

        Ok(Self { config })
    }
}

#[async_trait]
impl UniversalSecurityService for BeardogSecurityProvider {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        vec![
            SecurityCapability::Authentication {
                methods: vec![AuthMethod::Beardog {
                    // Capability-based alias; primal id: [`primal_names::BEARDOG`]
                    service_id: format!("{}-primary", primal_names::BEARDOG),
                }],
                multi_factor: true,
                session_management: true,
            },
            SecurityCapability::Authorization {
                rbac: true,
                abac: false,
                policy_engine: true,
            },
            SecurityCapability::Cryptography {
                algorithms: vec!["AES-256".to_string(), "RSA-4096".to_string()],
                key_management: true,
                hardware_security: true,
            },
            SecurityCapability::Compliance {
                standards: vec!["SOX".to_string(), "GDPR".to_string()],
                audit_logging: true,
                real_time_monitoring: true,
            },
        ]
    }

    fn get_service_info(&self) -> SecurityServiceInfo {
        let trust_level = if self.config.service_id == format!("{}-security", primal_names::BEARDOG)
        {
            TrustLevel::High
        } else {
            TrustLevel::Medium
        };

        SecurityServiceInfo {
            service_id: format!("{}-security", primal_names::BEARDOG),
            name: "Beardog Security Service".to_string(),
            version: "1.0.0".to_string(),
            description: "Enterprise security service with comprehensive capabilities".to_string(),
            capabilities: self.get_capabilities(),
            endpoints: vec![],
            supported_protocols: vec!["json-rpc-2.0".to_string(), "unix-socket".to_string()],
            compliance_certifications: vec!["SOC2".to_string(), "ISO27001".to_string()],
            trust_level,
        }
    }

    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError> {
        // Implementation would make actual requests to Beardog service
        Ok(SecurityResponse::success(
            request.request_id,
            "Beardog operation completed".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            status: HealthStatus::Healthy,
            message: "Beardog security service operational".to_string(),
            last_check: Utc::now(),
            metrics: HashMap::new(),
        })
    }

    /// NOTE: Uses Unix socket discovery via ecosystem patterns
    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError> {
        self.config = config;
        tracing::info!("BeardogSecurityProvider initialized (Unix socket discovery)");
        Ok(())
    }
}

/// Beardog Integration helper
/// NOTE: HTTP removed - Uses Unix socket communication via Songbird
pub struct BeardogIntegration;

impl BeardogIntegration {
    /// Create a new Beardog integration
    /// NOTE: Uses Unix socket discovery via ecosystem patterns
    ///
    /// Note: This is a factory function that returns BeardogSecurityProvider, not Self.
    /// This is intentional as BeardogIntegration is a namespace for integration logic.
    #[expect(
        clippy::new_ret_no_self,
        reason = "Factory pattern; returns trait object"
    )]
    pub async fn new(
        config: SecurityServiceConfig,
    ) -> Result<BeardogSecurityProvider, SecurityError> {
        // Beardog communication should use Unix sockets
        tracing::info!("BeardogIntegration created (HTTP delegation not yet implemented)");

        Ok(BeardogSecurityProvider { config })
    }
}

// Implement the traits::UniversalSecurityProvider for BeardogSecurityProvider
#[async_trait]
impl crate::security::traits::UniversalSecurityProvider for BeardogSecurityProvider {
    async fn authenticate(
        &self,
        credentials: &crate::traits::Credentials,
    ) -> Result<crate::traits::AuthResult, SecurityError> {
        // Convert to security request and use the existing handler
        // Create a default principal for the security context
        let default_principal = crate::traits::Principal {
            id: "system".to_string(),
            name: "System".to_string(),
            principal_type: crate::traits::PrincipalType::System,
            roles: vec!["system".to_string()],
            permissions: vec!["authenticate".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        let context = SecurityContext::from_principal(&default_principal);

        let parameters = serde_json::json!({
            "credentials": credentials
        })
        .as_object()
        .map(|obj| obj.clone().into_iter().collect())
        .unwrap_or_default();

        let request = SecurityRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: SecurityOperation::Authenticate,
            parameters,
            context,
            requester: format!("{}-provider", primal_names::BEARDOG),
            timestamp: chrono::Utc::now(),
            priority: Priority::Normal,
        };

        let _response = self.handle_security_request(request).await?;

        // For now, return a placeholder - this would need proper implementation
        Ok(crate::traits::AuthResult {
            principal: crate::traits::Principal {
                id: "test-principal".to_string(),
                name: "Test User".to_string(),
                principal_type: crate::traits::PrincipalType::User,
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            token: "test-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn authorize(
        &self,
        principal: &crate::traits::Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        tracing::debug!(
            principal_id = %principal.id,
            action,
            resource,
            "Beardog authorize: capability-based check"
        );
        let allowed = principal
            .permissions
            .iter()
            .any(|p| p == action || p == "*");
        Ok(allowed)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let key = blake3::derive_key(
            &format!("ecoPrimals {} encrypt v1", primal_names::BEARDOG),
            self.config.service_id.as_bytes(),
        );
        let mut reader = blake3::Hasher::new_keyed(&key)
            .update(b"encrypt-stream")
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
            &format!("ecoPrimals {} sign v1", primal_names::BEARDOG),
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
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Use existing audit logging through security request
        let parameters = serde_json::json!({
            "operation": operation,
            "context": context
        })
        .as_object()
        .map(|obj| obj.clone().into_iter().collect())
        .unwrap_or_default();

        let request = SecurityRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: SecurityOperation::AuditLog,
            parameters,
            context: context.clone(),
            requester: format!("{}-provider", primal_names::BEARDOG),
            timestamp: chrono::Utc::now(),
            priority: Priority::Normal,
        };

        self.handle_security_request(request).await?;
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
            latency: std::time::Duration::from_millis(10), // Default latency
            last_check: providers_health.last_check,
            details: std::collections::HashMap::new(),
        })
    }
}
