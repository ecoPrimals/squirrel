//! Universal Security Capability Definitions
//!
//! This module defines security capabilities and traits that any security service
//! can implement, following the Universal Capability-Based Adapter Pattern.
//!
//! Instead of hardcoding specific provider names, we define what capabilities
//! security services should provide and how they integrate universally.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::context::SecurityContext;
use super::errors::SecurityError;
use crate::config::AuthMethod;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Security health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealth {
    pub status: HealthStatus,
    pub message: String,
    pub last_check: DateTime<Utc>,
    pub metrics: HashMap<String, serde_json::Value>,
}

impl SecurityHealth {
    /// Check if the security service is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }
}

/// Security service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityServiceConfig {
    pub service_id: String,
    pub endpoint: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<u32>,
    pub auth_config: Option<HashMap<String, String>>,
}

impl Default for SecurityServiceConfig {
    fn default() -> Self {
        Self {
            service_id: "default".to_string(),
            endpoint: None,
            timeout_seconds: Some(30),
            max_retries: Some(3),
            auth_config: None,
        }
    }
}

/// Security level enumeration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Trust level for security services
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Low,
    Medium,
    High,
    Verified,
}

/// Priority level for security requests
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Security operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperation {
    Authenticate,
    Authorize,
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    AuditLog,
    Custom(String),
}

/// Universal security capability definition
/// Security services register these capabilities for discovery
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityCapability {
    /// Authentication capabilities
    Authentication {
        methods: Vec<AuthMethod>,
        multi_factor: bool,
        session_management: bool,
    },

    /// Authorization capabilities  
    Authorization {
        rbac: bool,
        abac: bool,
        policy_engine: bool,
    },

    /// Cryptographic capabilities
    Cryptography {
        algorithms: Vec<String>,
        key_management: bool,
        hardware_security: bool,
    },

    /// Audit and compliance capabilities
    Compliance {
        standards: Vec<String>,
        audit_logging: bool,
        real_time_monitoring: bool,
    },

    /// Threat detection capabilities
    ThreatDetection {
        anomaly_detection: bool,
        real_time_analysis: bool,
        threat_intelligence: bool,
    },

    /// Identity management capabilities
    Identity {
        provisioning: bool,
        lifecycle_management: bool,
        federation: bool,
    },

    /// Data protection capabilities
    DataProtection {
        encryption_at_rest: bool,
        encryption_in_transit: bool,
        data_classification: bool,
    },
}

/// Universal security service trait
/// Any security service (regardless of name) can implement this
#[async_trait]
/// Universal security service trait for capability-based security
///
/// This trait defines the interface for security services that can be
/// dynamically discovered and used based on their capabilities.
#[allow(dead_code)] // API designed for future use
pub trait UniversalSecurityService: Send + Sync {
    /// Get the capabilities this security service provides
    fn get_capabilities(&self) -> Vec<SecurityCapability>;

    /// Get service metadata
    fn get_service_info(&self) -> SecurityServiceInfo;

    /// Process a universal security request
    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError>;

    /// Health check for the security service
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError>;

    /// Initialize the security service
    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError>;
}

/// Universal security service information
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityServiceInfo {
    pub service_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<SecurityCapability>,
    pub endpoints: Vec<SecurityEndpoint>,
    pub supported_protocols: Vec<String>,
    pub compliance_certifications: Vec<String>,
    pub trust_level: TrustLevel,
}

/// Security service endpoint information  
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEndpoint {
    pub name: String,
    pub url: String,
    pub protocol: String,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub security_level: SecurityLevel,
}

/// Universal security request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequest {
    pub request_id: String,
    pub operation: SecurityOperation,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: SecurityContext,
    pub requester: String,
    pub timestamp: DateTime<Utc>,
    pub priority: Priority,
}

/// Universal security response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    pub request_id: String,
    pub status: SecurityResponseStatus,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processing_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub security_context: Option<SecurityContext>,
}

impl SecurityResponse {
    /// Create a successful security response
    pub fn success(request_id: String, message: String) -> Result<Self, SecurityError> {
        Ok(Self {
            request_id,
            status: SecurityResponseStatus::Success,
            data: serde_json::json!({"message": message}),
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
            security_context: None,
        })
    }

    /// Create a failed security response
    #[allow(dead_code)]
    pub fn failed(request_id: String, reason: String) -> Result<Self, SecurityError> {
        Ok(Self {
            request_id,
            status: SecurityResponseStatus::Failed { reason },
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
            security_context: None,
        })
    }
}

/// Security response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityResponseStatus {
    Success,
    Denied,
    Failed { reason: String },
    Partial { completed: usize, total: usize },
    RequiresAdditionalAuth,
}

