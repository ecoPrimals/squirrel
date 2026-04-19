// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core security capability types, requests/responses, and the universal service trait.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::context::SecurityContext;
use super::super::errors::SecurityError;
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Trust level for security services
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

/// Universal security service trait for capability-based security
///
/// This trait defines the interface for security services that can be
/// dynamically discovered and used based on their capabilities.
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
    #[allow(
        dead_code,
        reason = "trait contract: concrete adapters implement; only exercised in integration tests"
    )]
    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError>;
}

/// Universal security service information
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
    pub fn success(request_id: String, message: String) -> Self {
        Self {
            request_id,
            status: SecurityResponseStatus::Success,
            data: serde_json::json!({"message": message}),
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
            security_context: None,
        }
    }

    /// Create a failed security response
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "public API for error responses; tested in-crate")
    )]
    pub fn failed(request_id: String, reason: String) -> Self {
        Self {
            request_id,
            status: SecurityResponseStatus::Failed { reason },
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
            security_context: None,
        }
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

/// Compliance status for security audit results
#[expect(
    dead_code,
    reason = "public API for compliance integrations; no in-tree caller yet"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// All checks passed
    Compliant,
    /// Violations detected
    NonCompliant {
        /// List of violation descriptions
        violations: Vec<String>,
    },
    /// Audit in progress
    Pending {
        /// Number of checks remaining
        checks_remaining: usize,
    },
    /// Status not yet determined
    Unknown,
}

/// Default implementations and helper functions
impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Medium
    }
}
