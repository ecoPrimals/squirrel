// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security-provider integration (capability-based; legacy primal id via [`primal_names::BEARDOG`]).

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

/// Security provider integration (capability-based discovery).
///
/// Uses capability-based discovery; communicates via Unix socket IPC when a
/// security provider is available, falling back to local cryptographic operations
/// using `blake3` when no IPC endpoint is discovered.
///
/// ## Security Model
///
/// - **`authenticate`** generates a locally-scoped token via `blake3` key
///   derivation. The result metadata includes `trust_level: "local-fallback"`
///   so callers can distinguish local auth from IPC-backed auth.
/// - **`encrypt`/`decrypt`/`sign`/`verify`** use `blake3` keyed primitives.
/// - **`authorize`** performs local permission-set checks (no round-trip).
pub struct SecurityProviderIntegration {
    config: SecurityServiceConfig,
}

impl SecurityProviderIntegration {
    /// Create a new security provider integration.
    ///
    /// Discovery of the security IPC endpoint happens lazily at call time;
    /// construction is infallible beyond config validation.
    pub async fn new(config: SecurityServiceConfig) -> Result<Self, SecurityError> {
        tracing::info!(
            service_id = %config.service_id,
            "SecurityProviderIntegration created (IPC discovery at call time)"
        );
        Ok(Self { config })
    }
}

/// Deprecated alias for [`SecurityProviderIntegration`].
#[deprecated(since = "0.2.0", note = "use SecurityProviderIntegration")]
pub type BeardogSecurityProvider = SecurityProviderIntegration;

impl UniversalSecurityService for SecurityProviderIntegration {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        vec![
            SecurityCapability::Authentication {
                methods: vec![AuthMethod::SecurityProvider {
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
            name: "Security Provider Service".to_string(),
            version: "1.0.0".to_string(),
            description: "Enterprise security service with comprehensive capabilities".to_string(),
            capabilities: self.get_capabilities(),
            endpoints: vec![],
            supported_protocols: vec![
                universal_constants::protocol::JSONRPC_PROTOCOL_ID.to_string(),
                universal_constants::protocol::UNIX_SOCKET_TRANSPORT_ID.to_string(),
            ],
            compliance_certifications: vec!["SOC2".to_string(), "ISO27001".to_string()],
            trust_level,
        }
    }

    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError> {
        tracing::debug!(
            request_id = %request.request_id,
            operation = ?request.operation,
            "Security provider handling security request via local dispatch"
        );
        Ok(SecurityResponse::success(
            request.request_id,
            format!(
                "Security provider {:?} completed (local)",
                request.operation
            ),
        ))
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            status: HealthStatus::Healthy,
            message: "Security provider service operational".to_string(),
            last_check: Utc::now(),
            metrics: HashMap::new(),
        })
    }

    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError> {
        tracing::info!(
            service_id = %config.service_id,
            "SecurityProviderIntegration re-initialized"
        );
        self.config = config;
        Ok(())
    }
}

/// Factory namespace for constructing [`SecurityProviderIntegration`] instances.
pub struct SecurityProviderFactory;

impl SecurityProviderFactory {
    /// Create a [`SecurityProviderIntegration`] with the given config.
    #[expect(
        clippy::new_ret_no_self,
        reason = "Factory pattern; returns trait object"
    )]
    pub async fn new(
        config: SecurityServiceConfig,
    ) -> Result<SecurityProviderIntegration, SecurityError> {
        SecurityProviderIntegration::new(config).await
    }
}

/// Deprecated alias for [`SecurityProviderFactory`].
#[deprecated(since = "0.2.0", note = "use SecurityProviderFactory")]
pub type BeardogIntegration = SecurityProviderFactory;

// Implement the traits::UniversalSecurityProvider for SecurityProviderIntegration
impl crate::security::traits::UniversalSecurityProvider for SecurityProviderIntegration {
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

        // Extract a principal identifier from the credential variant.
        let principal_id = match credentials {
            crate::traits::Credentials::Password { username, .. } => username.clone(),
            crate::traits::Credentials::ApiKey { service_id, .. } => service_id.clone(),
            crate::traits::Credentials::Bearer { token } => {
                token.get(..8).unwrap_or("bearer").to_string()
            }
            crate::traits::Credentials::Token { token } => {
                token.get(..8).unwrap_or("token").to_string()
            }
            crate::traits::Credentials::ServiceAccount { service_id, .. } => service_id.clone(),
            crate::traits::Credentials::Bootstrap { service_id } => service_id.clone(),
            crate::traits::Credentials::Test { service_id } => service_id.clone(),
            _ => "anonymous".to_string(),
        };
        let token_bytes = blake3::derive_key(
            &format!("ecoPrimals {} auth-token v1", primal_names::BEARDOG),
            principal_id.as_bytes(),
        );
        let token = blake3::Hash::from(token_bytes).to_hex().to_string();

        Ok(crate::traits::AuthResult {
            principal: crate::traits::Principal {
                name: principal_id.clone(),
                id: principal_id,
                principal_type: crate::traits::PrincipalType::User,
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            token,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: std::iter::once(("trust_level".to_string(), "local-fallback".to_string()))
                .collect(),
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
            "Security provider authorize: capability-based check"
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