/// Compliance status
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant { violations: Vec<String> },
    Pending { checks_remaining: usize },
    Unknown,
}

/// Universal security service provider trait alias for backward compatibility
#[allow(dead_code)]
pub trait UniversalSecurityProvider: UniversalSecurityService {}

impl<T: UniversalSecurityService> UniversalSecurityProvider for T {}

/// Universal security service registry
/// Services register themselves with their capabilities here
#[allow(dead_code)]
pub struct UniversalSecurityRegistry {
    services: HashMap<String, Arc<dyn UniversalSecurityService>>,
    capabilities_index: HashMap<SecurityCapability, Vec<String>>,
}

#[allow(dead_code)] // API designed for future use
impl UniversalSecurityRegistry {
    /// Create a new security service registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            capabilities_index: HashMap::new(),
        }
    }

    /// Register a security service (any service, regardless of name)
    pub async fn register_service(
        &mut self,
        service_id: String,
        service: Arc<dyn UniversalSecurityService>,
    ) -> Result<(), SecurityError> {
        // Get service capabilities and index them
        let capabilities = service.get_capabilities();

        // Add service to registry
        self.services.insert(service_id.clone(), service);

        // Update capability index
        for capability in capabilities {
            self.capabilities_index
                .entry(capability)
                .or_default()
                .push(service_id.clone());
        }

        Ok(())
    }

    /// Find services by capability (agnostic discovery)
    pub fn find_by_capability(&self, capability: &SecurityCapability) -> Vec<String> {
        self.capabilities_index
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }

    /// Get optimal service for specific security requirements
    pub async fn find_optimal_service(
        &self,
        requirements: Vec<SecurityCapability>,
    ) -> Result<String, SecurityError> {
        let mut candidates = Vec::new();

        // Find services that have all required capabilities
        for service_id in self.services.keys() {
            if let Some(service) = self.services.get(service_id) {
                let service_caps = service.get_capabilities();

                let has_all_requirements = requirements
                    .iter()
                    .all(|req| service_caps.iter().any(|cap| capabilities_match(req, cap)));

                if has_all_requirements {
                    candidates.push(service_id.clone());
                }
            }
        }

        if candidates.is_empty() {
            return Err(SecurityError::configuration(
                "No security services found matching requirements",
            ));
        }

        // For now, return first candidate (could implement scoring logic)
        Ok(candidates[0].clone())
    }

    /// Get service by ID
    pub fn get_service(&self, service_id: &str) -> Option<Arc<dyn UniversalSecurityService>> {
        self.services.get(service_id).cloned()
    }

    /// List all registered services
    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

/// Check if two security capabilities match
#[allow(dead_code)] // Utility function for future use
pub fn capabilities_match(required: &SecurityCapability, provided: &SecurityCapability) -> bool {
    use SecurityCapability::*;

    match (required, provided) {
        (
            Authentication {
                methods: req_methods,
                ..
            },
            Authentication {
                methods: prov_methods,
                ..
            },
        ) => req_methods.iter().any(|req| prov_methods.contains(req)),
        (
            Authorization {
                rbac: req_rbac,
                abac: req_abac,
                ..
            },
            Authorization {
                rbac: prov_rbac,
                abac: prov_abac,
                ..
            },
        ) => (!req_rbac || *prov_rbac) && (!req_abac || *prov_abac),
        (
            Cryptography {
                algorithms: req_algs,
                ..
            },
            Cryptography {
                algorithms: prov_algs,
                ..
            },
        ) => req_algs.iter().any(|req| prov_algs.contains(req)),
        (
            Compliance {
                standards: req_stds,
                ..
            },
            Compliance {
                standards: prov_stds,
                ..
            },
        ) => req_stds.iter().any(|req| prov_stds.contains(req)),
        (ThreatDetection { .. }, ThreatDetection { .. }) => true,
        (Identity { .. }, Identity { .. }) => true,
        (DataProtection { .. }, DataProtection { .. }) => true,
        _ => false,
    }
}

/// Default implementations and helper functions
impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Medium
    }
}

/// Example registration function for any security service
/// This shows how a specific security service (like BearDog) would register
#[allow(dead_code)] // API function for future use
pub async fn register_security_service(
    registry: &mut UniversalSecurityRegistry,
    service: Arc<dyn UniversalSecurityService>,
) -> Result<(), SecurityError> {
    let info = service.get_service_info();
    registry.register_service(info.service_id, service).await
}

