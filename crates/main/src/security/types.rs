//! Security Types
//!
//! This module contains all security-related enums, types, and basic data structures
//! used throughout the security system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Security capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityCapability {
    /// Authentication and authorization
    Authentication,
    /// Encryption services
    Encryption,
    /// Access control
    AccessControl,
    /// Security auditing
    Auditing,
    /// Threat detection
    ThreatDetection,
    /// Compliance monitoring
    ComplianceMonitoring,
    /// Security analytics
    SecurityAnalytics,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Low security requirements
    Low,
    /// Standard security requirements
    Standard,
    /// High security requirements
    High,
    /// Critical security requirements
    Critical,
}

/// Authorization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorizationLevel {
    /// No authorization
    None,
    /// Basic user authorization
    User,
    /// Elevated authorization
    Elevated,
    /// Administrator authorization
    Admin,
    /// System-level authorization
    System,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User identifier
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Operation context
    pub operation: String,
    /// Resource being accessed
    pub resource: String,
    /// Client information
    pub client_info: HashMap<String, String>,
    /// Environment information
    pub environment: HashMap<String, String>,
    /// Timestamp of the operation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Authentication handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationHandler {
    /// Handler name
    pub name: String,
    /// Handler type
    pub handler_type: String,
    /// Handler configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Priority (lower numbers = higher priority)
    pub priority: u32,
    /// Enabled status
    pub enabled: bool,
}

/// Universal security request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequest {
    /// Request identifier
    pub request_id: String,
    /// Request type
    pub request_type: SecurityRequestType,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Security context
    pub context: SecurityContext,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRequestType {
    /// Authentication request
    Authentication,
    /// Authorization request
    Authorization,
    /// Encryption request
    Encryption,
    /// Decryption request
    Decryption,
    /// Audit request
    Audit,
    /// Policy check request
    PolicyCheck,
    /// Health check request
    HealthCheck,
    /// Token validation request
    TokenValidation,
}

/// Universal security response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    /// Request identifier
    pub request_id: String,
    /// Response status
    pub status: SecurityResponseStatus,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Processing time
    pub processing_time: Duration,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityResponseStatus {
    /// Success
    Success,
    /// Authentication failed
    AuthenticationFailed,
    /// Authorization failed
    AuthorizationFailed,
    /// Policy violation
    PolicyViolation,
    /// Threat detected
    ThreatDetected,
    /// Error
    Error { code: String, message: String },
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            operation: "unknown".to_string(),
            resource: "unknown".to_string(),
            client_info: HashMap::new(),
            environment: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(operation: &str, resource: &str) -> Self {
        Self {
            user_id: None,
            session_id: None,
            operation: operation.to_string(),
            resource: resource.to_string(),
            client_info: HashMap::new(),
            environment: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add client information
    pub fn with_client_info(mut self, key: String, value: String) -> Self {
        self.client_info.insert(key, value);
        self
    }

    /// Add environment information
    pub fn with_environment(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
}