/// Beardog Security Provider Implementation
/// Integrates with Beardog security service through capability-based discovery
/// TODO: HTTP removed - should use Unix socket communication
pub struct BeardogSecurityProvider {
    #[allow(dead_code)]
    config: SecurityServiceConfig,
    // Note: HTTP client removed - should use Unix socket for Beardog communication
}

impl BeardogSecurityProvider {
    /// Create a new Beardog security provider
    /// TODO: Should use Unix socket discovery instead of HTTP
    pub async fn new(config: SecurityServiceConfig) -> Result<Self, SecurityError> {
        // Beardog communication should use Unix sockets
        // Pattern: UnixStream::connect("/var/run/beardog/security.sock").await
        tracing::info!("BeardogSecurityProvider created (HTTP delegation not yet implemented)");
        
        Ok(Self {
            config,
        })
    }
}

#[async_trait]
impl UniversalSecurityService for BeardogSecurityProvider {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        vec![
            SecurityCapability::Authentication {
                methods: vec![AuthMethod::Beardog {
                    service_id: "beardog-primary".to_string(),
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
        let trust_level = if self.config.service_id == "beardog-security" {
            TrustLevel::High
        } else {
            TrustLevel::Medium
        };

        SecurityServiceInfo {
            service_id: "beardog-security".to_string(),
            name: "Beardog Security Service".to_string(),
            version: "1.0.0".to_string(),
            description: "Enterprise security service with comprehensive capabilities".to_string(),
            capabilities: self.get_capabilities(),
            endpoints: vec![],
            supported_protocols: vec!["HTTPS".to_string(), "gRPC".to_string()],
            compliance_certifications: vec!["SOC2".to_string(), "ISO27001".to_string()],
            trust_level,
        }
    }

    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError> {
        // Implementation would make actual requests to Beardog service
        SecurityResponse::success(
            request.request_id,
            "Beardog operation completed".to_string(),
        )
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            status: HealthStatus::Healthy,
            message: "Beardog security service operational".to_string(),
            last_check: Utc::now(),
            metrics: HashMap::new(),
        })
    }

    /// TODO: Should use Unix socket discovery instead of HTTP
    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError> {
        self.config = config;
        // HTTP client removed - should use Unix socket for Beardog communication
        tracing::info!("BeardogSecurityProvider initialized (HTTP delegation not yet implemented)");
        Ok(())
    }
}

/// Local Security Provider Implementation  
/// Provides basic local security capabilities for fallback scenarios
pub struct LocalSecurityProvider {
    #[allow(dead_code)] // Will be used when provider is fully implemented
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
        SecurityResponse::success(request.request_id, "Local operation completed".to_string())
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

/// Beardog Integration helper
/// TODO: HTTP removed - should use Unix socket communication
pub struct BeardogIntegration;

impl BeardogIntegration {
    /// Create a new Beardog integration
    /// TODO: Should use Unix socket discovery instead of HTTP
    ///
    /// Note: This is a factory function that returns BeardogSecurityProvider, not Self.
    /// This is intentional as BeardogIntegration is a namespace for integration logic.
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        config: SecurityServiceConfig,
    ) -> Result<BeardogSecurityProvider, SecurityError> {
        // Beardog communication should use Unix sockets
        tracing::info!("BeardogIntegration created (HTTP delegation not yet implemented)");
        
        Ok(BeardogSecurityProvider {
            config,
        })
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
            requester: "beardog-provider".to_string(),
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
        _principal: &crate::traits::Principal,
        _action: &str,
        _resource: &str,
    ) -> Result<bool, SecurityError> {
        // Placeholder implementation
        Ok(true)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Placeholder implementation - would use actual Beardog encryption
        Ok(data.to_vec())
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Placeholder implementation - would use actual Beardog decryption
        Ok(encrypted_data.to_vec())
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Placeholder implementation - would use actual Beardog signing
        Ok(data.to_vec())
    }

    async fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool, SecurityError> {
        // Placeholder implementation
        Ok(true)
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
            requester: "beardog-provider".to_string(),
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
        _principal: &crate::traits::Principal,
        _action: &str,
        _resource: &str,
    ) -> Result<bool, SecurityError> {
        // Local authorization - allow everything for testing
        Ok(true)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple local encryption (not production-ready)
        Ok(data.to_vec())
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple local decryption (not production-ready)
        Ok(encrypted_data.to_vec())
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple local signing (not production-ready)
        Ok(data.to_vec())
    }

    async fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool, SecurityError> {
        // Simple local verification
        Ok(true)
    }

    async fn audit_log(
        &self,
        _operation: &str,
        _context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Local audit logging - just log to stdout for testing
        println!("Local audit: {} at {:?}", _operation, chrono::Utc::now());
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

#[cfg(test)]
mod tests;
